//! Minimal node editor for project logic graphs.

use egui::{Align2, Color32, CornerRadius, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

use crate::logic::{LogicGraph, LogicNode, NodeKind, NodeProps, NodeWire};
use crate::project::FileGroupKind;
use crate::theme::{Tokens, regular, semibold};

#[derive(Clone, Debug, Default)]
pub struct NodeEditorState {
    pub pan: egui::Vec2,
    pub zoom: f32,
    pub dragging_node: Option<u64>,
    pub connect_from: Option<(u64, String)>,
    pub add_open: bool,
}

pub fn show(ui: &mut Ui, _t: &Tokens, group_kind: FileGroupKind, graph: &mut LogicGraph, st: &mut NodeEditorState) {
    let rect = ui.max_rect();
    let resp = ui.interact(rect, ui.scope_id().with("node_editor_canvas"), Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, if ui.visuals().dark_mode { Color32::from_rgb(0x18,0x18,0x18) } else { Color32::from_rgb(0xFA,0xFA,0xFA) });

    let mut pan = st.pan;
    let mut zoom = st.zoom.clamp(0.25, 3.0);
    if zoom == 0.0 { zoom = 1.0; }
    let pointer_in = resp.hover_pos().unwrap_or(rect.center());
    let scroll = ui.input(|i| i.smooth_scroll_delta);
    let pinch = ui.input(|i| i.zoom_delta());
    if let Some(mt) = ui.input(|i| i.multi_touch()) {
        if (mt.zoom_delta - 1.0).abs() > 0.001 {
            let old = zoom;
            zoom = (zoom * mt.zoom_delta).clamp(0.25, 3.0);
            let before = (mt.start_pos - rect.min - pan) / old;
            pan = mt.start_pos - rect.min - before * zoom;
        }
        pan += mt.translation_delta;
    } else if resp.dragged() && st.dragging_node.is_none() && st.connect_from.is_none() {
        pan += resp.drag_delta();
    } else if resp.hovered() {
        if (pinch - 1.0).abs() > 0.001 {
            let old = zoom;
            zoom = (zoom * pinch).clamp(0.25, 3.0);
            let before = (pointer_in - rect.min - pan) / old;
            pan = pointer_in - rect.min - before * zoom;
        } else if scroll != egui::Vec2::ZERO {
            if ui.input(|i| i.modifiers.command || i.modifiers.ctrl) {
                let old = zoom;
                let factor = (scroll.y / 200.0).exp();
                zoom = (zoom * factor).clamp(0.25, 3.0);
                let before = (pointer_in - rect.min - pan) / old;
                pan = pointer_in - rect.min - before * zoom;
            } else {
                pan += scroll;
            }
        }
    }
    st.pan = pan; st.zoom = zoom;

    draw_grid(&painter, rect, pan, zoom, ui.visuals().dark_mode);

    // wires
    for w in &graph.wires.clone() {
        let Some((from_p, _)) = port_pos(rect, pan, zoom, graph, w.from_node, &w.from_port, false) else { continue };
        let Some((to_p, _)) = port_pos(rect, pan, zoom, graph, w.to_node, &w.to_port, true) else { continue };
        draw_wire(&painter, from_p, to_p, Color32::from_rgb(140, 180, 255));
    }
    if let Some((nid, port)) = &st.connect_from.clone() {
        if let Some((from_p, _)) = port_pos(rect, pan, zoom, graph, *nid, port, false) {
            let to_p = ui.input(|i| i.pointer.hover_pos()).unwrap_or(from_p);
            draw_wire(&painter, from_p, to_p, Color32::from_rgba_unmultiplied(140,180,255,140));
        }
        if !ui.input(|i| i.pointer.any_down()) { st.connect_from = None; }
    }

    let mut remove_node: Option<u64> = None;
    let nodes = graph.nodes.clone();
    for n in &nodes {
        let nr = node_rect(rect, pan, zoom, n);
        let id = ui.scope_id().with(("node", n.id));
        let node_resp = ui.interact(nr, id, Sense::click_and_drag());
        let header_h = 28.0 * zoom;
        let header = Rect::from_min_size(nr.min, vec2(nr.width(), header_h));
        let accent = group_accent(group_kind);
        painter.rect_filled(nr, CornerRadius::same((8.0 * zoom).max(4.0) as u8), Color32::from_rgb(34, 34, 38));
        painter.rect_stroke(nr, CornerRadius::same((8.0 * zoom).max(4.0) as u8), Stroke::new(1.0, if node_resp.hovered() { accent } else { Color32::from_rgb(70,70,75) }), StrokeKind::Inside);
        painter.rect_filled(header, CornerRadius { nw: (8.0 * zoom).max(4.0) as u8, ne: (8.0 * zoom).max(4.0) as u8, sw: 0, se: 0 }, accent);
        painter.text(pos2(header.min.x + 8.0*zoom, header.center().y), Align2::LEFT_CENTER, n.kind.label(), semibold((12.0*zoom).max(4.0)), Color32::WHITE);

        // delete
        let del = Rect::from_min_size(pos2(header.max.x - 22.0*zoom, header.min.y + 4.0*zoom), vec2(18.0*zoom, 18.0*zoom));
        let del_resp = ui.interact(del, ui.scope_id().with(("node_del", n.id)), Sense::click());
        painter.text(del.center(), Align2::CENTER_CENTER, "×", regular((14.0*zoom).max(4.0)), if del_resp.hovered() { Color32::from_rgb(255,120,120) } else { Color32::WHITE });
        if del_resp.clicked() { remove_node = Some(n.id); }

        // ports
        for (i, p) in n.kind.inputs().iter().enumerate() {
            let (pc, pr) = input_port(nr, zoom, i);
            let prr = Rect::from_center_size(pc, vec2(pr*2.0, pr*2.0));
            let presp = ui.interact(prr, ui.scope_id().with(("in", n.id, *p)), Sense::click_and_drag());
            painter.circle_filled(pc, pr, Color32::from_rgb(120, 200, 255));
            painter.text(pos2(pc.x + 8.0*zoom, pc.y), Align2::LEFT_CENTER, *p, regular((10.0*zoom).max(4.0)), Color32::from_gray(210));
            if presp.hovered() && ui.input(|i| i.pointer.any_released()) {
                if let Some((fnid, fport)) = st.connect_from.clone() {
                    if fnid != n.id {
                        graph.wires.retain(|w| !(w.to_node==n.id && w.to_port==*p));
                        graph.wires.push(NodeWire { from_node: fnid, from_port: fport, to_node: n.id, to_port: (*p).to_string() });
                    }
                    st.connect_from = None;
                }
            }
        }
        for (i, p) in n.kind.outputs().iter().enumerate() {
            let (pc, pr) = output_port(nr, zoom, i);
            let prr = Rect::from_center_size(pc, vec2(pr*2.0, pr*2.0));
            let presp = ui.interact(prr, ui.scope_id().with(("out", n.id, *p)), Sense::click_and_drag());
            painter.circle_filled(pc, pr, Color32::from_rgb(255, 190, 120));
            painter.text(pos2(pc.x - 8.0*zoom, pc.y), Align2::RIGHT_CENTER, *p, regular((10.0*zoom).max(4.0)), Color32::from_gray(210));
            if presp.drag_started() { st.connect_from = Some((n.id, (*p).to_string())); }
        }

        // simple props preview
        let mut py = header.max.y + 10.0*zoom;
        let prop_lines = prop_preview(&n.props, n.kind);
        for line in prop_lines {
            painter.text(pos2(nr.min.x + 10.0*zoom, py), Align2::LEFT_TOP, line, regular((10.5*zoom).max(4.0)), Color32::from_gray(190));
            py += 16.0*zoom;
        }

        if node_resp.drag_started() && st.connect_from.is_none() { st.dragging_node = Some(n.id); }
        if st.dragging_node == Some(n.id) && node_resp.dragged() {
            if let Some(gn) = graph.nodes.iter_mut().find(|x| x.id == n.id) {
                let d = node_resp.drag_delta() / zoom;
                gn.x += d.x; gn.y += d.y;
            }
        }
    }
    if !ui.input(|i| i.pointer.any_down()) { st.dragging_node = None; }
    if let Some(id) = remove_node {
        graph.nodes.retain(|n| n.id != id);
        graph.wires.retain(|w| w.from_node != id && w.to_node != id);
    }

    // + Nodo button
    let add = Rect::from_min_size(pos2(rect.max.x - 124.0, rect.max.y - 60.0), vec2(104.0, 40.0));
    let add_resp = ui.interact(add, ui.scope_id().with("node_add_btn"), Sense::click());
    painter.rect_filled(add, CornerRadius::same(10), if add_resp.hovered() { Color32::from_rgb(0x1F,0xA8,0x6A) } else { Color32::from_rgb(0x0C,0xCE,0x87) });
    painter.text(add.center(), Align2::CENTER_CENTER, "+ Nodo", semibold(14.0), Color32::BLACK);
    if add_resp.clicked() { st.add_open = !st.add_open; }

    if st.add_open {
        let panel = Rect::from_min_size(pos2(rect.max.x - 280.0, rect.max.y - 320.0), vec2(260.0, 250.0));
        painter.rect_filled(panel, CornerRadius::same(10), Color32::from_rgb(28,28,32));
        painter.rect_stroke(panel, CornerRadius::same(10), Stroke::new(1.0, group_accent(group_kind)), StrokeKind::Inside);
        painter.text(pos2(panel.min.x + 12.0, panel.min.y + 18.0), Align2::LEFT_CENTER, "Agregar nodo", semibold(13.0), Color32::WHITE);
        let mut y = panel.min.y + 40.0;
        for kind in NodeKind::allowed_for(group_kind) {
            let r = Rect::from_min_size(pos2(panel.min.x + 10.0, y), vec2(panel.width()-20.0, 28.0));
            let rr = ui.interact(r, ui.scope_id().with(("add_kind", kind.label())), Sense::click());
            painter.rect_filled(r, CornerRadius::same(6), if rr.hovered() { Color32::from_rgb(45,45,52) } else { Color32::from_rgb(36,36,42) });
            painter.text(pos2(r.min.x + 8.0, r.center().y), Align2::LEFT_CENTER, kind.label(), regular(11.5), Color32::WHITE);
            if rr.clicked() {
                let id = graph.next_id();
                let world = (panel.center() - rect.min - pan) / zoom;
                graph.nodes.push(LogicNode { id, kind: *kind, x: world.x, y: world.y, props: default_props(*kind) });
                st.add_open = false;
            }
            y += 32.0;
            if y > panel.max.y - 10.0 { break; }
        }
    }
}

fn default_props(kind: NodeKind) -> NodeProps {
    let mut p = NodeProps::default();
    match kind {
        NodeKind::OpenScreen | NodeKind::NavigateTo => p.target = "screen".into(),
        NodeKind::InitModule | NodeKind::CallFunction | NodeKind::DefineFunction => p.name = "function".into(),
        NodeKind::SetVisible => { p.target = "element".into(); p.visible = true; }
        NodeKind::SetText => { p.target = "textNode".into(); p.value = "Hello".into(); }
        NodeKind::Variable | NodeKind::SetVariable => { p.name = "var".into(); p.value_type = "text".into(); p.value = "".into(); }
        NodeKind::MathOp => p.op = "+".into(),
        NodeKind::TimerDelay => p.ms = 500,
        NodeKind::SaveLocal | NodeKind::LoadLocal | NodeKind::DeleteLocal => { p.key = "key".into(); p.value = "".into(); }
        NodeKind::HttpRequest => { p.method = "GET".into(); p.url = "https://api.example.com".into(); }
        NodeKind::JsonGetField => p.field = "data".into(),
        _ => {}
    }
    p
}

fn prop_preview(p: &NodeProps, kind: NodeKind) -> Vec<String> {
    match kind {
        NodeKind::OpenScreen | NodeKind::NavigateTo => vec![format!("screen: {}", p.target)],
        NodeKind::InitModule => vec![format!("module: {}", p.target)],
        NodeKind::CallFunction => vec![format!("fn: {}", p.name)],
        NodeKind::DefineFunction => vec![format!("name: {}", p.name)],
        NodeKind::OnTap => vec![format!("element: {}", p.target)],
        NodeKind::SetVisible => vec![format!("target: {}", p.target), format!("visible: {}", p.visible)],
        NodeKind::SetText => vec![format!("target: {}", p.target), format!("text: {}", p.value)],
        NodeKind::Variable | NodeKind::SetVariable => vec![format!("{}: {} = {}", p.name, p.value_type, p.value)],
        NodeKind::MathOp => vec![format!("op: {}", p.op)],
        NodeKind::IfCondition => vec!["true/false".into()],
        NodeKind::TimerDelay => vec![format!("{} ms", p.ms)],
        NodeKind::SaveLocal | NodeKind::LoadLocal | NodeKind::DeleteLocal => vec![format!("key: {}", p.key)],
        NodeKind::HttpRequest => vec![format!("{} {}", p.method, p.url)],
        NodeKind::JsonGetField => vec![format!("field: {}", p.field)],
        _ => vec![],
    }
}

fn node_rect(rect: Rect, pan: egui::Vec2, zoom: f32, n: &LogicNode) -> Rect {
    let size = vec2(220.0 * zoom, 120.0 * zoom);
    let c = pos2(rect.min.x + pan.x + n.x * zoom, rect.min.y + pan.y + n.y * zoom);
    Rect::from_center_size(c, size)
}

fn input_port(nr: Rect, zoom: f32, i: usize) -> (Pos2, f32) {
    (pos2(nr.min.x, nr.min.y + 44.0*zoom + i as f32 * 22.0*zoom), 5.0*zoom)
}
fn output_port(nr: Rect, zoom: f32, i: usize) -> (Pos2, f32) {
    (pos2(nr.max.x, nr.min.y + 44.0*zoom + i as f32 * 22.0*zoom), 5.0*zoom)
}

fn port_pos(rect: Rect, pan: egui::Vec2, zoom: f32, g: &LogicGraph, node_id: u64, port: &str, input: bool) -> Option<(Pos2, f32)> {
    let n = g.nodes.iter().find(|n| n.id == node_id)?;
    let nr = node_rect(rect, pan, zoom, n);
    let ports = if input { n.kind.inputs() } else { n.kind.outputs() };
    let idx = ports.iter().position(|p| *p == port)?;
    Some(if input { input_port(nr, zoom, idx) } else { output_port(nr, zoom, idx) })
}

fn draw_wire(p: &egui::Painter, a: Pos2, b: Pos2, color: Color32) {
    let dx = ((b.x - a.x).abs() * 0.5).max(30.0);
    let c1 = pos2(a.x + dx, a.y);
    let c2 = pos2(b.x - dx, b.y);
    let mut pts = Vec::new();
    let steps = 24;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let u = 1.0 - t;
        let x = u*u*u*a.x + 3.0*u*u*t*c1.x + 3.0*u*t*t*c2.x + t*t*t*b.x;
        let y = u*u*u*a.y + 3.0*u*u*t*c1.y + 3.0*u*t*t*c2.y + t*t*t*b.y;
        pts.push(pos2(x,y));
    }
    p.add(egui::Shape::line(pts, Stroke::new(2.0, color)));
}

fn draw_grid(painter: &egui::Painter, rect: Rect, pan: egui::Vec2, zoom: f32, dark: bool) {
    let mut step = 48.0 * zoom;
    if step <= 0.0001 { return; }
    while step < 12.0 { step *= 4.0; }
    let grid_col = if dark { Color32::from_rgba_unmultiplied(255,255,255,16) } else { Color32::from_rgba_unmultiplied(0,0,0,16) };
    let ox = (pan.x % step + step) % step;
    let oy = (pan.y % step + step) % step;
    let mut x = rect.min.x + ox;
    while x < rect.max.x { painter.line_segment([pos2(x, rect.min.y), pos2(x, rect.max.y)], Stroke::new(1.0, grid_col)); x += step; }
    let mut y = rect.min.y + oy;
    while y < rect.max.y { painter.line_segment([pos2(rect.min.x, y), pos2(rect.max.x, y)], Stroke::new(1.0, grid_col)); y += step; }
}

fn group_accent(kind: FileGroupKind) -> Color32 {
    let s = kind.style();
    Color32::from_rgb(s.accent[0], s.accent[1], s.accent[2])
}
