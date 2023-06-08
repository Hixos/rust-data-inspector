use std::{
    cell::Cell,
    hash::Hash,
    ops::{Add, Div, Mul, Neg, Sub},
    rc::Rc,
};

use egui::{
    plot::{Plot, PlotBounds, PlotPoint, PlotUi, Legend},
    InnerResponse, Rect, Ui, Vec2,
};

use serde::{Deserialize, Serialize};

use crate::util::plothelper::{AxisBounds, PlotHelper};

pub struct RTPlot {
    id_source: egui::Id,
    x_bounds: Option<AxisBounds>,
    link_group: Option<LinkedAxisGroup>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlotResponse {
    pub interacted: bool,
    pub new_bounds: Option<PlotBounds>,
}

#[derive(Clone, Serialize, Deserialize)]
struct PlotMemory {
    x_bounds: Option<AxisBounds>,
    y_bounds: Option<AxisBounds>,
}

impl Default for PlotMemory {
    fn default() -> Self {
        PlotMemory {
            x_bounds: None,
            y_bounds: None,
        }
    }
}

impl PlotMemory {
    pub fn load(ctx: &egui::Context, id: egui::Id) -> Option<Self> {
        ctx.data_mut(|d| d.get_persisted(id))
    }

    pub fn store(self, ctx: &egui::Context, id: egui::Id) {
        ctx.data_mut(|d| d.insert_persisted(id, self));
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LinkedAxisGroup {
    pub link_x: bool,
    bounds: Rc<Cell<Option<AxisBounds>>>,
}

impl LinkedAxisGroup {
    pub fn new(link_x: bool) -> Self {
        LinkedAxisGroup {
            link_x: link_x,
            bounds: Rc::new(Cell::new(None)),
        }
    }
}

impl RTPlot {
    pub fn new(id_source: impl Hash) -> RTPlot {
        RTPlot {
            id_source: egui::Id::new(id_source),
            x_bounds: None,
            link_group: None,
        }
    }

    pub fn set_x_bounds(&mut self, x_bounds: AxisBounds) -> &mut Self {
        self.x_bounds = Some(x_bounds);
        self
    }

    pub fn link_axis(&mut self, link_group: LinkedAxisGroup) -> &mut Self {
        self.link_group = Some(link_group);
        self
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        plot_contents: impl FnOnce(&mut PlotUi),
    ) -> egui::InnerResponse<PlotResponse> {
        let plot_id = ui.make_persistent_id(self.id_source);
        ui.ctx().check_for_id_clash(
            plot_id,
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::ZERO),
            "RTPlot",
        );

        let mut memory = PlotMemory::load(ui.ctx(), plot_id).unwrap_or_default();

        struct InnerPlotResponse {
            bounds: PlotBounds,
            screen_bounds: Rect,
        }

        let response = Plot::new(self.id_source)
            .allow_double_click_reset(false)
            .allow_scroll(false)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_boxed_zoom(false)
            .link_cursor(egui::Id::new(0), true, false)
            .legend(Legend::default())
            .show::<InnerPlotResponse>(ui, |plot_ui| {
                let mut bounds = plot_ui.plot_bounds();
                
                let group_ref = self.link_group.as_ref();
                
                if group_ref.is_some() && group_ref.unwrap().link_x {
                    let x = group_ref.unwrap().bounds.clone();
                    
                    if x.get().is_some() {
                        bounds = PlotHelper::set_x_bounds(x.get().unwrap(), bounds);
                    }
                }
                
                let orig_bounds = bounds.clone();

                if memory.y_bounds.is_some() {
                    bounds = PlotHelper::set_y_bounds(memory.y_bounds.unwrap(), bounds);
                }

                if self.x_bounds.is_some() {
                    bounds = PlotHelper::set_x_bounds(self.x_bounds.unwrap(), bounds);
                } else if memory.x_bounds.is_some() {
                    bounds = PlotHelper::set_x_bounds(memory.x_bounds.unwrap(), bounds);
                }

                plot_ui.set_plot_bounds(bounds);

                plot_contents(plot_ui);

                // plot_ui.

                if bounds != orig_bounds && group_ref.is_some() {
                    group_ref.unwrap().bounds.set(Some(AxisBounds::from_x_bounds(bounds)));
                }

                // Plot bounds but in screen coordinates
                let screen_bounds = PlotHelper::get_screen_bounds(plot_ui);

                InnerPlotResponse {
                    bounds: bounds,
                    screen_bounds,
                }
            });

        let transform = ScreenTransform::new(response.inner.bounds, response.inner.screen_bounds);
        let plot_resp = Self::handle_input(&response.response, transform);

        match plot_resp.new_bounds {
            Some(bounds) => {
                memory.x_bounds = Some(PlotHelper::get_x_bounds(bounds));
                memory.y_bounds = Some(PlotHelper::get_y_bounds(bounds));

                
                //  group.bounds.set(Some(PlotHelper::get_x_bounds(bounds)));
                
            }
            None => {
                memory.x_bounds = None;
                memory.y_bounds = None;
            }
        }

        memory.store(ui.ctx(), plot_id);

        InnerResponse::new(plot_resp, response.response)
    }

    fn handle_input(response: &egui::Response, mut transform: ScreenTransform) -> PlotResponse {
        let mut plot_response = PlotResponse {
            interacted: false,
            new_bounds: None,
        };

        let mut handled = false;

        if response.dragged_by(egui::PointerButton::Primary) {
            plot_response.interacted = true;

            transform.translate(response.drag_delta());
            handled = true;
        }

        // Hackish way to handle custom zooming behaviour, since egui doesn't allow modyfing user interaction
        if response.hover_pos().is_some() {
            response.ctx.input(|input| {
                if input.scroll_delta.y != 0f32 {
                    plot_response.interacted = true;
                    transform.zoom_y(
                        Self::zoom_from_scroll(input.scroll_delta.y),
                        response.hover_pos().unwrap(),
                    );
                    handled = true;
                }
                if input.zoom_delta() != 1f32 {
                    plot_response.interacted = true;
                    transform.zoom_x(
                        1f64 / (input.zoom_delta() as f64),
                        response.hover_pos().unwrap(),
                    );
                    handled = true;
                }
            });
        }
        if handled {
            plot_response.new_bounds = Some(transform.bounds);
        }

        plot_response
    }

    fn zoom_from_scroll(scroll: f32) -> f64 {
        const ZOOM: f64 = 1.5f64;
        if scroll > 0f32 {
            1f64 / ZOOM
        } else {
            ZOOM
        }
    }
}

#[derive(Clone, Debug)]
struct RTPlotPoint {
    pub x: f64,
    pub y: f64,
}

impl RTPlotPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl Into<egui::plot::PlotPoint> for RTPlotPoint {
    fn into(self) -> egui::plot::PlotPoint {
        egui::plot::PlotPoint::new(self.x, self.y)
    }
}

impl Into<[f64; 2]> for RTPlotPoint {
    fn into(self) -> [f64; 2] {
        [self.x, self.y]
    }
}

impl From<egui::plot::PlotPoint> for RTPlotPoint {
    fn from(point: egui::plot::PlotPoint) -> Self {
        RTPlotPoint {
            x: point.x,
            y: point.y,
        }
    }
}

impl From<[f64; 2]> for RTPlotPoint {
    fn from(point: [f64; 2]) -> Self {
        RTPlotPoint {
            x: point[0],
            y: point[1],
        }
    }
}

impl From<Vec2> for RTPlotPoint {
    fn from(point: Vec2) -> Self {
        RTPlotPoint {
            x: point.x as f64,
            y: point.y as f64,
        }
    }
}

impl Add for RTPlotPoint {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Mul<f64> for RTPlotPoint {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Neg for RTPlotPoint {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Sub for RTPlotPoint {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

impl Copy for RTPlotPoint {}

#[derive(Clone)]
#[allow(non_snake_case)]
struct TransformMatrix {
    pub T: [[f64; 2]; 2],
}
impl Copy for TransformMatrix {}

impl TransformMatrix {
    fn new(a: f64, b: f64, c: f64, d: f64) -> Self {
        TransformMatrix {
            T: [[a, b], [c, d]],
        }
    }

    fn scale(x: f64, y: f64) -> Self {
        TransformMatrix::new(x, 0f64, 0f64, y)
    }

    #[allow(dead_code)]
    fn inv(self) -> Self {
        let a = self.T[0][0];
        let b = self.T[0][1];
        let c = self.T[1][0];
        let d = self.T[1][1];

        let out = TransformMatrix::new(d, -c, -b, a);
        out / (a * d - b * c)
    }
}

impl Mul for TransformMatrix {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let t00 = self.T[0][0] * rhs.T[0][0] + self.T[0][1] * rhs.T[1][0];
        let t01 = self.T[0][0] * rhs.T[0][1] + self.T[0][1] * rhs.T[1][1];

        let t10 = self.T[1][0] * rhs.T[0][0] + self.T[1][1] * rhs.T[1][0];
        let t11 = self.T[1][0] * rhs.T[0][1] + self.T[1][1] * rhs.T[1][1];

        TransformMatrix {
            T: [[t00, t01], [t10, t11]],
        }
    }
}

impl Mul<RTPlotPoint> for TransformMatrix {
    type Output = RTPlotPoint;

    fn mul(self, rhs: RTPlotPoint) -> Self::Output {
        let x = self.T[0][0] * rhs.x + self.T[0][1] * rhs.y;
        let y = self.T[1][0] * rhs.x + self.T[1][1] * rhs.y;
        RTPlotPoint::new(x, y)
    }
}

impl Mul<f64> for TransformMatrix {
    type Output = TransformMatrix;
    fn mul(self, rhs: f64) -> Self::Output {
        let mut out = self.clone();
        out.T[0][0] *= rhs;
        out.T[0][1] *= rhs;
        out.T[1][0] *= rhs;
        out.T[1][1] *= rhs;

        out
    }
}

impl Div<f64> for TransformMatrix {
    type Output = TransformMatrix;
    fn div(self, rhs: f64) -> Self::Output {
        TransformMatrix::new(
            self.T[0][0] / rhs,
            self.T[0][1] / rhs,
            self.T[1][0] / rhs,
            self.T[1][1] / rhs,
        )
    }
}

impl Neg for TransformMatrix {
    type Output = TransformMatrix;
    fn neg(self) -> Self::Output {
        TransformMatrix::new(-self.T[0][0], -self.T[0][1], -self.T[1][0], -self.T[1][1])
    }
}

struct ScreenTransform {
    bounds: egui::plot::PlotBounds,
    rect: egui::Rect,
}

impl ScreenTransform {
    pub fn new(bounds: egui::plot::PlotBounds, rect: egui::Rect) -> Self {
        ScreenTransform { bounds, rect }
    }

    #[allow(dead_code)]
    pub fn from_ui(plot_ui: &egui::plot::PlotUi) -> Self {
        let bounds = plot_ui.plot_bounds();
        // Y axis min/max are swapped since screen-space Y axis is positive downwards
        let topleft = PlotPoint::new(bounds.min()[0], bounds.max()[1]);
        let bottomright = PlotPoint::new(bounds.max()[0], bounds.min()[1]);

        let rect = egui::Rect {
            min: plot_ui.screen_from_plot(topleft),
            max: plot_ui.screen_from_plot(bottomright),
        };

        ScreenTransform { bounds, rect }
    }

    #[allow(dead_code)]
    pub fn zoom_x(&mut self, zoom: f64, hover_pos: egui::Pos2) {
        self.zoom_xy(zoom, 1f64, hover_pos)
    }

    pub fn zoom_y(&mut self, zoom: f64, hover_pos: egui::Pos2) {
        self.zoom_xy(1f64, zoom, hover_pos)
    }

    pub fn zoom_xy(&mut self, zoom_x: f64, zoom_y: f64, hover_pos: egui::Pos2) {
        let hover_pos = self.plot_from_screen(hover_pos);

        let zoom_factor = TransformMatrix::scale(zoom_x, zoom_y);

        let max = RTPlotPoint::from(self.bounds.max());
        let max = zoom_factor * (max - hover_pos) + hover_pos;

        let min = RTPlotPoint::from(self.bounds.min());
        let min = zoom_factor * (min - hover_pos) + hover_pos;

        self.bounds = egui::plot::PlotBounds::from_min_max(min.into(), max.into());
    }

    pub fn translate(&mut self, delta: egui::Vec2) {
        let delta = self.delta_from_screen(delta);

        let min = RTPlotPoint::from(self.bounds.min()) - delta;
        let max = RTPlotPoint::from(self.bounds.max()) - delta;

        self.bounds = egui::plot::PlotBounds::from_min_max(min.into(), max.into());
    }

    #[allow(dead_code)]
    pub fn screen_from_plot(&self, plot_point: RTPlotPoint) -> egui::Pos2 {
        let x = ((plot_point.x - self.bounds.min()[0]) * self.rect.width() as f64
            / self.bounds.width()) as f32
            + self.rect.min.x;
        let y = ((plot_point.y - self.bounds.min()[1]) * -self.rect.height() as f64
            / self.bounds.height()) as f32
            + self.rect.max.y;

        egui::Pos2 { x, y }
    }

    pub fn plot_from_screen(&self, pos: egui::Pos2) -> RTPlotPoint {
        let x = (pos.x - self.rect.min.x) as f64 * self.bounds.width() / self.rect.width() as f64
            + self.bounds.min()[0];
        let y = (pos.y - self.rect.max.y) as f64 * -self.bounds.height()
            / self.rect.height() as f64
            + self.bounds.min()[1];

        RTPlotPoint::new(x, y)
    }

    pub fn delta_from_screen(&self, delta: egui::Vec2) -> RTPlotPoint {
        let x = delta.x as f64 * self.bounds.width() / self.rect.width() as f64;
        let y = delta.y as f64 * -self.bounds.height() / self.rect.height() as f64;

        RTPlotPoint::new(x, y)
    }
}
