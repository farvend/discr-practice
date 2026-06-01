pub const MIN_RELATION_SIZE: usize = 1;
pub const MAX_RELATION_SIZE: usize = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationMatrixError {
    InvalidSize {
        size: usize,
        min: usize,
        max: usize,
    },
    ElementOutOfBounds {
        element: usize,
        size: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AntisymmetryWitness {
    pub left: usize,
    pub right: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitivityWitness {
    pub source: usize,
    pub middle: usize,
    pub target: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HasseCoverEdge {
    pub lower: usize,
    pub upper: usize,
}

impl std::fmt::Display for HasseCoverEdge {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "({}, {})", self.lower, self.upper)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialOrderDiagnostics {
    pub reflexivity_witness: Option<usize>,
    pub antisymmetry_witness: Option<AntisymmetryWitness>,
    pub transitivity_witness: Option<TransitivityWitness>,
}

impl PartialOrderDiagnostics {
    pub fn is_valid(&self) -> bool {
        self.reflexivity_witness.is_none()
            && self.antisymmetry_witness.is_none()
            && self.transitivity_witness.is_none()
    }

    pub fn report_messages(&self) -> Vec<String> {
        let mut messages = vec!["- Нарушены условия частичного порядка:".to_owned()];

        match self.reflexivity_witness {
            Some(element) => messages.push(format!(
                "  • Рефлексивность нарушена: ({0}, {0}) = 0.",
                element
            )),
            None => messages.push("  • Рефлексивность: OK.".to_owned()),
        }

        match self.antisymmetry_witness {
            Some(witness) => messages.push(format!(
                "  • Антисимметричность нарушена: ({}, {}) = 1 и ({}, {}) = 1.",
                witness.left, witness.right, witness.right, witness.left,
            )),
            None => messages.push("  • Антисимметричность: OK.".to_owned()),
        }

        match self.transitivity_witness {
            Some(witness) => messages.push(format!(
                "  • Транзитивность нарушена: ({}, {}) = 1 и ({}, {}) = 1, но ({}, {}) = 0.",
                witness.source,
                witness.middle,
                witness.middle,
                witness.target,
                witness.source,
                witness.target,
            )),
            None => messages.push("  • Транзитивность: OK.".to_owned()),
        }

        messages
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationMatrix {
    size: usize,
    cells: Vec<bool>,
}

impl RelationMatrix {
    pub fn new(size: usize) -> Result<Self, RelationMatrixError> {
        Self::validate_size(size)?;

        Ok(Self {
            size,
            cells: vec![false; size * size],
        })
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn get(&self, row: usize, column: usize) -> Result<bool, RelationMatrixError> {
        let index = self.index_of(row, column)?;
        Ok(self.cells[index])
    }

    pub fn set(
        &mut self,
        row: usize,
        column: usize,
        value: bool,
    ) -> Result<(), RelationMatrixError> {
        let index = self.index_of(row, column)?;
        self.cells[index] = value;
        Ok(())
    }

    pub fn validate_partial_order(&self) -> Result<(), PartialOrderDiagnostics> {
        let diagnostics = PartialOrderDiagnostics {
            reflexivity_witness: self.find_reflexivity_witness(),
            antisymmetry_witness: self.find_antisymmetry_witness(),
            transitivity_witness: self.find_transitivity_witness(),
        };

        if diagnostics.is_valid() {
            Ok(())
        } else {
            Err(diagnostics)
        }
    }

    pub fn hasse_cover_edges(&self) -> Result<Vec<HasseCoverEdge>, PartialOrderDiagnostics> {
        self.validate_partial_order()?;

        let mut edges = Vec::new();

        for lower in 1..=self.size {
            for upper in 1..=self.size {
                if lower == upper || !self.get_unchecked(lower, upper) {
                    continue;
                }

                if !self.has_intermediate_between(lower, upper) {
                    edges.push(HasseCoverEdge { lower, upper });
                }
            }
        }

        Ok(edges)
    }

    fn index_of(&self, row: usize, column: usize) -> Result<usize, RelationMatrixError> {
        self.validate_element(row)?;
        self.validate_element(column)?;

        Ok((row - 1) * self.size + (column - 1))
    }

    fn validate_size(size: usize) -> Result<(), RelationMatrixError> {
        if (MIN_RELATION_SIZE..=MAX_RELATION_SIZE).contains(&size) {
            Ok(())
        } else {
            Err(RelationMatrixError::InvalidSize {
                size,
                min: MIN_RELATION_SIZE,
                max: MAX_RELATION_SIZE,
            })
        }
    }

    fn validate_element(&self, element: usize) -> Result<(), RelationMatrixError> {
        if (1..=self.size).contains(&element) {
            Ok(())
        } else {
            Err(RelationMatrixError::ElementOutOfBounds {
                element,
                size: self.size,
            })
        }
    }

    fn get_unchecked(&self, row: usize, column: usize) -> bool {
        self.cells[(row - 1) * self.size + (column - 1)]
    }

    fn find_reflexivity_witness(&self) -> Option<usize> {
        (1..=self.size).find(|&element| !self.get_unchecked(element, element))
    }

    fn find_antisymmetry_witness(&self) -> Option<AntisymmetryWitness> {
        for left in 1..=self.size {
            for right in (left + 1)..=self.size {
                if self.get_unchecked(left, right) && self.get_unchecked(right, left) {
                    return Some(AntisymmetryWitness { left, right });
                }
            }
        }

        None
    }

    fn find_transitivity_witness(&self) -> Option<TransitivityWitness> {
        for source in 1..=self.size {
            for middle in 1..=self.size {
                if !self.get_unchecked(source, middle) {
                    continue;
                }

                for target in 1..=self.size {
                    if self.get_unchecked(middle, target) && !self.get_unchecked(source, target) {
                        return Some(TransitivityWitness {
                            source,
                            middle,
                            target,
                        });
                    }
                }
            }
        }

        None
    }

    fn has_intermediate_between(&self, lower: usize, upper: usize) -> bool {
        for middle in 1..=self.size {
            if middle == lower || middle == upper {
                continue;
            }

            if self.get_unchecked(lower, middle) && self.get_unchecked(middle, upper) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AntisymmetryWitness, HasseCoverEdge, MAX_RELATION_SIZE, MIN_RELATION_SIZE,
        PartialOrderDiagnostics, RelationMatrix, RelationMatrixError, TransitivityWitness,
    };

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
    fn relation_matrix_creation_accepts_boundary_sizes() {
        let single = RelationMatrix::new(MIN_RELATION_SIZE).expect("n=1 must be valid");
        let max = RelationMatrix::new(MAX_RELATION_SIZE).expect("n=10 must be valid");

        assert_eq!(single.size(), MIN_RELATION_SIZE);
        assert_eq!(max.size(), MAX_RELATION_SIZE);
        assert_eq!(single.get(1, 1), Ok(false));
        assert_eq!(max.get(MAX_RELATION_SIZE, MAX_RELATION_SIZE), Ok(false));
    }

    #[test]
    fn relation_matrix_creation_rejects_sizes_outside_supported_range() {
        assert_eq!(
            RelationMatrix::new(0),
            Err(RelationMatrixError::InvalidSize {
                size: 0,
                min: MIN_RELATION_SIZE,
                max: MAX_RELATION_SIZE,
            })
        );

        assert_eq!(
            RelationMatrix::new(MAX_RELATION_SIZE + 1),
            Err(RelationMatrixError::InvalidSize {
                size: MAX_RELATION_SIZE + 1,
                min: MIN_RELATION_SIZE,
                max: MAX_RELATION_SIZE,
            })
        );
    }

    #[test]
    fn relation_matrix_get_and_set_work_for_elements_numbered_from_one() {
        let mut matrix = RelationMatrix::new(3).expect("valid size must construct matrix");

        matrix.set(1, 2, true).expect("set within range must succeed");
        matrix.set(3, 1, true).expect("set within range must succeed");

        assert_eq!(matrix.get(1, 2), Ok(true));
        assert_eq!(matrix.get(2, 1), Ok(false));
        assert_eq!(matrix.get(3, 1), Ok(true));
    }

    #[test]
    fn relation_matrix_reports_bounds_errors_for_get_and_set() {
        let mut matrix = RelationMatrix::new(3).expect("valid size must construct matrix");

        assert_eq!(
            matrix.get(0, 1),
            Err(RelationMatrixError::ElementOutOfBounds {
                element: 0,
                size: 3,
            })
        );

        assert_eq!(
            matrix.get(1, 4),
            Err(RelationMatrixError::ElementOutOfBounds {
                element: 4,
                size: 3,
            })
        );

        assert_eq!(
            matrix.set(4, 2, true),
            Err(RelationMatrixError::ElementOutOfBounds {
                element: 4,
                size: 3,
            })
        );
    }

    #[test]
    fn validate_partial_order_accepts_valid_partial_order() {
        let matrix = matrix_with_true_pairs(
            3,
            &[
                cell(1, 1),
                cell(2, 2),
                cell(3, 3),
                cell(1, 2),
                cell(1, 3),
                cell(2, 3),
            ],
        )
        .expect("valid matrix must be created");

        assert_eq!(matrix.validate_partial_order(), Ok(()));
    }

    #[test]
    fn validate_partial_order_reports_reflexivity_witness() {
        let matrix = matrix_with_true_pairs(
            3,
            &[
                cell(1, 1),
                cell(3, 3),
            ],
        )
        .expect("valid matrix must be created");

        assert_eq!(
            matrix.validate_partial_order(),
            Err(PartialOrderDiagnostics {
                reflexivity_witness: Some(2),
                antisymmetry_witness: None,
                transitivity_witness: None,
            })
        );
    }

    #[test]
    fn validate_partial_order_reports_antisymmetry_witness() {
        let matrix = matrix_with_true_pairs(
            3,
            &[
                cell(1, 1),
                cell(2, 2),
                cell(3, 3),
                cell(1, 2),
                cell(2, 1),
            ],
        )
        .expect("valid matrix must be created");

        assert_eq!(
            matrix.validate_partial_order(),
            Err(PartialOrderDiagnostics {
                reflexivity_witness: None,
                antisymmetry_witness: Some(AntisymmetryWitness { left: 1, right: 2 }),
                transitivity_witness: None,
            })
        );
    }

    #[test]
    fn validate_partial_order_reports_transitivity_witness() {
        let matrix = matrix_with_true_pairs(
            3,
            &[
                cell(1, 1),
                cell(2, 2),
                cell(3, 3),
                cell(1, 2),
                cell(2, 3),
            ],
        )
        .expect("valid matrix must be created");

        assert_eq!(
            matrix.validate_partial_order(),
            Err(PartialOrderDiagnostics {
                reflexivity_witness: None,
                antisymmetry_witness: None,
                transitivity_witness: Some(TransitivityWitness {
                    source: 1,
                    middle: 2,
                    target: 3,
                }),
            })
        );
    }

    #[test]
    fn hasse_cover_edges_exclude_transitive_chain_edge() {
        let matrix = matrix_with_true_pairs(
            3,
            &[
                cell(1, 1),
                cell(2, 2),
                cell(3, 3),
                cell(1, 2),
                cell(2, 3),
                cell(1, 3),
            ],
        )
        .expect("valid matrix must be created");

        assert_eq!(
            matrix.hasse_cover_edges(),
            Ok(vec![edge(1, 2), edge(2, 3)])
        );
    }

    #[test]
    fn hasse_cover_edges_keep_branching_cover_edges_in_deterministic_order() {
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

        assert_eq!(
            matrix.hasse_cover_edges(),
            Ok(vec![edge(1, 2), edge(1, 3), edge(2, 4), edge(3, 4)])
        );
    }

    #[test]
    fn hasse_cover_edges_return_validation_error_for_invalid_relation() {
        let matrix = matrix_with_true_pairs(
            3,
            &[
                cell(1, 1),
                cell(2, 2),
                cell(3, 3),
                cell(1, 2),
                cell(2, 3),
            ],
        )
        .expect("valid matrix must be created");

        assert_eq!(
            matrix.hasse_cover_edges(),
            Err(PartialOrderDiagnostics {
                reflexivity_witness: None,
                antisymmetry_witness: None,
                transitivity_witness: Some(TransitivityWitness {
                    source: 1,
                    middle: 2,
                    target: 3,
                }),
            })
        );
    }
}
