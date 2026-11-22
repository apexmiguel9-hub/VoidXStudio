//! The VoidXStudio project layer: projects as groups of files in a world.
//!
//! A created project (`.vxs`) wraps the world (the layout and the connections between its
//! groups) and its files: one visual group per `Ui`/`Main` kind (`.figcraft` documents) and
//! one logic group per `Engine` / `Data` / `Network` kind (the IR node graph the Live Preview
//! runs as Lua and the final export compiles to Kotlin).

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

impl ProjectLanguage {
    pub const ALL: [ProjectLanguage; 4] = [
        ProjectLanguage::Kotlin,
        ProjectLanguage::Flutter,
        ProjectLanguage::Dart,
        ProjectLanguage::Slint,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ProjectLanguage::Kotlin => "Kotlin",
            ProjectLanguage::Flutter => "Flutter (Soon)",
            ProjectLanguage::Dart => "Dart (Soon)",
            ProjectLanguage::Slint => "Slint (Soon)",
        }
    }

    pub fn available(self) -> bool {
        matches!(self, ProjectLanguage::Kotlin)
    }
}

/// The kind of a file group: the kind of artifact the group holds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileGroupKind {
    /// Entry point of the app. Exactly one per project; anchored in World Space.
    Main,
    /// A visual group: the `.figcraft` document of the design.
    #[default]
    Ui,
    /// A logic group: main logic.
    Engine,
    /// Local storage / data.
    Data,
    /// Network and database.
    Network,
}

impl FileGroupKind {
    /// Kinds the user can create from the Add modal, in display order.
    pub const CREATE_ORDER: [FileGroupKind; 5] = [
        FileGroupKind::Main,
        FileGroupKind::Ui,
        FileGroupKind::Engine,
        FileGroupKind::Data,
        FileGroupKind::Network,
    ];

    /// The label the UI shows for a group's kind.
    pub fn label(self) -> &'static str {
        match self {
            FileGroupKind::Main => "Main",
            FileGroupKind::Ui => "UI",
            FileGroupKind::Engine => "Engine",
            FileGroupKind::Data => "Data",
            FileGroupKind::Network => "Network",
        }
    }

    /// Short description shown in add modal.
    pub fn description(self) -> &'static str {
        match self {
            FileGroupKind::Main => "Punto de entrada de la app",
            FileGroupKind::Ui => "Interfaz visual",
            FileGroupKind::Engine => "Lógica principal",
            FileGroupKind::Data => "Almacenamiento local",
            FileGroupKind::Network => "Network y Database",
        }
    }

    /// Whether this kind opens the FigCraft vector editor.
    pub fn opens_figcraft(self) -> bool {
        matches!(self, FileGroupKind::Ui)
    }

    /// Whether only one instance is allowed per project.
    pub fn unique(self) -> bool {
        matches!(self, FileGroupKind::Main)
    }

    /// Whether the card is anchored (not draggable) in World Space.
    pub fn anchored(self) -> bool {
        matches!(self, FileGroupKind::Main)
    }

    /// Visual style for the World Space card, derived from the reference XMLs.
    pub fn style(self) -> ContainerStyle {
        match self {
            FileGroupKind::Main => ContainerStyle {
                fill: [0x14, 0x1C, 0x1D],
                stroke: [0x0C, 0xCE, 0x87],
                accent: [0x11, 0xE9, 0x89],
                badge_fill: [0x19, 0x3D, 0x23],
                badge_text: [0x31, 0xEC, 0x8E],
                badge: "Main",
            },
            FileGroupKind::Ui => ContainerStyle {
                fill: [0x1D, 0x14, 0x1C],
                stroke: [0xE5, 0x11, 0xE9],
                accent: [0xE5, 0x11, 0xE9],
                badge_fill: [0xAC, 0x00, 0x83],
                badge_text: [0xFF, 0xFF, 0xFF],
                badge: "UI",
            },
            FileGroupKind::Engine => ContainerStyle {
                fill: [0x14, 0x18, 0x1D],
                stroke: [0x0C, 0x76, 0xCE],
                accent: [0x11, 0x9C, 0xE9],
                badge_fill: [0x00, 0x4C, 0xAC],
                badge_text: [0xFF, 0xFF, 0xFF],
                badge: "Engine",
            },
            FileGroupKind::Data => ContainerStyle {
                fill: [0x1C, 0x1D, 0x14],
                stroke: [0xE9, 0xDC, 0x11],
                accent: [0xE9, 0xDC, 0x11],
                badge_fill: [0xAC, 0xA0, 0x00],
                badge_text: [0xFF, 0xFF, 0xFF],
                badge: "Data",
            },
            FileGroupKind::Network => ContainerStyle {
                fill: [0x1D, 0x18, 0x14],
                stroke: [0xE9, 0x55, 0x11],
                accent: [0xE9, 0x55, 0x11],
                badge_fill: [0xE9, 0x55, 0x11],
                badge_text: [0xFF, 0xFF, 0xFF],
                badge: "Network",
            },
        }
    }
}

/// Visual style for a container card, taken from the reference XML designs.
#[derive(Clone, Copy, Debug)]
pub struct ContainerStyle {
    pub fill: [u8; 3],
    pub stroke: [u8; 3],
    pub accent: [u8; 3],
    pub badge_fill: [u8; 3],
    pub badge_text: [u8; 3],
    pub badge: &'static str,
}

/// One file group of the project: its kind, its visual document (a `Ui`/`Main` group) and its
/// logic graph (an `Engine` / `Data` / `Network` group).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileGroup {
    pub name: String,
    pub kind: FileGroupKind,
    /// The `.figcraft` document of a visual group (its name inside `files/`).
    #[serde(default)]
    pub visual: Option<String>,
    /// The IR node graph of a logic group (JSON).
    #[serde(default)]
    pub logic: Option<serde_json::Value>,
    /// Position in World Space canvas (logical units).
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    /// In-memory figcraft bytes for this group; persisted in .vxs as files/<visual>.
    #[serde(skip)]
    pub visual_data: Option<Vec<u8>>,
}

/// A link between two groups of the world.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldConnection {
    pub from: String,
    pub to: String,
}

/// The world: the groups of the project and the connections between them.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct World {
    #[serde(default)]
    pub connections: Vec<WorldConnection>,
}

/// The draft the Create Project dialog edits before the project is created.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectDraft {
    pub name: String,
    pub package_name: String,
    pub language: ProjectLanguage,
    pub icon: String,
}

impl Default for ProjectDraft {
    fn default() -> Self {
        Self {
            name: String::new(),
            package_name: String::new(),
            language: ProjectLanguage::Kotlin,
            icon: String::new(),
        }
    }
}

/// Draft for adding a new container/group in World Space.
#[derive(Clone, Debug)]
pub struct AddGroupDraft {
    pub kind: FileGroupKind,
    pub name: String,
}

impl Default for AddGroupDraft {
    fn default() -> Self {
        Self { kind: FileGroupKind::Ui, name: String::new() }
    }
}

/// A created VoidXStudio project.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoidXProject {
    pub name: String,
    pub package_name: String,
    pub language: ProjectLanguage,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub world: World,
    #[serde(default)]
    pub files: Vec<FileGroup>,
}

impl VoidXProject {
    /// Whether a kind can still be added (Main is unique).
    pub fn can_add_kind(&self, kind: FileGroupKind) -> bool {
        !kind.unique() || !self.files.iter().any(|f| f.kind == kind)
    }

    /// Default file name suggestion for a kind.
    pub fn default_group_name(kind: FileGroupKind) -> String {
        match kind {
            FileGroupKind::Main => "main".into(),
            FileGroupKind::Ui => "screen".into(),
            FileGroupKind::Engine => "engine".into(),
            FileGroupKind::Data => "data".into(),
            FileGroupKind::Network => "network".into(),
        }
    }

    /// Sanitize a file/group name for use as identifier.
    pub fn sanitize_name(name: &str) -> String {
        let mut out = String::new();
        for c in name.chars() {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                out.push(c);
            } else if c.is_whitespace() {
                out.push('_');
            }
        }
        let out = out.trim_matches('_').trim_matches('-').to_string();
        if out.is_empty() { "unnamed".to_string() } else { out }
    }

    /// Display name with .kt extension.
    pub fn display_kt(name: &str) -> String {
        let base = Self::sanitize_name(name);
        format!("{base}.kt")
    }

    /// Whether the package name looks like an Android application id: two segments minimum.
    pub fn package_name_ok(package_name: &str) -> bool {
        let bad = |s: &str| {
            !s.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
                || s.chars().any(|c| !(c.is_ascii_alphanumeric() || c == '_' ))
        };
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
        if let Some(logic) = &group.logic {
            let name = VoidXProject::sanitize_name(&group.name);
            let entry = format!("files/{name}.logic.json");
            let json = serde_json::to_vec_pretty(logic).map_err(|e| e.to_string())?;
            zip.start_file(&entry, options).map_err(|e| e.to_string())?;
            std::io::Write::write_all(&mut zip, &json).map_err(|e| e.to_string())?;
        }
        if let (Some(visual), Some(data)) = (&group.visual, &group.visual_data) {
            let entry = format!("files/{visual}");
            zip.start_file(&entry, options).map_err(|e| e.to_string())?;
            std::io::Write::write_all(&mut zip, data).map_err(|e| e.to_string())?;
        }
    }
    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

/// Load a project from its `.vxs` container.
pub fn load(path: &Path) -> Result<VoidXProject, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut project: Option<VoidXProject> = None;
    let mut visuals: std::collections::HashMap<String, Vec<u8>> = std::collections::HashMap::new();
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut data).map_err(|e| e.to_string())?;
        if name == "project.json" {
            project = Some(serde_json::from_slice(&data).map_err(|e| format!("project.json: {e}"))?);
        } else if let Some(file_name) = name.strip_prefix("files/") {
            if file_name.ends_with(".figcraft") || file_name.ends_with(".fig") {
                visuals.insert(file_name.to_string(), data);
            }
        }
    }
    let mut project = project.ok_or_else(|| "el .vxs no tiene project.json".to_string())?;
    for g in &mut project.files {
        if let Some(v) = &g.visual {
            g.visual_data = visuals.remove(v);
        }
    }
    Ok(project)
}
