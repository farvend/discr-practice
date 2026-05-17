pub const APP_TITLE: &str = "Диаграмма Хассе";
pub const READY_MESSAGE: &str =
    "Каркас приложения готов. Следующий шаг — ввод матрицы отношения на множестве 1..n.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapState {
    app_title: &'static str,
    status_message: String,
}

impl BootstrapState {
    pub fn new() -> Self {
        Self {
            app_title: APP_TITLE,
            status_message: READY_MESSAGE.to_owned(),
        }
    }

    pub fn app_title(&self) -> &'static str {
        self.app_title
    }

    pub fn status_message(&self) -> &str {
        &self.status_message
    }

    pub fn is_ready_for_matrix_input(&self) -> bool {
        self.status_message.contains("1..n")
    }
}

impl Default for BootstrapState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{APP_TITLE, BootstrapState, READY_MESSAGE};

    #[test]
    fn bootstrap_state_reports_readiness_for_matrix_input() {
        let state = BootstrapState::new();

        assert_eq!(state.app_title(), APP_TITLE);
        assert_eq!(state.status_message(), READY_MESSAGE);
        assert!(state.is_ready_for_matrix_input());
    }
}
