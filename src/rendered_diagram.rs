use crate::hasse_layout::HasseLayout;
use crate::relation_matrix::HasseCoverEdge;

#[derive(Debug, Clone)]
pub(crate) struct RenderedDiagram {
    pub(crate) cover_edges: Vec<HasseCoverEdge>,
    pub(crate) layout: HasseLayout,
}

impl RenderedDiagram {
    pub(crate) fn formatted_cover_edges(&self) -> String {
        if self.cover_edges.is_empty() {
            "нет".to_owned()
        } else {
            self.cover_edges
                .iter()
                .map(HasseCoverEdge::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        }
    }

    pub(crate) fn draw(&self, painter: &egui::Painter, rect: egui::Rect) {
        let style = DiagramStyle::default();
        let plot_rect = rect.shrink(36.0);

        let mut positions = vec![egui::Pos2::ZERO; self.layout.nodes().len() + 1];

        for node in self.layout.nodes() {
            let normalized_position = self
                .layout
                .normalized_position(node.label)
                .expect("every layout node must provide normalized coordinates");

            let x = egui::lerp(
                (plot_rect.left() + style.node_radius)..=(plot_rect.right() - style.node_radius),
                normalized_position.x,
            );
            let y = egui::lerp(
                (plot_rect.bottom() - style.node_radius)..=(plot_rect.top() + style.node_radius),
                normalized_position.y,
            );
            positions[node.label] = egui::pos2(x, y);
        }

        for edge in &self.cover_edges {
            painter.line_segment([positions[edge.lower], positions[edge.upper]], style.edge_stroke);
        }

        for node in self.layout.nodes() {
            let position = positions[node.label];
            painter.circle(
                position,
                style.node_radius,
                style.node_fill,
                style.node_stroke,
            );
            painter.text(
                position,
                egui::Align2::CENTER_CENTER,
                node.label.to_string(),
                egui::FontId::proportional(20.0),
                style.label_color,
            );
        }
    }
}

struct DiagramStyle {
    node_radius: f32,
    edge_stroke: egui::Stroke,
    node_fill: egui::Color32,
    node_stroke: egui::Stroke,
    label_color: egui::Color32,
}

impl Default for DiagramStyle {
    fn default() -> Self {
        Self {
            node_radius: 20.0,
            edge_stroke: egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(142, 178, 255)),
            node_fill: egui::Color32::from_rgb(201, 162, 76),
            node_stroke: egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(251, 232, 179)),
            label_color: egui::Color32::from_rgb(16, 18, 24),
        }
    }
}
