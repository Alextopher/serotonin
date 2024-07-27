use super::{positional::PositionalConstraint, Reduction, StackValue};

/// Each definition may have an associated set of constraints, such that if the current stack state
/// is in the set of constraints, the definition is applicable.
///
/// Mathematically, constraints are a set of tuples of bytes & quotations, and a definition is applicable
/// to a state S if S ∈ C, where C is the set of constraints. These sets are impossible to represent with
/// something akin to a HashSet as the cardinality for bytes goes by 256^n and the number of quotations
/// (bf programs) are more or less infinite. Therefore, the set behavior is implemented as a List of
/// positional constraints, limiting the set of possible constraints considerably.
///
/// The two most common operations to preform on a sets of constraints are:
///
/// - Inclusion: Check if a given state S is an element of the set of constraints C
/// - Union: Join two sets of constraints into a single "set". This is implemented by the [`Union`] struct.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Constraint(Vec<PositionalConstraint>);

/// Constraints are implemented as a list of positional constraints.
impl FromIterator<PositionalConstraint> for Constraint {
    fn from_iter<I: IntoIterator<Item = PositionalConstraint>>(iter: I) -> Self {
        let v: Vec<_> = iter.into_iter().collect();

        // In debug mode we check that positional elements make a disjoint set
        for e in &v {
            let index = match e {
                PositionalConstraint::PositionalByte(i) => i,
                PositionalConstraint::PositionalQuotation(i) => i,
                _ => continue,
            };

            // Find the first element that has this index and verify that it's index is correct
            let position = v.iter().position(|c| c == e).unwrap();
            debug_assert_eq!(
                position, *index,
                "Positional elements must point to their first occurrence"
            );
        }

        Constraint(v)
    }
}

impl Constraint {
    /// Create a new set of constraints
    pub fn new(constraints: impl IntoIterator<Item = PositionalConstraint>) -> Self {
        FromIterator::from_iter(constraints)
    }

    /// Returns the length of the constraint
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns if the constraint is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over the positional constraints
    pub fn iter(&self) -> impl Iterator<Item = &PositionalConstraint> {
        self.0.iter()
    }

    /// Utility function that creates a random constraint set for property testing
    #[cfg(test)]
    pub fn random(n: usize, heterogeneous: Option<bool>) -> Self {
        use rand::Rng;
        use PositionalConstraint as PC;

        use crate::random_brainfuck;
        let mut rng = rand::thread_rng();

        fn positional_helper(constraints: &[PC], is_byte: bool, idx: usize) -> PC {
            let mut rng = rand::thread_rng();

            let items = constraints
                .iter()
                .filter(|e| e.is_byte() == is_byte)
                .collect::<Vec<_>>();

            if items.is_empty() || rng.r#gen() {
                if is_byte {
                    return PC::PositionalByte(idx);
                } else {
                    return PC::PositionalQuotation(idx);
                }
            }

            items[rng.gen_range(0..items.len())].clone()
        }

        let mut constraints = Vec::with_capacity(n);
        for i in 0..n {
            let cmd = match heterogeneous {
                // bytes only
                Some(true) => match rng.gen_range(0..3) {
                    0 => PC::AnyByte,
                    1 => PC::ExactByte(rng.gen::<u8>()),
                    2 => positional_helper(&constraints, true, i),
                    _ => unreachable!(),
                },
                // quotations only
                Some(false) => match rng.gen_range(0..3) {
                    0 => PC::AnyQuotation,
                    1 => PC::ExactQuotation(random_brainfuck(rng.gen_range(0..100)).into()),
                    2 => positional_helper(&constraints, false, i),
                    _ => unreachable!(),
                },
                // anything
                None => match rng.gen_range(0..6) {
                    0 => PC::AnyByte,
                    1 => PC::AnyQuotation,
                    2 => positional_helper(&constraints, true, i),
                    3 => positional_helper(&constraints, false, i),
                    4 => PC::ExactByte(rand::random::<u8>()),
                    5 => PC::ExactQuotation(random_brainfuck(50).into()),
                    _ => unreachable!(),
                },
            };

            constraints.push(cmd);
        }

        Self::new(constraints)
    }

    /// Checks if a stack state is an element of this set
    ///
    /// Stack states are sliced to match the length of this constraint
    /// If the constraint is longer than the stack state this method returns false
    pub fn contains(&self, state: &[StackValue]) -> bool {
        use PositionalConstraint as PC;

        if self.len() > state.len() {
            return false;
        }

        let state = &state[state.len() - self.len()..];
        debug_assert_eq!(state.len(), self.len());

        for (element, constraint) in state.iter().cloned().zip(self.iter()) {
            match constraint {
                PC::AnyByte => {
                    if !element.is_byte() {
                        return false;
                    }
                }
                PC::AnyQuotation => {
                    if !element.is_quotation() {
                        return false;
                    }
                }
                PC::PositionalByte(index) | PC::PositionalQuotation(index) => {
                    if state[*index] != element {
                        return false;
                    }
                }
                PC::ExactByte(_) | PC::ExactQuotation(_) => {
                    if constraint.exact_value() != Some(element) {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Reduces this constraint by assigning a specific value to the first positional constraint
    ///
    /// This is used in [`Union::is_subset`] to recursively simplify subset problems.
    ///
    /// [`Union::is_subset`]: super::union::Union::is_subset
    pub(super) fn reduce(&self, value: &Reduction) -> Constraint {
        use PositionalConstraint as PC;

        debug_assert!(!self.0.is_empty(), "Cannot reduce an empty constraint");
        debug_assert_eq!(
            self.0[0].is_byte(),
            value.is_byte(),
            "Should not reduce bytes with quotations or vice versa"
        );

        if value.is_byte() {
            let k = self
                .0
                .iter()
                .skip(1)
                .position(|c| matches!(c, PC::PositionalByte(0)));

            self.0
                .iter()
                .skip(1)
                .map(|c| match c {
                    PC::PositionalByte(0) => value
                        .byte()
                        .map_or(PC::PositionalByte(k.unwrap()), PC::ExactByte),
                    PC::PositionalByte(n) => PC::PositionalByte(n - 1),
                    PC::PositionalQuotation(n) => {
                        debug_assert!(
                            *n > 0,
                            " PositionalQuotations cannot point to PositionalBytes"
                        );
                        PC::PositionalQuotation(n - 1)
                    }
                    _ => c.clone(),
                })
                .collect()
        } else {
            let k = self
                .0
                .iter()
                .skip(1)
                .position(|c| matches!(c, PC::PositionalQuotation(0)));

            self.0
                .iter()
                .skip(1)
                .map(|c| match c {
                    PC::PositionalQuotation(0) => value
                        .quotation()
                        .map_or(PC::PositionalQuotation(k.unwrap()), |s| {
                            PC::ExactQuotation(s.clone())
                        }),
                    PC::PositionalQuotation(n) => PC::PositionalQuotation(n - 1),
                    PC::PositionalByte(n) => {
                        debug_assert!(
                            *n > 0,
                            " PositionalBytes cannot point to PositionalQuotations"
                        );
                        PC::PositionalByte(n - 1)
                    }
                    _ => c.clone(),
                })
                .collect()
        }
    }
}

#[cfg(test)]
mod test {
    use crate::semantic::solver::{positional::PositionalConstraint as PC, Constraint, Reduction};

    fn make_tests() -> Vec<(Vec<PC>, Reduction, Vec<PC>)> {
        vec![
            // C(@, @) x @ -> C(@)
            (
                vec![PC::AnyByte, PC::AnyByte],
                Reduction::AnyByte,
                vec![PC::AnyByte],
            ),
            // C(@, @) x 0 -> C(@)
            (
                vec![PC::AnyByte, PC::AnyByte],
                Reduction::ExactByte(0),
                vec![PC::AnyByte],
            ),
            // C(a, a) x @ -> C(a)
            (
                vec![PC::PositionalByte(0), PC::PositionalByte(0)],
                Reduction::AnyByte,
                vec![PC::PositionalByte(0)],
            ),
            // C(a, a) x 0 -> C(0)
            (
                vec![PC::PositionalByte(0), PC::PositionalByte(0)],
                Reduction::ExactByte(0),
                vec![PC::ExactByte(0)],
            ),
            // C(a, b) x @ -> C(b)
            (
                vec![PC::PositionalByte(0), PC::PositionalByte(1)],
                Reduction::AnyByte,
                vec![PC::PositionalByte(0)],
            ),
            // C(a, b) x 0 -> C(b)
            (
                vec![PC::PositionalByte(0), PC::PositionalByte(1)],
                Reduction::ExactByte(0),
                vec![PC::PositionalByte(0)],
            ),
            // C(a, a, a) X @ -> C(a, a)
            (
                vec![
                    PC::PositionalByte(0),
                    PC::PositionalByte(0),
                    PC::PositionalByte(0),
                ],
                Reduction::AnyByte,
                vec![PC::PositionalByte(0), PC::PositionalByte(0)],
            ),
            // C(a, b, a, a) x @ -> C(b, a, a)
            (
                vec![
                    PC::PositionalByte(0),
                    PC::PositionalByte(1),
                    PC::PositionalByte(0),
                    PC::PositionalByte(0),
                ],
                Reduction::AnyByte,
                vec![
                    PC::PositionalByte(0),
                    PC::PositionalByte(1),
                    PC::PositionalByte(1),
                ],
            ),
        ]
    }

    #[test]
    fn reduce_bytes() {
        for (input, value, expected) in &make_tests() {
            let c = Constraint::new(input.clone());
            let reduced = c.reduce(value);
            assert_eq!(
                reduced,
                Constraint::new(expected.clone()),
                "Failed for {:?} {:?} {:?}",
                input,
                value,
                expected
            );
        }
    }

    #[test]
    fn reduce_quotations() {
        // run the same tests but convert from bytes to quotations
        for (input, value, expected) in &make_tests() {
            let input = input.iter().map(|p| match p {
                PC::AnyByte => PC::AnyQuotation,
                PC::PositionalByte(n) => PC::PositionalQuotation(*n),
                PC::ExactByte(_) => PC::ExactQuotation("...".into()),
                _ => unreachable!(),
            });

            let value = match value {
                Reduction::AnyByte => Reduction::AnyQuotation,
                Reduction::ExactByte(_) => Reduction::ExactQuotation("...".into()),
                _ => unreachable!(),
            };

            let expected = expected.iter().map(|p| match p {
                PC::AnyByte => PC::AnyQuotation,
                PC::PositionalByte(n) => PC::PositionalQuotation(*n),
                PC::ExactByte(_) => PC::ExactQuotation("...".into()),
                _ => unreachable!(),
            });

            let c = Constraint::new(input.clone());
            let reduced = c.reduce(&value);
            assert_eq!(
                reduced,
                Constraint::new(expected.clone()),
                "Failed for {:?} {:?} {:?}",
                input,
                value,
                expected
            );
        }
    }

    #[test]
    fn reduce_heterogenous() {
        let tests = vec![
            (
                vec![PC::AnyByte, PC::AnyQuotation],
                Reduction::AnyByte,
                vec![PC::AnyQuotation],
            ),
            (
                vec![PC::ExactByte(0), PC::AnyQuotation],
                Reduction::AnyByte,
                vec![PC::AnyQuotation],
            ),
        ];

        for (input, value, expected) in tests {
            let c = Constraint::new(input.clone());
            let reduced = c.reduce(&value);
            assert_eq!(
                reduced,
                Constraint::new(expected.clone()),
                "Failed for {:?} {:?} {:?}",
                input,
                value,
                expected
            );
        }
    }
}
