pub mod app;
pub mod bootstrap;
mod gui_state;
pub mod hasse_layout;
pub mod relation_matrix;
mod rendered_diagram;

pub type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

pub fn run() -> AppResult<()> {
    app::run()
}
