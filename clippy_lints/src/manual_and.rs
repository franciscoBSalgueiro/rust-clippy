use clippy_utils::diagnostics::span_lint_and_then;
use clippy_utils::visitors::for_each_expr;
use clippy_utils::{get_parent_expr, peel_blocks};
use core::ops::ControlFlow;
use rustc_ast::ast::LitKind;
use rustc_ast::BinOpKind;
use rustc_errors::Applicability;
use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::declare_lint_pass;

declare_clippy_lint! {
    /// ### What it does
    /// Detects `if`-then-`else` that can be replaced with `&&`.
    ///
    /// ### Why is this bad?
    /// `&&` is simpler than `if`-then-`else`.
    ///
    /// ### Example
    /// ```no_run
    /// if a {
    ///     b
    /// } else {
    ///     false
    /// }
    /// ```
    /// Use instead:
    /// ```no_run
    /// a && b
    /// ```
    #[clippy::version = "1.80.0"]
    pub MANUAL_AND,
    complexity,
    "this `if`-then-`else` that can be replaced with `&&`."
}

declare_lint_pass!(ManualAnd => [MANUAL_AND]);

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

fn contains_or(cond: &Expr<'_>) -> bool {
    for_each_expr(cond, |e| {
        if let ExprKind::Binary(ref n, _, _) = e.kind {
            if n.node == BinOpKind::Or {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        } else {
            ControlFlow::Continue(())
        }
    })
    .is_some()
}

impl<'tcx> LateLintPass<'tcx> for ManualAnd {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &Expr<'tcx>) {
        if let Some(parent) = get_parent_expr(cx, expr) {
            if let ExprKind::If(_, _, _) = parent.kind {
                return;
            }
        }
        if let ExprKind::If(cond, then, Some(else_expr)) = expr.kind {
            if contains_let(cond) || contains_or(cond) || contains_or(then) {
                return;
            }
            if fetch_bool_expr(then).is_some()
                || match then.kind {
                    ExprKind::Block(block, _) => !block.stmts.is_empty(),
                    _ => false,
                }
            {
                return;
            }
            if let Some(false) = fetch_bool_expr(else_expr) {
                let applicability = Applicability::MachineApplicable;
                let cond_snippet = cx
                    .sess()
                    .source_map()
                    .span_to_snippet(cond.span)
                    .unwrap_or_else(|_| "..".to_string());

                let then_snippet = extract_final_expression_snippet(cx, then).unwrap_or_else(|| "..".to_string());
                span_lint_and_then(
                    cx,
                    MANUAL_AND,
                    expr.span,
                    "this `if`-then-`else` expression can be simplified with `&&`",
                    |diag| {
                        diag.span_suggestion(
                            expr.span,
                            "try",
                            format!("{cond_snippet} && {then_snippet}"),
                            applicability,
                        );
                    },
                );
            }
        }
    }
}
