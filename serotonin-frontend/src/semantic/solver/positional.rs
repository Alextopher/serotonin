use std::rc::Rc;

use super::StackValue;

/// Types of constraints:
///
/// - AnyByte: @
/// - PositionalByte: \[a-z\] (all a's must be equal, all b's must be equal, etc)
/// - ExactByte: 0-255
/// - AnyQuotation: ?
/// - PositionalQuotation: \[A-Z\] (all A's must be equal, all B's must be equal, etc)
/// - ExactQuotation: "..."
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PositionalConstraint {
    AnyByte,
    PositionalByte(usize),
    ExactByte(u8),
    AnyQuotation,
    PositionalQuotation(usize),
    ExactQuotation(Rc<str>),
}

impl PositionalConstraint {
    /// Unwraps as ExactByte(u) or ExactQuotation(s)
    ///
    /// # Returns
    ///
    /// - Some(Byte(u)) if the constraint is ExactByte(u)
    /// - Some(Quotation(s)) if the constraint is ExactQuotation(s)
    /// - None otherwise
    pub fn exact_value(&self) -> Option<StackValue> {
        match self {
            PositionalConstraint::ExactByte(u) => Some(StackValue::Byte(*u)),
            PositionalConstraint::ExactQuotation(s) => Some(StackValue::Quotation(s.clone())),
            _ => None,
        }
    }

    /// Returns true if 2 positional constraints are of the same type
    pub fn variant_eq(&self, other: &Self) -> bool {
        use PositionalConstraint as PC;

        matches!(
            (self, other),
            (PC::AnyByte, PC::AnyByte)
                | (PC::AnyQuotation, PC::AnyQuotation)
                | (PC::PositionalByte(_), PC::PositionalByte(_))
                | (PC::PositionalQuotation(_), PC::PositionalQuotation(_))
                | (PC::ExactByte(_), PC::ExactByte(_))
                | (PC::ExactQuotation(_), PC::ExactQuotation(_))
        )
    }

    /// Returns if this PC is of type Byte
    pub fn is_byte(&self) -> bool {
        matches!(
            self,
            PositionalConstraint::AnyByte
                | PositionalConstraint::PositionalByte(_)
                | PositionalConstraint::ExactByte(_)
        )
    }

    /// Returns if this PC is of type Quotation
    pub fn is_quotation(&self) -> bool {
        matches!(
            self,
            PositionalConstraint::AnyQuotation
                | PositionalConstraint::PositionalQuotation(_)
                | PositionalConstraint::ExactQuotation(_)
        )
    }

    /// Returns if this PC is of type Any.
    ///
    /// Positional constraint 0 are considered as Any for the purposes of solving
    pub(super) fn is_any(&self) -> bool {
        use PositionalConstraint as PC;
        matches!(
            self,
            PC::AnyByte | PC::AnyQuotation | PC::PositionalByte(0) | PC::PositionalQuotation(0)
        )
    }
}
