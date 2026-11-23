//! Logic graph model: visual IR for project workspaces.
//! Serialized as `files/<name>.logic.json` inside .vxs.
//! IR is serde-pure and independent from egui/runtime backends.

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

pub type NodeWire = NodeLink;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    // MAIN - Lifecycle (Application Start keeps internal OnReady for compatibility)
    #[serde(alias = "on_app_start")]
    OnReady,
    ApplicationReady,
    OnApplicationResume,
    OnApplicationSuspend,
    OnApplicationShutdown,

    // MAIN - Containers
    InitializeContainer,
    CallContainer,
    CallContainerAsync,
    WaitForContainer,
    GetContainerStatus,
    SetContainerState,
    RestartContainer,

    // MAIN - Events
    EmitEvent,
    WaitForEvent,
    SubscribeToEvent,
    UnsubscribeFromEvent,
    ForwardEvent,
    FilterEvent,

    // MAIN - Flow Control
    Sequence,
    Branch,
    Switch,
    MergeFlow,
    Gate,
    Delay,
    RepeatFlow,

    // MAIN - Concurrency
    StartParallel,
    WaitForAll,
    WaitForAny,
    RaceExecution,
    LimitConcurrency,

    // MAIN - Error Recovery
    TryCatchFlow,
    RetryExecution,
    ExecutionTimeout,
    FallbackExecution,

    // MAIN - Execution Context
    SetLaunchContext,
    GetLaunchContext,
    ShareExecutionContext,

    // MAIN - Application Control
    ReturnResult,
    CompleteFlow,
    ApplicationControl,

    // Legacy MAIN nodes kept for project deserialization compatibility; not in MAIN catalog.
    #[serde(alias = "on_resume")]
    LegacyOnResume,
    #[serde(alias = "on_pause")]
    LegacyOnPause,
    #[serde(alias = "on_destroy")]
    LegacyOnDestroy,
    #[serde(alias = "on_back_pressed")]
    LegacyOnBackPressed,
    #[serde(alias = "open_screen", alias = "set_initial_screen")]
    LegacySetInitialScreen,
    #[serde(alias = "navigate_to_screen")]
    LegacyNavigateToScreen,
    #[serde(alias = "init_engine")]
    LegacyInitEngine,
    #[serde(alias = "init_data")]
    LegacyInitData,
    #[serde(alias = "init_network")]
    LegacyInitNetwork,
    #[serde(alias = "call_module_function")]
    LegacyCallModuleFunction,
    #[serde(alias = "request_permission")]
    LegacyRequestPermission,
    #[serde(alias = "check_permission")]
    LegacyCheckPermission,
    #[serde(alias = "set_status_bar")]
    LegacySetStatusBar,
    #[serde(alias = "keep_screen_on")]
    LegacyKeepScreenOn,
    #[serde(alias = "exit_app")]
    LegacyExitApp,
    #[serde(alias = "on_deep_link")]
    LegacyOnDeepLink,
    #[serde(alias = "open_external_url")]
    LegacyOpenExternalUrl,
    #[serde(alias = "share_text")]
    LegacyShareText,
    #[serde(alias = "legacy_if_condition")]
    LegacyIfCondition,
    LegacyDelayTimer,
    #[serde(alias = "log_debug")]
    LegacyLogDebug,

    // Existing non-MAIN nodes (kept; other containers unchanged by this task)
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MainCategory {
    Lifecycle,
    Containers,
    Events,
    FlowControl,
    Concurrency,
    ErrorRecovery,
    ExecutionContext,
    ApplicationControl,
    Legacy,
}

impl MainCategory {
    pub fn label(self) -> &'static str {
        match self {
            MainCategory::Lifecycle => "Lifecycle",
            MainCategory::Containers => "Containers",
            MainCategory::Events => "Events",
            MainCategory::FlowControl => "Flow Control",
            MainCategory::Concurrency => "Concurrency",
            MainCategory::ErrorRecovery => "Error Recovery",
            MainCategory::ExecutionContext => "Execution Context",
            MainCategory::ApplicationControl => "Application Control",
            MainCategory::Legacy => "Legacy",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeLevel {
    Basic,
    Intermediate,
    Advanced,
}

impl NodeLevel {
    pub fn label(self) -> &'static str {
        match self {
            NodeLevel::Basic => "Basic",
            NodeLevel::Intermediate => "Intermediate",
            NodeLevel::Advanced => "Advanced",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionStatus {
    Defined,
    PendingRuntime,
}

impl NodeKind {
    pub const MAIN_CATALOG: [NodeKind; 40] = [
        NodeKind::OnReady,
        NodeKind::ApplicationReady,
        NodeKind::OnApplicationResume,
        NodeKind::OnApplicationSuspend,
        NodeKind::OnApplicationShutdown,
        NodeKind::InitializeContainer,
        NodeKind::CallContainer,
        NodeKind::CallContainerAsync,
        NodeKind::WaitForContainer,
        NodeKind::GetContainerStatus,
        NodeKind::SetContainerState,
        NodeKind::RestartContainer,
        NodeKind::EmitEvent,
        NodeKind::WaitForEvent,
        NodeKind::SubscribeToEvent,
        NodeKind::UnsubscribeFromEvent,
        NodeKind::ForwardEvent,
        NodeKind::FilterEvent,
        NodeKind::Sequence,
        NodeKind::Branch,
        NodeKind::Switch,
        NodeKind::MergeFlow,
        NodeKind::Gate,
        NodeKind::Delay,
        NodeKind::RepeatFlow,
        NodeKind::StartParallel,
        NodeKind::WaitForAll,
        NodeKind::WaitForAny,
        NodeKind::RaceExecution,
        NodeKind::LimitConcurrency,
        NodeKind::TryCatchFlow,
        NodeKind::RetryExecution,
        NodeKind::ExecutionTimeout,
        NodeKind::FallbackExecution,
        NodeKind::SetLaunchContext,
        NodeKind::GetLaunchContext,
        NodeKind::ShareExecutionContext,
        NodeKind::ReturnResult,
        NodeKind::CompleteFlow,
        NodeKind::ApplicationControl,
    ];

    pub fn semantic_id(self) -> &'static str {
        match self {
            NodeKind::OnReady => "application.start",
            NodeKind::ApplicationReady => "application.ready",
            NodeKind::OnApplicationResume => "application.resume",
            NodeKind::OnApplicationSuspend => "application.suspend",
            NodeKind::OnApplicationShutdown => "application.shutdown",
            NodeKind::InitializeContainer => "container.initialize",
            NodeKind::CallContainer => "container.call",
            NodeKind::CallContainerAsync => "container.call_async",
            NodeKind::WaitForContainer => "container.wait",
            NodeKind::GetContainerStatus => "container.get_status",
            NodeKind::SetContainerState => "container.set_state",
            NodeKind::RestartContainer => "container.restart",
            NodeKind::EmitEvent => "event.emit",
            NodeKind::WaitForEvent => "event.wait",
            NodeKind::SubscribeToEvent => "event.subscribe",
            NodeKind::UnsubscribeFromEvent => "event.unsubscribe",
            NodeKind::ForwardEvent => "event.forward",
            NodeKind::FilterEvent => "event.filter",
            NodeKind::Sequence => "flow.sequence",
            NodeKind::Branch => "flow.branch",
            NodeKind::Switch => "flow.switch",
            NodeKind::MergeFlow => "flow.merge",
            NodeKind::Gate => "flow.gate",
            NodeKind::Delay => "flow.delay",
            NodeKind::RepeatFlow => "flow.repeat",
            NodeKind::StartParallel => "parallel.start",
            NodeKind::WaitForAll => "parallel.wait_all",
            NodeKind::WaitForAny => "parallel.wait_any",
            NodeKind::RaceExecution => "parallel.race",
            NodeKind::LimitConcurrency => "parallel.limit_concurrency",
            NodeKind::TryCatchFlow => "error.try_catch",
            NodeKind::RetryExecution => "error.retry",
            NodeKind::ExecutionTimeout => "error.timeout",
            NodeKind::FallbackExecution => "error.fallback",
            NodeKind::SetLaunchContext => "context.set_launch",
            NodeKind::GetLaunchContext => "context.get_launch",
            NodeKind::ShareExecutionContext => "context.share",
            NodeKind::ReturnResult => "flow.return_result",
            NodeKind::CompleteFlow => "flow.complete",
            NodeKind::ApplicationControl => "application.control",
            NodeKind::LegacyOnResume => "legacy.on_resume",
            NodeKind::LegacyOnPause => "legacy.on_pause",
            NodeKind::LegacyOnDestroy => "legacy.on_destroy",
            NodeKind::LegacyOnBackPressed => "legacy.on_back_pressed",
            NodeKind::LegacySetInitialScreen => "legacy.set_initial_screen",
            NodeKind::LegacyNavigateToScreen => "legacy.navigate_to_screen",
            NodeKind::LegacyInitEngine => "legacy.init_engine",
            NodeKind::LegacyInitData => "legacy.init_data",
            NodeKind::LegacyInitNetwork => "legacy.init_network",
            NodeKind::LegacyCallModuleFunction => "legacy.call_module_function",
            NodeKind::LegacyRequestPermission => "legacy.request_permission",
            NodeKind::LegacyCheckPermission => "legacy.check_permission",
            NodeKind::LegacySetStatusBar => "legacy.set_status_bar",
            NodeKind::LegacyKeepScreenOn => "legacy.keep_screen_on",
            NodeKind::LegacyExitApp => "legacy.exit_app",
            NodeKind::LegacyOnDeepLink => "legacy.on_deep_link",
            NodeKind::LegacyOpenExternalUrl => "legacy.open_external_url",
            NodeKind::LegacyShareText => "legacy.share_text",
            NodeKind::LegacyIfCondition => "legacy.if_condition",
            NodeKind::LegacyDelayTimer => "legacy.delay_timer",
            NodeKind::LegacyLogDebug => "legacy.log_debug",
            NodeKind::OnTap => "ui.on_tap",
            NodeKind::SetVisible => "ui.set_visible",
            NodeKind::SetText => "ui.set_text",
            NodeKind::NavigateTo => "ui.navigate_to",
            NodeKind::CallFunction => "ui.call_function",
            NodeKind::DefineFunction => "engine.define_function",
            NodeKind::SetVariable => "engine.set_variable",
            NodeKind::MathOp => "engine.math_op",
            NodeKind::IfCondition => "engine.if_condition",
            NodeKind::DelayTimer => "engine.delay_timer",
            NodeKind::SaveLocal => "data.save_local",
            NodeKind::LoadLocal => "data.load_local",
            NodeKind::DeleteLocal => "data.delete_local",
            NodeKind::HttpRequest => "network.http_request",
            NodeKind::JsonGetField => "network.json_get_field",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            NodeKind::OnReady => "Application Start",
            NodeKind::ApplicationReady => "Application Ready",
            NodeKind::OnApplicationResume => "On Application Resume",
            NodeKind::OnApplicationSuspend => "On Application Suspend",
            NodeKind::OnApplicationShutdown => "On Application Shutdown",
            NodeKind::InitializeContainer => "Initialize Container",
            NodeKind::CallContainer => "Call Container",
            NodeKind::CallContainerAsync => "Call Container Async",
            NodeKind::WaitForContainer => "Wait for Container",
            NodeKind::GetContainerStatus => "Get Container Status",
            NodeKind::SetContainerState => "Set Container State",
            NodeKind::RestartContainer => "Restart Container",
            NodeKind::EmitEvent => "Emit Event",
            NodeKind::WaitForEvent => "Wait for Event",
            NodeKind::SubscribeToEvent => "Subscribe to Event",
            NodeKind::UnsubscribeFromEvent => "Unsubscribe from Event",
            NodeKind::ForwardEvent => "Forward Event",
            NodeKind::FilterEvent => "Filter Event",
            NodeKind::Sequence => "Sequence",
            NodeKind::Branch => "Branch",
            NodeKind::Switch => "Switch",
            NodeKind::MergeFlow => "Merge Flow",
            NodeKind::Gate => "Gate",
            NodeKind::Delay => "Delay",
            NodeKind::RepeatFlow => "Repeat Flow",
            NodeKind::StartParallel => "Start Parallel",
            NodeKind::WaitForAll => "Wait for All",
            NodeKind::WaitForAny => "Wait for Any",
            NodeKind::RaceExecution => "Race Execution",
            NodeKind::LimitConcurrency => "Limit Concurrency",
            NodeKind::TryCatchFlow => "Try / Catch Flow",
            NodeKind::RetryExecution => "Retry Execution",
            NodeKind::ExecutionTimeout => "Execution Timeout",
            NodeKind::FallbackExecution => "Fallback Execution",
            NodeKind::SetLaunchContext => "Set Launch Context",
            NodeKind::GetLaunchContext => "Get Launch Context",
            NodeKind::ShareExecutionContext => "Share Execution Context",
            NodeKind::ReturnResult => "Return Result",
            NodeKind::CompleteFlow => "Complete Flow",
            NodeKind::ApplicationControl => "Application Control",
            NodeKind::LegacyOnResume => "Legacy: OnResume",
            NodeKind::LegacyOnPause => "Legacy: OnPause",
            NodeKind::LegacyOnDestroy => "Legacy: OnDestroy",
            NodeKind::LegacyOnBackPressed => "Legacy: OnBackPressed",
            NodeKind::LegacySetInitialScreen => "Legacy: SetInitialScreen",
            NodeKind::LegacyNavigateToScreen => "Legacy: NavigateToScreen",
            NodeKind::LegacyInitEngine => "Legacy: InitEngine",
            NodeKind::LegacyInitData => "Legacy: InitData",
            NodeKind::LegacyInitNetwork => "Legacy: InitNetwork",
            NodeKind::LegacyCallModuleFunction => "Legacy: CallModuleFunction",
            NodeKind::LegacyRequestPermission => "Legacy: RequestPermission",
            NodeKind::LegacyCheckPermission => "Legacy: CheckPermission",
            NodeKind::LegacySetStatusBar => "Legacy: SetStatusBar",
            NodeKind::LegacyKeepScreenOn => "Legacy: KeepScreenOn",
            NodeKind::LegacyExitApp => "Legacy: ExitApp",
            NodeKind::LegacyOnDeepLink => "Legacy: OnDeepLink",
            NodeKind::LegacyOpenExternalUrl => "Legacy: OpenExternalUrl",
            NodeKind::LegacyShareText => "Legacy: ShareText",
            NodeKind::LegacyIfCondition => "Legacy: IfCondition",
            NodeKind::LegacyDelayTimer => "Legacy: DelayTimer",
            NodeKind::LegacyLogDebug => "Legacy: LogDebug",
            NodeKind::OnTap => "On Tap",
            NodeKind::SetVisible => "Set Visible",
            NodeKind::SetText => "Set Text",
            NodeKind::NavigateTo => "Navigate To",
            NodeKind::CallFunction => "Call Function",
            NodeKind::DefineFunction => "Define Function",
            NodeKind::SetVariable => "Set Variable",
            NodeKind::MathOp => "Math Op",
            NodeKind::IfCondition => "If Condition",
            NodeKind::DelayTimer => "Delay Timer",
            NodeKind::SaveLocal => "Save Local",
            NodeKind::LoadLocal => "Load Local",
            NodeKind::DeleteLocal => "Delete Local",
            NodeKind::HttpRequest => "HTTP Request",
            NodeKind::JsonGetField => "JSON Get Field",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            NodeKind::OnReady => "Punto inicial de ejecución de la aplicación.",
            NodeKind::ApplicationReady => "Continúa cuando la app completó inicializaciones requeridas.",
            NodeKind::OnApplicationResume => "Se ejecuta cuando la app vuelve a primer plano.",
            NodeKind::OnApplicationSuspend => "Se ejecuta cuando la app pasa a segundo plano.",
            NodeKind::OnApplicationShutdown => "Coordina operaciones previas al cierre.",
            NodeKind::InitializeContainer => "Inicializa un contenedor del proyecto.",
            NodeKind::CallContainer => "Invoca una operación pública de otro contenedor.",
            NodeKind::CallContainerAsync => "Invoca una operación sin bloquear el flujo actual.",
            NodeKind::WaitForContainer => "Espera a que una operación o inicialización termine.",
            NodeKind::GetContainerStatus => "Consulta el estado operativo de un contenedor.",
            NodeKind::SetContainerState => "Habilita o deshabilita un contenedor permitido.",
            NodeKind::RestartContainer => "Reinicia un contenedor con control de runtime.",
            NodeKind::EmitEvent => "Publica un evento para otros componentes.",
            NodeKind::WaitForEvent => "Espera la recepción de un evento específico.",
            NodeKind::SubscribeToEvent => "Registra una suscripción persistente a un evento.",
            NodeKind::UnsubscribeFromEvent => "Elimina una suscripción existente.",
            NodeKind::ForwardEvent => "Reenvía un evento recibido a otro destino.",
            NodeKind::FilterEvent => "Filtra eventos según condiciones.",
            NodeKind::Sequence => "Ejecuta operaciones en orden establecido.",
            NodeKind::Branch => "Divide el flujo según una condición.",
            NodeKind::Switch => "Selecciona una ruta entre múltiples casos.",
            NodeKind::MergeFlow => "Reúne varias rutas de ejecución.",
            NodeKind::Gate => "Permite o bloquea una ruta de ejecución.",
            NodeKind::Delay => "Espera antes de continuar la ejecución.",
            NodeKind::RepeatFlow => "Repite una secuencia según política configurada.",
            NodeKind::StartParallel => "Inicia múltiples operaciones simultáneas.",
            NodeKind::WaitForAll => "Espera a que todas las operaciones terminen.",
            NodeKind::WaitForAny => "Continúa cuando termine cualquiera de las operaciones.",
            NodeKind::RaceExecution => "Selecciona el primer resultado válido entre tareas.",
            NodeKind::LimitConcurrency => "Limita la cantidad de operaciones simultáneas.",
            NodeKind::TryCatchFlow => "Ejecuta una secuencia controlando errores.",
            NodeKind::RetryExecution => "Reintenta una operación fallida.",
            NodeKind::ExecutionTimeout => "Establece un tiempo máximo para una operación.",
            NodeKind::FallbackExecution => "Ejecuta una alternativa si la primaria falla.",
            NodeKind::SetLaunchContext => "Establece información inicial de ejecución.",
            NodeKind::GetLaunchContext => "Recupera información de inicio.",
            NodeKind::ShareExecutionContext => "Comparte contexto con destinos autorizados.",
            NodeKind::ReturnResult => "Devuelve un resultado al flujo invocador.",
            NodeKind::CompleteFlow => "Marca la ejecución actual como completada.",
            NodeKind::ApplicationControl => "Solicita una acción global permitida por la app.",
            _ if self.is_legacy() => "Nodo legado conservado por compatibilidad; no disponible en catálogo MAIN.",
            NodeKind::OnTap => "Evento al tocar un elemento UI.",
            NodeKind::SetVisible => "Muestra u oculta un elemento UI.",
            NodeKind::SetText => "Cambia el texto de un elemento UI.",
            NodeKind::NavigateTo => "Navega a otra pantalla UI.",
            NodeKind::CallFunction => "Llama una función de Engine.",
            NodeKind::DefineFunction => "Define una función reutilizable.",
            NodeKind::SetVariable => "Asigna un valor a una variable.",
            NodeKind::MathOp => "Realiza una operación matemática.",
            NodeKind::IfCondition => "Bifurca el flujo True/False.",
            NodeKind::DelayTimer => "Espera X ms y continúa.",
            NodeKind::SaveLocal => "Guarda un valor local por clave.",
            NodeKind::LoadLocal => "Carga un valor local por clave.",
            NodeKind::DeleteLocal => "Elimina un valor local por clave.",
            NodeKind::HttpRequest => "Hace una petición HTTP.",
            NodeKind::JsonGetField => "Obtiene un campo desde JSON.",
            _ => "",
        }
    }

    pub fn category(self) -> Option<MainCategory> {
        match self {
            NodeKind::OnReady | NodeKind::ApplicationReady | NodeKind::OnApplicationResume | NodeKind::OnApplicationSuspend | NodeKind::OnApplicationShutdown => Some(MainCategory::Lifecycle),
            NodeKind::InitializeContainer | NodeKind::CallContainer | NodeKind::CallContainerAsync | NodeKind::WaitForContainer | NodeKind::GetContainerStatus | NodeKind::SetContainerState | NodeKind::RestartContainer => Some(MainCategory::Containers),
            NodeKind::EmitEvent | NodeKind::WaitForEvent | NodeKind::SubscribeToEvent | NodeKind::UnsubscribeFromEvent | NodeKind::ForwardEvent | NodeKind::FilterEvent => Some(MainCategory::Events),
            NodeKind::Sequence | NodeKind::Branch | NodeKind::Switch | NodeKind::MergeFlow | NodeKind::Gate | NodeKind::Delay | NodeKind::RepeatFlow => Some(MainCategory::FlowControl),
            NodeKind::StartParallel | NodeKind::WaitForAll | NodeKind::WaitForAny | NodeKind::RaceExecution | NodeKind::LimitConcurrency => Some(MainCategory::Concurrency),
            NodeKind::TryCatchFlow | NodeKind::RetryExecution | NodeKind::ExecutionTimeout | NodeKind::FallbackExecution => Some(MainCategory::ErrorRecovery),
            NodeKind::SetLaunchContext | NodeKind::GetLaunchContext | NodeKind::ShareExecutionContext => Some(MainCategory::ExecutionContext),
            NodeKind::ReturnResult | NodeKind::CompleteFlow | NodeKind::ApplicationControl => Some(MainCategory::ApplicationControl),
            _ if self.is_legacy() => Some(MainCategory::Legacy),
            _ => None,
        }
    }

    pub fn level(self) -> NodeLevel {
        match self {
            NodeKind::OnReady | NodeKind::InitializeContainer | NodeKind::CallContainer | NodeKind::GetContainerStatus | NodeKind::EmitEvent | NodeKind::Sequence | NodeKind::Branch | NodeKind::Delay | NodeKind::GetLaunchContext | NodeKind::CompleteFlow => NodeLevel::Basic,
            NodeKind::ApplicationReady | NodeKind::OnApplicationResume | NodeKind::OnApplicationSuspend | NodeKind::CallContainerAsync | NodeKind::WaitForContainer | NodeKind::SetContainerState | NodeKind::WaitForEvent | NodeKind::SubscribeToEvent | NodeKind::UnsubscribeFromEvent | NodeKind::FilterEvent | NodeKind::Switch | NodeKind::Gate | NodeKind::RepeatFlow | NodeKind::TryCatchFlow | NodeKind::SetLaunchContext | NodeKind::ReturnResult => NodeLevel::Intermediate,
            _ => NodeLevel::Advanced,
        }
    }

    pub fn execution_status(self) -> ExecutionStatus {
        ExecutionStatus::PendingRuntime
    }

    pub fn is_legacy(self) -> bool {
        matches!(self,
            NodeKind::LegacyOnResume | NodeKind::LegacyOnPause | NodeKind::LegacyOnDestroy | NodeKind::LegacyOnBackPressed |
            NodeKind::LegacySetInitialScreen | NodeKind::LegacyNavigateToScreen | NodeKind::LegacyInitEngine | NodeKind::LegacyInitData | NodeKind::LegacyInitNetwork | NodeKind::LegacyCallModuleFunction |
            NodeKind::LegacyRequestPermission | NodeKind::LegacyCheckPermission | NodeKind::LegacySetStatusBar | NodeKind::LegacyKeepScreenOn | NodeKind::LegacyExitApp |
            NodeKind::LegacyOnDeepLink | NodeKind::LegacyOpenExternalUrl | NodeKind::LegacyShareText |
            NodeKind::LegacyIfCondition | NodeKind::LegacyDelayTimer | NodeKind::LegacyLogDebug
        )
    }

    pub fn is_main_catalog(self) -> bool {
        Self::MAIN_CATALOG.contains(&self)
    }

    pub fn allowed_for(kind: crate::project::FileGroupKind) -> &'static [NodeKind] {
        use crate::project::FileGroupKind as G;
        match kind {
            G::Main => &Self::MAIN_CATALOG,
            G::Ui => &[NodeKind::OnTap, NodeKind::SetVisible, NodeKind::SetText, NodeKind::NavigateTo, NodeKind::CallFunction],
            G::Engine => &[NodeKind::DefineFunction, NodeKind::SetVariable, NodeKind::MathOp, NodeKind::IfCondition, NodeKind::DelayTimer],
            G::Data => &[NodeKind::SaveLocal, NodeKind::LoadLocal, NodeKind::DeleteLocal],
            G::Network => &[NodeKind::HttpRequest, NodeKind::JsonGetField],
        }
    }

    pub fn inputs(self) -> &'static [PortDef] {
        match self {
            NodeKind::OnReady => &[],
            NodeKind::ApplicationReady => &[P_EXEC_INIT, P_DATA_REQ],
            NodeKind::OnApplicationResume | NodeKind::OnApplicationSuspend | NodeKind::OnApplicationShutdown => &[],
            NodeKind::InitializeContainer => &[P_EXEC_IN, P_REF_CONTAINER],
            NodeKind::CallContainer => &[P_EXEC_IN, P_REF_TARGET_CONTAINER, P_REF_TARGET_OPERATION, P_DATA_PARAMS],
            NodeKind::CallContainerAsync => &[P_EXEC_IN, P_REF_TARGET_CONTAINER, P_REF_TARGET_OPERATION, P_DATA_PARAMS],
            NodeKind::WaitForContainer => &[P_EXEC_WAIT, P_REF_CONTAINER, P_DATA_EXEC_HANDLE_OPT],
            NodeKind::GetContainerStatus => &[P_REF_CONTAINER],
            NodeKind::SetContainerState => &[P_EXEC_IN, P_REF_CONTAINER, P_DATA_ENABLED],
            NodeKind::RestartContainer => &[P_EXEC_IN, P_REF_CONTAINER],
            NodeKind::EmitEvent => &[P_EXEC_IN, P_DATA_EVENT_NAME, P_DATA_PAYLOAD_OPT],
            NodeKind::WaitForEvent => &[P_EXEC_IN, P_DATA_EVENT_NAME, P_DATA_TIMEOUT_OPT],
            NodeKind::SubscribeToEvent => &[P_EXEC_IN, P_DATA_EVENT_NAME],
            NodeKind::UnsubscribeFromEvent => &[P_EXEC_IN, P_DATA_SUBSCRIPTION_ID],
            NodeKind::ForwardEvent => &[P_EXEC_IN, P_DATA_EVENT_REF, P_REF_TARGET],
            NodeKind::FilterEvent => &[P_EXEC_IN, P_DATA_EVENT_PAYLOAD, P_DATA_CONDITIONS],
            NodeKind::Sequence => &[P_EXEC_IN],
            NodeKind::Branch => &[P_EXEC_IN, P_DATA_CONDITION],
            NodeKind::Switch => &[P_EXEC_IN, P_DATA_VALUE, P_DATA_CASES],
            NodeKind::MergeFlow => &[P_EXEC_A, P_EXEC_B],
            NodeKind::Gate => &[P_EXEC_IN, P_DATA_ENABLED],
            NodeKind::Delay => &[P_EXEC_IN, P_DATA_DURATION],
            NodeKind::RepeatFlow => &[P_EXEC_IN, P_DATA_ITERATIONS_OPT, P_DATA_CONDITION_OPT],
            NodeKind::StartParallel => &[P_EXEC_IN, P_DATA_TASKS],
            NodeKind::WaitForAll => &[P_EXEC_IN, P_DATA_EXEC_GROUP],
            NodeKind::WaitForAny => &[P_EXEC_IN, P_DATA_EXEC_GROUP],
            NodeKind::RaceExecution => &[P_EXEC_IN, P_DATA_TASKS],
            NodeKind::LimitConcurrency => &[P_EXEC_IN, P_DATA_MAX_CONCURRENT, P_REF_TASK],
            NodeKind::TryCatchFlow => &[P_EXEC_IN, P_REF_OPERATION],
            NodeKind::RetryExecution => &[P_EXEC_IN, P_REF_OPERATION],
            NodeKind::ExecutionTimeout => &[P_EXEC_IN, P_REF_OPERATION, P_DATA_MAX_DURATION],
            NodeKind::FallbackExecution => &[P_EXEC_IN, P_REF_PRIMARY, P_REF_FALLBACK],
            NodeKind::SetLaunchContext => &[P_EXEC_IN, P_DATA_KEY, P_DATA_VALUE],
            NodeKind::GetLaunchContext => &[P_DATA_KEY],
            NodeKind::ShareExecutionContext => &[P_EXEC_IN, P_DATA_CONTEXT, P_REF_TARGET],
            NodeKind::ReturnResult => &[P_EXEC_IN, P_DATA_RESULT],
            NodeKind::CompleteFlow => &[P_EXEC_IN, P_DATA_RESULT_OPT],
            NodeKind::ApplicationControl => &[P_EXEC_IN, P_DATA_ACTION],

            NodeKind::OnTap | NodeKind::DefineFunction => &[],
            NodeKind::SetVisible => &[P_EXEC_IN, P_REF_TARGET, P_DATA_ENABLED],
            NodeKind::SetText => &[P_EXEC_IN, P_REF_TARGET, P_DATA_VALUE],
            NodeKind::NavigateTo => &[P_EXEC_IN, P_REF_TARGET],
            NodeKind::CallFunction => &[P_EXEC_IN, P_DATA_MODULE, P_DATA_FN, P_DATA_PARAMS],
            NodeKind::SetVariable => &[P_EXEC_IN, P_DATA_KEY, P_DATA_VALUE],
            NodeKind::MathOp => &[P_DATA_LEFT, P_DATA_RIGHT],
            NodeKind::IfCondition => &[P_EXEC_IN, P_DATA_CONDITION],
            NodeKind::DelayTimer => &[P_EXEC_IN, P_DATA_DURATION],
            NodeKind::SaveLocal => &[P_EXEC_IN, P_DATA_KEY, P_DATA_VALUE],
            NodeKind::LoadLocal => &[P_EXEC_IN, P_DATA_KEY],
            NodeKind::DeleteLocal => &[P_EXEC_IN, P_DATA_KEY],
            NodeKind::HttpRequest => &[P_EXEC_IN, P_DATA_URL, P_DATA_BODY],
            NodeKind::JsonGetField => &[P_EXEC_IN, P_DATA_PAYLOAD, P_DATA_FIELD],

            _ if self.is_legacy() => LEGACY_INPUTS,
            _ => &[P_EXEC_IN],
        }
    }

    pub fn outputs(self) -> &'static [PortDef] {
        match self {
            NodeKind::OnReady => &[P_EXEC_ON_START],
            NodeKind::ApplicationReady => &[P_EXEC_READY, P_EXEC_FAILED],
            NodeKind::OnApplicationResume => &[P_EXEC_ON_RESUME],
            NodeKind::OnApplicationSuspend => &[P_EXEC_ON_SUSPEND],
            NodeKind::OnApplicationShutdown => &[P_EXEC_ON_SHUTDOWN],
            NodeKind::InitializeContainer => &[P_EXEC_COMPLETED, P_EXEC_FAILED, P_DATA_RESULT_OUT],
            NodeKind::CallContainer => &[P_EXEC_COMPLETED, P_EXEC_FAILED, P_DATA_RESULT_OUT],
            NodeKind::CallContainerAsync => &[P_EXEC_STARTED, P_DATA_EXEC_HANDLE, P_EXEC_FAILED],
            NodeKind::WaitForContainer => &[P_EXEC_COMPLETED, P_EXEC_FAILED, P_DATA_RESULT_OUT],
            NodeKind::GetContainerStatus => &[P_DATA_STATUS, P_DATA_IS_READY, P_DATA_IS_RUNNING],
            NodeKind::SetContainerState => &[P_EXEC_COMPLETED, P_EXEC_FAILED],
            NodeKind::RestartContainer => &[P_EXEC_COMPLETED, P_EXEC_FAILED],
            NodeKind::EmitEvent => &[P_EXEC_EMITTED, P_EXEC_FAILED],
            NodeKind::WaitForEvent => &[P_EXEC_RECEIVED, P_EXEC_TIMEOUT, P_DATA_PAYLOAD_OUT],
            NodeKind::SubscribeToEvent => &[P_EXEC_ON_EVENT, P_DATA_PAYLOAD_OUT, P_DATA_SUBSCRIPTION_ID_OUT, P_EXEC_FAILED],
            NodeKind::UnsubscribeFromEvent => &[P_EXEC_COMPLETED, P_EXEC_FAILED],
            NodeKind::ForwardEvent => &[P_EXEC_FORWARDED, P_EXEC_FAILED],
            NodeKind::FilterEvent => &[P_EXEC_ACCEPTED, P_EXEC_REJECTED, P_DATA_PAYLOAD_OUT],
            NodeKind::Sequence => &[P_EXEC_STEP, P_EXEC_COMPLETED, P_EXEC_FAILED],
            NodeKind::Branch => &[P_EXEC_TRUE, P_EXEC_FALSE],
            NodeKind::Switch => &[P_EXEC_CASE_A, P_EXEC_CASE_B, P_EXEC_DEFAULT],
            NodeKind::MergeFlow => &[P_EXEC_MERGED],
            NodeKind::Gate => &[P_EXEC_ALLOWED, P_EXEC_BLOCKED],
            NodeKind::Delay => &[P_EXEC_COMPLETED],
            NodeKind::RepeatFlow => &[P_EXEC_LOOP_BODY, P_EXEC_COMPLETED, P_EXEC_FAILED],
            NodeKind::StartParallel => &[P_EXEC_PARALLEL_STARTED, P_DATA_EXEC_GROUP_OUT, P_EXEC_FAILED],
            NodeKind::WaitForAll => &[P_EXEC_ALL_COMPLETED, P_EXEC_FAILED, P_DATA_RESULTS],
            NodeKind::WaitForAny => &[P_EXEC_COMPLETED, P_DATA_RESULT_OUT, P_DATA_EXEC_ID],
            NodeKind::RaceExecution => &[P_EXEC_WINNER, P_DATA_RESULT_OUT, P_EXEC_FAILED],
            NodeKind::LimitConcurrency => &[P_EXEC_COMPLETED, P_EXEC_FAILED],
            NodeKind::TryCatchFlow => &[P_EXEC_SUCCESS, P_EXEC_CATCH, P_DATA_ERROR],
            NodeKind::RetryExecution => &[P_EXEC_SUCCESS, P_EXEC_FAILED, P_DATA_ATTEMPTS],
            NodeKind::ExecutionTimeout => &[P_EXEC_COMPLETED, P_EXEC_TIMEOUT, P_EXEC_FAILED],
            NodeKind::FallbackExecution => &[P_EXEC_PRIMARY_SUCCESS, P_EXEC_FALLBACK_SUCCESS, P_EXEC_FAILED, P_DATA_RESULT_OUT],
            NodeKind::SetLaunchContext => &[P_EXEC_COMPLETED, P_EXEC_FAILED],
            NodeKind::GetLaunchContext => &[P_DATA_VALUE_OUT, P_DATA_EXISTS],
            NodeKind::ShareExecutionContext => &[P_EXEC_COMPLETED, P_EXEC_FAILED],
            NodeKind::ReturnResult => &[P_EXEC_RETURNED],
            NodeKind::CompleteFlow => &[P_EXEC_COMPLETED],
            NodeKind::ApplicationControl => &[P_EXEC_COMPLETED, P_EXEC_FAILED],

            NodeKind::OnTap | NodeKind::DefineFunction => &[P_EXEC_OUT],
            NodeKind::SetVisible | NodeKind::SetText | NodeKind::NavigateTo | NodeKind::CallFunction | NodeKind::SetVariable | NodeKind::SaveLocal | NodeKind::DeleteLocal => &[P_EXEC_OUT],
            NodeKind::LoadLocal => &[P_EXEC_OUT, P_DATA_VALUE_OUT],
            NodeKind::MathOp => &[P_DATA_VALUE_OUT],
            NodeKind::IfCondition => &[P_EXEC_TRUE, P_EXEC_FALSE],
            NodeKind::DelayTimer => &[P_EXEC_OUT],
            NodeKind::HttpRequest => &[P_EXEC_ON_SUCCESS, P_EXEC_ON_ERROR, P_DATA_RESULT_OUT],
            NodeKind::JsonGetField => &[P_DATA_VALUE_OUT],
            _ if self.is_legacy() => LEGACY_OUTPUTS,
            _ => &[P_EXEC_OUT],
        }
    }

    pub fn input_port(self, name: &str) -> Option<&'static PortDef> { self.inputs().iter().find(|p| p.name == name) }
    pub fn output_port(self, name: &str) -> Option<&'static PortDef> { self.outputs().iter().find(|p| p.name == name) }

    pub fn port_kind(self, port: &str, is_output: bool) -> PortKind {
        let def = if is_output { self.output_port(port) } else { self.input_port(port) };
        def.map(|d| d.kind).unwrap_or(PortKind::Flow)
    }

    pub fn prop_defs(self) -> &'static [PropDef] {
        match self {
            NodeKind::ApplicationReady => &[PROP_REQUIRED_CONDITIONS],
            NodeKind::InitializeContainer | NodeKind::GetContainerStatus | NodeKind::RestartContainer => &[PROP_CONTAINER_REF],
            NodeKind::CallContainer | NodeKind::CallContainerAsync => &[PROP_TARGET_CONTAINER, PROP_TARGET_OPERATION, PROP_PARAMETERS],
            NodeKind::WaitForContainer => &[PROP_CONTAINER_REF, PROP_EXEC_HANDLE],
            NodeKind::SetContainerState => &[PROP_CONTAINER_REF, PROP_ENABLED],
            NodeKind::EmitEvent => &[PROP_EVENT_NAME, PROP_PAYLOAD],
            NodeKind::WaitForEvent => &[PROP_EVENT_NAME, PROP_TIMEOUT_MS],
            NodeKind::SubscribeToEvent => &[PROP_EVENT_NAME],
            NodeKind::UnsubscribeFromEvent => &[PROP_SUBSCRIPTION_ID],
            NodeKind::ForwardEvent => &[PROP_TARGET_REF],
            NodeKind::FilterEvent => &[PROP_CONDITIONS],
            NodeKind::Sequence => &[PROP_SEQUENCE_STEPS],
            NodeKind::Branch => &[PROP_CONDITION_EXPR],
            NodeKind::Switch => &[PROP_SWITCH_CASES],
            NodeKind::MergeFlow => &[PROP_MERGE_POLICY],
            NodeKind::Gate => &[PROP_ENABLED],
            NodeKind::Delay => &[PROP_DURATION_MS],
            NodeKind::RepeatFlow => &[PROP_REPEAT_MODE, PROP_ITERATIONS, PROP_CONDITION_EXPR],
            NodeKind::StartParallel => &[PROP_TASKS],
            NodeKind::WaitForAll | NodeKind::WaitForAny => &[PROP_EXEC_GROUP],
            NodeKind::RaceExecution => &[PROP_TASKS, PROP_RACE_POLICY],
            NodeKind::LimitConcurrency => &[PROP_MAX_CONCURRENT],
            NodeKind::TryCatchFlow => &[PROP_OPERATION_REF],
            NodeKind::RetryExecution => &[PROP_OPERATION_REF, PROP_MAX_ATTEMPTS, PROP_RETRY_DELAY_MS],
            NodeKind::ExecutionTimeout => &[PROP_OPERATION_REF, PROP_TIMEOUT_MS],
            NodeKind::FallbackExecution => &[PROP_PRIMARY_REF, PROP_FALLBACK_REF],
            NodeKind::SetLaunchContext => &[PROP_KEY, PROP_VALUE],
            NodeKind::GetLaunchContext => &[PROP_KEY],
            NodeKind::ShareExecutionContext => &[PROP_TARGET_REF],
            NodeKind::ReturnResult => &[PROP_RESULT_VALUE],
            NodeKind::CompleteFlow => &[PROP_RESULT_VALUE],
            NodeKind::ApplicationControl => &[PROP_ACTION],
            NodeKind::SetText => &[PROP_TARGET_REF, PROP_VALUE],
            NodeKind::SetVisible => &[PROP_TARGET_REF, PROP_ENABLED],
            NodeKind::NavigateTo => &[PROP_TARGET_REF],
            NodeKind::CallFunction => &[PROP_MODULE, PROP_FUNCTION, PROP_PARAMETERS],
            NodeKind::DefineFunction => &[PROP_NAME],
            NodeKind::SetVariable => &[PROP_NAME, PROP_VALUE_TYPE, PROP_VALUE],
            NodeKind::MathOp => &[PROP_LEFT, PROP_OP, PROP_RIGHT],
            NodeKind::IfCondition => &[PROP_LEFT, PROP_COND_OP, PROP_RIGHT],
            NodeKind::DelayTimer => &[PROP_DURATION_MS],
            NodeKind::SaveLocal | NodeKind::LoadLocal | NodeKind::DeleteLocal => &[PROP_KEY, PROP_VALUE],
            NodeKind::HttpRequest => &[PROP_METHOD, PROP_URL, PROP_BODY],
            NodeKind::JsonGetField => &[PROP_FIELD],
            _ if self.is_legacy() => LEGACY_PROPS,
            _ => &[],
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortKind {
    Flow,
    Number,
    Json,
    String,
    Bool,
    Any,
    Reference,
    Event,
    ExecutionHandle,
    Status,
    Result,
    Context,
    Condition,
}

impl PortKind {
    pub fn label(self) -> &'static str {
        match self {
            PortKind::Flow => "flow",
            PortKind::Number => "number",
            PortKind::Json => "json",
            PortKind::String => "string",
            PortKind::Bool => "bool",
            PortKind::Any => "any",
            PortKind::Reference => "reference",
            PortKind::Event => "event",
            PortKind::ExecutionHandle => "execution_handle",
            PortKind::Status => "status",
            PortKind::Result => "result",
            PortKind::Context => "context",
            PortKind::Condition => "condition",
        }
    }

    pub fn compatible(self, other: PortKind) -> bool {
        if self == other { return true; }
        if self == PortKind::Any || other == PortKind::Any { return true; }
        // Execution/data interoperability kept permissive for MVP IR evolution.
        matches!((self, other),
            (PortKind::Reference, PortKind::String) | (PortKind::String, PortKind::Reference) |
            (PortKind::Result, PortKind::Json) | (PortKind::Json, PortKind::Result) |
            (PortKind::Context, PortKind::Json) | (PortKind::Json, PortKind::Context) |
            (PortKind::Event, PortKind::Json) | (PortKind::Json, PortKind::Event) |
            (PortKind::ExecutionHandle, PortKind::String) | (PortKind::String, PortKind::ExecutionHandle) |
            (PortKind::Condition, PortKind::Bool) | (PortKind::Bool, PortKind::Condition) |
            (PortKind::Status, PortKind::String) | (PortKind::String, PortKind::Status)
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortDir { In, Out }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PortDef {
    pub name: &'static str,
    pub kind: PortKind,
    pub dir: PortDir,
}

impl PortDef {
    pub const fn exec_in(name: &'static str) -> Self { Self { name, kind: PortKind::Flow, dir: PortDir::In } }
    pub const fn exec_out(name: &'static str) -> Self { Self { name, kind: PortKind::Flow, dir: PortDir::Out } }
    pub const fn data_in(name: &'static str, kind: PortKind) -> Self { Self { name, kind, dir: PortDir::In } }
    pub const fn data_out(name: &'static str, kind: PortKind) -> Self { Self { name, kind, dir: PortDir::Out } }
}

const P_EXEC_IN: PortDef = PortDef::exec_in("execute");
const P_EXEC_INIT: PortDef = PortDef::exec_in("initialization");
const P_EXEC_WAIT: PortDef = PortDef::exec_in("wait");
const P_EXEC_A: PortDef = PortDef::exec_in("a");
const P_EXEC_B: PortDef = PortDef::exec_in("b");
const P_EXEC_ON_START: PortDef = PortDef::exec_out("on_start");
const P_EXEC_READY: PortDef = PortDef::exec_out("ready");
const P_EXEC_FAILED: PortDef = PortDef::exec_out("failed");
const P_EXEC_COMPLETED: PortDef = PortDef::exec_out("completed");
const P_EXEC_STARTED: PortDef = PortDef::exec_out("started");
const P_EXEC_ON_RESUME: PortDef = PortDef::exec_out("on_resume");
const P_EXEC_ON_SUSPEND: PortDef = PortDef::exec_out("on_suspend");
const P_EXEC_ON_SHUTDOWN: PortDef = PortDef::exec_out("on_shutdown");
const P_EXEC_EMITTED: PortDef = PortDef::exec_out("emitted");
const P_EXEC_RECEIVED: PortDef = PortDef::exec_out("received");
const P_EXEC_TIMEOUT: PortDef = PortDef::exec_out("timeout");
const P_EXEC_ON_EVENT: PortDef = PortDef::exec_out("on_event");
const P_EXEC_FORWARDED: PortDef = PortDef::exec_out("forwarded");
const P_EXEC_ACCEPTED: PortDef = PortDef::exec_out("accepted");
const P_EXEC_REJECTED: PortDef = PortDef::exec_out("rejected");
const P_EXEC_STEP: PortDef = PortDef::exec_out("step_completed");
const P_EXEC_TRUE: PortDef = PortDef::exec_out("true");
const P_EXEC_FALSE: PortDef = PortDef::exec_out("false");
const P_EXEC_CASE_A: PortDef = PortDef::exec_out("case_a");
const P_EXEC_CASE_B: PortDef = PortDef::exec_out("case_b");
const P_EXEC_DEFAULT: PortDef = PortDef::exec_out("default");
const P_EXEC_MERGED: PortDef = PortDef::exec_out("merged");
const P_EXEC_ALLOWED: PortDef = PortDef::exec_out("allowed");
const P_EXEC_BLOCKED: PortDef = PortDef::exec_out("blocked");
const P_EXEC_LOOP_BODY: PortDef = PortDef::exec_out("loop_body");
const P_EXEC_PARALLEL_STARTED: PortDef = PortDef::exec_out("parallel_started");
const P_EXEC_ALL_COMPLETED: PortDef = PortDef::exec_out("all_completed");
const P_EXEC_WINNER: PortDef = PortDef::exec_out("winner");
const P_EXEC_SUCCESS: PortDef = PortDef::exec_out("success");
const P_EXEC_CATCH: PortDef = PortDef::exec_out("catch");
const P_EXEC_PRIMARY_SUCCESS: PortDef = PortDef::exec_out("primary_success");
const P_EXEC_FALLBACK_SUCCESS: PortDef = PortDef::exec_out("fallback_success");
const P_EXEC_RETURNED: PortDef = PortDef::exec_out("returned");
const P_EXEC_OUT: PortDef = PortDef::exec_out("out");
const P_EXEC_ON_SUCCESS: PortDef = PortDef::exec_out("on_success");
const P_EXEC_ON_ERROR: PortDef = PortDef::exec_out("on_error");

const P_REF_CONTAINER: PortDef = PortDef::data_in("container", PortKind::Reference);
const P_REF_TARGET_CONTAINER: PortDef = PortDef::data_in("target_container", PortKind::Reference);
const P_REF_TARGET_OPERATION: PortDef = PortDef::data_in("target_operation", PortKind::Reference);
const P_REF_TARGET: PortDef = PortDef::data_in("target", PortKind::Reference);
const P_REF_TASK: PortDef = PortDef::data_in("task", PortKind::Reference);
const P_REF_OPERATION: PortDef = PortDef::data_in("operation", PortKind::Reference);
const P_REF_PRIMARY: PortDef = PortDef::data_in("primary_operation", PortKind::Reference);
const P_REF_FALLBACK: PortDef = PortDef::data_in("fallback_operation", PortKind::Reference);

const P_DATA_PARAMS: PortDef = PortDef::data_in("parameters", PortKind::Json);
const P_DATA_REQ: PortDef = PortDef::data_in("required_conditions", PortKind::Json);
const P_DATA_EXEC_HANDLE_OPT: PortDef = PortDef::data_in("execution_handle", PortKind::ExecutionHandle);
const P_DATA_ENABLED: PortDef = PortDef::data_in("enabled", PortKind::Bool);
const P_DATA_EVENT_NAME: PortDef = PortDef::data_in("event_name", PortKind::String);
const P_DATA_PAYLOAD_OPT: PortDef = PortDef::data_in("payload", PortKind::Json);
const P_DATA_TIMEOUT_OPT: PortDef = PortDef::data_in("timeout", PortKind::Number);
const P_DATA_SUBSCRIPTION_ID: PortDef = PortDef::data_in("subscription_id", PortKind::String);
const P_DATA_EVENT_REF: PortDef = PortDef::data_in("event", PortKind::Event);
const P_DATA_EVENT_PAYLOAD: PortDef = PortDef::data_in("event_payload", PortKind::Json);
const P_DATA_CONDITIONS: PortDef = PortDef::data_in("conditions", PortKind::Condition);
const P_DATA_CONDITION: PortDef = PortDef::data_in("condition", PortKind::Condition);
const P_DATA_CONDITION_OPT: PortDef = PortDef::data_in("condition", PortKind::Condition);
const P_DATA_VALUE: PortDef = PortDef::data_in("value", PortKind::Any);
const P_DATA_CASES: PortDef = PortDef::data_in("cases", PortKind::Json);
const P_DATA_DURATION: PortDef = PortDef::data_in("duration", PortKind::Number);
const P_DATA_ITERATIONS_OPT: PortDef = PortDef::data_in("iterations", PortKind::Number);
const P_DATA_TASKS: PortDef = PortDef::data_in("tasks", PortKind::Json);
const P_DATA_EXEC_GROUP: PortDef = PortDef::data_in("execution_group", PortKind::ExecutionHandle);
const P_DATA_MAX_CONCURRENT: PortDef = PortDef::data_in("maximum_concurrent_tasks", PortKind::Number);
const P_DATA_MAX_DURATION: PortDef = PortDef::data_in("maximum_duration", PortKind::Number);
const P_DATA_KEY: PortDef = PortDef::data_in("key", PortKind::String);
const P_DATA_CONTEXT: PortDef = PortDef::data_in("context", PortKind::Context);
const P_DATA_RESULT: PortDef = PortDef::data_in("result", PortKind::Result);
const P_DATA_RESULT_OPT: PortDef = PortDef::data_in("result", PortKind::Result);
const P_DATA_ACTION: PortDef = PortDef::data_in("action", PortKind::String);
const P_DATA_MODULE: PortDef = PortDef::data_in("module", PortKind::String);
const P_DATA_FN: PortDef = PortDef::data_in("function", PortKind::String);
const P_DATA_LEFT: PortDef = PortDef::data_in("left", PortKind::Any);
const P_DATA_RIGHT: PortDef = PortDef::data_in("right", PortKind::Any);
const P_DATA_URL: PortDef = PortDef::data_in("url", PortKind::String);
const P_DATA_BODY: PortDef = PortDef::data_in("body", PortKind::Json);
const P_DATA_FIELD: PortDef = PortDef::data_in("field", PortKind::String);
const P_DATA_PAYLOAD: PortDef = PortDef::data_in("payload", PortKind::Json);

const P_DATA_RESULT_OUT: PortDef = PortDef::data_out("result", PortKind::Result);
const P_DATA_EXEC_HANDLE: PortDef = PortDef::data_out("execution_handle", PortKind::ExecutionHandle);
const P_DATA_STATUS: PortDef = PortDef::data_out("status", PortKind::Status);
const P_DATA_IS_READY: PortDef = PortDef::data_out("is_ready", PortKind::Bool);
const P_DATA_IS_RUNNING: PortDef = PortDef::data_out("is_running", PortKind::Bool);
const P_DATA_PAYLOAD_OUT: PortDef = PortDef::data_out("payload", PortKind::Json);
const P_DATA_SUBSCRIPTION_ID_OUT: PortDef = PortDef::data_out("subscription_id", PortKind::String);
const P_DATA_EXEC_GROUP_OUT: PortDef = PortDef::data_out("execution_group", PortKind::ExecutionHandle);
const P_DATA_RESULTS: PortDef = PortDef::data_out("results", PortKind::Json);
const P_DATA_EXEC_ID: PortDef = PortDef::data_out("execution_id", PortKind::ExecutionHandle);
const P_DATA_ERROR: PortDef = PortDef::data_out("error", PortKind::Result);
const P_DATA_ATTEMPTS: PortDef = PortDef::data_out("attempts", PortKind::Number);
const P_DATA_VALUE_OUT: PortDef = PortDef::data_out("value", PortKind::Any);
const P_DATA_EXISTS: PortDef = PortDef::data_out("exists", PortKind::Bool);

const LEGACY_INPUTS: &[PortDef] = &[P_EXEC_IN];
const LEGACY_OUTPUTS: &[PortDef] = &[P_EXEC_OUT];


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PropDef {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: PropKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropKind { String, Number, Bool, Json, Enum(&'static [&'static str]) }

const PROP_REQUIRED_CONDITIONS: PropDef = PropDef { key: "conditions", label: "Required Conditions", kind: PropKind::Json };
const PROP_CONTAINER_REF: PropDef = PropDef { key: "container", label: "Container", kind: PropKind::String };
const PROP_TARGET_CONTAINER: PropDef = PropDef { key: "target_container", label: "Target Container", kind: PropKind::String };
const PROP_TARGET_OPERATION: PropDef = PropDef { key: "target_operation", label: "Target Operation", kind: PropKind::String };
const PROP_PARAMETERS: PropDef = PropDef { key: "parameters", label: "Parameters", kind: PropKind::Json };
const PROP_EXEC_HANDLE: PropDef = PropDef { key: "execution_handle", label: "Execution Handle", kind: PropKind::String };
const PROP_ENABLED: PropDef = PropDef { key: "enabled", label: "Enabled", kind: PropKind::Bool };
const PROP_EVENT_NAME: PropDef = PropDef { key: "event_name", label: "Event Name", kind: PropKind::String };
const PROP_PAYLOAD: PropDef = PropDef { key: "payload", label: "Payload", kind: PropKind::Json };
const PROP_TIMEOUT_MS: PropDef = PropDef { key: "timeout_ms", label: "Timeout (ms)", kind: PropKind::Number };
const PROP_SUBSCRIPTION_ID: PropDef = PropDef { key: "subscription_id", label: "Subscription ID", kind: PropKind::String };
const PROP_TARGET_REF: PropDef = PropDef { key: "target", label: "Target", kind: PropKind::String };
const PROP_CONDITIONS: PropDef = PropDef { key: "conditions", label: "Conditions", kind: PropKind::Json };
const PROP_SEQUENCE_STEPS: PropDef = PropDef { key: "steps", label: "Steps", kind: PropKind::Json };
const PROP_CONDITION_EXPR: PropDef = PropDef { key: "condition", label: "Condition", kind: PropKind::String };
const PROP_SWITCH_CASES: PropDef = PropDef { key: "cases", label: "Cases", kind: PropKind::Json };
const PROP_MERGE_POLICY: PropDef = PropDef { key: "merge_policy", label: "Merge Policy", kind: PropKind::Enum(&["first", "all", "condition"]) };
const PROP_DURATION_MS: PropDef = PropDef { key: "duration_ms", label: "Duration (ms)", kind: PropKind::Number };
const PROP_REPEAT_MODE: PropDef = PropDef { key: "repeat_mode", label: "Repeat Mode", kind: PropKind::Enum(&["count", "while", "until"]) };
const PROP_ITERATIONS: PropDef = PropDef { key: "iterations", label: "Iterations", kind: PropKind::Number };
const PROP_TASKS: PropDef = PropDef { key: "tasks", label: "Tasks", kind: PropKind::Json };
const PROP_EXEC_GROUP: PropDef = PropDef { key: "execution_group", label: "Execution Group", kind: PropKind::String };
const PROP_RACE_POLICY: PropDef = PropDef { key: "race_policy", label: "Race Policy", kind: PropKind::Enum(&["first_success", "first_complete"]) };
const PROP_MAX_CONCURRENT: PropDef = PropDef { key: "max_concurrent", label: "Max Concurrent", kind: PropKind::Number };
const PROP_OPERATION_REF: PropDef = PropDef { key: "operation", label: "Operation", kind: PropKind::String };
const PROP_MAX_ATTEMPTS: PropDef = PropDef { key: "max_attempts", label: "Max Attempts", kind: PropKind::Number };
const PROP_RETRY_DELAY_MS: PropDef = PropDef { key: "retry_delay_ms", label: "Retry Delay (ms)", kind: PropKind::Number };
const PROP_PRIMARY_REF: PropDef = PropDef { key: "primary", label: "Primary Operation", kind: PropKind::String };
const PROP_FALLBACK_REF: PropDef = PropDef { key: "fallback", label: "Fallback Operation", kind: PropKind::String };
const PROP_KEY: PropDef = PropDef { key: "key", label: "Key", kind: PropKind::String };
const PROP_VALUE: PropDef = PropDef { key: "value", label: "Value", kind: PropKind::String };
const PROP_RESULT_VALUE: PropDef = PropDef { key: "result", label: "Result", kind: PropKind::Json };
const PROP_ACTION: PropDef = PropDef { key: "action", label: "Action", kind: PropKind::Enum(&["request_close", "request_restart", "request_state_transition"]) };
const PROP_MODULE: PropDef = PropDef { key: "module", label: "Module", kind: PropKind::String };
const PROP_FUNCTION: PropDef = PropDef { key: "fn_name", label: "Function", kind: PropKind::String };
const PROP_NAME: PropDef = PropDef { key: "name", label: "Name", kind: PropKind::String };
const PROP_VALUE_TYPE: PropDef = PropDef { key: "value_type", label: "Value Type", kind: PropKind::Enum(&["text", "number", "bool", "json"]) };
const PROP_LEFT: PropDef = PropDef { key: "left", label: "Left", kind: PropKind::String };
const PROP_OP: PropDef = PropDef { key: "op", label: "Operator", kind: PropKind::Enum(&["+", "-", "*", "/"]) };
const PROP_COND_OP: PropDef = PropDef { key: "cond_op", label: "Condition Op", kind: PropKind::Enum(&["==", "!=", ">", "<", ">=", "<="]) };
const PROP_RIGHT: PropDef = PropDef { key: "right", label: "Right", kind: PropKind::String };
const PROP_METHOD: PropDef = PropDef { key: "method", label: "Method", kind: PropKind::Enum(&["GET", "POST", "PUT", "DELETE"]) };
const PROP_URL: PropDef = PropDef { key: "url", label: "URL", kind: PropKind::String };
const PROP_BODY: PropDef = PropDef { key: "body", label: "Body", kind: PropKind::Json };
const PROP_FIELD: PropDef = PropDef { key: "field", label: "Field", kind: PropKind::String };
const LEGACY_PROPS: &[PropDef] = &[PROP_NAME, PROP_VALUE];

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NodeProps {
    #[serde(default)] pub name: String,
    #[serde(default)] pub target: String,
    #[serde(default)] pub key: String,
    #[serde(default)] pub value: String,
    #[serde(default)] pub value_type: String,
    #[serde(default)] pub op: String,
    #[serde(default)] pub left: String,
    #[serde(default)] pub right: String,
    #[serde(default)] pub cond_op: String,
    #[serde(default)] pub method: String,
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
    // New semantic fields for MAIN v1.0
    #[serde(default)] pub container: String,
    #[serde(default)] pub target_container: String,
    #[serde(default)] pub target_operation: String,
    #[serde(default)] pub parameters: String,
    #[serde(default)] pub execution_handle: String,
    #[serde(default)] pub event_name: String,
    #[serde(default)] pub payload: String,
    #[serde(default)] pub timeout_ms: i32,
    #[serde(default)] pub subscription_id: String,
    #[serde(default)] pub conditions: String,
    #[serde(default)] pub condition: String,
    #[serde(default)] pub cases: String,
    #[serde(default)] pub merge_policy: String,
    #[serde(default)] pub duration_ms: i32,
    #[serde(default)] pub repeat_mode: String,
    #[serde(default)] pub iterations: i32,
    #[serde(default)] pub tasks: String,
    #[serde(default)] pub execution_group: String,
    #[serde(default)] pub race_policy: String,
    #[serde(default)] pub max_concurrent: i32,
    #[serde(default)] pub operation: String,
    #[serde(default)] pub max_attempts: i32,
    #[serde(default)] pub retry_delay_ms: i32,
    #[serde(default)] pub primary: String,
    #[serde(default)] pub fallback: String,
    #[serde(default)] pub result: String,
    #[serde(default)] pub action: String,
    #[serde(default)] pub steps: String,
}

impl NodeProps {
    pub fn get(&self, key: &str) -> String {
        match key {
            "name" => self.name.clone(),
            "target" => self.target.clone(),
            "key" => self.key.clone(),
            "value" => self.value.clone(),
            "value_type" => self.value_type.clone(),
            "op" => self.op.clone(),
            "left" => self.left.clone(),
            "right" => self.right.clone(),
            "cond_op" => self.cond_op.clone(),
            "method" => self.method.clone(),
            "url" => self.url.clone(),
            "body" => self.body.clone(),
            "field" => self.field.clone(),
            "ms" | "duration_ms" if key=="ms" => self.ms.to_string(),
            "duration_ms" => self.duration_ms.to_string(),
            "visible" => self.visible.to_string(),
            "module" => self.module.clone(),
            "fn_name" => self.fn_name.clone(),
            "permission" => self.permission.clone(),
            "clear_history" => self.clear_history.to_string(),
            "color" => self.color.clone(),
            "dark_icons" => self.dark_icons.to_string(),
            "enabled" => self.enabled.to_string(),
            "tag" => self.tag.clone(),
            "message" => self.message.clone(),
            "container" => self.container.clone(),
            "target_container" => self.target_container.clone(),
            "target_operation" => self.target_operation.clone(),
            "parameters" => self.parameters.clone(),
            "execution_handle" => self.execution_handle.clone(),
            "event_name" => self.event_name.clone(),
            "payload" => self.payload.clone(),
            "timeout_ms" => self.timeout_ms.to_string(),
            "subscription_id" => self.subscription_id.clone(),
            "conditions" => self.conditions.clone(),
            "condition" => self.condition.clone(),
            "cases" => self.cases.clone(),
            "merge_policy" => self.merge_policy.clone(),
            "repeat_mode" => self.repeat_mode.clone(),
            "iterations" => self.iterations.to_string(),
            "tasks" => self.tasks.clone(),
            "execution_group" => self.execution_group.clone(),
            "race_policy" => self.race_policy.clone(),
            "max_concurrent" => self.max_concurrent.to_string(),
            "operation" => self.operation.clone(),
            "max_attempts" => self.max_attempts.to_string(),
            "retry_delay_ms" => self.retry_delay_ms.to_string(),
            "primary" => self.primary.clone(),
            "fallback" => self.fallback.clone(),
            "result" => self.result.clone(),
            "action" => self.action.clone(),
            "steps" => self.steps.clone(),
            _ => String::new(),
        }
    }

    pub fn set(&mut self, key: &str, val: String) {
        match key {
            "name" => self.name = val,
            "target" => self.target = val,
            "key" => self.key = val,
            "value" => self.value = val,
            "value_type" => self.value_type = val,
            "op" => self.op = val,
            "left" => self.left = val,
            "right" => self.right = val,
            "cond_op" => self.cond_op = val,
            "method" => self.method = val,
            "url" => self.url = val,
            "body" => self.body = val,
            "field" => self.field = val,
            "ms" => self.ms = val.parse().unwrap_or(self.ms),
            "visible" => self.visible = val == "true",
            "module" => self.module = val,
            "fn_name" => self.fn_name = val,
            "permission" => self.permission = val,
            "clear_history" => self.clear_history = val == "true",
            "color" => self.color = val,
            "dark_icons" => self.dark_icons = val == "true",
            "enabled" => self.enabled = val == "true",
            "tag" => self.tag = val,
            "message" => self.message = val,
            "container" => self.container = val,
            "target_container" => self.target_container = val,
            "target_operation" => self.target_operation = val,
            "parameters" => self.parameters = val,
            "execution_handle" => self.execution_handle = val,
            "event_name" => self.event_name = val,
            "payload" => self.payload = val,
            "timeout_ms" => self.timeout_ms = val.parse().unwrap_or(self.timeout_ms),
            "subscription_id" => self.subscription_id = val,
            "conditions" => self.conditions = val,
            "condition" => self.condition = val,
            "cases" => self.cases = val,
            "merge_policy" => self.merge_policy = val,
            "duration_ms" => { self.duration_ms = val.parse().unwrap_or(self.duration_ms); self.ms = self.duration_ms; },
            "repeat_mode" => self.repeat_mode = val,
            "iterations" => self.iterations = val.parse().unwrap_or(self.iterations),
            "tasks" => self.tasks = val,
            "execution_group" => self.execution_group = val,
            "race_policy" => self.race_policy = val,
            "max_concurrent" => self.max_concurrent = val.parse().unwrap_or(self.max_concurrent),
            "operation" => self.operation = val,
            "max_attempts" => self.max_attempts = val.parse().unwrap_or(self.max_attempts),
            "retry_delay_ms" => self.retry_delay_ms = val.parse().unwrap_or(self.retry_delay_ms),
            "primary" => self.primary = val,
            "fallback" => self.fallback = val,
            "result" => self.result = val,
            "action" => self.action = val,
            "steps" => self.steps = val,
            _ => {}
        }
    }
}


impl LogicGraph {
    pub fn next_id(&self) -> u64 {
        self.nodes.iter().map(|n| n.id).max().unwrap_or(0) + 1
    }

    pub fn node(&self, id: u64) -> Option<&LogicNode> { self.nodes.iter().find(|n| n.id == id) }

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
        let Some(from_def) = src.kind.output_port(from_port) else { return Err("Sin salida"); };
        let Some(to_def) = dst.kind.input_port(to_port) else { return Err("Sin entrada"); };
        if !from_def.kind.compatible(to_def.kind) { return Err("Tipo inválido"); }
        if from_def.kind == PortKind::Flow && self.has_path(to_node, from_node) { return Err("Ciclo inválido"); }
        Ok(())
    }

    pub fn connect(&mut self, from_node: u64, from_port: &str, to_node: u64, to_port: &str) -> Result<(), &'static str> {
        self.can_connect(from_node, from_port, to_node, to_port)?;
        let to_flow = self.node(to_node)
            .and_then(|n| n.kind.input_port(to_port))
            .is_some_and(|p| p.kind == PortKind::Flow);
        if to_flow {
            self.links.retain(|l| !(l.to_node == to_node && l.to_port == to_port));
        }
        self.links.retain(|l| !(l.from_node == from_node && l.from_port == from_port && l.to_node == to_node && l.to_port == to_port));
        self.links.push(NodeLink { from_node, from_port: from_port.to_string(), to_node, to_port: to_port.to_string() });
        Ok(())
    }

    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();
        let ids: std::collections::HashSet<u64> = self.nodes.iter().map(|n| n.id).collect();
        for l in &self.links {
            if !ids.contains(&l.from_node) || !ids.contains(&l.to_node) {
                issues.push(format!("Referencia inexistente: {} -> {}", l.from_node, l.to_node));
                continue;
            }
            if let Err(e) = self.can_connect(l.from_node, &l.from_port, l.to_node, &l.to_port) {
                issues.push(format!("Link inválido {}:{} -> {}:{}: {}", l.from_node, l.from_port, l.to_node, l.to_port, e));
            }
        }
        for n in &self.nodes {
            if n.kind.is_legacy() && n.kind.category() == Some(MainCategory::Legacy) {
                issues.push(format!("Nodo legado en grafo: {} ({})", n.kind.label(), n.id));
            }
        }
        issues
    }

    /// Normalize node kinds using the owning group context.
    /// This resolves serde name collisions where old MAIN nodes (`if_condition`, `delay_timer`)
    /// share wire names with active non-MAIN nodes. The group kind is known from project.json.
    pub fn normalize_for_group(&mut self, kind: crate::project::FileGroupKind) {
        use crate::project::FileGroupKind as G;
        for n in &mut self.nodes {
            n.kind = normalize_kind_for_group(n.kind, kind);
        }
        // non-MAIN graphs should never contain MAIN-legacy variants; keep as-is except impossible case.
        if kind != G::Main {
            for n in &mut self.nodes {
                if matches!(n.kind, NodeKind::LegacyIfCondition) { n.kind = NodeKind::IfCondition; }
                if matches!(n.kind, NodeKind::LegacyDelayTimer) { n.kind = NodeKind::DelayTimer; }
            }
        }
    }

    pub fn from_value_for_group(v: serde_json::Value, kind: crate::project::FileGroupKind) -> Result<Self, serde_json::Error> {
        let mut g: LogicGraph = serde_json::from_value(v)?;
        g.normalize_for_group(kind);
        Ok(g)
    }
}

fn normalize_kind_for_group(k: NodeKind, group: crate::project::FileGroupKind) -> NodeKind {
    use crate::project::FileGroupKind as G;
    match group {
        G::Main => match k {
            // Old MAIN serialized names collide with active non-MAIN variants.
            NodeKind::IfCondition => NodeKind::LegacyIfCondition,
            NodeKind::DelayTimer => NodeKind::LegacyDelayTimer,
            _ => k,
        },
        G::Engine | G::Ui | G::Data | G::Network => match k {
            // Defensive: legacy MAIN variants should not appear in non-MAIN graphs.
            NodeKind::LegacyIfCondition => NodeKind::IfCondition,
            NodeKind::LegacyDelayTimer => NodeKind::DelayTimer,
            _ => k,
        },
    }
}


#[cfg(test)]
mod audit_tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn main_catalog_has_exactly_40_unique() {
        let c = NodeKind::MAIN_CATALOG;
        assert_eq!(c.len(), 40);
        let mut ids = HashSet::new();
        for k in c { assert!(ids.insert(k.semantic_id()), "dup {}", k.semantic_id()); }
    }

    #[test]
    fn main_catalog_ids_match_spec() {
        let expected = [
            "application.start","application.ready","application.resume","application.suspend","application.shutdown",
            "container.initialize","container.call","container.call_async","container.wait","container.get_status","container.set_state","container.restart",
            "event.emit","event.wait","event.subscribe","event.unsubscribe","event.forward","event.filter",
            "flow.sequence","flow.branch","flow.switch","flow.merge","flow.gate","flow.delay","flow.repeat",
            "parallel.start","parallel.wait_all","parallel.wait_any","parallel.race","parallel.limit_concurrency",
            "error.try_catch","error.retry","error.timeout","error.fallback",
            "context.set_launch","context.get_launch","context.share",
            "flow.return_result","flow.complete","application.control",
        ];
        let got: Vec<_> = NodeKind::MAIN_CATALOG.iter().map(|k| k.semantic_id()).collect();
        assert_eq!(got, expected);
    }

    #[test]
    fn application_start_is_onready_compat() {
        let k = NodeKind::OnReady;
        assert_eq!(k.semantic_id(), "application.start");
        assert_eq!(k.label(), "Application Start");
        assert!(k.is_main_catalog());
        // serde compat
        let n: NodeKind = serde_json::from_str("\"on_ready\"").unwrap();
        assert_eq!(n, NodeKind::OnReady);
        let n2: NodeKind = serde_json::from_str("\"on_app_start\"").unwrap();
        assert_eq!(n2, NodeKind::OnReady);
    }

    #[test]
    fn legacy_nodes_deserialize_and_not_in_catalog() {
        let cases = [
            ("\"on_resume\"", NodeKind::LegacyOnResume),
            ("\"on_pause\"", NodeKind::LegacyOnPause),
            ("\"on_destroy\"", NodeKind::LegacyOnDestroy),
            ("\"on_back_pressed\"", NodeKind::LegacyOnBackPressed),
            ("\"set_initial_screen\"", NodeKind::LegacySetInitialScreen),
            ("\"open_screen\"", NodeKind::LegacySetInitialScreen),
            ("\"navigate_to_screen\"", NodeKind::LegacyNavigateToScreen),
            ("\"init_engine\"", NodeKind::LegacyInitEngine),
            ("\"init_data\"", NodeKind::LegacyInitData),
            ("\"init_network\"", NodeKind::LegacyInitNetwork),
            ("\"call_module_function\"", NodeKind::LegacyCallModuleFunction),
            ("\"request_permission\"", NodeKind::LegacyRequestPermission),
            ("\"check_permission\"", NodeKind::LegacyCheckPermission),
            ("\"set_status_bar\"", NodeKind::LegacySetStatusBar),
            ("\"keep_screen_on\"", NodeKind::LegacyKeepScreenOn),
            ("\"exit_app\"", NodeKind::LegacyExitApp),
            ("\"on_deep_link\"", NodeKind::LegacyOnDeepLink),
            ("\"open_external_url\"", NodeKind::LegacyOpenExternalUrl),
            ("\"share_text\"", NodeKind::LegacyShareText),
            ("\"log_debug\"", NodeKind::LegacyLogDebug),
        ];
        for (j, exp) in cases {
            let got: NodeKind = serde_json::from_str(j).unwrap();
            assert_eq!(got, exp, "{}", j);
            assert!(!got.is_main_catalog(), "legacy in catalog: {}", j);
        }
        let if_raw: NodeKind = serde_json::from_str("\"if_condition\"").unwrap();
        let delay_raw: NodeKind = serde_json::from_str("\"delay_timer\"").unwrap();
        assert_eq!(if_raw, NodeKind::IfCondition);
        assert_eq!(delay_raw, NodeKind::DelayTimer);
        assert_eq!(normalize_kind_for_group(if_raw, crate::project::FileGroupKind::Main), NodeKind::LegacyIfCondition);
        assert_eq!(normalize_kind_for_group(delay_raw, crate::project::FileGroupKind::Main), NodeKind::LegacyDelayTimer);
        assert_eq!(normalize_kind_for_group(if_raw, crate::project::FileGroupKind::Engine), NodeKind::IfCondition);
        assert_eq!(normalize_kind_for_group(delay_raw, crate::project::FileGroupKind::Engine), NodeKind::DelayTimer);
    }

    #[test]
    fn colliding_node_names_resolve_by_group_context() {
        let main_graph = serde_json::json!({
            "nodes": [
                {"id":1,"kind":"if_condition","x":10.0,"y":20.0,"props":{"left":"a","cond_op":"==","right":"b"}},
                {"id":2,"kind":"delay_timer","x":30.0,"y":40.0,"props":{"ms":500}}
            ],
            "links": [{"from_node":1,"from_port":"true","to_node":2,"to_port":"execute"}]
        });
        let g_main = LogicGraph::from_value_for_group(main_graph, crate::project::FileGroupKind::Main).unwrap();
        assert_eq!(g_main.nodes[0].kind, NodeKind::LegacyIfCondition);
        assert_eq!(g_main.nodes[1].kind, NodeKind::LegacyDelayTimer);
        assert_eq!(g_main.links.len(), 1);
        assert_eq!(g_main.nodes[0].props.left, "a");
        let engine_graph = serde_json::json!({
            "nodes": [
                {"id":1,"kind":"if_condition","x":0.0,"y":0.0,"props":{"left":"a","cond_op":"==","right":"b"}},
                {"id":2,"kind":"delay_timer","x":1.0,"y":1.0,"props":{"ms":500}}
            ],
            "links": []
        });
        let g_engine = LogicGraph::from_value_for_group(engine_graph, crate::project::FileGroupKind::Engine).unwrap();
        assert_eq!(g_engine.nodes[0].kind, NodeKind::IfCondition);
        assert_eq!(g_engine.nodes[1].kind, NodeKind::DelayTimer);
    }

    #[test]
    fn vxs_persistence_normalizes_by_group_kind() {
        use crate::project::{FileGroup, FileGroupKind, VoidXProject, ProjectLanguage, World, VXS_EXT};
        let mut project = VoidXProject {
            name: "Test".into(), package_name: "com.test.app".into(), language: ProjectLanguage::Kotlin,
            icon: None, world: World::default(), files: vec![],
        };
        let main_logic = serde_json::json!({
            "nodes": [
                {"id":1,"kind":"if_condition","x":10.0,"y":20.0,"props":{"left":"a","cond_op":"==","right":"b"}},
                {"id":2,"kind":"delay_timer","x":30.0,"y":40.0,"props":{"ms":750}}
            ],
            "links": [{"from_node":1,"from_port":"true","to_node":2,"to_port":"execute"}]
        });
        let engine_logic = serde_json::json!({
            "nodes": [
                {"id":1,"kind":"if_condition","x":1.0,"y":2.0,"props":{"left":"x","cond_op":"!=","right":"y"}},
                {"id":2,"kind":"delay_timer","x":3.0,"y":4.0,"props":{"ms":250}}
            ],
            "links": [{"from_node":1,"from_port":"true","to_node":2,"to_port":"execute"}]
        });
        project.files.push(FileGroup { name: "main".into(), kind: FileGroupKind::Main, logic: Some(main_logic), visual: None, x: 0.0, y: 0.0, visual_data: None });
        project.files.push(FileGroup { name: "engine".into(), kind: FileGroupKind::Engine, logic: Some(engine_logic), visual: None, x: 0.0, y: 0.0, visual_data: None });
        let dir = std::env::temp_dir().join(format!("vxs_audit_{}_{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("test.{VXS_EXT}"));
        crate::project::save(&path, &project).unwrap();
        let loaded = crate::project::load(&path).unwrap();
        let main_g = LogicGraph::from_value_for_group(loaded.files[0].logic.clone().unwrap(), FileGroupKind::Main).unwrap();
        assert_eq!(main_g.nodes[0].kind, NodeKind::LegacyIfCondition);
        assert_eq!(main_g.nodes[1].kind, NodeKind::LegacyDelayTimer);
        assert_eq!(main_g.nodes[0].props.left, "a");
        assert_eq!(main_g.nodes[1].props.ms, 750);
        assert_eq!(main_g.links.len(), 1);
        let eng_g = LogicGraph::from_value_for_group(loaded.files[1].logic.clone().unwrap(), FileGroupKind::Engine).unwrap();
        assert_eq!(eng_g.nodes[0].kind, NodeKind::IfCondition);
        assert_eq!(eng_g.nodes[1].kind, NodeKind::DelayTimer);
        assert_eq!(eng_g.nodes[1].props.ms, 250);
        // save again and reopen
        crate::project::save(&path, &loaded).unwrap();
        let loaded2 = crate::project::load(&path).unwrap();
        let main_g2 = LogicGraph::from_value_for_group(loaded2.files[0].logic.clone().unwrap(), FileGroupKind::Main).unwrap();
        assert_eq!(main_g2.nodes[0].kind, NodeKind::LegacyIfCondition);
        assert_eq!(main_g2.nodes[1].kind, NodeKind::LegacyDelayTimer);
        assert_eq!(main_g2.nodes[0].x, 10.0);
        assert_eq!(main_g2.links[0].from_port, "true");
        let eng_g2 = LogicGraph::from_value_for_group(loaded2.files[1].logic.clone().unwrap(), FileGroupKind::Engine).unwrap();
        assert_eq!(eng_g2.nodes[0].kind, NodeKind::IfCondition);
        assert_eq!(eng_g2.nodes[1].kind, NodeKind::DelayTimer);
        let _ = std::fs::remove_dir_all(&dir);
    }
    #[test]
    fn main_allowed_for_only_catalog() {
        let allowed = NodeKind::allowed_for(crate::project::FileGroupKind::Main);
        assert_eq!(allowed.len(), 40);
        assert_eq!(allowed, &NodeKind::MAIN_CATALOG);
    }

    #[test]
    fn every_main_node_has_semantic_contract() {
        for k in NodeKind::MAIN_CATALOG {
            assert!(!k.semantic_id().is_empty());
            assert!(!k.label().is_empty());
            assert!(!k.description().is_empty());
            assert!(k.category().is_some(), "{:?}", k);
            let _ = k.level();
            assert_eq!(k.execution_status(), ExecutionStatus::PendingRuntime);
            // ports must have names and kinds
            for p in k.inputs() { assert!(!p.name.is_empty()); }
            for p in k.outputs() { assert!(!p.name.is_empty()); }
        }
    }

    #[test]
    fn can_connect_validates_types_and_cycles() {
        let mut g = LogicGraph::default();
        g.nodes.push(LogicNode { id: 1, kind: NodeKind::OnReady, x:0.0,y:0.0, props: NodeProps::default() });
        g.nodes.push(LogicNode { id: 2, kind: NodeKind::Delay, x:0.0,y:0.0, props: NodeProps::default() });
        g.nodes.push(LogicNode { id: 3, kind: NodeKind::Delay, x:0.0,y:0.0, props: NodeProps::default() });
        assert!(g.can_connect(1, "on_start", 2, "execute").is_ok());
        assert!(g.connect(1,"on_start",2,"execute").is_ok());
        assert!(g.connect(2,"completed",3,"execute").is_ok());
        // cycle
        assert_eq!(g.can_connect(3,"completed",2,"execute").unwrap_err(), "Ciclo inválido");
        // type mismatch
        g.nodes.push(LogicNode { id: 4, kind: NodeKind::GetLaunchContext, x:0.0,y:0.0, props: NodeProps::default() });
        g.nodes.push(LogicNode { id: 5, kind: NodeKind::SetLaunchContext, x:0.0,y:0.0, props: NodeProps::default() });
        // Any->String is compatible by design; Flow->data should fail
        assert!(g.can_connect(1,"on_start",5,"key").is_err());
    }

    #[test]
    fn graph_serde_roundtrip_preserves_links_and_props() {
        let mut g = LogicGraph::default();
        let mut p = NodeProps::default();
        p.event_name = "evt".into();
        p.timeout_ms = 1234;
        g.nodes.push(LogicNode { id: 1, kind: NodeKind::OnReady, x:1.0,y:2.0, props: NodeProps::default() });
        g.nodes.push(LogicNode { id: 2, kind: NodeKind::WaitForEvent, x:3.0,y:4.0, props: p });
        g.connect(1,"on_start",2,"execute").unwrap();
        let j = serde_json::to_string(&g).unwrap();
        let g2: LogicGraph = serde_json::from_str(&j).unwrap();
        assert_eq!(g2.nodes.len(), 2);
        assert_eq!(g2.links.len(), 1);
        assert_eq!(g2.nodes[1].props.event_name, "evt");
        assert_eq!(g2.nodes[1].props.timeout_ms, 1234);
    }

    #[test]
    fn legacy_nodes_in_graph_validate_reports() {
        let mut g = LogicGraph::default();
        g.nodes.push(LogicNode { id:1, kind: NodeKind::LegacyOnResume, x:0.0,y:0.0, props: NodeProps::default() });
        let issues = g.validate();
        assert!(issues.iter().any(|i| i.contains("Nodo legado")));
    }
}
