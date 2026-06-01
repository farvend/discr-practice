use crate::bootstrap::BootstrapState;
use crate::gui_state::HasseGuiState;
use crate::relation_matrix::{MAX_RELATION_SIZE, MIN_RELATION_SIZE};

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
            .resizable(true)
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
                    .show(ui, |ui| self.render_matrix_editor(ui));

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

        ui.label(format!(
            "Сейчас редактируется матрица {}×{}.",
            self.ui_state.size(),
            self.ui_state.size()
        ));
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

        for message in self.ui_state.result_messages() {
            ui.label(message);
        }
    }

    fn render_diagram_area(&self, ui: &mut egui::Ui) {
        let desired_size = egui::vec2(
            ui.available_width().max(320.0),
            ui.available_height().max(420.0),
        );
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
            Some(diagram) => diagram.draw(&painter, rect),
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
