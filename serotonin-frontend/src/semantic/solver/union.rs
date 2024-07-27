use std::collections::HashSet;

use super::{definition::Constraint, Reduction, StackValue};

/// Represents the set union of [`Constraint`] structs.
///
/// The [`Union::find_constraint`] finds the first constraint in the union that matches a given state.
///
/// The [`Union::is_subset`] (called via [`Union::add`]) method is used to check if a new definition constraint is already completely covered
/// by the existing constraints (and thus inaccessible).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Union(Vec<Constraint>);

impl FromIterator<Constraint> for Union {
    fn from_iter<I: IntoIterator<Item = Constraint>>(iter: I) -> Self {
        Union(iter.into_iter().collect())
    }
}

#[cfg(test)]
use super::positional::PositionalConstraint;

#[cfg(test)]
impl From<Vec<Vec<PositionalConstraint>>> for Union {
    fn from(v: Vec<Vec<PositionalConstraint>>) -> Self {
        v.into_iter().map(Constraint::new).collect()
    }
}

impl Union {
    /// Create a new empty union of constraints
    pub fn new() -> Self {
        Union(Vec::new())
    }

    /// Get the index of the first constraint in the union that contains the given state.
    ///
    /// Returns `Some(index)` if a matching constraint is found, or `None` otherwise.    
    pub fn find_constraint(&self, state: &[StackValue]) -> Option<usize> {
        self.0.iter().position(|c| c.contains(state))
    }

    /// Check that the inner constraints have the same length as the given state.
    pub fn check_len(&self, len: usize) -> bool {
        self.0.iter().all(|c| c.len() == len)
    }

    /// Adds a new constraint if it isn't already a subset of the existing constraints.
    pub fn try_push(&mut self, constraint: Constraint) -> bool {
        debug_assert!(
            self.check_len(constraint.len()),
            "Constraints must have the same length"
        );
        if self.is_subset(&constraint) {
            return false;
        }
        self.push(constraint);
        true
    }

    /// Adds a new constraint to the union.
    pub fn push(&mut self, constraint: Constraint) {
        self.0.push(constraint);
    }

    /// Reduces this union of constraints by giving a specific value to the first positional constraint
    fn reduce(&self, v: &Reduction) -> Union {
        self.0.iter().map(|c| c.reduce(v)).collect()
    }

    /// Checks if a new constraint is a subset of the existing constraints.
    ///
    /// This is a recursive procedure that reduces constraints step-by-step by either applying appropriate [`Reduction`] values.
    fn is_subset(&self, constraint: &Constraint) -> bool {
        // Base case (length 0 constraints)
        //
        // Length 0 constraint unions can only contain at most 1 constraint
        //
        // ```
        // foo == ...; // unreachable
        // foo == ...; // reachable
        // ```
        if constraint.is_empty() {
            return !self.0.is_empty();
        }

        // Get the first positional constraint
        let incoming_first = constraint.iter().next().unwrap();
        let union_firsts = self
            .0
            .iter()
            .map(|c| c.iter().next().unwrap())
            .filter(|p| p.is_byte() == incoming_first.is_byte())
            .collect::<Vec<_>>();

        // Nothing is a subset of the empty set
        if union_firsts.is_empty() {
            return false;
        }

        let incoming_is_any = incoming_first.is_any();
        // An 'Any' constraint can be created naively by having an Any / Position(0) constraint, or be enumerating all possible ExactByte values.
        let mut union_has_any = union_firsts.iter().any(|p| p.is_any());
        if !union_has_any && union_firsts.len() >= 256 {
            let num = union_firsts
                .iter()
                .filter_map(|p| p.exact_value().and_then(|v| v.byte()))
                .collect::<HashSet<_>>()
                .len();

            union_has_any = num >= 256;
        }

        let is_byte = incoming_first.is_byte();
        let (new_union, new_constraint) = match (union_has_any, incoming_is_any) {
            // Any == us >= g | continue
            (true, _) => (
                self.reduce(&Reduction::new_any(is_byte)),
                constraint.reduce(&Reduction::new_any(is_byte)),
            ),
            // us < g == Any | there is at least 1 element in the incoming constraint that is not in the Union
            (false, true) => {
                return false;
            }
            // neither are Any | so we need to check if the incoming constraint matches any of the union constraints
            (false, false) => {
                let value = incoming_first.exact_value().unwrap();
                let reduction = Reduction::from(value.clone());
                if union_firsts
                    .iter()
                    .any(|p| p.exact_value().unwrap() == value)
                {
                    (self.reduce(&reduction), constraint.reduce(&reduction))
                } else {
                    return false;
                }
            }
        };

        new_union.is_subset(&new_constraint)
    }
}

#[cfg(test)]
mod tests {
    use crate::semantic::solver::{positional::PositionalConstraint, Constraint, Union};

    /// Property test: Union::add(_) always returns true if the union is empty
    #[test]
    fn union_add_empty() {
        for n in 1..10 {
            for _ in 0..100 {
                let mut u = Union::new();
                let c = Constraint::random(n, None);
                assert!(
                    u.try_push(c),
                    "Union::add(_) should always return true if the union is empty"
                );
            }
        }
    }

    /// Property test: Union::add(_) always returns false if the union already contains the constraint [Any, Any, Any, ...]
    #[test]
    fn union_add_any() {
        for n in 1..10 {
            for _ in 0..100 {
                let bytes = rand::random();
                let mut u = Union::new();

                let anys = if bytes {
                    Constraint::new(vec![PositionalConstraint::AnyByte; n])
                } else {
                    Constraint::new(vec![PositionalConstraint::AnyQuotation; n])
                };
                assert!(
                    u.try_push(anys),
                    "Adding to an empty union should always return true"
                );

                let c = Constraint::random(n, Some(bytes));
                assert!(
                    !u.try_push(c),
                    "Adding a constraint to a union that already contains [Any, Any, ...] should always return false"
                );
            }
        }
    }

    /// Test some specific cases of Union::is_subset(_)
    #[test]
    fn union_is_subset() {
        use PositionalConstraint as PC;

        let tests = vec![
            (vec![], vec![], false),
            (vec![vec![]], vec![], true),
            // C(0, 0) is a subset of { C(@, @) }
            (
                vec![vec![PC::AnyByte, PC::AnyByte]],
                vec![PC::PositionalByte(0), PC::PositionalByte(0)],
                true,
            ),
            // C(0, 0) is a subset of { C(a, b) }
            (
                vec![vec![PC::PositionalByte(0), PC::PositionalByte(1)]],
                vec![PC::PositionalByte(0), PC::PositionalByte(0)],
                true,
            ),
            // C(0, 0) is a subset of { C(a, a) }
            (
                vec![vec![PC::PositionalByte(0), PC::PositionalByte(0)]],
                vec![PC::PositionalByte(0), PC::PositionalByte(0)],
                true,
            ),
            // C(a, a) is a subset of { C(a, b) }
            (
                vec![vec![PC::PositionalByte(0), PC::PositionalByte(1)]],
                vec![PC::PositionalByte(0), PC::PositionalByte(0)],
                true,
            ),
            // C(a, 0, a) is a subset of { C(a, b, a) }
            (
                vec![vec![
                    PC::PositionalByte(0),
                    PC::PositionalByte(1),
                    PC::PositionalByte(0),
                ]],
                vec![
                    PC::PositionalByte(0),
                    PC::ExactByte(0),
                    PC::PositionalByte(0),
                ],
                true,
            ),
            // C(0, 0) is not a subset of { C(1, 1) }
            (
                vec![vec![PC::ExactByte(1), PC::ExactByte(1)]],
                vec![PC::ExactByte(0), PC::ExactByte(0)],
                false,
            ),
        ];

        for (u, constraint, expected) in tests {
            let union = Union::from(u);
            let c = Constraint::new(constraint);
            assert_eq!(union.is_subset(&c), expected);
        }
    }
}
