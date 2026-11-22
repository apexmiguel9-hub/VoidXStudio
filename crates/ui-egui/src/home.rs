//! The home screen: Recents (new file, open, join with a code, the Getting started tips, open and
//! recent files ([`crate::home_recents`]), and templates to start from on a first run), Online (your online files:
//! [`crate::online`]) and Templates (the starter templates as cards with thumbnails rendered from
//! the documents themselves, searched and filtered by kind).

use std::collections::BTreeMap;

use egui::{Align2, Color32, CornerRadius, Id, Order, Rect, Response, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use figcraft_engine::templates::{Kind, TEMPLATES, Template};
use serde_json::json;

use crate::FigApp;
use crate::a11y;
use crate::i18n::{tr, tr_noop};
use crate::icons::{self, Icon};
use crate::theme::{ACCENT, Tokens, regular, semibold};
use crate::widgets;
pub mod home_recents;
pub mod online;

/// The section the home screen shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Section {
    #[default]
    Recents,
    Online,
    Templates,
}

impl Section {
    /// The sidebar, top to bottom.
    const ALL: [(Section, Icon, &str); 3] = [
        (Section::Recents, Icon::Home, tr_noop("Recents")),
        (Section::Online, Icon::Online, tr_noop("Online")),
        (Section::Templates, Icon::Assets, tr_noop("Templates")),
    ];

    /// The id `ui.set {homeSection}` and `ui.inspect` use.
    pub fn id(self) -> &'static str {
        match self {
            Section::Recents => "recents",
            Section::Online => "online",
            Section::Templates => "templates",
        }
    }

    pub fn parse(id: &str) -> Option<Section> {
        Section::ALL.into_iter().map(|(section, ..)| section).find(|section| section.id() == id)
    }
}

/// The home screen's own state.
#[derive(Default)]
pub struct Home {
    pub section: Section,
    /// Searches known files and starter templates together from the home sidebar.
    pub file_query: String,
    /// The Templates search text and kind filter.
    pub query: String,
    pub kind: Option<Kind>,
    /// The recent files: what is known about each, and the selection.
    pub recents: crate::home_recents::Recents,
}

const SIDEBAR: f32 = 240.0;
const CARD_W: f32 = 248.0;
const THUMB_H: f32 = 148.0;
const CARD_H: f32 = THUMB_H + 86.0;
const GAP: f32 = 20.0;
/// Templates offered on a first run.
const FIRST_RUN: usize = 4;

/// The Getting started tips: (title, what to do, the command whose shortcut shows).
const TIPS: [(&str, &str, &str); 6] = [
    (tr_noop("Draw a frame"), tr_noop("Pick the Frame tool, then drag on the canvas or choose a device size"), "tool.frame"),
    (tr_noop("Add auto layout"), tr_noop("Select layers and they space themselves as their content changes"), "layout.add"),
    (tr_noop("Make a component"), tr_noop("Turn a layer into a component, reuse it, and change every copy at once"), "component.create"),
    (tr_noop("Prototype"), tr_noop("Link frames on the Prototype tab, then present the flow"), "proto.present"),
    (tr_noop("Share with a code"), tr_noop("Start sharing with the Share button, then pass on the code it gives"), "collab.share"),
    (tr_noop("Export"), tr_noop("Export frames and layers as PNG, JPG, SVG or PDF"), "file.export"),
];

/// A row of the home screen's file lists.
pub(crate) struct FileRow<'a> {
    /// The mark before the name, and its colour.
    pub mark: Option<(Icon, Color32)>,
    pub name: &'a str,
    pub ink: Color32,
    /// What its right end says (a path, where the file is), with its size and colour.
    pub detail: &'a str,
    pub detail_size: f32,
    pub detail_ink: Color32,
}

/// Paint `row` in `r` (highlighted while `resp` is hovered): the name first, the detail in the
/// room left after it, cut at its start; true when either is cut.
pub(crate) fn file_row(ui: &Ui, t: &Tokens, r: Rect, resp: &Response, row: &FileRow) -> bool {
    /// Between the name and the detail.
    const GAP: f32 = 16.0;
    /// A detail with less room than this is left out rather than shown as a bare ellipsis.
    const MIN_DETAIL: f32 = 48.0;
    if resp.hovered() {
        ui.painter().rect_filled(r, CornerRadius::same(5), t.hover);
    }
    if let Some((icon, tint)) = row.mark {
        icons::paint(ui.painter(), Rect::from_min_size(pos2(r.min.x + 8.0, r.center().y - 8.0), vec2(16.0, 16.0)), icon, tint);
    }
    let (name_font, detail_font) = (regular(12.0), regular(row.detail_size));
    let width = |s: &str, font: &egui::FontId| ui.painter().layout_no_wrap(s.to_string(), font.clone(), Color32::WHITE).size().x;
    let room = (r.width() - 40.0).max(0.0);
    let name_w = width(row.name, &name_font);
    let detail_w = width(row.detail, &detail_font);
    // When both don't fit, the name keeps at least 60% of the row and the detail gets the rest.
    let name_max = if name_w + GAP + detail_w <= room { name_w } else { name_w.min((room - GAP - detail_w).max(room * 0.6)) };
    let name = widgets::elide_font(ui, row.name, name_max, name_font.clone());
    let detail_room = room - width(&name, &name_font) - GAP;
    let detail = if row.detail.is_empty() || detail_room < MIN_DETAIL {
        String::new()
    } else {
        widgets::elide_start_font(ui, row.detail, detail_room, detail_font.clone())
    };
    let cut = name != row.name || detail != row.detail;
    ui.painter().text(pos2(r.min.x + 32.0, r.center().y), Align2::LEFT_CENTER, name, name_font, row.ink);
    ui.painter().text(pos2(r.max.x - 8.0, r.center().y), Align2::RIGHT_CENTER, detail, detail_font, row.detail_ink);
    cut
}

/// A row of Recents' own lists: laid out in place, named for screen readers, ringed with the
/// keyboard on it; true when chosen.
fn listed(ui: &mut Ui, t: &Tokens, id: Id, height: f32, row: &FileRow) -> bool {
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width().min(640.0), height), Sense::hover());
    let mut resp = ui.interact(r, id, Sense::click());
    a11y::button(&resp, row.name);
    if file_row(ui, t, r, &resp, row) {
        // Cut to fit: in full on hover.
        resp = resp.on_hover_text(format!("{}\n{}", row.name, row.detail));
    }
    focus_ring(ui, t, r, &resp, 5);
    resp.clicked()
}

pub fn show(app: &mut FigApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    dropped(app, ui.ctx());
    egui::CentralPanel::default().frame(egui::Frame::NONE.fill(t.panel)).show(ui, |ui| {
        if app.ui.project.is_some() {
            project_view(app, ui, &t);
            return;
        }
        let rect = ui.max_rect();
        let left = Rect::from_min_size(rect.min, vec2(SIDEBAR, rect.height()));
        ui.painter().line_segment([left.right_top(), left.right_bottom()], Stroke::new(1.0, t.border));
        sidebar(app, ui, &t, left);
        if app.ui.home_view.file_query.trim().is_empty() && app.ui.home_view.section == Section::Online {
            let area = Rect::from_min_max(pos2(left.max.x + 40.0, rect.min.y + 24.0), rect.max - vec2(40.0, 24.0));
            crate::online::section(app, ui, area, &t);
            opened_join(app);
            return;
        }
        let main = Rect::from_min_max(pos2(left.max.x, rect.min.y), rect.max);
        ui.scope_builder(egui::UiBuilder::new().max_rect(main), |ui| {
            egui::ScrollArea::vertical().id_salt("home").auto_shrink([false, false]).show(ui, |ui| {
                egui::Frame::NONE.inner_margin(egui::Margin::symmetric(40, 24)).show(ui, |ui| {
                    ui.spacing_mut().item_spacing = vec2(GAP, 0.0);
                    if app.ui.home_view.file_query.trim().is_empty() {
                        match app.ui.home_view.section {
                            Section::Templates => templates(app, ui, &t),
                            _ => recents(app, ui, &t),
                        }
                    } else {
                        search_results(app, ui, &t);
                        opened_join(app);
                    }
                });
            });
        });
    });
    if app.ui.project_dialog.is_some() {
        create_project_dialog(app, ui);
    }
    if app.ui.add_group_dialog.is_some() {
        add_group_dialog(app, ui);
    }
    crate::drop_overlay(ui.ctx(), ui.ctx().content_rect(), tr("Drop to open"));
}

/// The Create Project dialog.
fn create_project_dialog(app: &mut FigApp, ui: &mut Ui) {
    let Some(mut draft) = app.ui.project_dialog.take() else { return };
    let mut open = true;
    let mut close_dialog = false;
    let ctx = ui.ctx().clone();
    crate::dialogs::window("Create Project", &ctx)
        .id(egui::Id::unique("create_project_dialog"))
        .order(Order::Foreground)
        .open(&mut open)
        .show(&ctx, |ui| {
            let t = Tokens::get(ui.ctx());
            ui.set_width(crate::dialogs::dialog_width(ui.ctx(), 440.0));
            ui.spacing_mut().item_spacing.y = 8.0;

            ui.label(egui::RichText::new("Lenguaje").font(semibold(12.0)).color(t.text_secondary));
            ui.horizontal_wrapped(|ui| {
                for lang in crate::project::ProjectLanguage::ALL {
                    let selected = draft.language == lang;
                    let available = lang.available();
                    let (fill, stroke, text_col) = if selected {
                        (Color32::from_rgb(0x19, 0x3D, 0x23), Color32::from_rgb(0x0C, 0xCE, 0x87), Color32::from_rgb(0x31, 0xEC, 0x8E))
                    } else if !available {
                        (t.field, t.border, t.text_tertiary)
                    } else {
                        (t.field, t.border, t.text)
                    };
                    let btn = egui::Button::new(egui::RichText::new(lang.label()).size(12.0).color(text_col))
                        .fill(fill)
                        .stroke(Stroke::new(if selected { 1.5 } else { 1.0 }, stroke))
                        .corner_radius(CornerRadius::same(6));
                    let resp = ui.add_enabled(available, btn);
                    if resp.clicked() && available {
                        draft.language = lang;
                    }
                }
            });

            ui.add_space(4.0);
            ui.label(egui::RichText::new("App Name").font(semibold(12.0)).color(t.text_secondary));
            ui.add(egui::TextEdit::singleline(&mut draft.name).hint_text("MiApp").desired_width(f32::INFINITY));

            ui.label(egui::RichText::new("Package Name").font(semibold(12.0)).color(t.text_secondary));
            ui.add(egui::TextEdit::singleline(&mut draft.package_name).hint_text("com.voidx.miapp").desired_width(f32::INFINITY));
            let pkg_ok = crate::project::VoidXProject::package_name_ok(&draft.package_name);
            if !draft.package_name.is_empty() && !pkg_ok {
                ui.label(egui::RichText::new("Debe tener al menos 2 segmentos, ej. com.voidx.miapp").size(11.0).color(t.warning));
            }

            ui.label(egui::RichText::new("Icon Path (opcional)").font(semibold(12.0)).color(t.text_secondary));
            ui.add(egui::TextEdit::singleline(&mut draft.icon).hint_text("/ruta/icon.png").desired_width(f32::INFINITY));

            ui.add_space(8.0);
            let can_create = !draft.name.trim().is_empty() && pkg_ok;
            ui.horizontal(|ui| {
                let create = egui::Button::new(egui::RichText::new("Create").color(if can_create { Color32::BLACK } else { t.text_tertiary }).strong())
                    .fill(if can_create { Color32::from_rgb(0x0C, 0xCE, 0x87) } else { t.field })
                    .corner_radius(CornerRadius::same(6));
                if ui.add_enabled(can_create, create).clicked() {
                    let name = draft.name.trim().to_string();
                    let project = crate::project::VoidXProject {
                        name: name.clone(),
                        package_name: draft.package_name.trim().to_string(),
                        language: draft.language,
                        icon: (!draft.icon.trim().is_empty()).then_some(draft.icon.trim().to_string()),
                        world: crate::project::World::default(),
                        files: Vec::new(),
                    };
                    let dir = app.services.documents_dir();
                    let _ = std::fs::create_dir_all(&dir);
                    let path = dir.join(format!("{}.vxs", crate::project::VoidXProject::sanitize_name(&name)));
                    match crate::project::save(&path, &project) {
                        Ok(()) => {
                            app.ui.project = Some(project);
                            app.ui.world_pan = egui::Vec2::ZERO;
                            app.ui.world_zoom = 1.0;
                            app.ui.project_workspace = None;
                            close_dialog = true;
                        }
                        Err(e) => app.toast(format!("No se pudo guardar .vxs: {e}")),
                    }
                }
                if ui.button("Cancel").clicked() {
                    close_dialog = true;
                }
            });
        });
    if open && !close_dialog {
        app.ui.project_dialog = Some(draft);
    }
}

fn c32(rgb: [u8; 3]) -> Color32 { Color32::from_rgb(rgb[0], rgb[1], rgb[2]) }

fn draw_adaptive_grid(painter: &egui::Painter, rect: Rect, pan: egui::Vec2, zoom: f32, dark: bool) {
    let mut step = 48.0 * zoom;
    if step <= 0.0001 { return; }
    while step < 12.0 { step *= 4.0; }
    let alpha = if dark { 18 } else { 18 };
    let grid_col = if dark { Color32::from_rgba_unmultiplied(255,255,255,alpha) } else { Color32::from_rgba_unmultiplied(0,0,0,alpha) };
    let ox = (pan.x % step + step) % step;
    let oy = (pan.y % step + step) % step;
    let mut x = rect.min.x + ox;
    while x < rect.max.x { painter.line_segment([pos2(x, rect.min.y), pos2(x, rect.max.y)], Stroke::new(1.0, grid_col)); x += step; }
    let mut y = rect.min.y + oy;
    while y < rect.max.y { painter.line_segment([pos2(rect.min.x, y), pos2(rect.max.x, y)], Stroke::new(1.0, grid_col)); y += step; }
}


pub(crate) fn project_view(app: &mut FigApp, ui: &mut Ui, t: &Tokens) {
    let Some(ws_idx) = app.ui.project_workspace else {
        world_space(app, ui, t);
        return;
    };
    group_workspace(app, ui, t, ws_idx);
}

fn world_space(app: &mut FigApp, ui: &mut Ui, t: &Tokens) {
    let rect = ui.max_rect();
    let resp = ui.interact(rect, ui.scope_id().with("world_space_canvas"), Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, if ui.visuals().dark_mode { Color32::from_rgb(0x16, 0x16, 0x16) } else { Color32::from_rgb(0xF3, 0xF3, 0xF3) });

    let mut pan = app.ui.world_pan;
    let mut zoom = app.ui.world_zoom.clamp(0.25, 4.0);
    let pointer_in = resp.hover_pos().unwrap_or(rect.center());
    let scroll = ui.input(|i| i.smooth_scroll_delta);
    let pinch = ui.input(|i| i.zoom_delta());
    if let Some(mt) = ui.input(|i| i.multi_touch()) {
        if (mt.zoom_delta - 1.0).abs() > 0.001 {
            let old = zoom;
            zoom = (zoom * mt.zoom_delta).clamp(0.25, 4.0);
            let world_before = (mt.start_pos - rect.min - pan) / old;
            pan = mt.start_pos - rect.min - world_before * zoom;
        }
        pan += mt.translation_delta;
    } else if resp.dragged() && app.ui.world_drag.is_none() {
        pan += resp.drag_delta();
    } else if resp.hovered() {
        if (pinch - 1.0).abs() > 0.001 {
            let old = zoom;
            zoom = (zoom * pinch).clamp(0.25, 4.0);
            let world_before = (pointer_in - rect.min - pan) / old;
            pan = pointer_in - rect.min - world_before * zoom;
        } else if scroll != egui::Vec2::ZERO {
            if ui.input(|i| i.modifiers.command || i.modifiers.ctrl) {
                let old = zoom;
                let factor = (scroll.y / 200.0).exp();
                zoom = (zoom * factor).clamp(0.25, 4.0);
                let world_before = (pointer_in - rect.min - pan) / old;
                pan = pointer_in - rect.min - world_before * zoom;
            } else {
                pan += scroll;
            }
        }
    }
    app.ui.world_pan = pan;
    app.ui.world_zoom = zoom;

    draw_adaptive_grid(&painter, rect, pan, zoom, ui.visuals().dark_mode);

    let title = app.ui.project.as_ref().map(|p| p.name.clone()).unwrap_or_else(|| "Project".to_string());
    painter.text(pos2(rect.min.x + 20.0, rect.min.y + 24.0), Align2::LEFT_CENTER, format!("World Space — {title}"), semibold(18.0), t.text);
    let back_rect = Rect::from_min_size(pos2(rect.min.x + 20.0, rect.min.y + 44.0), vec2(120.0, 30.0));
    let back_resp = ui.interact(back_rect, ui.scope_id().with("ws_back_home"), Sense::click());
    painter.rect_filled(back_rect, CornerRadius::same(6), if back_resp.hovered() { t.hover } else { t.field });
    painter.rect_stroke(back_rect, CornerRadius::same(6), Stroke::new(1.0, t.border), StrokeKind::Inside);
    painter.text(back_rect.center(), Align2::CENTER_CENTER, "← Home", regular(12.0), t.text);
    if back_resp.clicked() {
        save_project(app);
        app.ui.project = None;
        app.ui.project_workspace = None;
        app.ui.add_group_dialog = None;
        return;
    }

    let export_rect = Rect::from_min_size(pos2(rect.max.x - 220.0, rect.min.y + 18.0), vec2(200.0, 32.0));
    let export_resp = ui.interact(export_rect, ui.scope_id().with("ws_export_vxs"), Sense::click());
    painter.rect_filled(export_rect, CornerRadius::same(8), if export_resp.hovered() { Color32::from_rgb(0x1F,0xA8,0x6A) } else { Color32::from_rgb(0x0C,0xCE,0x87) });
    painter.text(export_rect.center(), Align2::CENTER_CENTER, "Exportar / Guardar .vxs", regular(12.0), Color32::BLACK);
    if export_resp.clicked() {
        export_project_vxs(app);
    }

    let card_w = 360.0 * zoom;
    let card_h = 110.0 * zoom;
    let mut open_ws: Option<usize> = None;
    let mut drag_idx = app.ui.world_drag;
    if !ui.input(|i| i.pointer.any_down()) { drag_idx = None; }

    let group_count = app.ui.project.as_ref().map(|p| p.files.len()).unwrap_or(0);
    for i in 0..group_count {
        let Some(project) = app.ui.project.as_ref() else { break };
        let g = project.files[i].clone();
        let center = pos2(rect.min.x + pan.x + g.x * zoom, rect.min.y + pan.y + g.y * zoom);
        let card_rect = Rect::from_center_size(center, vec2(card_w, card_h));
        let id = ui.scope_id().with(("ws_card", i, &g.name));
        let card_resp = ui.interact(card_rect, id, Sense::click_and_drag());
        if card_resp.drag_started() && !g.kind.anchored() { app.ui.world_drag = Some(i); drag_idx = Some(i); }
        if drag_idx == Some(i) && card_resp.dragged() && !g.kind.anchored() {
            let delta = card_resp.drag_delta() / zoom;
            if let Some(p) = app.ui.project.as_mut() { if let Some(gg) = p.files.get_mut(i) { gg.x += delta.x; gg.y += delta.y; } }
        }

        let style = g.kind.style();
        painter.rect_filled(card_rect, CornerRadius::same(10), c32(style.fill));
        painter.rect_stroke(card_rect, CornerRadius::same(10), Stroke::new(if card_resp.hovered() { 2.0 } else { 1.5 }, c32(style.stroke)), StrokeKind::Inside);
        let pad = 14.0 * zoom;
        let icon_size = 28.0 * zoom;
        let icon_rect = Rect::from_min_size(pos2(card_rect.min.x + pad, card_rect.min.y + 16.0 * zoom), vec2(icon_size, icon_size));
        icons::paint(&painter, icon_rect, g.kind.icon(), c32(style.accent));
        let text_x = icon_rect.max.x + 10.0 * zoom;
        painter.text(pos2(text_x, card_rect.min.y + 26.0 * zoom), Align2::LEFT_CENTER, crate::project::VoidXProject::display_kt(&g.name), semibold((16.0 * zoom).max(10.0)), Color32::WHITE);
        let badge_font = regular((11.0 * zoom).max(8.5));
        let badge_g = painter.layout_no_wrap(style.badge.to_string(), badge_font.clone(), c32(style.badge_text));
        let badge_rect = Rect::from_min_size(pos2(text_x, card_rect.min.y + 44.0 * zoom), vec2(badge_g.size().x + 18.0 * zoom, 22.0 * zoom));
        painter.rect_filled(badge_rect, CornerRadius::same((badge_rect.height() / 2.0).round().clamp(0.0, 255.0) as u8), c32(style.badge_fill));
        painter.text(badge_rect.center(), Align2::CENTER_CENTER, style.badge, badge_font, c32(style.badge_text));
        painter.text(pos2(text_x, card_rect.min.y + 74.0 * zoom), Align2::LEFT_CENTER, g.kind.description(), regular((11.5 * zoom).max(8.5)), Color32::from_rgba_unmultiplied(255, 255, 255, 170));

        let edit_font = regular((13.0 * zoom).max(9.0));
        let eg = painter.layout_no_wrap("Editar >".to_string(), edit_font.clone(), c32(style.accent));
        let er = Rect::from_min_size(pos2(card_rect.max.x - eg.size().x - 18.0*zoom, card_rect.max.y - 30.0*zoom), eg.size() + vec2(8.0*zoom, 6.0*zoom));
        let edit_resp = ui.interact(er, ui.scope_id().with(("ws_edit", i)), Sense::click());
        painter.text(er.left_top(), Align2::LEFT_TOP, "Editar >", edit_font, if edit_resp.hovered() { Color32::WHITE } else { c32(style.accent) });
        if edit_resp.clicked() { open_ws = Some(i); }
        if g.kind.anchored() {
            painter.text(pos2(card_rect.max.x - pad, card_rect.min.y + 20.0*zoom), Align2::RIGHT_CENTER, "PIN", regular((10.0*zoom).max(8.0)), t.text_secondary);
        }
    }
    app.ui.world_drag = drag_idx;
    if let Some(idx) = open_ws { open_group_workspace(app, idx); }

    let add_rect = Rect::from_min_size(pos2(rect.max.x - 144.0, rect.max.y - 68.0), vec2(120.0, 44.0));
    let add_resp = ui.interact(add_rect, ui.scope_id().with("ws_add_btn"), Sense::click());
    painter.rect_filled(add_rect, CornerRadius::same(10), if add_resp.hovered() { Color32::from_rgb(0x1F,0xA8,0x6A) } else { Color32::from_rgb(0x0C,0xCE,0x87) });
    painter.text(add_rect.center(), Align2::CENTER_CENTER, "+ Añadir", semibold(14.0), Color32::BLACK);
    if add_resp.clicked() {
        let default_kind = app.ui.project.as_ref().and_then(|p| crate::project::FileGroupKind::CREATE_ORDER.iter().copied().find(|k| p.can_add_kind(*k))).unwrap_or(crate::project::FileGroupKind::Ui);
        app.ui.add_group_dialog = Some(crate::project::AddGroupDraft { kind: default_kind, name: crate::project::VoidXProject::default_group_name(default_kind) });
    }
}

pub(crate) fn open_group_workspace(app: &mut FigApp, idx: usize) {
    let Some(project) = app.ui.project.as_ref() else { return };
    let Some(group) = project.files.get(idx).cloned() else { return };
    let name = crate::project::VoidXProject::sanitize_name(&group.name);
    let visual = group.visual.clone().unwrap_or_else(|| format!("{name}.figcraft"));
    if let Some(p) = app.ui.project.as_mut() {
        if let Some(g) = p.files.get_mut(idx) { g.visual = Some(visual.clone()); }
    }

    if group.kind.opens_figcraft() {
        let mut loaded = false;
        if let Some(bytes) = group.visual_data.as_ref() {
            let dir = app.services.documents_dir().join(".vxs_tmp");
            let _ = std::fs::create_dir_all(&dir);
            let tmp = dir.join(&visual);
            if std::fs::write(&tmp, bytes).is_ok() {
                loaded = app.run("file.open", json!({"path": tmp.to_string_lossy()})).is_some();
            }
        }
        if !loaded {
            let _ = app.run("file.new", json!({"name": group.name.clone()}));
        }
        app.ui.home = false;
    } else {
        app.ui.home = true; // clean workspace rendered by home/project_view
        app.ui.world_pan = egui::Vec2::ZERO;
        app.ui.world_zoom = 1.0;
    }
    app.ui.project_workspace = Some(idx);
}

fn add_group_dialog(app: &mut FigApp, ui: &mut Ui) {
    let Some(mut draft) = app.ui.add_group_dialog.take() else { return };
    let mut open = true;
    let mut close_dialog = false;
    let ctx = ui.ctx().clone();
    crate::dialogs::window("Añadir contenedor", &ctx)
        .id(egui::Id::unique("add_group_dialog"))
        .order(Order::Foreground)
        .open(&mut open)
        .show(&ctx, |ui| {
            let t = Tokens::get(ui.ctx());
            ui.set_width(crate::dialogs::dialog_width(ui.ctx(), 460.0));
            ui.label(egui::RichText::new("Tipo de contenedor").font(semibold(12.0)).color(t.text_secondary));
            ui.add_space(4.0);
            for kind in crate::project::FileGroupKind::CREATE_ORDER {
                let can_add = app.ui.project.as_ref().is_some_and(|p| p.can_add_kind(kind));
                let selected = draft.kind == kind;
                let style = kind.style();
                let label = if kind.unique() && !can_add { format!("{} (máx 1)", kind.label()) } else { kind.label().to_string() };
                let fill = if selected { c32(style.badge_fill) } else { t.field };
                let text_col = if selected { c32(style.badge_text) } else if can_add { t.text } else { t.text_tertiary };
                let btn = egui::Button::new(egui::RichText::new(format!("{label} — {}", kind.description())).size(12.0).color(text_col))
                    .fill(fill).stroke(Stroke::new(1.0, c32(style.stroke))).corner_radius(CornerRadius::same(6));
                let resp = ui.add_enabled(can_add, btn);
                if resp.clicked() && can_add {
                    let old_default = crate::project::VoidXProject::default_group_name(draft.kind);
                    let was_default = draft.name.trim().is_empty() || draft.name.trim() == old_default;
                    draft.kind = kind;
                    if was_default { draft.name = crate::project::VoidXProject::default_group_name(kind); }
                }
            }
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Nombre del archivo").font(semibold(12.0)).color(t.text_secondary));
            ui.add(egui::TextEdit::singleline(&mut draft.name).hint_text("main").desired_width(f32::INFINITY));
            let sanitized = crate::project::VoidXProject::sanitize_name(&draft.name);
            let duplicate = app.ui.project.as_ref().is_some_and(|p| p.files.iter().any(|f| crate::project::VoidXProject::sanitize_name(&f.name) == sanitized));
            ui.label(egui::RichText::new(format!("Se mostrará como: {}", crate::project::VoidXProject::display_kt(&draft.name))).size(11.0).color(t.text_secondary));
            if duplicate {
                ui.label(egui::RichText::new("Ya existe un archivo con ese nombre").size(11.0).color(t.warning));
            }
            ui.add_space(8.0);
            let can_create = !draft.name.trim().is_empty() && !duplicate && app.ui.project.as_ref().is_some_and(|p| p.can_add_kind(draft.kind));
            ui.horizontal(|ui| {
                let create = egui::Button::new(egui::RichText::new("Crear").color(if can_create { Color32::BLACK } else { t.text_tertiary }).strong())
                    .fill(if can_create { Color32::from_rgb(0x0C,0xCE,0x87) } else { t.field }).corner_radius(CornerRadius::same(6));
                if ui.add_enabled(can_create, create).clicked() {
                    if let Some(p) = app.ui.project.as_mut() {
                        let idx = p.files.len();
                        let x = 220.0 + (idx as f32) * 40.0;
                        let y = 180.0 + (idx as f32) * 40.0;
                        let name = crate::project::VoidXProject::sanitize_name(&draft.name);
                        let visual = matches!(draft.kind, crate::project::FileGroupKind::Ui | crate::project::FileGroupKind::Main).then(|| format!("{name}.figcraft"));
                        p.files.push(crate::project::FileGroup { name, kind: draft.kind, visual, logic: None, x, y, visual_data: None });
                    }
                    save_project(app);
                    close_dialog = true;
                }
                if ui.button("Cancel").clicked() { close_dialog = true; }
            });
        });
    if open && !close_dialog { app.ui.add_group_dialog = Some(draft); }
}

pub(crate) fn group_workspace(app: &mut FigApp, ui: &mut Ui, t: &Tokens, idx: usize) {
    let Some(project) = app.ui.project.as_ref() else { app.ui.project_workspace = None; return; };
    let Some(group) = project.files.get(idx).cloned() else { app.ui.project_workspace = None; return; };

    if group.kind == crate::project::FileGroupKind::Ui {
        // UI workspace uses normal FigCraft editor; draw a top overlay bar to return.
        let rect = ui.max_rect();
        let bar_h = 40.0;
        let bar_rect = Rect::from_min_size(rect.min, vec2(rect.width(), bar_h));
        let painter = ui.painter_at(bar_rect);
        painter.rect_filled(bar_rect, 0.0, t.panel);
        painter.rect_stroke(bar_rect, 0.0, Stroke::new(1.0, t.border), StrokeKind::Inside);
        painter.text(pos2(bar_rect.min.x + 16.0, bar_rect.center().y), Align2::LEFT_CENTER, format!("UI: {}.kt", group.name), semibold(13.0), t.text);
        let back = Rect::from_min_size(pos2(bar_rect.max.x - 220.0, bar_rect.min.y + 6.0), vec2(200.0, 28.0));
        let resp = ui.interact(back, ui.scope_id().with("ui_ws_back"), Sense::click());
        painter.rect_filled(back, CornerRadius::same(6), if resp.hovered() { Color32::from_rgb(0x1F,0xA8,0x6A) } else { Color32::from_rgb(0x0C,0xCE,0x87) });
        painter.text(back.center(), Align2::CENTER_CENTER, "Guardar y volver al World Space", regular(11.5), Color32::BLACK);
        if resp.clicked() {
            save_project(app);
            app.ui.project_workspace = None;
            app.ui.home = true;
        }
        // The actual editor is rendered by app frame; we only reserve no space here.
        return;
    }

    // Clean canvas for Main/Engine/Data/Network
    let rect = ui.max_rect();
    let resp = ui.interact(rect, ui.scope_id().with(("clean_ws", idx)), Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, if ui.visuals().dark_mode { Color32::from_rgb(0x18,0x18,0x18) } else { Color32::from_rgb(0xFA,0xFA,0xFA) });
    let mut pan = app.ui.world_pan;
    let mut zoom = app.ui.world_zoom.clamp(0.25, 4.0);
    let pointer_in = resp.hover_pos().unwrap_or(rect.center());
    let scroll = ui.input(|i| i.smooth_scroll_delta);
    let pinch = ui.input(|i| i.zoom_delta());
    if let Some(mt) = ui.input(|i| i.multi_touch()) {
        if (mt.zoom_delta - 1.0).abs() > 0.001 {
            let old = zoom; zoom = (zoom * mt.zoom_delta).clamp(0.25, 4.0);
            let before = (mt.start_pos - rect.min - pan) / old; pan = mt.start_pos - rect.min - before * zoom;
        }
        pan += mt.translation_delta;
    } else if resp.dragged() {
        pan += resp.drag_delta();
    } else if resp.hovered() {
        if (pinch - 1.0).abs() > 0.001 {
            let old = zoom; zoom = (zoom * pinch).clamp(0.25, 4.0);
            let before = (pointer_in - rect.min - pan) / old; pan = pointer_in - rect.min - before * zoom;
        } else if scroll != egui::Vec2::ZERO {
            if ui.input(|i| i.modifiers.command || i.modifiers.ctrl) {
                let old = zoom; let factor = (scroll.y / 200.0).exp(); zoom = (zoom * factor).clamp(0.25, 4.0);
                let before = (pointer_in - rect.min - pan) / old; pan = pointer_in - rect.min - before * zoom;
            } else { pan += scroll; }
        }
    }
    app.ui.world_pan = pan; app.ui.world_zoom = zoom;

    draw_adaptive_grid(&painter, rect, pan, zoom, ui.visuals().dark_mode);

    let style = group.kind.style();
    let header_h = 72.0;
    let header = Rect::from_min_size(rect.min, vec2(rect.width(), header_h));
    painter.rect_filled(header, 0.0, if ui.visuals().dark_mode { Color32::from_rgba_unmultiplied(20,20,20,220) } else { Color32::from_rgba_unmultiplied(255,255,255,220) });
    painter.line_segment([header.left_bottom(), header.right_bottom()], Stroke::new(1.0, t.border));
    painter.text(pos2(header.min.x + 20.0, header.min.y + 24.0), Align2::LEFT_CENTER, format!("{} — {}", group.kind.label(), crate::project::VoidXProject::display_kt(&group.name)), semibold(18.0), c32(style.accent));
    painter.text(pos2(header.min.x + 20.0, header.min.y + 48.0), Align2::LEFT_CENTER, "Canvas limpio (pan/zoom). Paleta de nodos: próximo hito.", regular(12.0), t.text_secondary);

    let back = Rect::from_min_size(pos2(header.max.x - 220.0, header.min.y + 20.0), vec2(200.0, 32.0));
    let back_resp = ui.interact(back, ui.scope_id().with(("clean_ws_back", idx)), Sense::click());
    painter.rect_filled(back, CornerRadius::same(8), if back_resp.hovered() { Color32::from_rgb(0x1F,0xA8,0x6A) } else { Color32::from_rgb(0x0C,0xCE,0x87) });
    painter.text(back.center(), Align2::CENTER_CENTER, "Guardar y volver al World Space", regular(12.0), Color32::BLACK);
    if back_resp.clicked() {
        save_project(app);
        app.ui.project_workspace = None;
        app.ui.home = true;
    }
}



pub(crate) fn active_workspace_kind(app: &FigApp) -> Option<crate::project::FileGroupKind> {
    let idx = app.ui.project_workspace?;
    let p = app.ui.project.as_ref()?;
    p.files.get(idx).map(|g| g.kind)
}

pub(crate) fn show_project_workspace(app: &mut FigApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    project_view(app, ui, &t);
}
fn capture_active_group_figcraft(app: &mut FigApp) {
    use base64::Engine as _;
    let Some(idx) = app.ui.project_workspace else { return };
    let Some(kind) = app.ui.project.as_ref().and_then(|p| p.files.get(idx).map(|g| g.kind)) else { return };
    if !kind.opens_figcraft() { return; }
    let Some(r) = app.run("file.saveAs", serde_json::json!({"data": true})) else { return };
    let Some(data_b64) = r.get("data").and_then(|v| v.as_str()) else { return };
    let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(data_b64) else { return };
    let Some(p) = app.ui.project.as_mut() else { return };
    if let Some(g) = p.files.get_mut(idx) {
        let name = crate::project::VoidXProject::sanitize_name(&g.name);
        g.visual = Some(format!("{name}.figcraft"));
        g.visual_data = Some(bytes);
    }
}


fn export_project_vxs(app: &mut FigApp) {
    save_project(app);
    let Some(project) = app.ui.project.clone() else { return };
    let name = format!("{}.vxs", crate::project::VoidXProject::sanitize_name(&project.name));
    let src = app.services.documents_dir().join(&name);
    let download = std::path::PathBuf::from("/storage/emulated/0/Download").join(&name);
    let mut out = download.clone();
    let target = if std::fs::write(&download, std::fs::read(&src).unwrap_or_default()).is_ok() {
        Some(download)
    } else {
        // fallback: keep in documents dir and tell user
        out = src.clone();
        None
    };
    if let Some(p) = target {
        app.toast(format!("Proyecto exportado: {}", p.display()));
    } else {
        app.toast(format!("Guardado en: {}", out.display()));
    }
}

fn vxs_project_files(app: &FigApp) -> Vec<std::path::PathBuf> {
    let mut dirs = vec![app.services.documents_dir()];
    dirs.push(std::path::PathBuf::from("/storage/emulated/0/VoidXStudio"));
    let mut out = Vec::new();
    for dir in dirs {
        let Ok(rd) = std::fs::read_dir(dir) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("vxs")) {
                out.push(p);
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

fn vxs_recents_section(app: &mut FigApp, ui: &mut Ui, t: &Tokens, query: &str) {
    let mut files = vxs_project_files(app);
    if files.is_empty() { return; }
    files.sort_by(|a,b| b.metadata().and_then(|m| m.modified()).ok().cmp(&a.metadata().and_then(|m| m.modified()).ok()));
    subheading(ui, t, "Proyectos VoidX (.vxs)");
    for p in files.into_iter().take(12) {
        let name = p.file_stem().and_then(|s| s.to_str()).unwrap_or("proyecto").to_string();
        let loc = p.parent().and_then(|p| p.to_str()).unwrap_or("").to_string();
        if !query.is_empty() && !file_matches(&name, &loc, query) { continue; }
        let row = FileRow {
            mark: Some((Icon::Assets, Color32::from_rgb(0x0C,0xCE,0x87))),
            name: &name,
            ink: t.text,
            detail: &loc,
            detail_size: 11.0,
            detail_ink: t.text_secondary,
        };
        if listed(ui, t, ui.scope_id().with(("vxs_recent", &loc, &name)), 36.0, &row) {
            match crate::project::load(&p) {
                Ok(project) => {
                    app.ui.project = Some(project);
                    app.ui.home = true;
                    app.ui.project_workspace = None;
                    app.ui.world_pan = egui::Vec2::ZERO;
                    app.ui.world_zoom = 1.0;
                }
                Err(e) => app.toast(format!("No se pudo abrir .vxs: {e}")),
            }
        }
    }
    ui.add_space(8.0);
}
pub(crate) fn save_project(app: &mut FigApp) {
    capture_active_group_figcraft(app);
    let Some(project) = app.ui.project.clone() else { return };
    let dir = app.services.documents_dir();
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(format!("{}.vxs", crate::project::VoidXProject::sanitize_name(&project.name)));
    if let Err(e) = crate::project::save(&path, &project) {
        app.toast(format!("No se pudo guardar .vxs: {e}"));
    }
}

/// Files dropped on the home screen: each one the app opens becomes a document of its own (the
/// editor then shows it), behind an "Opening as a new file" card ([`crate::pending_open`]); anything
/// else is turned away with a toast. A file that fails to open keeps the engine's own error.
fn dropped(app: &mut FigApp, ctx: &egui::Context) {
    let files = ctx.input(|i| i.raw.dropped_files.clone());
    for f in files {
        if !crate::opens(f.path()) {
            app.toast(tr("Drop a .figcraft, .fig, .jam, .sketch, .json, .svg or image file."));
            continue;
        }
        crate::pending_open::open_dropped(app, &f);
    }
}

/// Draw the Home sidebar: app icon and wordmark, file/template search (Escape clears it)
/// and section tabs; choosing a section clears the file query.
fn sidebar(app: &mut FigApp, ui: &mut Ui, t: &Tokens, left: Rect) {
    let mut y = left.min.y + 20.0;
    crate::theme::paint_app_icon(ui, Rect::from_min_size(pos2(left.min.x + 12.0, y - 4.0), vec2(32.0, 32.0)));
    crate::theme::paint_wordmark(ui.painter(), pos2(left.min.x + 52.0, y + 12.0), 16.0, t);
    y += 52.0;
    let search = Rect::from_min_size(pos2(left.min.x + 8.0, y), vec2(SIDEBAR - 16.0, widgets::FIELD_H));
    ui.scope_builder(egui::UiBuilder::new().max_rect(search), |ui| {
        let response = widgets::text_input(
            ui,
            Id::unique("home_file_search"),
            &mut app.ui.home_view.file_query,
            search.width(),
            tr("Search files and templates"),
        );
        a11y::text(&response, tr("Search files and templates"), &app.ui.home_view.file_query);
        if (response.has_focus() || response.lost_focus()) && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            app.ui.home_view.file_query.clear();
        }
    });
    y += widgets::FIELD_H + 12.0;
    for (section, icon, label) in Section::ALL {
        let r = Rect::from_min_size(pos2(left.min.x + 8.0, y), vec2(SIDEBAR - 16.0, 32.0));
        let resp = ui.interact(r, ui.scope_id().with(("home_section", label)), Sense::click());
        let current = app.ui.home_view.file_query.trim().is_empty() && app.ui.home_view.section == section;
        a11y::tab(&resp, tr(label), current);
        if current || resp.hovered() {
            ui.painter().rect_filled(r, CornerRadius::same(5), if current { t.selected_row } else { t.hover });
        }
        focus_ring(ui, t, r, &resp, 5);
        icons::paint(ui.painter(), Rect::from_min_size(pos2(r.min.x + 10.0, r.center().y - 8.0), vec2(16.0, 16.0)), icon, t.icon);
        ui.painter().text(pos2(r.min.x + 36.0, r.center().y), Align2::LEFT_CENTER, tr(label), regular(12.0), t.text);
        if resp.clicked() {
            app.ui.home_view.section = section;
            app.ui.home_view.file_query.clear();
        }
        y += 34.0;
    }
}

fn file_matches(name: &str, location: &str, query: &str) -> bool {
    name.to_lowercase().contains(query) || location.to_lowercase().contains(query)
}

/// One search over known files and starter templates. It never sends a query or owner token to a
/// server; online files are opened through their existing owner-token command only on click.
fn search_results(app: &mut FigApp, ui: &mut Ui, t: &Tokens) {
    heading(ui, t, tr("Search results"));
    let query = app.ui.home_view.file_query.trim().to_lowercase();
    let mut count = open_files(app, ui, t, &query);
    count += crate::home_recents::section(app, ui, t, &query);
    vxs_recents_section(app, ui, t, &query);

    let online: Vec<_> = app.session.prefs.online_files.iter().filter(|f| file_matches(&f.name, &f.server, &query)).cloned().collect();
    let mut open_online = None;
    if !online.is_empty() {
        subheading(ui, t, tr("Online files"));
        for f in &online {
            count += 1;
            let host = figcraft_engine::online::server_host(&f.server);
            let icon = crate::online::mark(f.shared.is_some());
            let row = FileRow {
                mark: Some((icon, if f.shared.is_some() { t.success } else { t.icon_secondary })),
                name: &f.name,
                ink: t.text,
                detail: host,
                detail_size: 11.0,
                detail_ink: t.text_secondary,
            };
            if listed(ui, t, ui.scope_id().with(("search_online", &f.server, &f.file)), 36.0, &row) {
                open_online = Some(f.clone());
            }
        }
    }
    let templates: Vec<&Template> = TEMPLATES.iter().filter(|template| matches(template, &query, None)).collect();
    if !templates.is_empty() {
        subheading(ui, t, tr("Templates"));
        template_grid(app, ui, t, &templates);
    }
    if count == 0 && templates.is_empty() {
        ui.label(egui::RichText::new(tr("No files or templates match your search")).font(semibold(13.0)).color(t.text));
        ui.add_space(6.0);
        widgets::label(ui, tr("Try another search, or browse templates."), t.text_secondary);
        ui.add_space(16.0);
        if widgets::button(ui, tr("Show all templates"), false, true).clicked() {
            app.ui.home_view.file_query.clear();
            app.ui.home_view.section = Section::Templates;
        }
    }
    if let Some(file) = open_online {
        crate::online::open(app, &file);
    }
}

/// A ring around a card or row that has the keyboard.
fn focus_ring(ui: &Ui, t: &Tokens, rect: Rect, resp: &Response, radius: u8) {
    if resp.has_focus() {
        ui.painter().rect_stroke(rect, CornerRadius::same(radius), Stroke::new(2.0, t.focus), StrokeKind::Outside);
    }
}

/// A card's background and border (lit on hover, ringed with the keyboard on it).
fn card_frame(ui: &Ui, t: &Tokens, rect: Rect, resp: &Response) {
    ui.painter().rect_filled(rect, CornerRadius::same(8), if resp.hovered() { t.hover } else { t.panel });
    ui.painter().rect_stroke(rect, CornerRadius::same(8), Stroke::new(1.0, t.border), StrokeKind::Inside);
    focus_ring(ui, t, rect, resp, 8);
}

fn heading(ui: &mut Ui, t: &Tokens, text: &str) {
    ui.label(egui::RichText::new(text).font(semibold(18.0)).color(t.text));
    ui.add_space(24.0);
}

fn subheading(ui: &mut Ui, t: &Tokens, text: &str) {
    ui.add_space(28.0);
    ui.label(egui::RichText::new(text).font(semibold(12.0)).color(t.text_secondary));
    ui.add_space(12.0);
}

/// A card with an icon tile, a title and a line under it; true when chosen.
fn action_card(ui: &mut Ui, t: &Tokens, icon: Icon, title: &str, sub: &str) -> bool {
    let (r, mut resp) = ui.allocate_exact_size(vec2(260.0, 72.0), Sense::click());
    a11y::button(&resp, title);
    // One line each inside the card; a long translation ends with an ellipsis, in full on hover.
    let text_w = r.width() - 72.0;
    let (title_shown, sub_shown) = (widgets::elide_font(ui, title, text_w, semibold(12.0)), widgets::elide(ui, sub, text_w));
    if title_shown != title || sub_shown != sub {
        resp = resp.on_hover_text(format!("{title}\n{sub}"));
    }
    card_frame(ui, t, r, &resp);
    let ir = Rect::from_min_size(pos2(r.min.x + 16.0, r.center().y - 16.0), vec2(32.0, 32.0));
    ui.painter().rect_filled(ir, CornerRadius::same(6), t.accent);
    icons::paint(ui.painter(), ir.shrink(8.0), icon, t.accent_on);
    ui.painter().text(pos2(r.min.x + 60.0, r.center().y - 8.0), Align2::LEFT_CENTER, title_shown, semibold(12.0), t.text);
    ui.painter().text(pos2(r.min.x + 60.0, r.center().y + 9.0), Align2::LEFT_CENTER, sub_shown, regular(11.0), t.text_secondary);
    resp.clicked()
}

fn recents(app: &mut FigApp, ui: &mut Ui, t: &Tokens) {
    heading(ui, t, tr("Recents"));
    let (mut new_file, mut new_board, mut open, mut create_project) = (false, false, false, false);
    ui.horizontal_wrapped(|ui| {
        new_file = action_card(ui, t, Icon::PenNib, tr("New design file"), tr("Start from a blank canvas"));
        new_board = action_card(ui, t, Icon::Board, tr("New board"), tr("Sticky notes, shapes and connectors"));
        open = action_card(ui, t, Icon::Page, tr("Open file"), tr("Open .figcraft, .svg or images"));
        create_project = action_card(ui, t, Icon::Layers, tr("Create Project"), tr("A .vxs project for an app"));
    });
    if new_file || new_board {
        app.run(if new_board { "file.newBoard" } else { "file.new" }, json!({}));
        app.ui.home = false;
    }
    if open {
        app.open_dialog();
    }
    if create_project {
        app.ui.project_dialog = Some(crate::project::ProjectDraft::default());
    }
    // Join a file someone shared (the Join dialog shows what the join is doing).
    subheading(ui, t, tr("Join with a code"));
    ui.scope(|ui| {
        ui.set_max_width(540.0);
        crate::live::join_row(app, ui, 260.0);
    });
    opened_join(app);
    if !app.session.prefs.getting_started_dismissed {
        ui.add_space(28.0);
        getting_started(app, ui, t);
    }
    open_files(app, ui, t, "");
    if crate::home_recents::listed(&app.session.prefs.recent_files).next().is_none() {
        // A first run: nothing to reopen yet, so offer something to start from.
        subheading(ui, t, tr("Start from a template"));
        let shown: Vec<&Template> = TEMPLATES.iter().take(FIRST_RUN).collect();
        template_grid(app, ui, t, &shown);
        ui.add_space(12.0);
        if widgets::button(ui, tr("See all templates"), false, true).clicked() {
            app.ui.home_view.section = Section::Templates;
        }
        return;
    }
    crate::home_recents::section(app, ui, t, "");
    vxs_recents_section(app, ui, t, "");
}

/// The Open files list: the documents whose name or path has `query` (lower case; all for ""), a
/// row each; choosing one shows its tab. How many are listed.
fn open_files(app: &mut FigApp, ui: &mut Ui, t: &Tokens, query: &str) -> usize {
    let hit = |d: &&figcraft_engine::DocState| file_matches(&d.doc.name, d.path.as_deref().unwrap_or_default(), query);
    let count = app.session.docs.iter().filter(hit).count();
    if count == 0 {
        return 0;
    }
    subheading(ui, t, tr("Open files"));
    let mut switch = None;
    for (i, d) in app.session.docs.iter().enumerate().filter(|(_, d)| hit(d)) {
        let detail = d.path.as_deref().unwrap_or_default();
        let row = FileRow {
            mark: Some((icons::file_mark(d.doc.kind), ACCENT)),
            name: &d.doc.name,
            ink: t.text,
            detail,
            detail_size: 11.0,
            detail_ink: t.text_secondary,
        };
        if listed(ui, t, ui.scope_id().with(("open", d.uid)), 36.0, &row) {
            switch = Some(i);
        }
    }
    if let Some(i) = switch {
        app.run("doc.switch", json!({"index": i}));
        app.ui.home = false;
    }
    count
}

/// The file joined from the home screen (a code, or one of your online files) opened in a tab of
/// its own: show it. (A join that was left is forgotten.)
fn opened_join(app: &mut FigApp) {
    match app.ui.live.home_join.map(|id| app.session.live.get(id).map(figcraft_engine::live::Live::syncing)) {
        Some(Some(true)) => {
            app.ui.live.home_join = None;
            app.ui.home = false;
        }
        Some(None) => app.ui.live.home_join = None,
        _ => {}
    }
}

/// The Getting started card: short tips with their shortcuts as the keymap has them now, and a
/// button that puts the card away (Help ▸ Getting started brings it back).
fn getting_started(app: &mut FigApp, ui: &mut Ui, t: &Tokens) {
    let width = ui.available_width().min(3.0 * CARD_W + 2.0 * GAP);
    let columns = if width >= 720.0 { 3 } else { 1 };
    let (cell_w, cell_h) = ((width - 32.0) / columns as f32, 62.0);
    let rows = TIPS.len().div_ceil(columns);
    let (rect, _) = ui.allocate_exact_size(vec2(width, 52.0 + rows as f32 * cell_h + 8.0), Sense::hover());
    a11y::container_bounds(ui, rect);
    ui.painter().rect_filled(rect, CornerRadius::same(8), t.field);
    ui.painter().text(rect.min + vec2(16.0, 24.0), Align2::LEFT_CENTER, tr("Getting started"), semibold(13.0), t.text);
    let close = Rect::from_min_size(pos2(rect.max.x - 36.0, rect.min.y + 12.0), vec2(24.0, 24.0));
    if widgets::icon_button_at(
        ui,
        ui.scope_id().with("tips_close"),
        close,
        Icon::Close,
        tr("Hide these tips (Getting started in the Help menu shows them again)"),
    ) {
        app.run("help.gettingStarted", json!({"show": false}));
        app.services.save_prefs(&app.session.prefs);
    }
    for (i, (title, line, command)) in TIPS.into_iter().enumerate() {
        let cell = Rect::from_min_size(
            rect.min + vec2(16.0 + (i % columns) as f32 * cell_w, 48.0 + (i / columns) as f32 * cell_h),
            vec2(cell_w - 16.0, cell_h),
        );
        let badge = Rect::from_min_size(cell.min + vec2(0.0, 2.0), vec2(20.0, 20.0));
        ui.painter().rect_filled(badge, CornerRadius::same(10), t.panel);
        ui.painter().text(badge.center(), Align2::CENTER_CENTER, (i + 1).to_string(), semibold(10.0), t.text_secondary);
        let x = cell.min.x + 30.0;
        let title_end = ui.painter().text(pos2(x, cell.min.y + 12.0), Align2::LEFT_CENTER, tr(title), semibold(12.0), t.text).max.x;
        if let Some(key) = shortcut(command) {
            ui.painter().text(pos2(title_end + 8.0, cell.min.y + 12.0), Align2::LEFT_CENTER, key, regular(11.0), t.text_tertiary);
        }
        let galley = lines(ui, tr(line), regular(11.0), t.text_secondary, cell.max.x - x, 2);
        ui.painter().galley(pos2(x, cell.min.y + 24.0), galley, t.text_secondary);
    }
}

/// A command's shortcut as this platform writes it, as the keymap has it now.
pub(crate) fn shortcut(command: &str) -> Option<String> {
    crate::shortcuts::shortcut(command).map(|key| figcraft_engine::keymap::display(key, crate::shortcuts::platform()))
}

/// `text` wrapped at `width`, at most `rows` lines (the last one ends with an ellipsis).
fn lines(ui: &Ui, text: &str, font: egui::FontId, color: Color32, width: f32, rows: usize) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::simple(text.to_string(), font, color, width.max(1.0));
    job.wrap.max_rows = rows;
    ui.fonts_mut(|f| f.layout_job(job))
}

fn matches(template: &Template, query: &str, kind: Option<Kind>) -> bool {
    let q = query.trim().to_lowercase();
    let has = |s: &str| s.to_lowercase().contains(&q);
    kind.is_none_or(|k| k == template.kind)
        && (q.is_empty()
            || [template.name, template.description, template.kind.label()].into_iter().any(|s| has(s) || has(tr(s)))
            || has(template.size))
}

/// Draw template search and kind filter, show no-match guidance and a "Show all templates"
/// button that clears the query and kind, and lay matching templates out in a grid.
fn templates(app: &mut FigApp, ui: &mut Ui, t: &Tokens) {
    heading(ui, t, tr("Templates"));
    ui.horizontal(|ui| {
        let search = widgets::text_input(ui, Id::unique("home_template_search"), &mut app.ui.home_view.query, 260.0, tr("Search templates"));
        a11y::text(&search, tr("Search templates"), &app.ui.home_view.query);
        let labels: Vec<&str> = std::iter::once(tr("All")).chain(Kind::ALL.iter().map(|k| tr(k.label()))).collect();
        let active = app.ui.home_view.kind.and_then(|k| Kind::ALL.iter().position(|x| *x == k)).map_or(0, |i| i + 1);
        ui.scope(|ui| {
            ui.set_max_width((labels.len() as f32 * 124.0).min(ui.available_width()));
            if let Some(i) = widgets::segmented_text(ui, &labels, active) {
                app.ui.home_view.kind = i.checked_sub(1).and_then(|i| Kind::ALL.get(i).copied());
            }
        });
    });
    ui.add_space(24.0);
    let (query, kind) = (app.ui.home_view.query.clone(), app.ui.home_view.kind);
    let shown: Vec<&Template> = TEMPLATES.iter().filter(|x| matches(x, &query, kind)).collect();
    if shown.is_empty() {
        ui.add_space(24.0);
        ui.label(egui::RichText::new(tr("No templates match your search")).font(semibold(13.0)).color(t.text));
        ui.add_space(6.0);
        ui.label(egui::RichText::new(tr("Try another word, or look at every kind.")).font(regular(12.0)).color(t.text_secondary));
        ui.add_space(14.0);
        if widgets::button(ui, tr("Show all templates"), false, true).clicked() {
            app.ui.home_view.query.clear();
            app.ui.home_view.kind = None;
        }
        return;
    }
    template_grid(app, ui, t, &shown);
}

/// Template cards in as many columns as fit; choosing one opens a new document made from it.
fn template_grid(app: &mut FigApp, ui: &mut Ui, t: &Tokens, list: &[&Template]) {
    let columns = (((ui.available_width() + GAP) / (CARD_W + GAP)).floor() as usize).max(1);
    let dark = ui.visuals().dark_mode;
    let mut thumbs: Thumbs = ui.data(|d| d.get_temp(Id::unique("template_thumbs"))).unwrap_or_default();
    let mut chosen = None;
    let mut rendered = false;
    for row in list.chunks(columns) {
        ui.horizontal(|ui| {
            for template in row {
                let key = (template.id, dark);
                if !thumbs.0.contains_key(&key) && ui.is_rect_visible(Rect::from_min_size(ui.cursor().min, vec2(CARD_W, CARD_H))) {
                    if rendered {
                        // One a frame keeps the screen moving; the rest follow.
                        ui.ctx().request_repaint();
                    } else {
                        thumbs.0.insert(key, thumbnail(ui.ctx(), template, app.session.new_page_background()));
                        rendered = true;
                    }
                }
                if template_card(ui, t, template, thumbs.0.get(&key).and_then(Option::as_ref)) {
                    chosen = Some(*template);
                }
            }
        });
        ui.add_space(GAP);
    }
    ui.data_mut(|d| d.insert_temp(Id::unique("template_thumbs"), thumbs));
    if let Some(template) = chosen
        && app.run("file.newFromTemplate", json!({"template": template.id, "name": tr(template.name)})).is_some()
    {
        app.ui.home = false;
    }
}

fn template_card(ui: &mut Ui, t: &Tokens, template: &Template, thumb: Option<&Thumb>) -> bool {
    let (r, resp) = ui.allocate_exact_size(vec2(CARD_W, CARD_H), Sense::click());
    let (name, description) = (tr(template.name), tr(template.description));
    a11y::button(&resp, &format!("{name}: {description}"));
    card_frame(ui, t, r, &resp);
    let picture = Rect::from_min_size(r.min, vec2(CARD_W, THUMB_H)).shrink(1.0);
    let top = CornerRadius { nw: 7, ne: 7, sw: 0, se: 0 };
    match thumb {
        Some(thumb) => {
            ui.painter().rect_filled(picture, top, thumb.page);
            let size = thumb.texture.size_vec2();
            let inner = picture.shrink(10.0);
            let fit = (inner.width() / size.x).min(inner.height() / size.y);
            let image = Rect::from_center_size(inner.center(), size * fit);
            ui.painter().image(thumb.texture.id(), image, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
        }
        None => {
            ui.painter().rect_filled(picture, top, t.field);
            icons::paint(ui.painter(), Rect::from_center_size(picture.center(), vec2(20.0, 20.0)), Icon::Frame, t.icon_secondary);
        }
    }
    ui.painter().line_segment([picture.left_bottom(), picture.right_bottom()], Stroke::new(1.0, t.border));
    let (x, w) = (r.min.x + 14.0, CARD_W - 28.0);
    let y = picture.max.y + 20.0;
    ui.painter().text(pos2(x, y), Align2::LEFT_CENTER, widgets::elide_font(ui, name, w, semibold(12.0)), semibold(12.0), t.text);
    let galley = lines(ui, description, regular(11.0), t.text_secondary, w, 2);
    ui.painter().galley(pos2(x, y + 11.0), galley, t.text_secondary);
    let tag = format!("{} · {}", tr(template.kind.label()), template.size);
    ui.painter().text(pos2(x, r.max.y - 15.0), Align2::LEFT_CENTER, widgets::elide_font(ui, &tag, w, regular(10.0)), regular(10.0), t.text_tertiary);
    resp.clicked()
}

/// A template's picture: its first page's frames, and the page colour they sit on.
#[derive(Clone)]
struct Thumb {
    texture: egui::TextureHandle,
    page: Color32,
}

/// Pictures by template and theme (the page colour follows the theme); `None`: nothing to show.
#[derive(Clone, Default)]
struct Thumbs(BTreeMap<(&'static str, bool), Option<Thumb>>);

/// Build `template` and render the top of its first page to fit a card.
fn thumbnail(ctx: &egui::Context, template: &Template, background: figcraft_color::Color) -> Option<Thumb> {
    let doc = template.build(background).ok()?;
    let frames = doc.node(doc.first_page()?)?.children.clone();
    let bounds = frames.iter().filter_map(|id| figcraft_render::export_bounds(&doc, *id)).reduce(|a, b| a.union(b))?;
    if !(bounds.width() > 0.0 && bounds.height() > 0.0) {
        return None;
    }
    // A page much taller than a card shows its top, as wide as the card.
    let (w, h) = (f64::from(CARD_W - 22.0), f64::from(THUMB_H - 22.0));
    let area = figcraft_geom::Rect::new(bounds.x0, bounds.y0, bounds.x1, bounds.y1.min(bounds.y0 + bounds.width() * h / w * 1.5));
    let scale = (w / area.width()).min(h / area.height()) * f64::from(ctx.pixels_per_point());
    let pix = figcraft_render::render_area(&doc, &frames, area, scale.min(1.0))?;
    let image = crate::render_worker::screen_image(&pix, doc.color_profile);
    let texture = ctx.load_texture(format!("template-thumb-{}", template.id), image, egui::TextureOptions::LINEAR);
    Some(Thumb { texture, page: crate::theme::c32(background) })
}

/// Tests: the templates whose pictures are cached for the light or dark theme.
#[cfg(test)]
pub(crate) fn cached_thumbnails(ctx: &egui::Context, dark: bool) -> Vec<&'static str> {
    let thumbs: Thumbs = ctx.data(|d| d.get_temp(Id::unique("template_thumbs"))).unwrap_or_default();
    thumbs.0.iter().filter(|((_, d), thumb)| *d == dark && thumb.is_some()).map(|((id, _), _)| *id).collect()
}
