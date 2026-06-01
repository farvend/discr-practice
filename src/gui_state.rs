use crate::hasse_layout::HasseLayout;
use crate::relation_matrix::{MIN_RELATION_SIZE, PartialOrderDiagnostics, RelationMatrix};
use crate::rendered_diagram::RenderedDiagram;

#[derive(Debug, Clone)]
pub(crate) enum BuildState {
    Idle,
    Dirty,
    Invalid(PartialOrderDiagnostics),
    Valid(RenderedDiagram),
}

#[derive(Debug, Clone)]
pub(crate) struct HasseGuiState {
    relation_matrix: RelationMatrix,
    status_message: String,
    pub(crate) build_state: BuildState,
}

impl HasseGuiState {
    pub(crate) fn new(initial_status_message: &str) -> Self {
        Self {
            relation_matrix: RelationMatrix::new(MIN_RELATION_SIZE)
                .expect("minimum supported size must create the initial matrix"),
            status_message: initial_status_message.to_owned(),
            build_state: BuildState::Idle,
        }
    }

    pub(crate) fn size(&self) -> usize {
        self.relation_matrix.size()
    }

    pub(crate) fn cell(&self, row: usize, column: usize) -> bool {
        self.relation_matrix
            .get(row, column)
            .expect("matrix access from the UI must stay within 1-based bounds")
    }

    pub(crate) fn set_cell(&mut self, row: usize, column: usize, value: bool) {
        self.relation_matrix
            .set(row, column, value)
            .expect("matrix writes from the UI must stay within 1-based bounds");
        self.invalidate_build("Матрица изменена. Нажмите кнопку проверки, чтобы обновить диаграмму.");
    }

    pub(crate) fn resize(&mut self, size: usize) {
        self.relation_matrix = RelationMatrix::new(size)
            .expect("size slider only allows values from the supported range");
        self.invalidate_build("Размер матрицы изменён. Заполните значения и запустите проверку.");
    }

    pub(crate) fn build_diagram(&mut self) {
        match self.relation_matrix.validate_partial_order() {
            Ok(()) => self.build_valid_diagram(),
            Err(diagnostics) => {
                self.status_message =
                    "Отношение не является частичным порядком. Диаграмма не построена.".to_owned();
                self.build_state = BuildState::Invalid(diagnostics);
            }
        }
    }

    pub(crate) fn status_message(&self) -> &str {
        &self.status_message
    }

    pub(crate) fn rendered_diagram(&self) -> Option<&RenderedDiagram> {
        match &self.build_state {
            BuildState::Valid(diagram) => Some(diagram),
            _ => None,
        }
    }

    pub(crate) fn result_messages(&self) -> Vec<String> {
        match &self.build_state {
            BuildState::Idle => vec!["- Диаграмма ещё не строилась.".to_owned()],
            BuildState::Dirty => vec!["- Текущая диаграмма сброшена: матрица изменилась.".to_owned()],
            BuildState::Invalid(diagnostics) => diagnostics.report_messages(),
            BuildState::Valid(diagram) => vec![
                "- Проверка частичного порядка пройдена.".to_owned(),
                format!("- Рёбра покрытия: {}.", diagram.formatted_cover_edges()),
                format!("- Уровней layout: {}.", diagram.layout.level_count()),
            ],
        }
    }

    fn build_valid_diagram(&mut self) {
        let cover_edges = self
            .relation_matrix
            .hasse_cover_edges()
            .expect("validated partial order must produce cover edges");

        match HasseLayout::new(self.relation_matrix.size(), &cover_edges) {
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

    fn invalidate_build(&mut self, status_message: &str) {
        self.status_message = status_message.to_owned();
        self.build_state = BuildState::Dirty;
    }
}

#[cfg(test)]
mod tests {
    use super::{BuildState, HasseGuiState};
    use crate::relation_matrix::HasseCoverEdge;

    fn edge(lower: usize, upper: usize) -> HasseCoverEdge {
        HasseCoverEdge { lower, upper }
    }

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

        assert_eq!(diagram.cover_edges, vec![edge(1, 2), edge(2, 3)]);
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
        let messages = state.result_messages();
        assert!(messages
            .iter()
            .any(|message| message.contains("Транзитивность нарушена")));
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
