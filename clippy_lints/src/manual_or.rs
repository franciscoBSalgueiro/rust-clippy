use clippy_utils::diagnostics::span_lint_and_sugg;
use clippy_utils::peel_blocks;
use clippy_utils::visitors::for_each_expr;
use core::ops::ControlFlow;
use rustc_ast::ast::LitKind;
use rustc_errors::Applicability;
use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::declare_lint_pass;

declare_clippy_lint! {
    /// ### What it does
    /// Detects `if`-then-`else` that can be replaced with `||`.
    ///
    /// ### Why is this bad?
    /// `||` is simpler than `if`-then-`else`.
    ///
    /// ### Example
    /// ```no_run
    /// if a {
    ///     true
    /// } else {
    ///     b
    /// }
    /// ```
    /// Use instead:
    /// ```no_run
    /// a || b
    /// ```
    #[clippy::version = "1.80.0"]
    pub MANUAL_OR,
    complexity,
    "this `if`-then-`else` expression can be simplified with `||`"
}

declare_lint_pass!(ManualOr => [MANUAL_OR]);

fn extract_final_expression_snippet<'tcx>(cx: &LateContext<'tcx>, expr: &Expr<'tcx>) -> Option<String> {
    if let ExprKind::Block(block, _) = expr.kind {
        if let Some(final_expr) = block.expr {
            return cx.sess().source_map().span_to_snippet(final_expr.span).ok();
        }
    }
    cx.sess().source_map().span_to_snippet(expr.span).ok()
}

fn fetch_bool_expr(expr: &Expr<'_>) -> Option<bool> {
    if let ExprKind::Lit(lit_ptr) = peel_blocks(expr).kind {
        if let LitKind::Bool(value) = lit_ptr.node {
            return Some(value);
        }
    }
    None
}

fn contains_let(cond: &Expr<'_>) -> bool {
    for_each_expr(cond, |e| {
        if matches!(e.kind, ExprKind::Let(_)) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    })
    .is_some()
}

impl<'tcx> LateLintPass<'tcx> for ManualOr {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &Expr<'tcx>) {
        if let ExprKind::If(cond, then, Some(else_expr)) = expr.kind {
            if contains_let(cond) {
                return;
            }
            if matches!(else_expr.kind, ExprKind::If(..))
                || match else_expr.kind {
                    ExprKind::Block(block, _) => !block.stmts.is_empty(),
                    _ => false,
                }
                || fetch_bool_expr(else_expr).is_some()
            {
                return;
            }
            if let Some(true) = fetch_bool_expr(then) {
                let applicability = Applicability::MachineApplicable;
                let cond_snippet = cx
                    .sess()
                    .source_map()
                    .span_to_snippet(cond.span)
                    .unwrap_or_else(|_| "..".to_string());

                let else_snippet = extract_final_expression_snippet(cx, else_expr).unwrap_or_else(|| "..".to_string());
                let suggestion = format!("{cond_snippet} || {else_snippet}");
                span_lint_and_sugg(
                    cx,
                    MANUAL_OR,
                    expr.span,
                    "this `if`-then-`else` expression can be simplified with `||`",
                    "try",
                    suggestion,
                    applicability,
                );
            }
        }
    }
}
