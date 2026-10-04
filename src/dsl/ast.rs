use proc_macro2::Span;
use syn::LitStr;

use super::predicates::Predicate;

pub struct Guard {
    pub condition: Expr,
    pub message: LitStr,
    pub span: Span,
}

pub enum Expr {
    Predicate(Predicate, Span),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}
