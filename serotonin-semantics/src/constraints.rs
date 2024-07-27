use either::Either::{self, Left, Right};

use crate::ast::{Definition, Stack, StackArg};

use super::{
    errors::{SemanticError, SemanticWarning},
    solver::PositionalConstraint,
    SemanticAnalyzer,
};

impl SemanticAnalyzer<'_> {
    fn stack_arg_to_constraint(
        &mut self,
        arg: &StackArg,
    ) -> Result<PositionalConstraint, Either<SemanticError, SemanticWarning>> {
        match arg {
            StackArg::UnnamedByte(_) => Ok(SingleConstraint::UnnamedByte),
            StackArg::UnnamedQuotation(_) => Ok(SingleConstraint::UnnamedQuotation),
            StackArg::NamedByte(token) => token
                .text(self.rodeo)
                .chars()
                .next()
                .map(SingleConstraint::NamedByte)
                .ok_or(Left(SemanticError::ICENamedByteHasLengthNotOne(
                    token.clone(),
                ))),
            StackArg::NamedQuotation(token) => token
                .text(self.rodeo)
                .chars()
                .next()
                .map(SingleConstraint::NamedQuotation)
                .ok_or(Left(SemanticError::ICENamedQuotationHasLengthNotOne(
                    token.clone(),
                ))),
            StackArg::Integer(token) => token
                .data()
                .get_byte()
                .map(SingleConstraint::Byte)
                .ok_or(Left(SemanticError::ICEByteMissingValue(token.clone()))),
            StackArg::Quotation(q) => Err(Right(SemanticWarning::SpecificQuotationsNotSupported(
                q.span(),
            ))),
        }
    }

    /// Converts a definitions stack args to a list of Constraints
    pub fn stack_to_constraints(
        &mut self,
        stack: &Stack,
    ) -> Result<Vec<SingleConstraint>, Either<SemanticError, SemanticWarning>> {
        stack
            .args()
            .iter()
            .map(|arg| self.stack_arg_to_constraint(arg))
            .collect()
    }

    /// Analyze a name to ensure all of it's definitions are reachable
    ///
    /// test (0 0) == ...;
    /// test (a a) == ...; # covers
    ///
    /// flow:
    ///   add (named(a), named(a))
    ///   compare (0, 0), (named(a), named(a))
    ///     try a == 0
    ///     
    ///   done
    ///
    /// test (0 0) == ...;
    /// test (a a) == ...; # covers
    /// test (a b) == ...; # covers
    ///
    /// flow:
    ///   add (named(a), named(a))
    ///   compare (a, b), (a, a)
    ///   a = a
    ///   b = b
    ///   check a == a, b == a
    #[allow(unused_variables)]
    pub fn reachablity(&mut self, defs: Vec<(&Definition, Vec<SingleConstraint>)>) {
        todo!()
    }
}
