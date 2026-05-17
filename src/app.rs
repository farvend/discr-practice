use crate::bootstrap::BootstrapState;
use crate::hasse_layout::{HasseLayout, layout_hasse_nodes};
use crate::relation_matrix::{
    HasseCoverEdge, MAX_RELATION_SIZE, MIN_RELATION_SIZE, PartialOrderDiagnostics, RelationMatrix,
};

pub fn run() -> eframe::Result<()> {
    let state = BootstrapState::new();
    let title = state.app_title().to_owned();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1080.0, 760.0])
            .with_min_inner_size([820.0, 560.0]),
        ..Default::default()
    };

    eframe::run_native(
        &title,
        options,
        Box::new(move |_cc| Ok(Box::new(HasseBootstrapApp::new(state)))),
    )
}

struct HasseBootstrapApp {
    state: BootstrapState,
    ui_state: HasseGuiState,
}

impl HasseBootstrapApp {
    fn new(state: BootstrapState) -> Self {
        Self {
            ui_state: HasseGuiState::new(state.status_message()),
            state,
        }
    }
}

impl eframe::App for HasseBootstrapApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = egui::Color32::from_rgb(14, 18, 26);
        visuals.extreme_bg_color = egui::Color32::from_rgb(20, 27, 39);
        visuals.override_text_color = Some(egui::Color32::from_rgb(235, 237, 242));
        visuals.selection.bg_fill = egui::Color32::from_rgb(201, 162, 76);
        visuals.selection.stroke.color = egui::Color32::from_rgb(16, 16, 20);
        ctx.set_visuals(visuals);

        egui::SidePanel::left("matrix_controls")
            .resizable(false)
            .default_width(360.0)
            .min_width(320.0)
            .show(ctx, |ui| {
                ui.add_space(10.0);
                ui.heading(self.state.app_title());
                ui.label(
                    egui::RichText::new("Редактор отношения и сборка диаграммы Хассе")
                        .size(15.0)
                        .color(egui::Color32::from_rgb(201, 162, 76)),
                );
                ui.add_space(8.0);
                ui.label(self.ui_state.status_message());
                ui.add_space(12.0);

                self.render_size_controls(ui);

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                ui.label(egui::RichText::new("Матрица бинарного отношения").strong());
                ui.label("Строка i и столбец j задают истинность пары (i, j).");
                ui.add_space(8.0);

                egui::ScrollArea::both()
                    .id_salt("matrix_scroll")
                    .max_height(420.0)
                    .show(ui, |ui| {
                        self.render_matrix_editor(ui);
                    });

                ui.add_space(12.0);

                if ui
                    .add_sized(
                        [ui.available_width(), 40.0],
                        egui::Button::new(
                            egui::RichText::new("Проверить и построить диаграмму").strong(),
                        ),
                    )
                    .clicked()
                {
                    self.ui_state.build_diagram();
                }

                ui.add_space(10.0);
                self.render_status_panel(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            ui.heading("Область диаграммы");
            ui.label("Рёбра берутся только из покрытия Хассе; позиции узлов — из детерминированного layout-модуля.");
            ui.add_space(12.0);
            self.render_diagram_area(ui);
        });
    }
}

impl HasseBootstrapApp {
    fn render_size_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Размер множества n").strong());

            let mut size = self.ui_state.size() as u32;
            let response = ui.add(
                egui::Slider::new(
                    &mut size,
                    MIN_RELATION_SIZE as u32..=MAX_RELATION_SIZE as u32,
                )
                .text("n")
                .integer(),
            );

            if response.changed() {
                self.ui_state.resize(size as usize);
            }
        });

        ui.label(format!("Сейчас редактируется матрица {}×{}.", self.ui_state.size(), self.ui_state.size()));
    }

    fn render_matrix_editor(&mut self, ui: &mut egui::Ui) {
        let size = self.ui_state.size();

        egui::Grid::new("relation_matrix_grid")
            .striped(true)
            .spacing([10.0, 6.0])
            .show(ui, |ui| {
                ui.label(egui::RichText::new("i\\j").strong());
                for column in 1..=size {
                    ui.label(egui::RichText::new(column.to_string()).strong());
                }
                ui.end_row();

                for row in 1..=size {
                    ui.label(egui::RichText::new(row.to_string()).strong());

                    for column in 1..=size {
                        let mut value = self.ui_state.cell(row, column);
                        let response = ui.checkbox(&mut value, "");

                        if response.changed() {
                            self.ui_state.set_cell(row, column, value);
                        }
                    }

                    ui.end_row();
                }
            });
    }

    fn render_status_panel(&self, ui: &mut egui::Ui) {
        ui.separator();
        ui.add_space(6.0);
        ui.label(egui::RichText::new("Результат проверки").strong());
        ui.add_space(4.0);

        for line in self.ui_state.result_lines() {
            ui.label(line);
        }
    }

    fn render_diagram_area(&self, ui: &mut egui::Ui) {
        let desired_size = egui::vec2(ui.available_width().max(320.0), ui.available_height().max(420.0));
        let (response, painter) = ui.allocate_painter(desired_size, egui::Sense::hover());
        let rect = response.rect;

        painter.rect_filled(rect, 24.0, egui::Color32::from_rgb(18, 24, 34));
        painter.rect_stroke(
            rect,
            24.0,
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(59, 72, 92)),
            egui::StrokeKind::Inside,
        );

        match self.ui_state.rendered_diagram() {
            Some(diagram) => draw_hasse_diagram(&painter, rect, diagram),
            None => {
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "После успешной проверки здесь появится диаграмма Хассе",
                    egui::FontId::proportional(22.0),
                    egui::Color32::from_rgb(150, 158, 171),
                );
            }
        }
    }
}

#[derive(Debug, Clone)]
struct RenderedDiagram {
    cover_edges: Vec<HasseCoverEdge>,
    layout: HasseLayout,
}

#[derive(Debug, Clone)]
enum BuildState {
    Idle,
    Dirty,
    Invalid(PartialOrderDiagnostics),
    Valid(RenderedDiagram),
}

#[derive(Debug, Clone)]
struct HasseGuiState {
    relation_matrix: RelationMatrix,
    status_message: String,
    build_state: BuildState,
}

impl HasseGuiState {
    fn new(initial_status_message: &str) -> Self {
        Self {
            relation_matrix: RelationMatrix::new(MIN_RELATION_SIZE)
                .expect("minimum supported size must create the initial matrix"),
            status_message: initial_status_message.to_owned(),
            build_state: BuildState::Idle,
        }
    }

    fn size(&self) -> usize {
        self.relation_matrix.size()
    }

    fn cell(&self, row: usize, column: usize) -> bool {
        self.relation_matrix
            .get(row, column)
            .expect("matrix access from the UI must stay within 1-based bounds")
    }

    fn set_cell(&mut self, row: usize, column: usize, value: bool) {
        self.relation_matrix
            .set(row, column, value)
            .expect("matrix writes from the UI must stay within 1-based bounds");
        self.invalidate_build("Матрица изменена. Нажмите кнопку проверки, чтобы обновить диаграмму.");
    }

    fn resize(&mut self, size: usize) {
        self.relation_matrix = RelationMatrix::new(size)
            .expect("size slider only allows values from the supported range");
        self.invalidate_build("Размер матрицы изменён. Заполните значения и запустите проверку.");
    }

    fn build_diagram(&mut self) {
        match self.relation_matrix.validate_partial_order() {
            Ok(()) => {
                let cover_edges = self
                    .relation_matrix
                    .hasse_cover_edges()
                    .expect("validated partial order must produce cover edges");

                match layout_hasse_nodes(self.relation_matrix.size(), &cover_edges) {
                    Ok(layout) => {
                        let edge_count = cover_edges.len();
                        let level_count = layout.level_count();
                        self.status_message = format!(
                            "Отношение корректно. Построена диаграмма Хассе для n = {}.",
                            self.relation_matrix.size()
                        );
                        self.build_state = BuildState::Valid(RenderedDiagram { cover_edges, layout });

                        self.status_message.push_str(&format!(
                            " Узлов: {}, рёбер покрытия: {}, уровней: {}.",
                            self.relation_matrix.size(),
                            edge_count,
                            level_count
                        ));
                    }
                    Err(error) => {
                        self.status_message = format!(
                            "Layout не построен: {:?}. Проверьте матрицу и повторите попытку.",
                            error
                        );
                        self.build_state = BuildState::Dirty;
                    }
                }
            }
            Err(diagnostics) => {
                self.status_message =
                    "Отношение не является частичным порядком. Диаграмма не построена.".to_owned();
                self.build_state = BuildState::Invalid(diagnostics);
            }
        }
    }

    fn status_message(&self) -> &str {
        &self.status_message
    }

    fn rendered_diagram(&self) -> Option<&RenderedDiagram> {
        match &self.build_state {
            BuildState::Valid(diagram) => Some(diagram),
            _ => None,
        }
    }

    fn result_lines(&self) -> Vec<String> {
        match &self.build_state {
            BuildState::Idle => vec!["- Диаграмма ещё не строилась.".to_owned()],
            BuildState::Dirty => vec!["- Текущая диаграмма сброшена: матрица изменилась.".to_owned()],
            BuildState::Invalid(diagnostics) => diagnostics_to_lines(diagnostics),
            BuildState::Valid(diagram) => vec![
                "- Проверка частичного порядка пройдена.".to_owned(),
                format!("- Рёбра покрытия: {}.", format_edges(&diagram.cover_edges)),
                format!("- Уровней layout: {}.", diagram.layout.level_count()),
            ],
        }
    }

    fn invalidate_build(&mut self, status_message: &str) {
        self.status_message = status_message.to_owned();
        self.build_state = BuildState::Dirty;
    }
}

fn diagnostics_to_lines(diagnostics: &PartialOrderDiagnostics) -> Vec<String> {
    let mut lines = vec!["- Нарушены условия частичного порядка:".to_owned()];

    match diagnostics.reflexivity_witness {
        Some(element) => lines.push(format!(
            "  • Рефлексивность нарушена: ({0}, {0}) = 0.",
            element
        )),
        None => lines.push("  • Рефлексивность: OK.".to_owned()),
    }

    match diagnostics.antisymmetry_witness {
        Some((left, right)) => lines.push(format!(
            "  • Антисимметричность нарушена: ({left}, {right}) = 1 и ({right}, {left}) = 1."
        )),
        None => lines.push("  • Антисимметричность: OK.".to_owned()),
    }

    match diagnostics.transitivity_witness {
        Some((source, middle, target)) => lines.push(format!(
            "  • Транзитивность нарушена: ({source}, {middle}) = 1 и ({middle}, {target}) = 1, но ({source}, {target}) = 0."
        )),
        None => lines.push("  • Транзитивность: OK.".to_owned()),
    }

    lines
}

fn format_edges(edges: &[HasseCoverEdge]) -> String {
    if edges.is_empty() {
        "нет".to_owned()
    } else {
        edges.iter()
            .map(|(lower, upper)| format!("({lower}, {upper})"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn draw_hasse_diagram(painter: &egui::Painter, rect: egui::Rect, diagram: &RenderedDiagram) {
    let plot_rect = rect.shrink(36.0);
    let node_radius = 20.0;
    let edge_stroke = egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(142, 178, 255));
    let node_fill = egui::Color32::from_rgb(201, 162, 76);
    let node_stroke = egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(251, 232, 179));
    let label_color = egui::Color32::from_rgb(16, 18, 24);

    let mut positions = vec![egui::Pos2::ZERO; diagram.layout.nodes().len() + 1];

    for node in diagram.layout.nodes() {
        let (normalized_x, normalized_y) = diagram
            .layout
            .normalized_position(node.label)
            .expect("every layout node must provide normalized coordinates");

        let x = egui::lerp((plot_rect.left() + node_radius)..=(plot_rect.right() - node_radius), normalized_x);
        let y = egui::lerp((plot_rect.bottom() - node_radius)..=(plot_rect.top() + node_radius), normalized_y);
        positions[node.label] = egui::pos2(x, y);
    }

    for &(lower, upper) in &diagram.cover_edges {
        painter.line_segment([positions[lower], positions[upper]], edge_stroke);
    }

    for node in diagram.layout.nodes() {
        let position = positions[node.label];
        painter.circle(position, node_radius, node_fill, node_stroke);
        painter.text(
            position,
            egui::Align2::CENTER_CENTER,
            node.label.to_string(),
            egui::FontId::proportional(20.0),
            label_color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{BuildState, HasseGuiState};

    #[test]
    fn valid_relation_builds_renderable_diagram() {
        let mut state = HasseGuiState::new("ready");
        state.resize(3);

        state.set_cell(1, 1, true);
        state.set_cell(2, 2, true);
        state.set_cell(3, 3, true);
        state.set_cell(1, 2, true);
        state.set_cell(2, 3, true);
        state.set_cell(1, 3, true);

        state.build_diagram();

        let diagram = state
            .rendered_diagram()
            .expect("valid partial order must render a diagram");

        assert_eq!(diagram.cover_edges, vec![(1, 2), (2, 3)]);
        assert_eq!(diagram.layout.level_count(), 3);
        assert!(matches!(state.build_state, BuildState::Valid(_)));
    }

    #[test]
    fn invalid_relation_reports_diagnostics_and_clears_diagram() {
        let mut state = HasseGuiState::new("ready");
        state.resize(3);

        state.set_cell(1, 1, true);
        state.set_cell(2, 2, true);
        state.set_cell(3, 3, true);
        state.set_cell(1, 2, true);
        state.set_cell(2, 3, true);

        state.build_diagram();

        assert!(state.rendered_diagram().is_none());
        let lines = state.result_lines();
        assert!(lines
            .iter()
            .any(|line| line.contains("Транзитивность нарушена")));
        assert!(matches!(state.build_state, BuildState::Invalid(_)));
    }

    #[test]
    fn resizing_or_editing_matrix_invalidates_previous_build() {
        let mut state = HasseGuiState::new("ready");

        state.set_cell(1, 1, true);
        state.build_diagram();
        assert!(state.rendered_diagram().is_some());

        state.set_cell(1, 1, false);
        assert!(state.rendered_diagram().is_none());
        assert!(matches!(state.build_state, BuildState::Dirty));

        state.resize(2);
        assert_eq!(state.size(), 2);
        assert!(state.rendered_diagram().is_none());
        assert!(matches!(state.build_state, BuildState::Dirty));
    }
}
