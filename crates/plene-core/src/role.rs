//! Syntactic roles: what a token does where it appears. The glossary keys expansions
//! by token and role.
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use std::fmt;
use std::str::FromStr;

use ra_ap_syntax::SyntaxKind::*;
use ra_ap_syntax::ast;
use ra_ap_syntax::{AstNode, Direction, SyntaxNode, SyntaxToken};

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
    ImplBlock => "impl_block",
    ImplTraitType => "impl_trait_type",
    RefExpr => "ref_expr",
    RefType => "ref_type",
    RefPattern => "ref_pattern",
    SelfParam => "self_param",
    Binary => "binary",
    CompoundAssign => "compound_assign",
    Deref => "deref",
    PtrType => "ptr_type",
    Not => "not",
    NegativeImpl => "negative_impl",
    MatchArm => "match_arm",
    PatternOr => "pattern_or",
    PatternBinding => "pattern_binding",
    Wildcard => "wildcard",
    Discard => "discard",
    InferredType => "inferred_type",
    RestPattern => "rest_pattern",
    RestBinding => "rest_binding",
    StructUpdate => "struct_update",
    Range => "range",
    RangeFrom => "range_from",
    RangeFull => "range_full",
    RangeInclusive => "range_inclusive",
    TraitBound => "trait_bound",
    LifetimeBound => "lifetime_bound",
    BoundSeparator => "bound_separator",
    MaybeBound => "maybe_bound",
    RetType => "ret_type",
    Try => "try",
    TryChained => "try_chained",
    ClosureOpen => "closure_open",
    ClosureClose => "closure_close",
    Lifetime => "lifetime",
    LifetimeAnonymous => "lifetime_anonymous",
    Label => "label",
    FnEnd => "fn_end",
    ImplEnd => "impl_end",
    ModEnd => "mod_end",
    TraitEnd => "trait_end",
    StructEnd => "struct_end",
    EnumEnd => "enum_end",
}

impl Role {
    /// Whether the role's expansion carries a name through `{name}`: the lifetime or
    /// label itself, or the item a closing brace closes. One glossary entry then
    /// covers every name, so the entry is keyed by the role alone.
    pub fn takes_name(self) -> bool {
        matches!(
            self,
            Role::Lifetime
                | Role::Label
                | Role::FnEnd
                | Role::ImplEnd
                | Role::ModEnd
                | Role::TraitEnd
                | Role::StructEnd
                | Role::EnumEnd
        )
    }
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

/// The most lines of an item's block that go without a label on its closing brace.
const SHORT_BLOCK_LINES: usize = 19;

/// A token's role, and for a role that takes a name, the name to fill in.
pub(crate) struct Classified {
    pub role: Role,
    pub name: Option<String>,
}

pub(crate) fn classify(token: &SyntaxToken) -> Option<Classified> {
    let parent = token.parent()?;
    // Attribute arguments sit in token trees too, like macro arguments.
    if parent
        .ancestors()
        .any(|node| matches!(node.kind(), ERROR | TOKEN_TREE))
    {
        return None;
    }
    if token.kind() == R_CURLY {
        return closing_brace(&parent);
    }
    let role = role(token, &parent)?;
    let name = matches!(role, Role::Lifetime | Role::Label).then(|| {
        token
            .text()
            .strip_prefix('\'')
            .unwrap_or(token.text())
            .to_string()
    });
    Some(Classified { role, name })
}

fn role(token: &SyntaxToken, parent: &SyntaxNode) -> Option<Role> {
    match (token.kind(), parent.kind()) {
        (FN_KW | PUB_KW | MUT_KW | MOD_KW | DYN_KW | REF_KW | EXTERN_KW, _) => Some(Role::Keyword),
        (IMPL_KW, IMPL) => Some(Role::ImplBlock),
        (IMPL_KW, IMPL_TRAIT_TYPE) => Some(Role::ImplTraitType),
        (AMP, REF_EXPR) => Some(Role::RefExpr),
        (AMP, REF_TYPE) => Some(Role::RefType),
        (AMP, REF_PAT) => Some(Role::RefPattern),
        (AMP, SELF_PARAM) => Some(Role::SelfParam),
        // Multiplication `*` is kept, so it takes no role.
        (AMP | PIPE | CARET | SHL | SHR, BIN_EXPR) => Some(Role::Binary),
        (AMPEQ | PIPEEQ | CARETEQ | SHLEQ | SHREQ, BIN_EXPR) => Some(Role::CompoundAssign),
        (STAR, PREFIX_EXPR) => Some(Role::Deref),
        (STAR, PTR_TYPE) => Some(Role::PtrType),
        (BANG, PREFIX_EXPR) => Some(Role::Not),
        (BANG, IMPL) => Some(Role::NegativeImpl),
        (FAT_ARROW, MATCH_ARM) => Some(Role::MatchArm),
        (PIPE, OR_PAT) => Some(Role::PatternOr),
        (AT, IDENT_PAT) => Some(Role::PatternBinding),
        (UNDERSCORE, WILDCARD_PAT) if is_let_pattern(parent) => Some(Role::Discard),
        (UNDERSCORE, UNDERSCORE_EXPR) if is_assignment_target(parent) => Some(Role::Discard),
        (UNDERSCORE, UNDERSCORE_EXPR) => Some(Role::Wildcard),
        (UNDERSCORE, WILDCARD_PAT) => Some(Role::Wildcard),
        (UNDERSCORE, INFER_TYPE) => Some(Role::InferredType),
        (DOT2, REST_PAT) if is_bound(parent) => Some(Role::RestBinding),
        (DOT2, REST_PAT) => Some(Role::RestPattern),
        (DOT2, RECORD_EXPR_FIELD_LIST) if has_base(token) => Some(Role::StructUpdate),
        (DOT2, RECORD_EXPR_FIELD_LIST) => Some(Role::RestPattern),
        (DOT2, RANGE_EXPR) if is_destructuring_rest(parent) => Some(Role::RestPattern),
        (DOT2 | DOT2EQ, RANGE_EXPR | RANGE_PAT) => Some(range_role(token, parent)),
        (COLON, _) => bound_colon_role(token),
        (PLUS, TYPE_BOUND_LIST) => Some(Role::BoundSeparator),
        (QUESTION, TYPE_BOUND) => Some(Role::MaybeBound),
        (THIN_ARROW, RET_TYPE) => Some(Role::RetType),
        (QUESTION, TRY_EXPR) if is_receiver(parent) => Some(Role::TryChained),
        (QUESTION, TRY_EXPR) => Some(Role::Try),
        (PIPE, PARAM_LIST) if is_closure_params(parent) => {
            if parent.first_token().as_ref() == Some(token) {
                Some(Role::ClosureOpen)
            } else {
                Some(Role::ClosureClose)
            }
        }
        (LIFETIME_IDENT, _) => Some(classify_lifetime(token, parent)),
        _ => None,
    }
}

/// The role and name of the closing brace of `block`, when `block` is the body of an
/// item long enough that its opening is out of sight: the item's name, or for an `impl`
/// the type it is for, with its trait.
fn closing_brace(block: &SyntaxNode) -> Option<Classified> {
    let item = if block.kind() == STMT_LIST {
        block
            .parent()
            .filter(|body| body.kind() == BLOCK_EXPR)?
            .parent()?
    } else {
        block.parent()?
    };
    let role = match (item.kind(), block.kind()) {
        (FN, STMT_LIST) => Role::FnEnd,
        (IMPL, ASSOC_ITEM_LIST) => Role::ImplEnd,
        (MODULE, ITEM_LIST) => Role::ModEnd,
        (TRAIT, ASSOC_ITEM_LIST) => Role::TraitEnd,
        (STRUCT, RECORD_FIELD_LIST) => Role::StructEnd,
        (ENUM, VARIANT_LIST) => Role::EnumEnd,
        _ => return None,
    };
    if block.text().to_string().lines().count() <= SHORT_BLOCK_LINES {
        return None;
    }
    let name = if role == Role::ImplEnd {
        impl_name(&item)?
    } else {
        item.children()
            .find(|child| child.kind() == NAME)?
            .text()
            .to_string()
    };
    Some(Classified {
        role,
        name: Some(name),
    })
}

/// What an `impl` is for, read as `Type` or `Trait for Type` with the last segment of
/// each path and no generic arguments. An `impl` for anything but a named type has no
/// name.
fn impl_name(item: &SyntaxNode) -> Option<String> {
    let item = ast::Impl::cast(item.clone())?;
    let segment = |ty: ast::Type| match ty {
        ast::Type::PathType(path) => Some(path.path()?.segment()?.name_ref()?.text().to_string()),
        _ => None,
    };
    let target = segment(item.self_ty()?)?;
    match item.trait_() {
        Some(trait_) => Some(format!("{} for {target}", segment(trait_)?)),
        None => Some(target),
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

/// Whether the rest pattern is bound to a name, as in `rest @ ..`.
fn is_bound(rest: &SyntaxNode) -> bool {
    rest.parent()
        .is_some_and(|binding| binding.kind() == IDENT_PAT)
}

/// Whether a `..` in a record expression is followed by a base value, as in
/// `Point { x: 1, ..base }`; without one it is the rest of a destructuring assignment.
fn has_base(dots: &SyntaxToken) -> bool {
    dots.siblings_with_tokens(Direction::Next)
        .skip(1)
        .any(|element| element.into_node().is_some())
}

/// Whether a bare `..` is the rest of a destructuring assignment, as in
/// `(a, ..) = t;`. On the right of an `=` the same text is the full range, so the
/// tuple, array, call or record holding it has to be the whole left side.
fn is_destructuring_rest(range: &SyntaxNode) -> bool {
    let target = range
        .ancestors()
        .skip(1)
        .take_while(|node| {
            matches!(
                node.kind(),
                TUPLE_EXPR
                    | ARRAY_EXPR
                    | CALL_EXPR
                    | ARG_LIST
                    | RECORD_EXPR
                    | RECORD_EXPR_FIELD_LIST
                    | RECORD_EXPR_FIELD
            )
        })
        .last();
    range.children().next().is_none() && target.is_some_and(|node| is_assignment_target(&node))
}

/// Whether `pattern` is the whole pattern of a `let`, as in `let _ = f();`.
fn is_let_pattern(pattern: &SyntaxNode) -> bool {
    pattern
        .parent()
        .is_some_and(|owner| owner.kind() == LET_STMT)
}

/// A range reads by which ends it has, found from the positions of its children
/// around the operator: `a..b` and `..b` go up to, `a..` goes onward, `..` alone
/// is everything, and `..=` always goes through its end.
fn range_role(token: &SyntaxToken, range: &SyntaxNode) -> Role {
    let at = token.text_range();
    let has_start = range
        .children()
        .any(|end| end.text_range().end() <= at.start());
    let has_end = range
        .children()
        .any(|end| end.text_range().start() >= at.end());
    match (token.kind(), has_start, has_end) {
        (DOT2EQ, _, _) => Role::RangeInclusive,
        (_, true, false) => Role::RangeFrom,
        (_, false, false) => Role::RangeFull,
        _ => Role::Range,
    }
}

/// A colon introduces bounds when a bound list follows it, as in `T: Clone`,
/// `'a: 'b`, `trait A: B` and `where T: 'a`; every other colon is left alone. The
/// first bound decides the reading: a lifetime is outlived, a trait implemented.
fn bound_colon_role(colon: &SyntaxToken) -> Option<Role> {
    let next = colon
        .siblings_with_tokens(Direction::Next)
        .skip(1)
        .find(|element| !matches!(element.kind(), WHITESPACE | COMMENT))?;
    let bounds = next
        .into_node()
        .filter(|node| node.kind() == TYPE_BOUND_LIST)?;
    let first_is_lifetime = bounds
        .children()
        .next()
        .is_some_and(|bound| bound.children().any(|child| child.kind() == LIFETIME));
    Some(if first_is_lifetime {
        Role::LifetimeBound
    } else {
        Role::TraitBound
    })
}

/// Whether `expr` is the whole left side of an assignment, as in `_ = f();`.
fn is_assignment_target(expr: &SyntaxNode) -> bool {
    expr.parent().is_some_and(|assignment| {
        assignment.kind() == BIN_EXPR
            && assignment.first_child().as_ref() == Some(expr)
            && assignment
                .children_with_tokens()
                .any(|element| element.kind() == EQ)
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
