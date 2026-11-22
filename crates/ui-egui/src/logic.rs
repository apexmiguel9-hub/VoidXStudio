//! Logic graph model: visual IR for project workspaces.
//! Serialized as `files/<name>.logic.json` inside .vxs.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LogicGraph {
    #[serde(default)]
    pub nodes: Vec<LogicNode>,
    #[serde(default, alias = "wires")]
    pub links: Vec<NodeLink>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogicNode {
    pub id: u64,
    pub kind: NodeKind,
    pub x: f32,
    pub y: f32,
    #[serde(default)]
    pub props: NodeProps,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeLink {
    pub from_node: u64,
    pub from_port: String,
    pub to_node: u64,
    pub to_port: String,
}

// Back-compat alias used in older code paths.
pub type NodeWire = NodeLink;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    // Main - Lifecycle
    #[serde(alias = "on_app_start")]
    OnReady,
    OnResume,
    OnPause,
    OnDestroy,
    OnBackPressed,
    // Main - Orchestration
    #[serde(alias = "open_screen")]
    SetInitialScreen,
    NavigateToScreen,
    InitEngine,
    InitData,
    InitNetwork,
    CallModuleFunction,
    // Main - Android / Permissions
    RequestPermission,
    CheckPermission,
    SetStatusBar,
    KeepScreenOn,
    ExitApp,
    // Main - Intents / External
    OnDeepLink,
    OpenExternalUrl,
    ShareText,
    // Main/UI/Engine/Data/Network shared & others
    OnTap,
    SetVisible,
    SetText,
    NavigateTo,
    CallFunction,
    DefineFunction,
    SetVariable,
    MathOp,
    IfCondition,
    #[serde(alias = "timer_delay")]
    DelayTimer,
    SaveLocal,
    LoadLocal,
    DeleteLocal,
    HttpRequest,
    JsonGetField,
    LogDebug,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MainCategory {
    Lifecycle,
    Orchestration,
    Android,
    Intents,
    Flow,
}

impl MainCategory {
    pub fn label(self) -> &'static str {
        match self {
            MainCategory::Lifecycle => "Ciclo de Vida",
            MainCategory::Orchestration => "Orquestación",
            MainCategory::Android => "Sistema Android",
            MainCategory::Intents => "Intents",
            MainCategory::Flow => "Flujo",
        }
    }
}

impl NodeKind {
    pub fn label(self) -> &'static str {
        match self {
            NodeKind::OnReady => "OnReady",
            NodeKind::OnResume => "OnResume",
            NodeKind::OnPause => "OnPause",
            NodeKind::OnDestroy => "OnDestroy",
            NodeKind::OnBackPressed => "OnBackPressed",
            NodeKind::SetInitialScreen => "SetInitialScreen",
            NodeKind::NavigateToScreen => "NavigateToScreen",
            NodeKind::InitEngine => "InitEngine",
            NodeKind::InitData => "InitData",
            NodeKind::InitNetwork => "InitNetwork",
            NodeKind::CallModuleFunction => "CallModuleFunction",
            NodeKind::RequestPermission => "RequestPermission",
            NodeKind::CheckPermission => "CheckPermission",
            NodeKind::SetStatusBar => "SetStatusBar",
            NodeKind::KeepScreenOn => "KeepScreenOn",
            NodeKind::ExitApp => "ExitApp",
            NodeKind::OnDeepLink => "OnDeepLink",
            NodeKind::OpenExternalUrl => "OpenExternalUrl",
            NodeKind::ShareText => "ShareText",
            NodeKind::IfCondition => "IfCondition",
            NodeKind::DelayTimer => "DelayTimer",
            NodeKind::LogDebug => "LogDebug",
            NodeKind::OnTap => "On Tap",
            NodeKind::SetVisible => "Set Visible",
            NodeKind::SetText => "Set Text",
            NodeKind::NavigateTo => "Navigate To",
            NodeKind::CallFunction => "Call Function",
            NodeKind::DefineFunction => "Define Function",
            NodeKind::SetVariable => "Set Variable",
            NodeKind::MathOp => "Math Op",
            NodeKind::SaveLocal => "Save Local",
            NodeKind::LoadLocal => "Load Local",
            NodeKind::DeleteLocal => "Delete Local",
            NodeKind::HttpRequest => "HTTP Request",
            NodeKind::JsonGetField => "JSON Get Field",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            NodeKind::OnReady => "Inicio del Main: dispara la inicialización.",
            NodeKind::OnResume => "Se ejecuta cuando la app vuelve al frente.",
            NodeKind::OnPause => "Se ejecuta cuando la app pasa a segundo plano.",
            NodeKind::OnDestroy => "Se ejecuta antes de destruir la app.",
            NodeKind::OnBackPressed => "Se ejecuta al pulsar atrás.",
            NodeKind::SetInitialScreen => "Define la pantalla UI inicial.",
            NodeKind::NavigateToScreen => "Navega a una pantalla UI.",
            NodeKind::InitEngine => "Inicializa un contenedor Engine.",
            NodeKind::InitData => "Inicializa un contenedor Data.",
            NodeKind::InitNetwork => "Inicializa un contenedor Network.",
            NodeKind::CallModuleFunction => "Llama una función de otro contenedor.",
            NodeKind::RequestPermission => "Solicita un permiso Android.",
            NodeKind::CheckPermission => "Comprueba si un permiso está concedido.",
            NodeKind::SetStatusBar => "Configura color e iconos de status bar.",
            NodeKind::KeepScreenOn => "Mantiene la pantalla encendida.",
            NodeKind::ExitApp => "Cierra la aplicación.",
            NodeKind::OnDeepLink => "Trigger por deep link; entrega URL.",
            NodeKind::OpenExternalUrl => "Abre una URL externa.",
            NodeKind::ShareText => "Comparte texto por intent.",
            NodeKind::IfCondition => "Bifurca el flujo True/False.",
            NodeKind::DelayTimer => "Espera X ms y continúa.",
            NodeKind::LogDebug => "Escribe un log de depuración.",
            NodeKind::OnTap => "Evento al tocar un elemento UI.",
            NodeKind::SetVisible => "Muestra u oculta un elemento UI.",
            NodeKind::SetText => "Cambia el texto de un elemento UI.",
            NodeKind::NavigateTo => "Navega a otra pantalla UI.",
            NodeKind::CallFunction => "Llama una función de un módulo Engine.",
            NodeKind::DefineFunction => "Define una función reutilizable.",
            NodeKind::SetVariable => "Asigna un valor a una variable.",
            NodeKind::MathOp => "Realiza una operación matemática.",
            NodeKind::SaveLocal => "Guarda un valor local por clave.",
            NodeKind::LoadLocal => "Carga un valor local por clave.",
            NodeKind::DeleteLocal => "Elimina un valor local por clave.",
            NodeKind::HttpRequest => "Hace una petición HTTP y devuelve éxito/error.",
            NodeKind::JsonGetField => "Obtiene un campo desde un JSON.",
        }
    }

    pub fn category(self) -> Option<MainCategory> {
        match self {
            NodeKind::OnReady | NodeKind::OnResume | NodeKind::OnPause | NodeKind::OnDestroy | NodeKind::OnBackPressed => Some(MainCategory::Lifecycle),
            NodeKind::SetInitialScreen | NodeKind::NavigateToScreen | NodeKind::InitEngine | NodeKind::InitData | NodeKind::InitNetwork | NodeKind::CallModuleFunction => Some(MainCategory::Orchestration),
            NodeKind::RequestPermission | NodeKind::CheckPermission | NodeKind::SetStatusBar | NodeKind::KeepScreenOn | NodeKind::ExitApp => Some(MainCategory::Android),
            NodeKind::OnDeepLink | NodeKind::OpenExternalUrl | NodeKind::ShareText => Some(MainCategory::Intents),
            NodeKind::IfCondition | NodeKind::DelayTimer | NodeKind::LogDebug => Some(MainCategory::Flow),
            _ => None,
        }
    }

    pub fn allowed_for(kind: crate::project::FileGroupKind) -> &'static [NodeKind] {
        use crate::project::FileGroupKind as G;
        match kind {
            G::Main => &[
                NodeKind::OnReady, NodeKind::OnResume, NodeKind::OnPause, NodeKind::OnDestroy, NodeKind::OnBackPressed,
                NodeKind::SetInitialScreen, NodeKind::NavigateToScreen, NodeKind::InitEngine, NodeKind::InitData, NodeKind::InitNetwork, NodeKind::CallModuleFunction,
                NodeKind::RequestPermission, NodeKind::CheckPermission, NodeKind::SetStatusBar, NodeKind::KeepScreenOn, NodeKind::ExitApp,
                NodeKind::OnDeepLink, NodeKind::OpenExternalUrl, NodeKind::ShareText,
                NodeKind::IfCondition, NodeKind::DelayTimer, NodeKind::LogDebug,
            ],
            G::Ui => &[NodeKind::OnTap, NodeKind::SetVisible, NodeKind::SetText, NodeKind::NavigateTo, NodeKind::CallFunction],
            G::Engine => &[NodeKind::DefineFunction, NodeKind::SetVariable, NodeKind::MathOp, NodeKind::IfCondition, NodeKind::DelayTimer],
            G::Data => &[NodeKind::SaveLocal, NodeKind::LoadLocal, NodeKind::DeleteLocal],
            G::Network => &[NodeKind::HttpRequest, NodeKind::JsonGetField],
        }
    }

    pub fn inputs(self) -> &'static [&'static str] {
        match self {
            NodeKind::OnReady | NodeKind::OnResume | NodeKind::OnPause | NodeKind::OnDestroy | NodeKind::OnBackPressed | NodeKind::OnDeepLink | NodeKind::OnTap | NodeKind::DefineFunction => &[],
            NodeKind::IfCondition | NodeKind::CheckPermission | NodeKind::MathOp | NodeKind::JsonGetField => &["in"],
            _ => &["in"],
        }
    }

    pub fn outputs(self) -> &'static [&'static str] {
        match self {
            NodeKind::IfCondition | NodeKind::CheckPermission => &["True", "False"],
            NodeKind::RequestPermission => &["OnGranted", "OnDenied"],
            NodeKind::HttpRequest => &["OnSuccess", "OnError"],
            NodeKind::OnDeepLink => &["out", "url"],
            NodeKind::JsonGetField => &["value"],
            NodeKind::MathOp => &["value"],
            NodeKind::ExitApp => &[],
            _ => &["out"],
        }
    }

    pub fn port_kind(self, port: &str, is_output: bool) -> PortKind {
        let p = port.to_ascii_lowercase();
        if matches!(p.as_str(), "out" | "in" | "true" | "false" | "ongranted" | "ondenied" | "onsuccess" | "onerror") {
            return PortKind::Flow;
        }
        if is_output {
            match self {
                NodeKind::OnDeepLink if p == "url" => PortKind::String,
                NodeKind::MathOp | NodeKind::JsonGetField | NodeKind::LoadLocal => PortKind::Any,
                _ => PortKind::Flow,
            }
        } else {
            match self {
                NodeKind::MathOp => PortKind::Number,
                NodeKind::JsonGetField => PortKind::Json,
                NodeKind::OpenExternalUrl | NodeKind::ShareText | NodeKind::LogDebug => PortKind::Flow, // inputs are flow; props hold string values
                _ => PortKind::Flow,
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortKind {
    Flow,
    Number,
    Json,
    String,
    Any,
}

impl PortKind {
    pub fn compatible(self, other: PortKind) -> bool {
        self == other || self == PortKind::Any || other == PortKind::Any
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NodeProps {
    #[serde(default)] pub name: String,
    #[serde(default)] pub target: String,
    #[serde(default)] pub key: String,
    #[serde(default)] pub value: String,
    #[serde(default)] pub value_type: String, // text|number|bool
    #[serde(default)] pub op: String, // + - * /
    #[serde(default)] pub left: String,
    #[serde(default)] pub right: String,
    #[serde(default)] pub cond_op: String, // == != > < >= <=
    #[serde(default)] pub method: String, // GET|POST
    #[serde(default)] pub url: String,
    #[serde(default)] pub body: String,
    #[serde(default)] pub field: String,
    #[serde(default)] pub ms: i32,
    #[serde(default)] pub visible: bool,
    #[serde(default)] pub module: String,
    #[serde(default)] pub fn_name: String,
    #[serde(default)] pub permission: String,
    #[serde(default)] pub clear_history: bool,
    #[serde(default)] pub color: String,
    #[serde(default)] pub dark_icons: bool,
    #[serde(default)] pub enabled: bool,
    #[serde(default)] pub tag: String,
    #[serde(default)] pub message: String,
}

impl LogicGraph {
    pub fn next_id(&self) -> u64 {
        self.nodes.iter().map(|n| n.id).max().unwrap_or(0) + 1
    }

    pub fn has_path(&self, from: u64, to: u64) -> bool {
        if from == to { return true; }
        let mut stack = vec![from];
        let mut seen = std::collections::HashSet::new();
        while let Some(n) = stack.pop() {
            if !seen.insert(n) { continue; }
            for l in &self.links {
                if l.from_node == n {
                    if l.to_node == to { return true; }
                    stack.push(l.to_node);
                }
            }
        }
        false
    }

    pub fn can_connect(&self, from_node: u64, from_port: &str, to_node: u64, to_port: &str) -> Result<(), &'static str> {
        let Some(src) = self.nodes.iter().find(|n| n.id == from_node) else { return Err("Nodo inválido"); };
        let Some(dst) = self.nodes.iter().find(|n| n.id == to_node) else { return Err("Nodo inválido"); };
        if from_node == to_node { return Err("Mismo nodo"); }
        if !src.kind.outputs().iter().any(|p| *p == from_port) { return Err("Puerto inválido"); }
        if !dst.kind.inputs().iter().any(|p| *p == to_port) { return Err("Puerto inválido"); }
        let from_kind = src.kind.port_kind(from_port, true);
        let to_kind = dst.kind.port_kind(to_port, false);
        if !from_kind.compatible(to_kind) { return Err("Tipo inválido"); }
        if from_kind == PortKind::Flow && self.has_path(to_node, from_node) { return Err("Ciclo inválido"); }
        Ok(())
    }
}
