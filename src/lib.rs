pub mod app;
pub mod bootstrap;
pub mod hasse_layout;
pub mod relation_matrix;

pub fn run() -> eframe::Result<()> {
    app::run()
}
