//! The VoidXStudio project layer: projects as groups of files in a world.
//!
//! A created project (`.vxs`) wraps the world (the layout and the connections between its
//! groups) and its files: one visual group per `Ui` kind (`.figcraft` documents) and one
//! logic group per `Engine` / `Network` kind (the IR node graph the Live Preview runs as Lua
//! and the final export compiles to Kotlin).

use std::path::Path;

use serde::{Deserialize, Serialize};

/// The extension of a saved project container: a ZIP like the `.figcraft` one.
pub const VXS_EXT: &str = ".vxs";

/// The language a created project's groups compile to / preview in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectLanguage {
    /// The only active target for now; the others are `ComingSoon`.
    #[default]
    Kotlin,
    Flutter,
    Dart,
    Slint,
}

/// The kind of a file group: the kind of artifact the group holds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileGroupKind {
    /// A visual group: the `.figcraft` document of the design.
    #[default]
    Ui,
    /// A logic group: the IR node graph.
    Engine,
    /// A network group: the backend/data graph.
    Network,
}

impl FileGroupKind {
    /// The label the UI shows for a group's kind.
    pub fn label(&self) -> &'static str {
        match self {
            FileGroupKind::Ui => "UI",
            FileGroupKind::Engine => "Engine",
            FileGroupKind::Network => "Network",
        }
    }
}

/// One file group of the project: its kind, its visual document (a `Ui` group) and its logic
/// graph (an `Engine` / `Network` group).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileGroup {
    pub name: String,
    pub kind: FileGroupKind,
    /// The `.figcraft` document of a `Ui` group (its name inside `files/`).
    pub visual: Option<String>,
    /// The IR node graph of an `Engine` / `Network` group (JSON).
    pub logic: Option<serde_json::Value>,
}

/// A link between two groups of the world.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldConnection {
    /// The group being driven (the source the event comes from).
    pub from: String,
    /// The group receiving the event (the target of the wire).
    pub to: String,
}

/// The world: the groups of the project and the connections between them.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct World {
    pub connections: Vec<WorldConnection>,
}

/// The draft the Create Project dialog edits before the project is created.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectDraft {
    pub name: String,
    pub package_name: String,
    pub language: ProjectLanguage,
    pub icon: String,
}

/// A created VoidXStudio project.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoidXProject {
    /// The app's name.
    pub name: String,
    /// The Android application id: two segments minimum (`com.example.app`).
    pub package_name: String,
    /// The target language of the created project's groups.
    pub language: ProjectLanguage,
    /// The app's icon, when the creator chose one.
    pub icon: Option<String>,
    /// The world: groups and their connections.
    pub world: World,
    /// The project's files.
    pub files: Vec<FileGroup>,
}

impl VoidXProject {
    /// The name for a new group of the given kind.
    pub fn new_group_name(kind: FileGroupKind) -> String {
        match kind {
            FileGroupKind::Ui => "Main".into(),
            FileGroupKind::Engine => "Main".into(),
            FileGroupKind::Network => "Main".into(),
        }
    }

    /// Whether the package name looks like an Android application id: two segments minimum,
    /// each starting with a letter and holding alphanumerics, `-` or `_`.
    pub fn package_name_ok(package_name: &str) -> bool {
        let bad = |s: &str| !s.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) || s.chars().any(|c| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'));
        let parts: Vec<&str> = package_name.split('.').collect();
        parts.len() >= 2 && parts.iter().all(|p| !p.is_empty() && !bad(p))
    }
}

/// Save the project as a `.vxs` container: `project.json` plus one entry per file group.
pub fn save(path: &Path, project: &VoidXProject) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("project.json", options).map_err(|e| e.to_string())?;
    let json = serde_json::to_vec_pretty(project).map_err(|e| e.to_string())?;
    std::io::Write::write_all(&mut zip, &json).map_err(|e| e.to_string())?;
    for group in &project.files {
        let Some(name) = group.visual.as_ref().map(|v| v.trim_end_matches(".figcraft").to_string()) else { continue };
        let entry = format!("files/{name}.logic.json");
        let json = group.logic.clone().unwrap_or(serde_json::json!({}));
        let json = serde_json::to_vec_pretty(&json).map_err(|e| e.to_string())?;
        zip.start_file(&entry, options).map_err(|e| e.to_string())?;
        std::io::Write::write_all(&mut zip, &json).map_err(|e| e.to_string())?;
    }
    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

/// Load a project from its `.vxs` container.
pub fn load(path: &Path) -> Result<VoidXProject, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut project: Option<VoidXProject> = None;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut data).map_err(|e| e.to_string())?;
        if name == "project.json" {
            project = Some(serde_json::from_slice(&data).map_err(|e| format!("project.json: {e}"))?);
        }
    }
    project.ok_or_else(|| "el .vxs no tiene project.json".to_string())
}
