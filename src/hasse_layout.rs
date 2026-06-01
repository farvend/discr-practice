use crate::relation_matrix::HasseCoverEdge;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HasseLayoutError {
    InvalidNodeCount { node_count: usize },
    EdgeOutOfBounds {
        edge: HasseCoverEdge,
        node_count: usize,
    },
    CycleDetected,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NormalizedPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HasseLayoutNode {
    pub label: usize,
    pub level: usize,
    pub index_in_level: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HasseLayout {
    nodes: Vec<HasseLayoutNode>,
    level_widths: Vec<usize>,
}

impl HasseLayout {
    pub fn new(
        node_count: usize,
        cover_edges: &[HasseCoverEdge],
    ) -> Result<Self, HasseLayoutError> {
        HasseLayoutBuilder::new(node_count)?
            .with_cover_edges(cover_edges)?
            .build()
    }

    pub fn nodes(&self) -> &[HasseLayoutNode] {
        &self.nodes
    }

    pub fn node(&self, label: usize) -> Option<&HasseLayoutNode> {
        self.nodes.get(label.checked_sub(1)?)
    }

    pub fn level_count(&self) -> usize {
        self.level_widths.len()
    }

    pub fn level_width(&self, level: usize) -> Option<usize> {
        self.level_widths.get(level).copied()
    }

    pub fn normalized_position(&self, label: usize) -> Option<NormalizedPosition> {
        let node = self.node(label)?;
        let level_width = self.level_width(node.level)?;

        let x = if level_width <= 1 {
            0.5
        } else {
            node.index_in_level as f32 / (level_width - 1) as f32
        };

        let y = if self.level_count() <= 1 {
            0.0
        } else {
            node.level as f32 / (self.level_count() - 1) as f32
        };

        Some(NormalizedPosition { x, y })
    }
}

struct HasseLayoutBuilder {
    node_count: usize,
    successors: Vec<Vec<usize>>,
    incoming_counts: Vec<usize>,
}

impl HasseLayoutBuilder {
    fn new(node_count: usize) -> Result<Self, HasseLayoutError> {
        if node_count == 0 {
            return Err(HasseLayoutError::InvalidNodeCount { node_count });
        }

        Ok(Self {
            node_count,
            successors: vec![Vec::new(); node_count + 1],
            incoming_counts: vec![0usize; node_count + 1],
        })
    }

    fn with_cover_edges(mut self, cover_edges: &[HasseCoverEdge]) -> Result<Self, HasseLayoutError> {
        for edge in cover_edges {
            self.add_edge(*edge)?;
        }

        for targets in self.successors.iter_mut().skip(1) {
            targets.sort_unstable();
        }

        Ok(self)
    }

    fn build(self) -> Result<HasseLayout, HasseLayoutError> {
        let topological_order = self.topological_order()?;
        let levels = self.levels_by_label(&topological_order);

        Ok(self.into_layout(levels))
    }

    fn add_edge(&mut self, edge: HasseCoverEdge) -> Result<(), HasseLayoutError> {
        if !(1..=self.node_count).contains(&edge.lower)
            || !(1..=self.node_count).contains(&edge.upper)
        {
            return Err(HasseLayoutError::EdgeOutOfBounds {
                edge,
                node_count: self.node_count,
            });
        }

        self.successors[edge.lower].push(edge.upper);
        self.incoming_counts[edge.upper] += 1;

        Ok(())
    }

    fn topological_order(&self) -> Result<Vec<usize>, HasseLayoutError> {
        let mut incoming_counts = self.incoming_counts.clone();
        let mut ready = std::collections::BTreeSet::new();

        for label in 1..=self.node_count {
            if incoming_counts[label] == 0 {
                ready.insert(label);
            }
        }

        let mut order = Vec::with_capacity(self.node_count);

        while let Some(&label) = ready.iter().next() {
            ready.remove(&label);
            order.push(label);

            for &upper in &self.successors[label] {
                incoming_counts[upper] -= 1;
                if incoming_counts[upper] == 0 {
                    ready.insert(upper);
                }
            }
        }

        if order.len() == self.node_count {
            Ok(order)
        } else {
            Err(HasseLayoutError::CycleDetected)
        }
    }

    fn levels_by_label(&self, topological_order: &[usize]) -> Vec<usize> {
        let mut levels = vec![0usize; self.node_count + 1];

        for &lower in topological_order {
            let next_level = levels[lower] + 1;
            for &upper in &self.successors[lower] {
                levels[upper] = levels[upper].max(next_level);
            }
        }

        levels
    }

    fn into_layout(self, levels: Vec<usize>) -> HasseLayout {
        let mut levels_to_labels = self.labels_by_level(&levels);
        let mut indices_in_level = vec![0usize; self.node_count + 1];

        for labels in &mut levels_to_labels {
            labels.sort_unstable();

            for (index_in_level, &label) in labels.iter().enumerate() {
                indices_in_level[label] = index_in_level;
            }
        }

        HasseLayout {
            nodes: (1..=self.node_count)
                .map(|label| HasseLayoutNode {
                    label,
                    level: levels[label],
                    index_in_level: indices_in_level[label],
                })
                .collect(),
            level_widths: levels_to_labels.iter().map(Vec::len).collect(),
        }
    }

    fn labels_by_level(&self, levels: &[usize]) -> Vec<Vec<usize>> {
        let level_count = levels.iter().skip(1).copied().max().unwrap_or(0) + 1;
        let mut levels_to_labels = vec![Vec::new(); level_count];

        for label in 1..=self.node_count {
            levels_to_labels[levels[label]].push(label);
        }

        levels_to_labels
    }
}

#[cfg(test)]
mod tests {
    use super::{HasseLayout, HasseLayoutNode};
    use crate::relation_matrix::{HasseCoverEdge, RelationMatrix, RelationMatrixError};

    #[derive(Debug, Clone, Copy)]
    struct RelationCell {
        row: usize,
        column: usize,
    }

    fn cell(row: usize, column: usize) -> RelationCell {
        RelationCell { row, column }
    }

    fn edge(lower: usize, upper: usize) -> HasseCoverEdge {
        HasseCoverEdge { lower, upper }
    }

    fn matrix_with_true_pairs(
        size: usize,
        pairs: &[RelationCell],
    ) -> Result<RelationMatrix, RelationMatrixError> {
        let mut matrix = RelationMatrix::new(size)?;

        for pair in pairs {
            matrix.set(pair.row, pair.column, true)?;
        }

        Ok(matrix)
    }

    #[test]
    fn layout_is_deterministic_for_same_cover_edges() {
        let matrix = matrix_with_true_pairs(
            4,
            &[
                cell(1, 1),
                cell(2, 2),
                cell(3, 3),
                cell(4, 4),
                cell(1, 2),
                cell(1, 3),
                cell(1, 4),
                cell(2, 4),
                cell(3, 4),
            ],
        )
        .expect("valid matrix must be created");
        let cover_edges = matrix
            .hasse_cover_edges()
            .expect("valid partial order must yield cover edges");

        let first = HasseLayout::new(matrix.size(), &cover_edges)
            .expect("layout should succeed for valid cover edges");
        let second = HasseLayout::new(matrix.size(), &cover_edges)
            .expect("layout should be repeatable");

        assert_eq!(first, second);
    }

    #[test]
    fn layout_covers_every_node_label_from_one_to_n() {
        let matrix = matrix_with_true_pairs(
            4,
            &[
                cell(1, 1),
                cell(2, 2),
                cell(3, 3),
                cell(4, 4),
                cell(1, 2),
            ],
        )
        .expect("valid matrix must be created");
        let cover_edges = matrix
            .hasse_cover_edges()
            .expect("valid partial order must yield cover edges");

        let layout = HasseLayout::new(matrix.size(), &cover_edges)
            .expect("layout should succeed for valid cover edges");
        let labels: Vec<usize> = layout.nodes().iter().map(|node| node.label).collect();

        assert_eq!(labels, vec![1, 2, 3, 4]);
    }

    #[test]
    fn cover_edges_always_point_upward_in_layout() {
        let matrix = matrix_with_true_pairs(
            5,
            &[
                cell(1, 1),
                cell(2, 2),
                cell(3, 3),
                cell(4, 4),
                cell(5, 5),
                cell(1, 2),
                cell(1, 3),
                cell(2, 4),
                cell(3, 4),
                cell(4, 5),
                cell(1, 4),
                cell(1, 5),
                cell(2, 5),
                cell(3, 5),
            ],
        )
        .expect("valid matrix must be created");
        let cover_edges = matrix
            .hasse_cover_edges()
            .expect("valid partial order must yield cover edges");

        let layout = HasseLayout::new(matrix.size(), &cover_edges)
            .expect("layout should succeed for valid cover edges");

        for edge in &cover_edges {
            let lower_node = layout.node(edge.lower).expect("lower node must exist");
            let upper_node = layout.node(edge.upper).expect("upper node must exist");
            let lower_position = layout
                .normalized_position(edge.lower)
                .expect("lower node must have coordinates");
            let upper_position = layout
                .normalized_position(edge.upper)
                .expect("upper node must have coordinates");

            assert!(lower_node.level < upper_node.level);
            assert!(lower_position.y < upper_position.y);
        }
    }

    #[test]
    fn diamond_layout_uses_stable_levels_and_ordering_inside_level() {
        let matrix = matrix_with_true_pairs(
            4,
            &[
                cell(1, 1),
                cell(2, 2),
                cell(3, 3),
                cell(4, 4),
                cell(1, 2),
                cell(1, 3),
                cell(1, 4),
                cell(2, 4),
                cell(3, 4),
            ],
        )
        .expect("valid matrix must be created");
        let cover_edges = matrix
            .hasse_cover_edges()
            .expect("valid partial order must yield cover edges");

        let layout = HasseLayout::new(matrix.size(), &cover_edges)
            .expect("layout should succeed for valid cover edges");

        assert_eq!(layout.level_count(), 3);
        assert_eq!(layout.level_width(0), Some(1));
        assert_eq!(layout.level_width(1), Some(2));
        assert_eq!(layout.level_width(2), Some(1));
        assert_eq!(
            layout.nodes(),
            &[
                HasseLayoutNode {
                    label: 1,
                    level: 0,
                    index_in_level: 0,
                },
                HasseLayoutNode {
                    label: 2,
                    level: 1,
                    index_in_level: 0,
                },
                HasseLayoutNode {
                    label: 3,
                    level: 1,
                    index_in_level: 1,
                },
                HasseLayoutNode {
                    label: 4,
                    level: 2,
                    index_in_level: 0,
                },
            ]
        );

        assert_eq!(
            cover_edges,
            vec![edge(1, 2), edge(1, 3), edge(2, 4), edge(3, 4)]
        );
    }
}
