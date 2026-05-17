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

    pub fn normalized_position(&self, label: usize) -> Option<(f32, f32)> {
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

        Some((x, y))
    }
}

pub fn layout_hasse_nodes(
    node_count: usize,
    cover_edges: &[HasseCoverEdge],
) -> Result<HasseLayout, HasseLayoutError> {
    if node_count == 0 {
        return Err(HasseLayoutError::InvalidNodeCount { node_count });
    }

    let mut successors = vec![Vec::new(); node_count + 1];
    let mut incoming_counts = vec![0usize; node_count + 1];

    for &(lower, upper) in cover_edges {
        if !(1..=node_count).contains(&lower) || !(1..=node_count).contains(&upper) {
            return Err(HasseLayoutError::EdgeOutOfBounds {
                edge: (lower, upper),
                node_count,
            });
        }

        successors[lower].push(upper);
        incoming_counts[upper] += 1;
    }

    for targets in successors.iter_mut().skip(1) {
        targets.sort_unstable();
    }

    let mut ready = std::collections::BTreeSet::new();

    for label in 1..=node_count {
        if incoming_counts[label] == 0 {
            ready.insert(label);
        }
    }

    let mut topological_order = Vec::with_capacity(node_count);

    while let Some(&label) = ready.iter().next() {
        ready.remove(&label);
        topological_order.push(label);

        for &upper in &successors[label] {
            incoming_counts[upper] -= 1;
            if incoming_counts[upper] == 0 {
                ready.insert(upper);
            }
        }
    }

    if topological_order.len() != node_count {
        return Err(HasseLayoutError::CycleDetected);
    }

    let mut levels = vec![0usize; node_count + 1];

    for &lower in &topological_order {
        let next_level = levels[lower] + 1;
        for &upper in &successors[lower] {
            levels[upper] = levels[upper].max(next_level);
        }
    }

    let level_count = levels.iter().skip(1).copied().max().unwrap_or(0) + 1;
    let mut levels_to_labels = vec![Vec::new(); level_count];

    for label in 1..=node_count {
        levels_to_labels[levels[label]].push(label);
    }

    let mut indices_in_level = vec![0usize; node_count + 1];

    for labels in &mut levels_to_labels {
        labels.sort_unstable();

        for (index_in_level, &label) in labels.iter().enumerate() {
            indices_in_level[label] = index_in_level;
        }
    }

    Ok(HasseLayout {
        nodes: (1..=node_count)
            .map(|label| HasseLayoutNode {
                label,
                level: levels[label],
                index_in_level: indices_in_level[label],
            })
            .collect(),
        level_widths: levels_to_labels.iter().map(Vec::len).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::{layout_hasse_nodes, HasseLayoutNode};
    use crate::relation_matrix::{RelationMatrix, RelationMatrixError};

    fn matrix_with_true_pairs(
        size: usize,
        pairs: &[(usize, usize)],
    ) -> Result<RelationMatrix, RelationMatrixError> {
        let mut matrix = RelationMatrix::new(size)?;

        for &(row, column) in pairs {
            matrix.set(row, column, true)?;
        }

        Ok(matrix)
    }

    #[test]
    fn layout_is_deterministic_for_same_cover_edges() {
        let matrix = matrix_with_true_pairs(
            4,
            &[
                (1, 1),
                (2, 2),
                (3, 3),
                (4, 4),
                (1, 2),
                (1, 3),
                (1, 4),
                (2, 4),
                (3, 4),
            ],
        )
        .expect("valid matrix must be created");
        let cover_edges = matrix
            .hasse_cover_edges()
            .expect("valid partial order must yield cover edges");

        let first = layout_hasse_nodes(matrix.size(), &cover_edges)
            .expect("layout should succeed for valid cover edges");
        let second = layout_hasse_nodes(matrix.size(), &cover_edges)
            .expect("layout should be repeatable");

        assert_eq!(first, second);
    }

    #[test]
    fn layout_covers_every_node_label_from_one_to_n() {
        let matrix = matrix_with_true_pairs(4, &[(1, 1), (2, 2), (3, 3), (4, 4), (1, 2)])
            .expect("valid matrix must be created");
        let cover_edges = matrix
            .hasse_cover_edges()
            .expect("valid partial order must yield cover edges");

        let layout = layout_hasse_nodes(matrix.size(), &cover_edges)
            .expect("layout should succeed for valid cover edges");
        let labels: Vec<usize> = layout.nodes().iter().map(|node| node.label).collect();

        assert_eq!(labels, vec![1, 2, 3, 4]);
    }

    #[test]
    fn cover_edges_always_point_upward_in_layout() {
        let matrix = matrix_with_true_pairs(
            5,
            &[
                (1, 1),
                (2, 2),
                (3, 3),
                (4, 4),
                (5, 5),
                (1, 2),
                (1, 3),
                (2, 4),
                (3, 4),
                (4, 5),
                (1, 4),
                (1, 5),
                (2, 5),
                (3, 5),
            ],
        )
        .expect("valid matrix must be created");
        let cover_edges = matrix
            .hasse_cover_edges()
            .expect("valid partial order must yield cover edges");

        let layout = layout_hasse_nodes(matrix.size(), &cover_edges)
            .expect("layout should succeed for valid cover edges");

        for (lower, upper) in cover_edges {
            let lower_node = layout.node(lower).expect("lower node must exist");
            let upper_node = layout.node(upper).expect("upper node must exist");
            let (_, lower_y) = layout
                .normalized_position(lower)
                .expect("lower node must have coordinates");
            let (_, upper_y) = layout
                .normalized_position(upper)
                .expect("upper node must have coordinates");

            assert!(lower_node.level < upper_node.level);
            assert!(lower_y < upper_y);
        }
    }

    #[test]
    fn diamond_layout_uses_stable_levels_and_ordering_inside_level() {
        let matrix = matrix_with_true_pairs(
            4,
            &[
                (1, 1),
                (2, 2),
                (3, 3),
                (4, 4),
                (1, 2),
                (1, 3),
                (1, 4),
                (2, 4),
                (3, 4),
            ],
        )
        .expect("valid matrix must be created");
        let cover_edges = matrix
            .hasse_cover_edges()
            .expect("valid partial order must yield cover edges");

        let layout = layout_hasse_nodes(matrix.size(), &cover_edges)
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
    }
}
