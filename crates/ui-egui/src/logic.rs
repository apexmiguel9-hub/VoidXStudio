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
    // Main
    OnAppStart,
    OpenScreen,
    InitModule,
    // UI
    OnTap,
    SetVisible,
    SetText,
    NavigateTo,
    CallFunction,
    // Engine
    DefineFunction,
    SetVariable,
    MathOp,
    IfCondition,
    TimerDelay,
    // Data
    SaveLocal,
    LoadLocal,
    DeleteLocal,
    // Network
    HttpRequest,
    JsonGetField,
}

impl NodeKind {
    pub fn label(self) -> &'static str {
        match self {
            NodeKind::OnAppStart => "On App Start",
            NodeKind::OpenScreen => "Open Screen",
            NodeKind::InitModule => "Init Module",
            NodeKind::OnTap => "On Tap",
            NodeKind::SetVisible => "Set Visible",
            NodeKind::SetText => "Set Text",
            NodeKind::NavigateTo => "Navigate To",
            NodeKind::CallFunction => "Call Function",
            NodeKind::DefineFunction => "Define Function",
            NodeKind::SetVariable => "Set Variable",
            NodeKind::MathOp => "Math Op",
            NodeKind::IfCondition => "If Condition",
            NodeKind::TimerDelay => "Timer Delay",
            NodeKind::SaveLocal => "Save Local",
            NodeKind::LoadLocal => "Load Local",
            NodeKind::DeleteLocal => "Delete Local",
            NodeKind::HttpRequest => "HTTP Request",
            NodeKind::JsonGetField => "JSON Get Field",
        }
    }

    pub fn allowed_for(kind: crate::project::FileGroupKind) -> &'static [NodeKind] {
        use crate::project::FileGroupKind as G;
        match kind {
            G::Main => &[NodeKind::OnAppStart, NodeKind::OpenScreen, NodeKind::InitModule],
            G::Ui => &[NodeKind::OnTap, NodeKind::SetVisible, NodeKind::SetText, NodeKind::NavigateTo, NodeKind::CallFunction],
            G::Engine => &[NodeKind::DefineFunction, NodeKind::SetVariable, NodeKind::MathOp, NodeKind::IfCondition, NodeKind::TimerDelay],
            G::Data => &[NodeKind::SaveLocal, NodeKind::LoadLocal, NodeKind::DeleteLocal],
            G::Network => &[NodeKind::HttpRequest, NodeKind::JsonGetField],
        }
    }

    pub fn inputs(self) -> &'static [&'static str] {
        match self {
            NodeKind::OnAppStart | NodeKind::OnTap | NodeKind::DefineFunction => &[],
            NodeKind::IfCondition => &["in"],
            NodeKind::MathOp => &["in"],
            NodeKind::JsonGetField => &["in"],
            _ => &["in"],
        }
    }

    pub fn outputs(self) -> &'static [&'static str] {
        match self {
            NodeKind::IfCondition => &["True", "False"],
            NodeKind::HttpRequest => &["OnSuccess", "OnError"],
            NodeKind::OnAppStart | NodeKind::OnTap | NodeKind::DefineFunction => &["out"],
            NodeKind::JsonGetField => &["value"],
            NodeKind::MathOp => &["value"],
            _ => &["out"],
        }
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
}

impl LogicGraph {
    pub fn next_id(&self) -> u64 {
        self.nodes.iter().map(|n| n.id).max().unwrap_or(0) + 1
    }
}
