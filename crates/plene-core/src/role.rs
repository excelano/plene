//! Syntactic roles: what a token does where it appears. The glossary keys expansions
//! by token and role.

use std::fmt;
use std::str::FromStr;

use ra_ap_syntax::SyntaxKind::*;
use ra_ap_syntax::{SyntaxNode, SyntaxToken};

/// Declares `Role` with each variant's stable string identifier, used in glossary files.
macro_rules! roles {
    ($($variant:ident => $id:literal,)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum Role {
            $($variant,)*
        }

        impl Role {
            pub const ALL: &[Role] = &[$(Role::$variant,)*];

            pub fn as_str(self) -> &'static str {
                match self {
                    $(Role::$variant => $id,)*
                }
            }
        }

        impl FromStr for Role {
            type Err = UnknownRole;

            fn from_str(id: &str) -> Result<Role, UnknownRole> {
                match id {
                    $($id => Ok(Role::$variant),)*
                    _ => Err(UnknownRole(id.to_string())),
                }
            }
        }
    };
}

roles! {
    Keyword => "keyword",
    RefExpr => "ref_expr",
    RefType => "ref_type",
    RefPattern => "ref_pattern",
    SelfParam => "self_param",
    RetType => "ret_type",
    Try => "try",
    TryChained => "try_chained",
    ClosureOpen => "closure_open",
    ClosureClose => "closure_close",
    Lifetime => "lifetime",
    LifetimeAnonymous => "lifetime_anonymous",
    Label => "label",
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownRole(pub String);

impl fmt::Display for UnknownRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown role `{}`", self.0)
    }
}

impl std::error::Error for UnknownRole {}

pub(crate) fn classify(token: &SyntaxToken) -> Option<Role> {
    let parent = token.parent()?;
    // Attribute arguments sit in token trees too, like macro arguments.
    if parent
        .ancestors()
        .any(|node| matches!(node.kind(), ERROR | TOKEN_TREE))
    {
        return None;
    }
    match (token.kind(), parent.kind()) {
        (FN_KW | PUB_KW | MUT_KW, _) => Some(Role::Keyword),
        (AMP, REF_EXPR) => Some(Role::RefExpr),
        (AMP, REF_TYPE) => Some(Role::RefType),
        (AMP, REF_PAT) => Some(Role::RefPattern),
        (AMP, SELF_PARAM) => Some(Role::SelfParam),
        (THIN_ARROW, RET_TYPE) => Some(Role::RetType),
        (QUESTION, TRY_EXPR) if is_receiver(&parent) => Some(Role::TryChained),
        (QUESTION, TRY_EXPR) => Some(Role::Try),
        (PIPE, PARAM_LIST) if is_closure_params(&parent) => {
            if parent.first_token().as_ref() == Some(token) {
                Some(Role::ClosureOpen)
            } else {
                Some(Role::ClosureClose)
            }
        }
        (LIFETIME_IDENT, _) => Some(classify_lifetime(token, &parent)),
        _ => None,
    }
}

/// Whether `expr` is the receiver of a method call, field access, index or `.await`,
/// so that an expansion after it would run into the `.` or `[` that follows.
fn is_receiver(expr: &SyntaxNode) -> bool {
    expr.parent().is_some_and(|outer| {
        matches!(
            outer.kind(),
            METHOD_CALL_EXPR | FIELD_EXPR | INDEX_EXPR | AWAIT_EXPR
        ) && outer.first_child().as_ref() == Some(expr)
    })
}

fn is_closure_params(param_list: &SyntaxNode) -> bool {
    param_list
        .parent()
        .is_some_and(|owner| owner.kind() == CLOSURE_EXPR)
}

/// Labels and lifetimes share a token kind; a label sits in a `LABEL` definition or
/// directly under `break`/`continue`.
fn classify_lifetime(token: &SyntaxToken, parent: &SyntaxNode) -> Role {
    let owner = parent.parent().map(|owner| owner.kind());
    if matches!(owner, Some(LABEL | BREAK_EXPR | CONTINUE_EXPR)) {
        Role::Label
    } else if token.text() == "'_" {
        Role::LifetimeAnonymous
    } else {
        Role::Lifetime
    }
}
