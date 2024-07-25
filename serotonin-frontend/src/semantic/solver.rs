//! Resolves byte constraints on definitions
//!
//! Constraints define a set of byte vectors that a definition can be called with
//!
//! C(a, b) is the set of all (a, b)
//! C(a, 0) is the set of all (a, b) where b is 0
//! C(a, a) is the set of all (a, b) where a == b
//!
//! So when I go to test a stack state like (0, 0) the verifier will check if (0, 0) is an element of C(a, b).
//! If it is, then the stack state is valid. Nothing is special about 0, it's just any byte 0 <= b < 256.
//!
//! The examples I've used here are for pairs, but the same logic needs to work for any number of arguments.

/// Types of constraints:
/// - Any: @
/// - Positional Equality: \[a-z\] (all a's must be equal, all b's must be equal, etc)
/// - Exact Value: 0-255
///
/// Positional equality constraints can only point to other position equalities. For example,
/// (Any, PE(0)) is invalid as PE(0) must point to another PE(0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PositionalConstraint {
    AnyByte,
    Positional(usize),
    ExactValue(u8),
}

impl PositionalConstraint {
    /// Unwraps as ExactValue if the constraint is an exact value.
    ///
    /// # Returns
    ///
    /// - `Some(u)`: If the constraint is an ExactValue(u).
    /// - `None`: If the constraint is not an ExactValue(_).
    pub fn exact_value(&self) -> Option<u8> {
        match self {
            PositionalConstraint::ExactValue(v) => Some(*v),
            _ => None,
        }
    }
}

/// Each definition has a set of constraints that define the applicable stack states
/// this struct represents one such set. The [`Union`] struct represents set union
/// of these Constraint structs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Constraint(Vec<PositionalConstraint>);

impl From<Vec<PositionalConstraint>> for Constraint {
    fn from(constraints: Vec<PositionalConstraint>) -> Self {
        use PositionalConstraint as PC;

        // Very the positional constraints are valid
        for c in constraints.iter() {
            if let PC::Positional(n) = c {
                debug_assert_eq!(
                    constraints[*n],
                    PC::Positional(*n),
                    "Positional constraints must point to identical positional constraints"
                );
            }
        }

        Constraint(constraints)
    }
}

impl Constraint {
    /// Create a new set of constraints
    pub fn new(constraints: Vec<PositionalConstraint>) -> Self {
        constraints.into()
    }

    /// Creates a random set of constraints for testing fulfilling the requirement
    /// that PositionalEq constraints are disjoint.
    #[cfg(test)]
    pub fn random(n: usize) -> Self {
        use PositionalConstraint as PC;

        let mut constraints = vec![PC::AnyByte; n];

        for i in 0..n {
            constraints[i] = match rand::random::<u8>() % 3 {
                0 => PC::AnyByte,
                1 => {
                    // 50-50 to point to another positional equality or create a new one
                    if i == 0 || rand::random::<bool>() {
                        PC::Positional(i)
                    } else {
                        match constraints
                            .iter()
                            .cloned()
                            .filter(|c| matches!(c, PC::Positional(_)))
                            .cycle()
                            .nth(rand::random::<usize>() % n)
                        {
                            Some(r) => r,
                            None => PC::Positional(i),
                        }
                    }
                }
                2 => PC::ExactValue(rand::random::<u8>()),
                _ => unreachable!(),
            };
        }

        Constraint(constraints)
    }

    /// Check if an element exists in the set of constraints
    pub fn contains(&self, element: &[u8]) -> bool {
        for (e, constraint) in element.iter().zip(self.0.iter()) {
            match constraint {
                PositionalConstraint::AnyByte => {},
                PositionalConstraint::Positional(i) => {
                    if *e != element[*i] {
                        return false;
                    }
                }
                PositionalConstraint::ExactValue(v) => {
                    if e != v {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Reduces the constraint by one element, by optionally assigning the first element to a specific value.
    pub fn reduce(&self, value: Option<u8>) -> Constraint {
        use PositionalConstraint as PC;

        if self.0.is_empty() {
            return self.clone();
        }

        let k = self.0.iter().skip(1).position(|c| matches!(c, PC::Positional(0)));
        let constraints = 
                self.0.iter().skip(1).map(|c| match c {
                    PC::Positional(0) => value.map_or_else(|| PC::Positional(k.unwrap()), PC::ExactValue),
                    PC::Positional(n) => PC::Positional(n - 1),
                    _ => *c,
                }).collect::<Vec<_>>();

        Constraint(constraints)
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
}

/// Represents the set union of [`Constraint`] structs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Union {
    constraints: Vec<Constraint>,
}

#[cfg(test)]
impl<T> From<Vec<T>> for Union
where
    T: Into<Constraint>,
{
    fn from(constraints: Vec<T>) -> Self {
        Union {
            constraints: constraints.into_iter().map(|c| c.into()).collect(),
        }
    }
}

impl Default for Union {
    fn default() -> Self {
        Union::new()
    }
}

impl Union {
    /// Create a new empty union of constraints
    pub fn new() -> Self {
        Union {
            constraints: Vec::new(),
        }
    }

    /// Adds a new constraint to the union if it is not already a subset of the existing constraints.
    pub fn add(&mut self, constraint: Constraint) -> bool {
        if self.is_subset(&constraint) {
            return false;
        }
        self.constraints.push(constraint);
        true
    }

    /// Reduces this union of constraints by giving a specific value to the first positional constraint
    fn reduce(&self, v: Option<u8>) -> Union {
        Union {
            constraints: self.constraints.iter().map(|c| c.reduce(v)).collect(),
        }
    }

    /// Check if a new definition constraint is already a subset of the existing constraints.
    ///
    /// This should be reported as a warning as it means the new definition is unreachable.
    fn is_subset(&self, constraint: &Constraint) -> bool {
        use PositionalConstraint as PC;

        if !self.constraints.is_empty() {
            debug_assert_eq!(
                self.constraints[0].0.len(),
                constraint.0.len(),
                "Constraints must have the same length"
            );
        }

        // Base case (length 0 constraints)
        if constraint.0.is_empty() {
            return !self.constraints.is_empty();
        }

        let big_g = &self.constraints;
        let big_u = &constraint;

        let u = big_u.0[0];
        let u_is_any = matches!(u, PC::AnyByte | PC::Positional(0));

        debug_assert!(matches!(
            u,
            PC::AnyByte | PC::Positional(0) | PC::ExactValue(_)
        ));

        let gs = big_g.iter().map(|c| c.0[0]).collect::<Vec<_>>();
        let gs_are_any = gs.iter().any(|c| {
            matches!(
                c,
                PositionalConstraint::AnyByte | PositionalConstraint::Positional(0)
            )
        });

        let (new_u, new_g) = match (u_is_any, gs_are_any) {
            // gs == u == Any | continue
            (true, true) => {
                let value = u.exact_value();
                let new_u = big_u.reduce(value);
                let new_g = self.reduce(value);
                (new_u, new_g)
            }
            // Any == u > gs  | there is at least 1 element in U that is not in G
            (true, false) => {
                return false;
            }
            // u < gs == Any  | continue
            (false, true) => {
                let value = u.exact_value();
                let new_u = big_u.reduce(value);
                let new_g = self.reduce(value);
                (new_u, new_g)
            }
            // Neither are Any, so we need to check if u is in gs. If it is, continue, if it's not, return false
            (false, false) => {
                // check if any g equals u
                let u = u.exact_value().unwrap();
                if gs.iter().map(|g| g.exact_value().unwrap()).any(|g| g == u) {
                    let new_u = big_u.reduce(Some(u));
                    let new_g = self.reduce(Some(u));
                    (new_u, new_g)
                } else {
                    return false;
                }
            }
        };

        new_g.is_subset(&new_u)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs a few constraint::reduce tests as a sanity check
    #[test]
    fn constraint_reduce_specific() {
        use PositionalConstraint as PC;

        let tests = vec![
            // C(@, @) x @ -> C(@)
            (vec![PC::AnyByte, PC::AnyByte], None, vec![PC::AnyByte]),
            // C(@, @) x 0 -> C(@)
            (vec![PC::AnyByte, PC::AnyByte], Some(0), vec![PC::AnyByte]),
            // C(a, a) x @ -> C(a)
            (
                vec![PC::Positional(0), PC::Positional(0)],
                None,
                vec![PC::Positional(0)],
            ),
            // C(a, a) x 0 -> C(0)
            (
                vec![PC::Positional(0), PC::Positional(0)],
                Some(0),
                vec![PC::ExactValue(0)],
            ),
            // C(a, b) x @ -> C(b)
            (
                vec![PC::Positional(0), PC::Positional(1)],
                None,
                vec![PC::Positional(0)],
            ),
            // C(a, b) x 0 -> C(b)
            (
                vec![PC::Positional(0), PC::Positional(1)],
                Some(0),
                vec![PC::Positional(0)],
            ),
            // C(a, a, a) X @ -> C(a, a)
            (
                vec![PC::Positional(0), PC::Positional(0), PC::Positional(0)],
                None,
                vec![PC::Positional(0), PC::Positional(0)],
            ),
            // C(a, b, a, a) x @ -> C(b, a, a)
            (
                vec![
                    PC::Positional(0),
                    PC::Positional(1),
                    PC::Positional(0),
                    PC::Positional(0),
                ],
                None,
                vec![
                    PC::Positional(0),
                    PC::Positional(1),
                    PC::Positional(1),
                ],
            ),
        ];

        for (input, value, expected) in tests {
            let c = Constraint::new(input.clone());
            let reduced = c.reduce(value);
            assert_eq!(reduced, Constraint::new(expected.clone()), "Failed for {:?} {:?} {:?}", input, value, expected);
        }
    }

    /// Property test: Union::add(_) always returns true if the union is empty
    #[test]
    fn union_add_empty() {
        for n in 1..10 {
            for _ in 0..100 {
                let mut u = Union::new();
                let c = Constraint::random(n);
                assert!(
                    u.add(c),
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
                let mut u = Union::new();
                let anys = Constraint::new(vec![PositionalConstraint::AnyByte; n]);
                assert!(
                    u.add(anys),
                    "Adding to an empty union should always return true"
                );
                let c = Constraint::random(n);
                assert!(
                    !u.add(c),
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
                vec![PC::Positional(0), PC::Positional(0)],
                true,
            ),
            // C(0, 0) is a subset of { C(a, b) }
            (
                vec![vec![PC::Positional(0), PC::Positional(1)]],
                vec![PC::Positional(0), PC::Positional(0)],
                true,
            ),
            // C(0, 0) is a subset of { C(a, a) }
            (
                vec![vec![PC::Positional(0), PC::Positional(0)]],
                vec![PC::Positional(0), PC::Positional(0)],
                true,
            ),
            // C(a, a) is a subset of { C(a, b) }
            (
                vec![vec![PC::Positional(0), PC::Positional(1)]],
                vec![PC::Positional(0), PC::Positional(0)],
                true,
            ),
            // C(a, 0, a) is a subset of { C(a, b, a) }
            (
                vec![vec![
                    PC::Positional(0),
                    PC::Positional(1),
                    PC::Positional(0),
                ]],
                vec![PC::Positional(0), PC::ExactValue(0), PC::Positional(0)],
                true,
            ),
            // C(0, 0) is not a subset of { C(1, 1) }
            (
                vec![vec![PC::Positional(1), PC::Positional(1)]],
                vec![PC::Positional(0), PC::Positional(0)],
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
