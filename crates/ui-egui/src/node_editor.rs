//! Node editor for project logic graphs: mini-panel connections and animated flow cables.

use egui::{Align2, Color32, CornerRadius, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

use crate::logic::{LogicGraph, LogicNode, NodeKind, NodeLink, NodeProps};
use crate::project::FileGroupKind;
use crate::theme::{Tokens, regular, semibold};

#[derive(Clone, Debug, Default)]
pub struct NodeEditorState {
    pub pan: egui::Vec2,
    pub zoom: f32,
    pub dragging_node: Option<u64>,
    pub selected_node: Option<u64>,
    pub add_open: bool,
    pub add_query: String,
    pub add_selected: Option<NodeKind>,
    pub connect_panel: Option<ConnectPanel>,
}

#[derive(Clone, Debug)]
pub struct ConnectPanel {
    pub node_id: u64,
    pub port: String,
    pub is_output: bool,
    pub create_mode: bool,
}

#[derive(Clone, Debug, Default)]
pub struct NodeEditorActions {
    pub auto_order: bool,
    pub save_back: bool,
}

pub fn show(ui: &mut Ui, t: &Tokens, group_kind: FileGroupKind, graph: &mut LogicGraph, st: &mut NodeEditorState) -> NodeEditorActions {
    let actions = NodeEditorActions::default();
    let rect = ui.max_rect();
    let modal_open = st.add_open || st.connect_panel.is_some();
    let resp = ui.interact(rect, ui.scope_id().with("node_editor_canvas"), if modal_open { Sense::hover() } else { Sense::click_and_drag() });
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, if ui.visuals().dark_mode { Color32::from_rgb(0x18,0x18,0x18) } else { Color32::from_rgb(0xFA,0xFA,0xFA) });

    let mut pan = st.pan;
    let mut zoom = if st.zoom == 0.0 { 1.0 } else { st.zoom.clamp(0.25, 3.0) };
    let pointer_in = resp.hover_pos().unwrap_or(rect.center());
    let scroll = ui.input(|i| i.smooth_scroll_delta);
    let pinch = ui.input(|i| i.zoom_delta());
    if !modal_open {
        if let Some(mt) = ui.input(|i| i.multi_touch()) {
            if (mt.zoom_delta - 1.0).abs() > 0.001 {
                let old = zoom;
                zoom = (zoom * mt.zoom_delta).clamp(0.25, 3.0);
                let before = (mt.start_pos - rect.min - pan) / old;
                pan = mt.start_pos - rect.min - before * zoom;
            }
            pan += mt.translation_delta;
        } else if resp.dragged() && st.dragging_node.is_none() {
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
    }
    st.pan = pan;
    st.zoom = zoom;

    draw_grid(&painter, rect, pan, zoom, ui.visuals().dark_mode);

    if resp.clicked() && !resp.dragged() {
        st.selected_node = None;
        st.connect_panel = None;
        // keep add modal state unless explicitly closed
    }

    let accent = group_accent(group_kind);
    let time = ui.input(|i| i.time);
    let selected = st.selected_node;

    // Cables behind nodes
    let links = graph.links.clone();
    for l in &links {
        let Some((from_p, _)) = port_pos(rect, pan, zoom, graph, l.from_node, &l.from_port, false) else { continue };
        let Some((to_p, _)) = port_pos(rect, pan, zoom, graph, l.to_node, &l.to_port, true) else { continue };
        let related = selected.is_some_and(|s| s == l.from_node || s == l.to_node);
        let (alpha, width, pulse_alpha, speed, pulse_r) = if let Some(_) = selected {
            if related { (1.0f32, 2.2f32, 220u8, 1.8, 3.2 * zoom.max(0.7)) } else { (0.10, 1.2, 40, 0.8, 2.0 * zoom.max(0.7)) }
        } else {
            (0.35, 1.2, 110, 1.0, 2.4 * zoom.max(0.7))
        };
        let base = with_alpha(accent, (alpha * 255.0) as u8);
        let pts = bezier_points(from_p, to_p);
        painter.add(egui::Shape::line(pts.clone(), Stroke::new(width.max(1.0), base)));
        // energy pulse
        let t_anim = ((time * speed) as f32).fract();
        let pulse_pos = sample_bezier(from_p, to_p, t_anim);
        let glow = with_alpha(accent, pulse_alpha / 3);
        painter.circle_filled(pulse_pos, pulse_r * 1.8, glow);
        painter.circle_filled(pulse_pos, pulse_r, with_alpha(Color32::WHITE, pulse_alpha));
    }
    ui.ctx().request_repaint();

    let mut remove_node: Option<u64> = None;
    let nodes = graph.nodes.clone();
    for n in &nodes {
        let nr = node_rect(rect, pan, zoom, n);
        let id = ui.scope_id().with(("node", n.id));
        let node_resp = ui.interact(nr, id, Sense::click_and_drag());
        let header_h = 28.0 * zoom;
        let header = Rect::from_min_size(nr.min, vec2(nr.width(), header_h));
        let is_sel = st.selected_node == Some(n.id);
        painter.rect_filled(nr, CornerRadius::same((10.0 * zoom).max(4.0) as u8), Color32::from_rgb(34, 34, 38));
        painter.rect_stroke(nr, CornerRadius::same((10.0 * zoom).max(4.0) as u8), Stroke::new(if is_sel { 2.0 } else { 1.0 }, if is_sel || node_resp.hovered() { accent } else { Color32::from_rgb(70,70,75) }), StrokeKind::Inside);
        painter.rect_filled(header, CornerRadius { nw: (10.0 * zoom).max(4.0) as u8, ne: (10.0 * zoom).max(4.0) as u8, sw: 0, se: 0 }, accent);
        painter.text(pos2(header.min.x + 8.0*zoom, header.center().y), Align2::LEFT_CENTER, n.kind.label(), semibold((12.0*zoom).max(4.0)), Color32::WHITE);

        let del = Rect::from_min_size(pos2(header.max.x - 22.0*zoom, header.min.y + 4.0*zoom), vec2(18.0*zoom, 18.0*zoom));
        let del_resp = ui.interact(del, ui.scope_id().with(("node_del", n.id)), Sense::click());
        painter.text(del.center(), Align2::CENTER_CENTER, "×", regular((14.0*zoom).max(4.0)), if del_resp.hovered() { Color32::from_rgb(255,120,120) } else { Color32::WHITE });
        if del_resp.clicked() { remove_node = Some(n.id); }

        if node_resp.clicked() {
            st.selected_node = Some(n.id);
        }
        if node_resp.drag_started() { st.dragging_node = Some(n.id); st.selected_node = Some(n.id); }
        if st.dragging_node == Some(n.id) && node_resp.dragged() {
            if let Some(gn) = graph.nodes.iter_mut().find(|x| x.id == n.id) {
                let d = node_resp.drag_delta() / zoom;
                gn.x += d.x; gn.y += d.y;
            }
        }

        // Connection buttons (mobile touch target >= 36px)
        let btn_h = (36.0 * zoom).max(30.0);
        let btn_w = (nr.width() - 18.0*zoom) / 2.0;
        let in_btn = Rect::from_min_size(pos2(nr.min.x + 6.0*zoom, nr.max.y - btn_h - 6.0*zoom), vec2(btn_w, btn_h));
        let out_btn = Rect::from_min_size(pos2(in_btn.max.x + 6.0*zoom, nr.max.y - btn_h - 6.0*zoom), vec2(btn_w, btn_h));
        let has_in = !n.kind.inputs().is_empty();
        let has_out = !n.kind.outputs().is_empty();
        if has_in {
            let r = ui.interact(in_btn, ui.scope_id().with(("in_btn", n.id)), Sense::click());
            painter.rect_filled(in_btn, CornerRadius::same(8), if r.hovered() { Color32::from_rgb(55,55,62) } else { Color32::from_rgb(45,45,52) });
            painter.text(in_btn.center(), Align2::CENTER_CENTER, "⬅ Entrada", regular((11.0*zoom).max(10.0)), t.text);
            if r.clicked() {
                st.selected_node = Some(n.id);
                let port = n.kind.inputs().get(0).unwrap_or(&"in").to_string();
                st.connect_panel = Some(ConnectPanel { node_id: n.id, port, is_output: false, create_mode: false });
                st.add_open = false;
            }
        }
        if has_out {
            let r = ui.interact(out_btn, ui.scope_id().with(("out_btn", n.id)), Sense::click());
            painter.rect_filled(out_btn, CornerRadius::same(8), if r.hovered() { Color32::from_rgb(55,55,62) } else { Color32::from_rgb(45,45,52) });
            painter.text(out_btn.center(), Align2::CENTER_CENTER, "Salida ➡", regular((11.0*zoom).max(10.0)), t.text);
            if r.clicked() {
                st.selected_node = Some(n.id);
                let port = n.kind.outputs().get(0).unwrap_or(&"out").to_string();
                st.connect_panel = Some(ConnectPanel { node_id: n.id, port, is_output: true, create_mode: false });
                st.add_open = false;
            }
        }

        // Connected pills
        let mut py = header.max.y + 8.0*zoom;
        let prop_lines = prop_preview(&n.props, n.kind);
        for line in prop_lines {
            painter.text(pos2(nr.min.x + 10.0*zoom, py), Align2::LEFT_TOP, line, regular((10.0*zoom).max(4.0)), Color32::from_gray(190));
            py += 14.0*zoom;
        }
        py += 2.0*zoom;
        for l in links.iter().filter(|l| l.from_node == n.id || l.to_node == n.id) {
            let (label, remove_link) = if l.from_node == n.id {
                let other = graph.nodes.iter().find(|x| x.id == l.to_node).map(|x| x.kind.label()).unwrap_or("node");
                (format!("{} → {}", l.from_port, other), l.clone())
            } else {
                let other = graph.nodes.iter().find(|x| x.id == l.from_node).map(|x| x.kind.label()).unwrap_or("node");
                (format!("{} ← {}", l.to_port, other), l.clone())
            };
            let font = regular((9.5*zoom).max(4.0));
            let g = painter.layout_no_wrap(label.clone(), font.clone(), Color32::WHITE);
            let pill = Rect::from_min_size(pos2(nr.min.x + 8.0*zoom, py), vec2((g.size().x + 28.0*zoom).min(nr.width()-16.0*zoom), 18.0*zoom));
            painter.rect_filled(pill, CornerRadius::same(9), Color32::from_rgb(50,50,58));
            painter.text(pos2(pill.min.x + 6.0*zoom, pill.center().y), Align2::LEFT_CENTER, label, font, Color32::from_gray(220));
            let x_rect = Rect::from_min_size(pos2(pill.max.x - 16.0*zoom, pill.min.y), vec2(16.0*zoom, pill.height()));
            let x_resp = ui.interact(x_rect, ui.scope_id().with(("unlink", remove_link.from_node, &remove_link.from_port, remove_link.to_node, &remove_link.to_port, n.id, py as i32)), Sense::click());
            painter.text(x_rect.center(), Align2::CENTER_CENTER, "×", regular((10.0*zoom).max(4.0)), if x_resp.hovered() { Color32::from_rgb(255,120,120) } else { Color32::from_gray(180) });
            if x_resp.clicked() {
                graph.links.retain(|w| !(w.from_node==remove_link.from_node && w.from_port==remove_link.from_port && w.to_node==remove_link.to_node && w.to_port==remove_link.to_port));
            }
            py += 20.0*zoom;
            if py > nr.max.y - btn_h - 10.0*zoom { break; }
        }
    }

    if !ui.input(|i| i.pointer.any_down()) { st.dragging_node = None; }
    if let Some(id) = remove_node {
        graph.nodes.retain(|n| n.id != id);
        graph.links.retain(|w| w.from_node != id && w.to_node != id);
        if st.selected_node == Some(id) { st.selected_node = None; }
        if st.connect_panel.as_ref().is_some_and(|p| p.node_id == id) { st.connect_panel = None; }
    }

    // Connect mini-panel with semantic compatibility
    if let Some(panel_state) = st.connect_panel.clone() {
        if let Some(node) = graph.nodes.iter().find(|n| n.id == panel_state.node_id).cloned() {
            let nr = node_rect(rect, pan, zoom, &node);
            let panel_w = 300.0f32.min(rect.width() - 20.0);
            let panel_h = 260.0f32.min(rect.height() - 20.0);
            let anchor = if panel_state.is_output { pos2(nr.max.x + 8.0, nr.min.y) } else { pos2(nr.min.x - panel_w - 8.0, nr.min.y) };
            let panel_rect = Rect::from_min_size(anchor.clamp(rect.min + vec2(8.0,8.0), rect.max - vec2(panel_w + 8.0, panel_h + 8.0)), vec2(panel_w, panel_h));
            let _panel_resp = ui.interact(panel_rect, ui.scope_id().with(("conn_panel", panel_state.node_id, &panel_state.port, panel_state.is_output)), Sense::click());
            painter.rect_filled(panel_rect, CornerRadius::same(12), Color32::from_rgb(30,30,34));
            painter.rect_stroke(panel_rect, CornerRadius::same(12), Stroke::new(1.0, accent), StrokeKind::Inside);
            painter.text(pos2(panel_rect.min.x + 12.0, panel_rect.min.y + 18.0), Align2::LEFT_CENTER, if panel_state.is_output { "Conectar salida" } else { "Conectar entrada" }, semibold(12.5), Color32::WHITE);
            let close_r = Rect::from_min_size(pos2(panel_rect.max.x - 34.0, panel_rect.min.y + 8.0), vec2(26.0, 26.0));
            let close_resp = ui.interact(close_r, ui.scope_id().with(("conn_close", panel_state.node_id)), Sense::click());
            painter.text(close_r.center(), Align2::CENTER_CENTER, "×", regular(15.0), if close_resp.hovered() { Color32::WHITE } else { Color32::from_gray(180) });
            if close_resp.clicked() { st.connect_panel = None; }

            let mut y = panel_rect.min.y + 38.0;
            if !panel_state.create_mode {
                let candidates: Vec<LogicNode> = graph.nodes.iter().filter(|cand| cand.id != panel_state.node_id).cloned().collect();
                if candidates.is_empty() {
                    painter.text(pos2(panel_rect.min.x + 12.0, y + 8.0), Align2::LEFT_TOP, "No hay otros nodos en este lienzo", regular(12.0), Color32::from_gray(190));
                    let add_r = Rect::from_min_size(pos2(panel_rect.min.x + 12.0, y + 44.0), vec2(panel_rect.width()-24.0, 40.0));
                    let add_resp = ui.interact(add_r, ui.scope_id().with(("conn_empty_add", panel_state.node_id)), Sense::click());
                    painter.rect_filled(add_r, CornerRadius::same(8), if add_resp.hovered() { with_alpha(accent, 220) } else { accent });
                    painter.text(add_r.center(), Align2::CENTER_CENTER, "+ Añadir Nodo", semibold(12.0), Color32::BLACK);
                    if add_resp.clicked() { st.add_open = true; st.add_query.clear(); st.add_selected = None; st.connect_panel = None; }
                } else {
                    painter.text(pos2(panel_rect.min.x + 12.0, y), Align2::LEFT_TOP, "Nodos disponibles", regular(10.5), Color32::from_gray(170));
                    y += 20.0;
                    for cand in candidates.iter().take(5) {
                        let (ok, reason) = connection_status(graph, &panel_state, cand);
                        let r = Rect::from_min_size(pos2(panel_rect.min.x + 8.0, y), vec2(panel_rect.width()-16.0, 36.0));
                        let rr = ui.interact(r, ui.scope_id().with(("conn_pick", panel_state.node_id, cand.id, &panel_state.port, panel_state.is_output)), Sense::click());
                        let bg = if ok { Color32::from_rgba_unmultiplied(16,185,129, if rr.hovered() { 55 } else { 35 }) } else { Color32::from_rgba_unmultiplied(239,68,68, if rr.hovered() { 45 } else { 28 }) };
                        let stroke = if ok { Color32::from_rgb(16,185,129) } else { Color32::from_rgb(239,68,68) };
                        painter.rect_filled(r, CornerRadius::same(8), bg);
                        painter.rect_stroke(r, CornerRadius::same(8), Stroke::new(1.0, stroke), StrokeKind::Inside);
                        painter.text(pos2(r.min.x + 8.0, r.center().y), Align2::LEFT_CENTER, cand.kind.label(), regular(11.5), Color32::WHITE);
                        let tag = if ok { "Compatible" } else { reason };
                        let tag_font = regular(9.5);
                        let tg = painter.layout_no_wrap(tag.to_string(), tag_font.clone(), Color32::WHITE);
                        let tr = Rect::from_min_size(pos2(r.max.x - tg.size().x - 16.0, r.center().y - 9.0), vec2(tg.size().x + 10.0, 18.0));
                        painter.rect_filled(tr, CornerRadius::same(9), with_alpha(stroke, 90));
                        painter.text(tr.center(), Align2::CENTER_CENTER, tag, tag_font, Color32::WHITE);
                        if rr.clicked() && ok {
                            create_link(graph, &panel_state, cand.id);
                            st.connect_panel = None;
                        }
                        y += 40.0;
                    }
                    let create_r = Rect::from_min_size(pos2(panel_rect.min.x + 8.0, panel_rect.max.y - 44.0), vec2(panel_rect.width()-16.0, 36.0));
                    let create_resp = ui.interact(create_r, ui.scope_id().with(("conn_create_mode", panel_state.node_id)), Sense::click());
                    painter.rect_filled(create_r, CornerRadius::same(8), if create_resp.hovered() { with_alpha(accent, 220) } else { accent });
                    painter.text(create_r.center(), Align2::CENTER_CENTER, "+ Crear y conectar nuevo nodo", regular(11.0), Color32::BLACK);
                    if create_resp.clicked() {
                        if let Some(cp) = st.connect_panel.as_mut() { cp.create_mode = true; }
                    }
                }
            } else {
                painter.text(pos2(panel_rect.min.x + 12.0, y), Align2::LEFT_TOP, "Tipo de nuevo nodo", regular(10.5), Color32::from_gray(170));
                y += 20.0;
                for kind in NodeKind::allowed_for(group_kind).iter().take(6) {
                    let r = Rect::from_min_size(pos2(panel_rect.min.x + 8.0, y), vec2(panel_rect.width()-16.0, 30.0));
                    let rr = ui.interact(r, ui.scope_id().with(("conn_new_kind", panel_state.node_id, kind.label(), panel_state.is_output)), Sense::click());
                    painter.rect_filled(r, CornerRadius::same(8), if rr.hovered() { Color32::from_rgb(52,52,60) } else { Color32::from_rgb(42,42,48) });
                    painter.text(pos2(r.min.x + 8.0, r.center().y), Align2::LEFT_CENTER, kind.label(), regular(11.0), Color32::WHITE);
                    if rr.clicked() {
                        let id = graph.next_id();
                        let nx = node.x + if panel_state.is_output { 280.0 } else { -280.0 };
                        let ny = node.y + 40.0 * (graph.nodes.len() % 4) as f32;
                        graph.nodes.push(LogicNode { id, kind: *kind, x: nx, y: ny, props: default_props(*kind) });
                        let new_panel = ConnectPanel { node_id: panel_state.node_id, port: panel_state.port.clone(), is_output: panel_state.is_output, create_mode: false };
                        if connection_status(graph, &new_panel, graph.nodes.iter().find(|n| n.id==id).expect("new")).0 {
                            create_link(graph, &new_panel, id);
                        }
                        st.selected_node = Some(id);
                        st.connect_panel = None;
                    }
                    y += 34.0;
                }
            }
        } else {
            st.connect_panel = None;
        }
    }

    // FAB + Añadir Nodo (always visible, mobile thumb-friendly)
    let fab = Rect::from_min_size(pos2(rect.max.x - 84.0, rect.max.y - 84.0), vec2(64.0, 64.0));
    let fab_resp = ui.interact(fab, ui.scope_id().with("add_node_fab"), Sense::click());
    painter.circle_filled(fab.center(), 30.0, if fab_resp.hovered() { with_alpha(accent, 230) } else { accent });
    painter.text(fab.center(), Align2::CENTER_CENTER, "+", semibold(30.0), Color32::BLACK);

    // Add node modal (centered, container accent, searchable)
    if st.add_open {
        let overlay = rect;
        let overlay_resp = ui.interact(overlay, ui.scope_id().with("add_modal_overlay"), Sense::click());
        painter.rect_filled(overlay, 0.0, Color32::from_rgba_unmultiplied(0,0,0,110));
        let panel_w = 420.0f32.min(rect.width() - 24.0);
        let panel_h = 460.0f32.min(rect.height() - 24.0);
        let panel = Rect::from_center_size(rect.center(), vec2(panel_w, panel_h));
        let panel_resp = ui.interact(panel, ui.scope_id().with("add_modal_panel"), Sense::click());
        painter.rect_filled(panel, CornerRadius::same(14), Color32::from_rgb(28,28,32));
        painter.rect_stroke(panel, CornerRadius::same(14), Stroke::new(1.5, accent), StrokeKind::Inside);

        painter.text(pos2(panel.min.x + 16.0, panel.min.y + 24.0), Align2::LEFT_CENTER, "Añadir Nodo", semibold(18.0), Color32::WHITE);
        let count = NodeKind::allowed_for(group_kind).len();
        let badge_text = format!("{count} nodos disponibles");
        let bf = regular(11.0);
        let bg = painter.layout_no_wrap(badge_text.clone(), bf.clone(), Color32::BLACK);
        let badge_r = Rect::from_min_size(pos2(panel.min.x + 132.0, panel.min.y + 14.0), vec2(bg.size().x + 14.0, 22.0));
        painter.rect_filled(badge_r, CornerRadius::same(11), accent);
        painter.text(badge_r.center(), Align2::CENTER_CENTER, badge_text, bf, Color32::BLACK);
        let close_r = Rect::from_min_size(pos2(panel.max.x - 40.0, panel.min.y + 10.0), vec2(28.0, 28.0));
        let close_resp = ui.interact(close_r, ui.scope_id().with("add_modal_close"), Sense::click());
        painter.text(close_r.center(), Align2::CENTER_CENTER, "×", regular(18.0), if close_resp.hovered() { Color32::WHITE } else { Color32::from_gray(180) });

        let search_r = Rect::from_min_size(pos2(panel.min.x + 14.0, panel.min.y + 48.0), vec2(panel.width()-28.0, 40.0));
        painter.rect_filled(search_r, CornerRadius::same(8), Color32::from_rgb(40,40,46));
        painter.rect_stroke(search_r, CornerRadius::same(8), Stroke::new(1.0, with_alpha(accent, 120)), StrokeKind::Inside);
        let search_id = ui.scope_id().with("add_node_query");
        let mut query = st.add_query.clone();
        let mut te_resp = None;
        ui.scope_builder(egui::UiBuilder::new().max_rect(search_r.shrink2(vec2(10.0, 6.0))), |ui| {
            let te = egui::TextEdit::singleline(&mut query).id(search_id).hint_text("Buscar nodo...").frame(egui::Frame::NONE).desired_width(f32::INFINITY);
            te_resp = Some(ui.add(te));
        });
        st.add_query = query;

        let q = st.add_query.trim().to_ascii_lowercase();
        let filtered: Vec<NodeKind> = NodeKind::allowed_for(group_kind)
            .iter()
            .copied()
            .filter(|k| {
                q.is_empty()
                    || k.label().to_ascii_lowercase().contains(&q)
                    || k.description().to_ascii_lowercase().contains(&q)
                    || k.category().map(|c| c.label().to_ascii_lowercase().contains(&q)).unwrap_or(false)
            })
            .collect();

        let list_top = search_r.max.y + 10.0;
        let create_r = Rect::from_min_size(pos2(panel.min.x + 14.0, panel.max.y - 54.0), vec2(panel.width()-28.0, 40.0));
        let list_rect = Rect::from_min_max(pos2(panel.min.x + 14.0, list_top), pos2(panel.max.x - 14.0, create_r.min.y - 10.0));
        painter.rect_filled(list_rect, CornerRadius::same(8), Color32::from_rgb(34,34,39));

        let mut create_clicked_kind: Option<NodeKind> = None;
        ui.scope_builder(egui::UiBuilder::new().max_rect(list_rect.shrink2(vec2(6.0, 6.0))), |ui| {
            egui::ScrollArea::vertical()
                .id_salt("add_node_scroll")
                .auto_shrink([false, false])
                .max_height((list_rect.height() - 12.0).max(120.0))
                .show(ui, |ui| {
                    if filtered.is_empty() {
                        ui.add_space(24.0);
                        ui.label(egui::RichText::new("Sin resultados").color(Color32::from_gray(170)));
                    } else {
                        let mut last_cat: Option<crate::logic::MainCategory> = None;
                        for kind in filtered.iter().copied() {
                            if let Some(cat) = kind.category() {
                                if last_cat != Some(cat) {
                                    ui.add_space(4.0);
                                    ui.label(egui::RichText::new(cat.label()).font(semibold(11.0)).color(with_alpha(accent, 220)));
                                    last_cat = Some(cat);
                                }
                            }
                            let selected = st.add_selected == Some(kind);
                            let (row_rect, row_resp) = ui.allocate_exact_size(vec2(ui.available_width(), 48.0), Sense::click());
                            let rp = ui.painter_at(row_rect);
                            rp.rect_filled(row_rect, CornerRadius::same(8), if selected { with_alpha(accent, 55) } else if row_resp.hovered() { Color32::from_rgb(48,48,55) } else { Color32::TRANSPARENT });
                            if selected { rp.rect_stroke(row_rect, CornerRadius::same(8), Stroke::new(1.0, accent), StrokeKind::Inside); }
                            rp.text(pos2(row_rect.min.x + 10.0, row_rect.min.y + 15.0), Align2::LEFT_CENTER, kind.label(), semibold(12.5), Color32::WHITE);
                            rp.text(pos2(row_rect.min.x + 10.0, row_rect.min.y + 33.0), Align2::LEFT_CENTER, kind.description(), regular(10.0), Color32::from_gray(185));
                            if row_resp.clicked() { st.add_selected = Some(kind); }
                            ui.add_space(2.0);
                        }
                    }
                });
        });

        let can_create = st.add_selected.is_some();
        let create_resp = ui.interact(create_r, ui.scope_id().with("add_modal_create"), Sense::click());
        painter.rect_filled(create_r, CornerRadius::same(8), if can_create { if create_resp.hovered() { with_alpha(accent, 220) } else { accent } } else { Color32::from_rgb(70,70,76) });
        painter.text(create_r.center(), Align2::CENTER_CENTER, "Create", semibold(13.0), Color32::WHITE);
        if create_resp.clicked() {
            create_clicked_kind = st.add_selected;
        }
        if let Some(kind) = create_clicked_kind {
            let id = graph.next_id();
            let world = (rect.center() - rect.min - pan) / zoom;
            graph.nodes.push(LogicNode { id, kind, x: world.x, y: world.y, props: default_props(kind) });
            st.selected_node = Some(id);
            st.add_open = false;
            st.add_query.clear();
            st.add_selected = None;
        }

        if close_resp.clicked() || (overlay_resp.clicked() && !panel_resp.clicked() && te_resp.as_ref().is_none_or(|r| !r.clicked())) {
            st.add_open = false;
        }
    }

    if fab_resp.clicked() {
        st.add_open = !st.add_open;
        st.connect_panel = None;
        if st.add_open && st.add_selected.is_none() {
            st.add_selected = NodeKind::allowed_for(group_kind).first().copied();
        }
    }

    actions
}

fn connection_status(graph: &LogicGraph, panel: &ConnectPanel, cand: &LogicNode) -> (bool, &'static str) {
    let (from_node, from_port, to_node, to_port) = if panel.is_output {
        let Some(to_port) = cand.kind.inputs().get(0).copied() else { return (false, "Sin entrada"); };
        (panel.node_id, panel.port.as_str(), cand.id, to_port)
    } else {
        let Some(from_port) = cand.kind.outputs().get(0).copied() else { return (false, "Sin salida"); };
        (cand.id, from_port, panel.node_id, panel.port.as_str())
    };
    match graph.can_connect(from_node, from_port, to_node, to_port) {
        Ok(()) => (true, "Compatible"),
        Err(e) => (false, e),
    }
}

fn create_link(graph: &mut LogicGraph, panel: &ConnectPanel, other_id: u64) {
    let (from_node, from_port, to_node, to_port) = if panel.is_output {
        let to_port = graph.nodes.iter().find(|n| n.id == other_id).and_then(|n| n.kind.inputs().get(0).copied()).unwrap_or("in").to_string();
        (panel.node_id, panel.port.clone(), other_id, to_port)
    } else {
        let from_port = graph.nodes.iter().find(|n| n.id == other_id).and_then(|n| n.kind.outputs().get(0).copied()).unwrap_or("out").to_string();
        (other_id, from_port, panel.node_id, panel.port.clone())
    };
    if graph.can_connect(from_node, &from_port, to_node, &to_port).is_err() { return; }
    graph.links.retain(|l| !(l.to_node == to_node && l.to_port == to_port));
    graph.links.retain(|l| !(l.from_node == from_node && l.from_port == from_port && l.to_node == to_node && l.to_port == to_port));
    graph.links.push(NodeLink { from_node, from_port, to_node, to_port });
}

pub fn auto_order(graph: &mut LogicGraph) {
    if graph.nodes.is_empty() { return; }
    // simple left-to-right layering by BFS from roots
    let mut indeg: std::collections::HashMap<u64, usize> = graph.nodes.iter().map(|n| (n.id, 0)).collect();
    let mut adj: std::collections::HashMap<u64, Vec<u64>> = graph.nodes.iter().map(|n| (n.id, vec![])).collect();
    for l in &graph.links {
        if adj.contains_key(&l.from_node) && indeg.contains_key(&l.to_node) {
            adj.entry(l.from_node).or_default().push(l.to_node);
            *indeg.entry(l.to_node).or_insert(0) += 1;
        }
    }
    let mut q: std::collections::VecDeque<(u64, usize)> = std::collections::VecDeque::new();
    let mut roots: Vec<u64> = indeg.iter().filter(|(_, d)| **d == 0).map(|(id, _)| *id).collect();
    roots.sort();
    for r in roots { q.push_back((r, 0)); }
    if q.is_empty() { // cycle/all connected
        for n in &graph.nodes { q.push_back((n.id, 0)); }
    }
    let mut layer: std::collections::HashMap<u64, usize> = std::collections::HashMap::new();
    while let Some((id, d)) = q.pop_front() {
        if layer.contains_key(&id) { continue; }
        layer.insert(id, d);
        if let Some(ns) = adj.get(&id) {
            for n in ns { if !layer.contains_key(n) { q.push_back((*n, d+1)); } }
        }
    }
    for n in &graph.nodes { layer.entry(n.id).or_insert(0); }
    let mut by_layer: std::collections::BTreeMap<usize, Vec<u64>> = std::collections::BTreeMap::new();
    for (id, l) in layer { by_layer.entry(l).or_default().push(id); }
    for ids in by_layer.values_mut() { ids.sort(); }
    for (l, ids) in by_layer {
        for (i, id) in ids.iter().enumerate() {
            if let Some(n) = graph.nodes.iter_mut().find(|n| n.id == *id) {
                n.x = 180.0 + l as f32 * 320.0;
                n.y = 120.0 + i as f32 * 170.0;
            }
        }
    }
}

fn with_alpha(c: Color32, a: u8) -> Color32 { Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a) }

fn bezier_points(a: Pos2, b: Pos2) -> Vec<Pos2> {
    let dx = ((b.x - a.x).abs() * 0.5).max(30.0);
    let c1 = pos2(a.x + dx, a.y);
    let c2 = pos2(b.x - dx, b.y);
    let mut pts = Vec::new();
    for i in 0..=28 {
        let t = i as f32 / 28.0;
        pts.push(sample_bezier_ctrl(a, c1, c2, b, t));
    }
    pts
}

fn sample_bezier(a: Pos2, b: Pos2, t: f32) -> Pos2 {
    let dx = ((b.x - a.x).abs() * 0.5).max(30.0);
    let c1 = pos2(a.x + dx, a.y);
    let c2 = pos2(b.x - dx, b.y);
    sample_bezier_ctrl(a, c1, c2, b, t)
}

fn sample_bezier_ctrl(a: Pos2, c1: Pos2, c2: Pos2, b: Pos2, t: f32) -> Pos2 {
    let u = 1.0 - t;
    let x = u*u*u*a.x + 3.0*u*u*t*c1.x + 3.0*u*t*t*c2.x + t*t*t*b.x;
    let y = u*u*u*a.y + 3.0*u*u*t*c1.y + 3.0*u*t*t*c2.y + t*t*t*b.y;
    pos2(x, y)
}

pub fn default_props(kind: NodeKind) -> NodeProps {
    let mut p = NodeProps::default();
    match kind {
        NodeKind::SetInitialScreen | NodeKind::NavigateTo => { p.target = "screen".into(); }
        NodeKind::NavigateToScreen => { p.target = "screen".into(); p.clear_history = false; }
        NodeKind::InitEngine | NodeKind::InitData | NodeKind::InitNetwork => { p.module = "module".into(); p.target = "module".into(); }
        NodeKind::CallModuleFunction | NodeKind::CallFunction => { p.module = "module".into(); p.fn_name = "function".into(); }
        NodeKind::DefineFunction => p.name = "function".into(),
        NodeKind::OnTap => p.target = "element".into(),
        NodeKind::SetVisible => { p.target = "element".into(); p.visible = true; }
        NodeKind::SetText => { p.target = "textNode".into(); p.value = "Hello".into(); }
        NodeKind::SetVariable => { p.name = "var".into(); p.value_type = "text".into(); p.value = "".into(); }
        NodeKind::MathOp => { p.op = "+".into(); p.left = "a".into(); p.right = "b".into(); }
        NodeKind::IfCondition => { p.left = "a".into(); p.cond_op = "==".into(); p.right = "b".into(); }
        NodeKind::DelayTimer => p.ms = 500,
        NodeKind::SaveLocal | NodeKind::LoadLocal | NodeKind::DeleteLocal => { p.key = "key".into(); p.value = "".into(); }
        NodeKind::HttpRequest => { p.method = "GET".into(); p.url = "https://api.example.com".into(); }
        NodeKind::JsonGetField => { p.field = "data".into(); }
        NodeKind::RequestPermission | NodeKind::CheckPermission => p.permission = "Camera".into(),
        NodeKind::SetStatusBar => { p.color = "#0CCE87".into(); p.dark_icons = false; }
        NodeKind::KeepScreenOn => p.enabled = true,
        NodeKind::OpenExternalUrl => p.url = "https://example.com".into(),
        NodeKind::ShareText => p.value = "Hello from VoidX".into(),
        NodeKind::LogDebug => { p.tag = "Main".into(); p.message = "debug".into(); }
        _ => {}
    }
    p
}

fn prop_preview(p: &NodeProps, kind: NodeKind) -> Vec<String> {
    match kind {
        NodeKind::OnReady => vec!["Inicio del contenedor".into()],
        NodeKind::OnResume | NodeKind::OnPause | NodeKind::OnDestroy | NodeKind::OnBackPressed | NodeKind::ExitApp => vec![kind.description().into()],
        NodeKind::SetInitialScreen | NodeKind::NavigateTo => vec![format!("screen: {}", p.target)],
        NodeKind::NavigateToScreen => vec![format!("screen: {}", p.target), format!("clear: {}", p.clear_history)],
        NodeKind::InitEngine | NodeKind::InitData | NodeKind::InitNetwork => vec![format!("module: {}", if p.module.is_empty() { &p.target } else { &p.module })],
        NodeKind::CallModuleFunction | NodeKind::CallFunction => vec![format!("{}::{}", p.module, p.fn_name)],
        NodeKind::RequestPermission | NodeKind::CheckPermission => vec![format!("permiso: {}", p.permission)],
        NodeKind::SetStatusBar => vec![format!("color: {}", p.color), format!("dark_icons: {}", p.dark_icons)],
        NodeKind::KeepScreenOn => vec![format!("enabled: {}", p.enabled)],
        NodeKind::OnDeepLink => vec!["salida: url".into()],
        NodeKind::OpenExternalUrl => vec![format!("url: {}", p.url)],
        NodeKind::ShareText => vec![format!("text: {}", p.value)],
        NodeKind::LogDebug => vec![format!("{}: {}", p.tag, p.message)],
        NodeKind::DefineFunction => vec![format!("name: {}", p.name)],
        NodeKind::OnTap => vec![format!("element: {}", p.target)],
        NodeKind::SetVisible => vec![format!("target: {}", p.target), format!("visible: {}", p.visible)],
        NodeKind::SetText => vec![format!("target: {}", p.target), format!("text: {}", p.value)],
        NodeKind::SetVariable => vec![format!("{}: {} = {}", p.name, p.value_type, p.value)],
        NodeKind::MathOp => vec![format!("{} {} {}", p.left, p.op, p.right)],
        NodeKind::IfCondition => vec![format!("{} {} {}", p.left, p.cond_op, p.right)],
        NodeKind::DelayTimer => vec![format!("{} ms", p.ms)],
        NodeKind::SaveLocal | NodeKind::LoadLocal | NodeKind::DeleteLocal => vec![format!("key: {}", p.key)],
        NodeKind::HttpRequest => vec![format!("{} {}", p.method, p.url)],
        NodeKind::JsonGetField => vec![format!("field: {}", p.field)],
    }
}

fn node_rect(rect: Rect, pan: egui::Vec2, zoom: f32, n: &LogicNode) -> Rect {
    let in_count = n.kind.inputs().len().max(1) as f32;
    let out_count = n.kind.outputs().len().max(1) as f32;
    let port_rows = in_count.max(out_count);
    let extra = (prop_preview(&n.props, n.kind).len() as f32) * 14.0 + 40.0;
    let h = (86.0 + port_rows * 10.0 + extra).max(120.0) * zoom;
    let size = vec2(230.0 * zoom, h);
    let c = pos2(rect.min.x + pan.x + n.x * zoom, rect.min.y + pan.y + n.y * zoom);
    Rect::from_center_size(c, size)
}

fn input_port(nr: Rect, zoom: f32, i: usize) -> (Pos2, f32) { (pos2(nr.min.x, nr.min.y + 44.0*zoom + i as f32 * 22.0*zoom), 5.0*zoom) }
fn output_port(nr: Rect, zoom: f32, i: usize) -> (Pos2, f32) { (pos2(nr.max.x, nr.min.y + 44.0*zoom + i as f32 * 22.0*zoom), 5.0*zoom) }

fn port_pos(rect: Rect, pan: egui::Vec2, zoom: f32, g: &LogicGraph, node_id: u64, port: &str, input: bool) -> Option<(Pos2, f32)> {
    let n = g.nodes.iter().find(|n| n.id == node_id)?;
    let nr = node_rect(rect, pan, zoom, n);
    let ports = if input { n.kind.inputs() } else { n.kind.outputs() };
    let idx = ports.iter().position(|p| *p == port).unwrap_or(0);
    Some(if input { input_port(nr, zoom, idx) } else { output_port(nr, zoom, idx) })
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
