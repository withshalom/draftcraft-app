//! The navigation overlays on the drawing area: the ViewCube (Top face, Home, compass ring and
//! the WCS menu) and the UCS icon.
//!
//! Every overlay registers a widget that senses clicks *and* drags, so egui hands the pointer to
//! the overlay instead of the canvas underneath (a click on the ViewCube must never also pick or
//! start a selection window in the drawing). What the overlays do goes through commands:
//! `zoom.extents` (Top face, Home), `ui.toggle.viewcube`, `ui.toggle.ucsicon`.
//!
//! RustDraft's model view is a 2D plan view of the World Coordinate System: there is no view
//! twist, no 3D view direction and no user coordinate system yet, so the compass ring explains
//! that instead of rotating, and the WCS menu lists the only coordinate system there is.

use egui::{Color32, CursorIcon, Pos2, Rect, Sense, Stroke, pos2, vec2};
use serde_json::json;

use crate::CadApp;
use crate::theme::Tokens;

/// Shown when the compass ring is clicked or dragged.
pub const NO_TWIST: &str = "ViewCube: rotating the view needs a view twist, which the 2D plan view doesn't support yet.";

/// Shown when an edge, corner, arrow or roll arrow of the cube is clicked.
pub const NO_3D: &str = "ViewCube: other views of the cube need 3D views, which the 2D plan (Top) view doesn't support yet.";

/// Where the ViewCube's parts are on the canvas.
#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub center: Pos2,
    /// Radius of the compass ring's centre line.
    pub ring: f32,
    /// The cube as seen from above: the Top face with its chamfered edges and corners around it.
    pub cube: Rect,
    /// The Top face (the cube without its edges).
    pub face: Rect,
    pub home: Rect,
    /// The "WCS" coordinate-system menu under the cube.
    pub pill: Rect,
    /// Everything that belongs to the cube (the area kept away from the canvas).
    pub bounds: Rect,
}

/// Width of the cube's chamfered edges as seen from above.
const BEVEL: f32 = 9.0;
/// Half the width of the compass ring's band.
const BAND: f32 = 7.0;

impl Layout {
    pub fn new(canvas: Rect) -> Self {
        let center = pos2(canvas.right() - 92.0, canvas.top() + 82.0);
        let ring = 58.0;
        let cube = Rect::from_center_size(center, vec2(56.0, 56.0));
        Layout {
            center,
            ring,
            cube,
            face: cube.shrink(BEVEL),
            home: Rect::from_center_size(center + vec2(-ring + 2.0, -ring + 2.0), vec2(20.0, 20.0)),
            pill: Rect::from_center_size(center + vec2(0.0, ring + 26.0), vec2(56.0, 16.0)),
            bounds: Rect::from_center_size(center, vec2(2.0 * (ring + 10.0), 2.0 * (ring + 10.0))),
        }
    }

    /// The four arrows that point at the Top face from beyond its edges (N, E, S, W), each as
    /// its three corners.
    pub fn arrows(&self) -> [[Pos2; 3]; 4] {
        let c = self.center;
        let (near, far, half) = (self.cube.width() / 2.0 + 4.0, self.cube.width() / 2.0 + 13.0, 7.0);
        [
            [c + vec2(0.0, -near), c + vec2(-half, -far), c + vec2(half, -far)],
            [c + vec2(near, 0.0), c + vec2(far, -half), c + vec2(far, half)],
            [c + vec2(0.0, near), c + vec2(half, far), c + vec2(-half, far)],
            [c + vec2(-near, 0.0), c + vec2(-far, half), c + vec2(-far, -half)],
        ]
    }

    /// The roll arrows' hit area, above and right of the cube.
    pub fn roll(&self) -> Rect {
        Rect::from_center_size(self.cube.right_top() + vec2(10.0, -10.0), vec2(20.0, 20.0))
    }

    /// The part of the cube at `p` (not the pill, which is its own widget).
    pub fn part_at(&self, p: Pos2) -> Option<Part> {
        if self.home.contains(p) {
            return Some(Part::Home);
        }
        if self.face.contains(p) {
            return Some(Part::Face);
        }
        if self.cube.contains(p) {
            let left = p.x < self.face.left();
            let right = p.x > self.face.right();
            let top = p.y < self.face.top();
            let bottom = p.y > self.face.bottom();
            return Some(if (left || right) && (top || bottom) { Part::Corner } else { Part::Edge });
        }
        if self.roll().contains(p) {
            return Some(Part::Roll);
        }
        if self.arrows().iter().any(|a| Rect::from_points(a).expand(2.0).contains(p)) {
            return Some(Part::Arrow);
        }
        let d = p.distance(self.center);
        ((self.ring - 10.0..=self.ring + 10.0).contains(&d)).then_some(Part::Ring)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Face,
    Edge,
    Corner,
    Arrow,
    Roll,
    Home,
    Ring,
}

impl Part {
    fn tip(self) -> &'static str {
        match self {
            Part::Face => "Top: plan view of the drawing (zoom extents)",
            Part::Edge | Part::Corner => "Edge and corner views need 3D views, which aren't available yet",
            Part::Arrow | Part::Roll => "Rotating to another face needs 3D views, which aren't available yet",
            Part::Home => "Home: zoom to the drawing extents",
            Part::Ring => "Compass: view rotation isn't available in the 2D plan view yet",
        }
    }
}

/// Colours of the cube, drawn in code. `a` fades the whole cube when the pointer is away.
struct Shades {
    face_top: Color32,
    face_bottom: Color32,
    lit_edge: Color32,
    shaded_edge: Color32,
    outline: Color32,
    label: Color32,
    ring: Color32,
    ring_line: Color32,
    letters: Color32,
    hot: Color32,
}

impl Shades {
    fn new(a: f32) -> Self {
        let c = |r, g, b| Color32::from_rgb(r, g, b).gamma_multiply(a);
        Shades {
            face_top: c(0xf2, 0xf4, 0xf6),
            face_bottom: c(0xcf, 0xd3, 0xd8),
            lit_edge: c(0xdd, 0xe0, 0xe4),
            shaded_edge: c(0xa6, 0xab, 0xb2),
            outline: c(0x5a, 0x60, 0x68),
            label: c(0x3a, 0x3f, 0x46),
            ring: Color32::from_rgba_unmultiplied(0xc6, 0xca, 0xd0, (170.0 * a) as u8),
            ring_line: c(0x6e, 0x75, 0x7e),
            letters: Color32::from_rgb(0x22, 0x26, 0x2c).gamma_multiply(a.max(0.85)),
            hot: Color32::from_rgba_unmultiplied(0x6c, 0xb4, 0xff, (200.0 * a) as u8),
        }
    }
}

/// A quadrilateral filled with one colour per corner (a gradient).
fn quad(p: &egui::Painter, pts: [Pos2; 4], cols: [Color32; 4]) {
    let mut mesh = egui::Mesh::default();
    for (pt, col) in pts.iter().zip(cols) {
        mesh.colored_vertex(*pt, col);
    }
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    p.add(egui::Shape::mesh(mesh));
}

/// Points round a circle, enough that a thick ring stays round.
fn circle_pts(c: Pos2, r: f32) -> Vec<Pos2> {
    (0..96)
        .map(|k| {
            let a = std::f32::consts::TAU * k as f32 / 96.0;
            c + vec2(a.cos(), a.sin()) * r
        })
        .collect()
}

/// The compass ring: a translucent band with a line at each edge and the four letters on it.
fn paint_ring(p: &egui::Painter, l: &Layout, s: &Shades, hot: bool) {
    let c = l.center;
    let band = if hot { s.hot } else { s.ring };
    p.add(egui::Shape::closed_line(circle_pts(c, l.ring), Stroke::new(2.0 * BAND, band)));
    p.add(egui::Shape::closed_line(circle_pts(c, l.ring - BAND), Stroke::new(1.0, s.ring_line)));
    p.add(egui::Shape::closed_line(circle_pts(c, l.ring + BAND), Stroke::new(1.0, s.ring_line)));
    let f = egui::FontId::proportional(12.0);
    for (text, dir) in [("N", vec2(0.0, -1.0)), ("E", vec2(1.0, 0.0)), ("S", vec2(0.0, 1.0)), ("W", vec2(-1.0, 0.0))] {
        let at = c + dir * l.ring;
        p.text(at, egui::Align2::CENTER_CENTER, crate::i18n::t(text), f.clone(), s.letters);
    }
}

/// The cube seen from above: a soft shadow, the four chamfered edges lit from the upper left,
/// the corners between them, and the Top face with its label. `hot` is the hovered part.
fn paint_cube(p: &egui::Painter, l: &Layout, s: &Shades, hot: Option<Part>, pointer: Option<Pos2>) {
    let (o, i) = (l.cube, l.face);
    p.rect_filled(o.translate(vec2(2.0, 3.0)).expand(1.5), 4.0, Color32::from_black_alpha(70).gamma_multiply(s.outline.a() as f32 / 255.0));
    // Edges: top and left catch the light, right and bottom fall into shade.
    let (lit, dark) = (s.lit_edge, s.shaded_edge);
    let mid = |a: Color32, b: Color32| a.lerp_to_gamma(b, 0.5);
    quad(p, [o.left_top(), o.right_top(), i.right_top(), i.left_top()], [lit, lit, lit, lit]);
    quad(p, [o.left_top(), i.left_top(), i.left_bottom(), o.left_bottom()], [lit, lit, mid(lit, dark), mid(lit, dark)]);
    quad(p, [i.right_top(), o.right_top(), o.right_bottom(), i.right_bottom()], [mid(lit, dark), mid(lit, dark), dark, dark]);
    quad(p, [i.left_bottom(), i.right_bottom(), o.right_bottom(), o.left_bottom()], [dark, dark, dark, dark]);
    // The Top face: a gentle gradient, lighter towards the light.
    quad(p, [i.left_top(), i.right_top(), i.right_bottom(), i.left_bottom()], [s.face_top, s.face_top, s.face_bottom, s.face_bottom]);
    // The hovered region turns blue.
    if let (Some(part), Some(at)) = (hot, pointer) {
        let region = match part {
            Part::Face => Some(i),
            Part::Corner => {
                let x = if at.x < i.left() { o.left()..=i.left() } else { i.right()..=o.right() };
                let y = if at.y < i.top() { o.top()..=i.top() } else { i.bottom()..=o.bottom() };
                Some(Rect::from_x_y_ranges(x, y))
            }
            Part::Edge => Some(if at.y < i.top() {
                Rect::from_min_max(pos2(i.left(), o.top()), i.right_top())
            } else if at.y > i.bottom() {
                Rect::from_min_max(i.left_bottom(), pos2(i.right(), o.bottom()))
            } else if at.x < i.left() {
                Rect::from_min_max(pos2(o.left(), i.top()), i.left_bottom())
            } else {
                Rect::from_min_max(i.right_top(), pos2(o.right(), i.bottom()))
            }),
            _ => None,
        };
        if let Some(r) = region {
            p.rect_filled(r, 0.0, s.hot);
        }
    }
    // Crisp lines where the edges meet the face and round the cube.
    let line = Stroke::new(1.0, s.outline);
    p.rect_stroke(o, 1.0, line, egui::StrokeKind::Inside);
    p.rect_stroke(i, 0.0, Stroke::new(1.0, s.outline.gamma_multiply(0.6)), egui::StrokeKind::Middle);
    for (a, b) in
        [(o.left_top(), i.left_top()), (o.right_top(), i.right_top()), (o.right_bottom(), i.right_bottom()), (o.left_bottom(), i.left_bottom())]
    {
        p.line_segment([a, b], Stroke::new(1.0, s.outline.gamma_multiply(0.6)));
    }
    p.text(i.center(), egui::Align2::CENTER_CENTER, crate::i18n::t("TOP"), egui::FontId::proportional(12.0), s.label);
}

/// The arrows that rotate to the neighbouring faces, and the roll arrows: shown while the
/// pointer is over the cube, as on Windows.
fn paint_arrows(p: &egui::Painter, l: &Layout, s: &Shades, hot: Option<Part>, pointer: Option<Pos2>) {
    for tri in l.arrows() {
        let over = hot == Some(Part::Arrow) && pointer.is_some_and(|at| Rect::from_points(&tri).expand(2.0).contains(at));
        p.add(egui::Shape::convex_polygon(tri.to_vec(), if over { s.hot } else { s.lit_edge }, Stroke::new(1.0, s.outline)));
    }
    // Two curved arrows bending round the upper-right corner.
    let r = l.roll();
    let col = if hot == Some(Part::Roll) { s.hot } else { s.lit_edge };
    let c = r.center() + vec2(-6.0, 6.0);
    let arc: Vec<Pos2> = (0..=10)
        .map(|k| {
            let a = std::f32::consts::PI * (1.05 + 0.4 * k as f32 / 10.0);
            c + vec2(a.cos(), a.sin()) * -11.0
        })
        .collect();
    p.line(arc.clone(), Stroke::new(3.0, s.outline));
    p.line(arc.clone(), Stroke::new(1.8, col));
    if let (Some(first), Some(last)) = (arc.first(), arc.last()) {
        for (tip, back) in [(*first, arc.get(1).copied().unwrap_or(*first)), (*last, arc.get(arc.len().saturating_sub(2)).copied().unwrap_or(*last))]
        {
            let d = (tip - back).normalized();
            let n = vec2(-d.y, d.x);
            let pts = vec![tip + d * 4.0, tip + n * 3.5, tip - n * 3.5];
            p.add(egui::Shape::convex_polygon(pts, col, Stroke::new(1.0, s.outline)));
        }
    }
}

/// Paint the ViewCube's ring, compass letters, cube and Home button, and handle the pointer.
/// The canvas paints the WCS pill itself. Away from the pointer the cube is half transparent,
/// so it doesn't hide the drawing underneath; it comes up fully when the pointer reaches it.
pub fn show(app: &mut CadApp, ui: &mut egui::Ui, l: &Layout) {
    let resp = ui.interact(l.bounds, ui.scope_id().with("viewcube"), Sense::click_and_drag());
    let pointer = resp.hover_pos();
    let part = pointer.and_then(|p| l.part_at(p));
    let near = pointer.is_some();
    let s = Shades::new(if near { 1.0 } else { 0.6 });
    let p = ui.painter_at(l.bounds.union(l.pill));
    paint_ring(&p, l, &s, part == Some(Part::Ring));
    paint_cube(&p, l, &s, part, pointer);
    if near {
        paint_arrows(&p, l, &s, part, pointer);
    }
    paint_home(&p, l.home, part == Some(Part::Home));

    let resp = match part {
        Some(part) => resp.on_hover_cursor(CursorIcon::PointingHand).on_hover_text(part.tip()),
        None => resp,
    };
    let pressed_on = resp.interact_pointer_pos().and_then(|o| l.part_at(o));
    if resp.clicked() {
        match pressed_on {
            Some(Part::Face | Part::Home) => {
                let _ = app.run("zoom.extents", json!({}));
            }
            Some(Part::Edge | Part::Corner | Part::Arrow | Part::Roll) => app.session.echo(NO_3D),
            Some(Part::Ring) => app.session.echo(NO_TWIST),
            None => {}
        }
    }
    if resp.drag_started() {
        match pressed_on {
            Some(Part::Ring) => app.session.echo(NO_TWIST),
            Some(Part::Face | Part::Edge | Part::Corner) => app.session.echo(NO_3D),
            _ => {}
        }
    }
    let mut action = None;
    resp.context_menu(|ui| {
        if ui.button("Home").clicked() {
            action = Some(("zoom.extents", json!({})));
            ui.close();
        }
        if ui.button("Hide ViewCube").clicked() {
            action = Some(("ui.toggle.viewcube", json!({"on": false})));
            ui.close();
        }
    });
    if let Some((id, params)) = action {
        let _ = app.run(id, params);
    }
    // The WCS pill (painted by the canvas) opens the coordinate-system menu.
    let pill = ui.interact(l.pill, ui.scope_id().with("viewcube_wcs"), Sense::click_and_drag()).on_hover_cursor(CursorIcon::PointingHand);
    let pill = pill.on_hover_text("Coordinate system");
    ucs_menu(app, egui::Popup::menu(&pill));
}

/// A small house, drawn in code.
fn paint_home(p: &egui::Painter, r: Rect, hot: bool) {
    let t = Tokens::get();
    if hot {
        p.rect_filled(r, 3.0, t.control_hover);
    }
    let st = Stroke::new(1.3, if hot { t.text } else { t.text_dim });
    let c = r.center();
    let roof = [c + vec2(-6.5, 0.5), c + vec2(0.0, -6.0), c + vec2(6.5, 0.5)];
    p.line(roof.to_vec(), st);
    p.line(vec![c + vec2(-4.5, -1.0), c + vec2(-4.5, 6.0), c + vec2(4.5, 6.0), c + vec2(4.5, -1.0)], st);
    p.line_segment([c + vec2(-1.2, 6.0), c + vec2(-1.2, 2.5)], st);
    p.line_segment([c + vec2(1.2, 6.0), c + vec2(1.2, 2.5)], st);
    p.line_segment([c + vec2(-1.2, 2.5), c + vec2(1.2, 2.5)], st);
}

/// The UCS icon's arms (as drawn by the canvas at the lower-left corner): the X arm and the Y arm.
pub fn ucs_arms(canvas: Rect) -> [Rect; 2] {
    let o = pos2(canvas.left() + 36.0, canvas.bottom() - 34.0);
    [Rect::from_min_max(o + vec2(-8.0, -8.0), o + vec2(80.0, 8.0)), Rect::from_min_max(o + vec2(-8.0, -82.0), o + vec2(8.0, 8.0))]
}

/// Make the UCS icon clickable: hovering highlights it, a click (either button) opens the
/// coordinate-system menu.
pub fn ucs_icon(app: &mut CadApp, ui: &mut egui::Ui, canvas: Rect) {
    let t = Tokens::get();
    let [x, y] = ucs_arms(canvas);
    let rx = ui.interact(x, ui.scope_id().with("ucs_x"), Sense::click_and_drag());
    let ry = ui.interact(y, ui.scope_id().with("ucs_y"), Sense::click_and_drag());
    let resp = (rx | ry).on_hover_cursor(CursorIcon::PointingHand).on_hover_text("UCS icon: World Coordinate System");
    if resp.hovered() {
        let o = pos2(x.left() + 8.0, x.center().y);
        let st = Stroke::new(2.0, t.hover);
        let p = ui.painter_at(canvas);
        p.line_segment([o, o + vec2(60.0, 0.0)], st);
        p.line_segment([o, o + vec2(0.0, -60.0)], st);
    }
    // Opened with `Bool(true)`: a pointer-anchored popup records its position only then.
    let open = (resp.clicked() || resp.secondary_clicked()).then_some(egui::SetOpenCommand::Bool(true));
    ucs_menu(app, egui::Popup::menu(&resp).open_memory(open).at_pointer_fixed());
}

/// The coordinate-system menu: the WCS (the only coordinate system so far) and the icon switch.
fn ucs_menu(app: &mut CadApp, popup: egui::Popup<'_>) {
    let mut hide = false;
    popup.show(|ui| {
        ui.radio(true, "World (WCS)").on_hover_text("RustDraft draws in the World Coordinate System; user coordinate systems aren't available yet.");
        ui.separator();
        if ui.button("Hide UCS Icon").clicked() {
            hide = true;
            ui.close();
        }
    });
    if hide {
        let _ = app.run("ui.toggle.ucsicon", json!({"on": false}));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustdraft_engine::Session;

    /// The app with the sample drawing, laid out once in a 1400×900 window.
    fn app_and_ctx() -> (CadApp, egui::Context) {
        let mut app = CadApp::new(Session::empty(), crate::Services::default());
        app.ui.in_window_menu = true;
        app.start("ui.sample");
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, Vec::new());
        frame(&mut app, &ctx, Vec::new());
        (app, ctx)
    }

    fn frame(app: &mut CadApp, ctx: &egui::Context, events: Vec<egui::Event>) {
        let raw = egui::RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1400.0, 900.0))), events, ..Default::default() };
        let mut out = ctx.run_ui(raw, |ui| {
            app.logic(ui.ctx());
            app.ui(ui);
        });
        out.textures_delta.clear();
    }

    /// A real pointer click (move, press, release in separate frames, like the control channel).
    pub(crate) fn click(app: &mut CadApp, ctx: &egui::Context, at: Pos2, button: egui::PointerButton) {
        let m = egui::Modifiers::default();
        frame(app, ctx, vec![egui::Event::PointerMoved(at)]);
        frame(app, ctx, vec![egui::Event::PointerButton { pos: at, button, pressed: true, modifiers: m }]);
        frame(app, ctx, vec![egui::Event::PointerButton { pos: at, button, pressed: false, modifiers: m }]);
        frame(app, ctx, Vec::new());
    }

    /// A menu popup was shown in the last frame.
    fn menu_shown(ctx: &egui::Context) -> bool {
        ctx.memory(|m| m.areas().visible_layer_ids().iter().any(|l| l.order == egui::Order::Foreground))
    }

    fn canvas(app: &CadApp) -> Rect {
        app.canvas.rect.unwrap_or(Rect::NOTHING)
    }

    fn view(app: &CadApp) -> (f64, f64, f64) {
        let v = app.session.state().map(|s| s.view()).unwrap_or_default();
        (v.center.x, v.center.y, v.height)
    }

    #[test]
    fn parts_are_found_where_they_are_drawn() {
        let l = Layout::new(Rect::from_min_size(pos2(200.0, 80.0), vec2(1000.0, 800.0)));
        assert_eq!(l.part_at(l.center), Some(Part::Face));
        assert_eq!(l.part_at(l.home.center()), Some(Part::Home));
        assert_eq!(l.part_at(l.center + vec2(0.0, -l.ring)), Some(Part::Ring), "N");
        assert_eq!(l.part_at(l.center + vec2(l.ring + 3.0, 0.0)), Some(Part::Ring), "E");
        assert_eq!(l.part_at(l.center + vec2(30.0, 30.0)), None, "between the face and the ring");
        assert!(l.bounds.contains_rect(l.home) && l.bounds.contains_rect(l.face));
        assert_eq!(l.part_at(pos2(l.center.x, l.cube.top() + 2.0)), Some(Part::Edge), "top edge");
        assert_eq!(l.part_at(l.cube.right_bottom() - vec2(2.0, 2.0)), Some(Part::Corner), "lower-right corner");
        assert_eq!(l.part_at(l.arrows()[1][0] + vec2(4.0, 0.0)), Some(Part::Arrow), "east arrow");
        assert_eq!(l.part_at(l.roll().center()), Some(Part::Roll));
        assert!(l.bounds.contains_rect(l.cube) && l.bounds.contains_rect(l.roll()));
    }

    #[test]
    fn edges_corners_and_arrows_explain_instead_of_touching_the_drawing() {
        let (mut app, ctx) = app_and_ctx();
        let l = Layout::new(canvas(&app));
        let fitted = view(&app);
        for at in [pos2(l.center.x, l.cube.top() + 2.0), l.cube.left_top() + vec2(2.0, 2.0), l.arrows()[0][0] + vec2(0.0, -4.0), l.roll().center()] {
            app.session.log.clear();
            click(&mut app, &ctx, at, egui::PointerButton::Primary);
            assert_eq!(app.session.log.last().map(String::as_str), Some(NO_3D), "{at:?}");
            assert!(app.session.pending_window.is_none());
            assert_eq!(view(&app), fitted);
        }
    }

    #[test]
    fn viewcube_clicks_reach_the_cube_not_the_drawing() {
        let (mut app, ctx) = app_and_ctx();
        let l = Layout::new(canvas(&app));
        let fitted = view(&app);
        // Zoomed in, the Top face fits the drawing again (zoom extents) and nothing is picked.
        let _ = app.run("zoom.in", json!({}));
        assert_ne!(view(&app), fitted);
        click(&mut app, &ctx, l.center, egui::PointerButton::Primary);
        assert_eq!(view(&app), fitted, "Top face zooms to extents");
        assert!(app.session.pending_window.is_none(), "the click didn't start a selection window");
        assert!(app.session.selection().is_empty());
        // Home does the same.
        let _ = app.run("zoom.in", json!({}));
        click(&mut app, &ctx, l.home.center(), egui::PointerButton::Primary);
        assert_eq!(view(&app), fitted, "Home zooms to extents");
        // The ring explains that the 2D view can't rotate, and doesn't touch the drawing.
        click(&mut app, &ctx, l.center + vec2(0.0, -l.ring), egui::PointerButton::Primary);
        assert_eq!(app.session.log.last().map(String::as_str), Some(NO_TWIST));
        assert!(app.session.pending_window.is_none());
        assert_eq!(view(&app), fitted);
        // A click in the drawing still reaches the drawing.
        let empty = canvas(&app).center() + vec2(0.0, -300.0);
        click(&mut app, &ctx, empty, egui::PointerButton::Primary);
        assert!(app.session.pending_window.is_some(), "an empty-space click starts a selection window");
    }

    #[test]
    fn ucs_icon_and_wcs_pill_open_the_coordinate_system_menu() {
        let (mut app, ctx) = app_and_ctx();
        let [x, _] = ucs_arms(canvas(&app));
        click(&mut app, &ctx, x.center(), egui::PointerButton::Primary);
        assert!(menu_shown(&ctx), "UCS icon menu");
        assert!(app.session.pending_window.is_none(), "the click stayed on the icon");
        let l = Layout::new(canvas(&app));
        click(&mut app, &ctx, l.pill.center(), egui::PointerButton::Primary);
        assert!(menu_shown(&ctx), "WCS menu");
        assert!(app.session.pending_window.is_none());
    }
}
