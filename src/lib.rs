pub mod app;
pub mod bootstrap;
mod gui_state;
pub mod hasse_layout;
pub mod relation_matrix;
mod rendered_diagram;

pub fn run() -> eframe::Result<()> {
    app::run()
}
