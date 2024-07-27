pub mod definition;
pub mod positional;
pub mod union;

pub use definition::Constraint;
pub use union::Union;

use std::rc::Rc;

use positional::PositionalConstraint;

/// Serotonin constraints can only be applied when the stack arguments are constant byte values or quotations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StackValue {
    Byte(u8),
    Quotation(Rc<str>),
}

impl StackValue {
    /// Returns the byte value if the stack value is a byte
    pub fn byte(&self) -> Option<u8> {
        match self {
            StackValue::Byte(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the quotation value if the stack value is a quotation
    pub fn quotation(&self) -> Option<&Rc<str>> {
        match self {
            StackValue::Quotation(s) => Some(s),
            _ => None,
        }
    }

    /// Returns if the stack value is a byte
    pub fn is_byte(&self) -> bool {
        matches!(self, StackValue::Byte(_))
    }

    /// Returns if the stack value is a quotation
    pub fn is_quotation(&self) -> bool {
        matches!(self, StackValue::Quotation(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Reduction {
    ExactByte(u8),
    AnyByte,
    ExactQuotation(Rc<str>),
    AnyQuotation,
}

impl From<StackValue> for Reduction {
    fn from(value: StackValue) -> Self {
        match value {
            StackValue::Byte(b) => Reduction::ExactByte(b),
            StackValue::Quotation(s) => Reduction::ExactQuotation(s),
        }
    }
}

impl Reduction {
    fn new_any(is_byte: bool) -> Self {
        if is_byte {
            Reduction::AnyByte
        } else {
            Reduction::AnyQuotation
        }
    }

    fn is_byte(&self) -> bool {
        matches!(self, Reduction::ExactByte(_) | Reduction::AnyByte)
    }

    fn is_quotation(&self) -> bool {
        matches!(self, Reduction::ExactQuotation(_) | Reduction::AnyQuotation)
    }

    fn byte(&self) -> Option<u8> {
        match self {
            Reduction::ExactByte(b) => Some(*b),
            _ => None,
        }
    }

    fn quotation(&self) -> Option<&Rc<str>> {
        match self {
            Reduction::ExactQuotation(s) => Some(s),
            _ => None,
        }
    }
}

impl TryFrom<PositionalConstraint> for Reduction {
    type Error = ();

    fn try_from(value: PositionalConstraint) -> Result<Self, Self::Error> {
        use PositionalConstraint as PC;

        match value {
            PC::AnyByte => Ok(Reduction::AnyByte),
            PC::ExactByte(b) => Ok(Reduction::ExactByte(b)),
            PC::AnyQuotation => Ok(Reduction::AnyQuotation),
            PC::ExactQuotation(s) => Ok(Reduction::ExactQuotation(s)),
            PC::PositionalByte(0) => Ok(Reduction::AnyByte),
            PC::PositionalQuotation(0) => Ok(Reduction::AnyQuotation),
            _ => Err(()),
        }
    }
}
