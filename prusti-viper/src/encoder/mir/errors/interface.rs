use crate::encoder::errors::{ErrorCtxt, SpannedEncodingResult};
use prusti_interface::data::ProcedureDefId;
use rustc_span::MultiSpan;
use vir_crate::high::{
    self as vir_high, ast::statement::visitors::StatementFolder, visitors::ExpressionFolder,
};

pub(crate) trait ErrorInterface {
    fn register_error<T: Into<MultiSpan>>(
        &mut self,
        span: T,
        error_ctxt: ErrorCtxt,
        def_id: ProcedureDefId,
    ) -> vir_high::Position;
    /// Takes a position of an already registered error and registers a new
    /// error with the same span, but different error context.
    fn change_error_context(
        &mut self,
        position: vir_high::Position,
        error_ctxt: ErrorCtxt,
    ) -> vir_high::Position;
    fn set_surrounding_error_context(
        &mut self,
        position: vir_high::Position,
        error_ctxt: ErrorCtxt,
    ) -> vir_high::Position;
    fn set_surrounding_error_context_for_expression(
        &mut self,
        expression: vir_high::Expression,
        default_position: vir_high::Position,
        error_ctxt: ErrorCtxt,
    ) -> vir_high::Expression;
    fn set_statement_error_ctxt<T: Into<MultiSpan>>(
        &mut self,
        statement: vir_high::Statement,
        span: T,
        error_ctxt: ErrorCtxt,
        def_id: ProcedureDefId,
    ) -> SpannedEncodingResult<vir_high::Statement>;
    fn set_statement_error_ctxt_from_position(
        &mut self,
        statement: vir_high::Statement,
        span_position: vir_high::Position,
        error_ctxt: ErrorCtxt,
    ) -> SpannedEncodingResult<vir_high::Statement>;
}

impl<'v, 'tcx: 'v> ErrorInterface for super::super::super::Encoder<'v, 'tcx> {
    fn register_error<T: Into<MultiSpan>>(
        &mut self,
        span: T,
        error_ctxt: ErrorCtxt,
        def_id: ProcedureDefId,
    ) -> vir_high::Position {
        self.error_manager()
            .register_error(span, error_ctxt, def_id)
            .into()
    }
    fn change_error_context(
        &mut self,
        position: vir_high::Position,
        error_ctxt: ErrorCtxt,
    ) -> vir_high::Position {
        let new_position = self.error_manager().duplicate_position(position.into());
        self.error_manager().set_error(new_position, error_ctxt);
        new_position.into()
    }
    fn set_surrounding_error_context(
        &mut self,
        position: vir_high::Position,
        error_ctxt: ErrorCtxt,
    ) -> vir_high::Position {
        self.error_manager()
            .set_surrounding_error_context(position.into(), error_ctxt)
            .into()
    }
    /// Replaces all positions with:
    /// 1. `default_position` if `position.is_default()`.
    /// 2. With surrounding error context otherwise.
    fn set_surrounding_error_context_for_expression(
        &mut self,
        expression: vir_high::Expression,
        default_position: vir_high::Position,
        error_ctxt: ErrorCtxt,
    ) -> vir_high::Expression {
        struct Visitor<'p, 'v: 'p, 'tcx: 'v> {
            encoder: &'p mut super::super::super::Encoder<'v, 'tcx>,
            default_position: vir_high::Position,
            error_ctxt: ErrorCtxt,
        }
        impl<'p, 'v: 'p, 'tcx: 'v> ExpressionFolder for Visitor<'p, 'v, 'tcx> {
            fn fold_position(&mut self, position: vir_high::Position) -> vir_high::Position {
                if position.is_default() {
                    self.default_position
                } else {
                    self.encoder
                        .set_surrounding_error_context(position, self.error_ctxt.clone())
                }
            }
        }
        let mut visitor = Visitor {
            encoder: self,
            default_position,
            error_ctxt,
        };
        visitor.fold_expression(expression)
    }
    fn set_statement_error_ctxt<T: Into<MultiSpan>>(
        &mut self,
        statement: vir_high::Statement,
        span: T,
        error_ctxt: ErrorCtxt,
        def_id: ProcedureDefId,
    ) -> SpannedEncodingResult<vir_high::Statement> {
        struct Visitor<'p, 'v: 'p, 'tcx: 'v> {
            encoder: &'p mut super::super::super::Encoder<'v, 'tcx>,
            default_position: vir_high::Position,
            error_ctxt: ErrorCtxt,
        }
        impl<'p, 'v: 'p, 'tcx: 'v> StatementFolder for Visitor<'p, 'v, 'tcx> {
            fn fold_position(&mut self, position: vir_high::Position) -> vir_high::Position {
                self.encoder
                    .set_surrounding_error_context(position, self.error_ctxt.clone())
            }
            fn fold_expression(
                &mut self,
                expression: vir_high::Expression,
            ) -> vir_high::Expression {
                self.encoder.set_surrounding_error_context_for_expression(
                    expression,
                    self.default_position,
                    self.error_ctxt.clone(),
                )
            }
        }
        let default_position = self.register_error(span, error_ctxt.clone(), def_id);
        let mut visitor = Visitor {
            encoder: self,
            default_position,
            error_ctxt,
        };
        Ok(visitor.fold_statement(statement))
    }
    fn set_statement_error_ctxt_from_position(
        &mut self,
        statement: vir_high::Statement,
        span_position: vir_high::Position,
        error_ctxt: ErrorCtxt,
    ) -> SpannedEncodingResult<vir_high::Statement> {
        let span = self
            .error_manager()
            .position_manager()
            .get_span(span_position.into())
            .unwrap()
            .clone();
        let def_id = self
            .error_manager()
            .position_manager()
            .get_def_id(span_position.into())
            .unwrap();
        self.set_statement_error_ctxt(statement, span, error_ctxt, def_id)
    }
}
