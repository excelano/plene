//! Syntactic roles: what a token does where it appears. The glossary keys expansions
//! by token and role.
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use std::fmt;
use std::str::FromStr;

use ra_ap_syntax::SyntaxKind::*;
use ra_ap_syntax::ast;
use ra_ap_syntax::{AstNode, Direction, NodeOrToken, SyntaxElement, SyntaxNode, SyntaxToken};

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
    FragmentSpecifier => "fragment_specifier",
    ZeroOrMore => "zero_or_more",
    OneOrMore => "one_or_more",
    ZeroOrOne => "zero_or_one",
    ElidedFromSelf => "elided_from_self",
    ElidedFromParam => "elided_from_param",
    ElidedNamed => "elided_named",
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
                | Role::ElidedFromParam
                | Role::ElidedNamed
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

/// A token's role, and for a role that takes a name, the name to fill in. A role that
/// adds to another one names it as its `fallback`, for the token to read as when the
/// glossary has no entry for the added one.
pub(crate) struct Classified {
    pub role: Role,
    pub name: Option<String>,
    pub fallback: Option<Role>,
}

pub(crate) fn classify(token: &SyntaxToken) -> Option<Classified> {
    let parent = token.parent()?;
    if parent.ancestors().any(|node| node.kind() == ERROR) {
        return None;
    }
    // Macro arguments and attribute arguments sit in token trees, which syntax alone
    // can't read as Rust. A `macro_rules!` definition is the one place their
    // notation is fixed.
    if parent.ancestors().any(|node| node.kind() == TOKEN_TREE) {
        return macro_rules_role(token, &parent).map(|role| Classified {
            role,
            name: None,
            fallback: None,
        });
    }
    if token.kind() == R_CURLY {
        return closing_brace(&parent);
    }
    if token.kind() == AMP
        && parent.kind() == REF_TYPE
        && let Some(elided) = elided_output(&parent)
    {
        return Some(elided);
    }
    let role = role(token, &parent)?;
    let name = matches!(role, Role::Lifetime | Role::Label).then(|| {
        token
            .text()
            .strip_prefix('\'')
            .unwrap_or(token.text())
            .to_string()
    });
    Some(Classified {
        role,
        name,
        fallback: None,
    })
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

/// The role and name of the `&` of a reference with no lifetime written, in a
/// function's return type, when syntax alone says where the compiler takes its lifetime
/// from: `&self`, or the one lifetime in the parameters. With none, or several and no
/// `&self`, the code would not compile, or syntax cannot see which, and the `&` keeps
/// its ordinary reading. A reference inside a function pointer or an `Fn(..)` bound has
/// a scope of its own and is left alone.
fn elided_output(ref_type: &SyntaxNode) -> Option<Classified> {
    if ref_type.children().any(|child| child.kind() == LIFETIME) {
        return None;
    }
    let mut ancestors = ref_type.ancestors().skip(1);
    let ret_type = loop {
        let node = ancestors.next()?;
        match node.kind() {
            FN_PTR_TYPE | PARENTHESIZED_ARG_LIST => return None,
            RET_TYPE => break node,
            _ => {}
        }
    };
    let function = ret_type.parent().filter(|node| node.kind() == FN)?;
    let params = function
        .children()
        .find(|child| child.kind() == PARAM_LIST)?;
    let self_param = params.children().find(|child| child.kind() == SELF_PARAM);
    let source = match self_param.as_ref().and_then(borrowed_self) {
        Some(lifetime) => match lifetime {
            Some(name) => Source::Named(name),
            None => Source::Param("self".to_string()),
        },
        None => {
            let mut positions = Vec::new();
            for param in params.children() {
                let owner = param_name(&param);
                collect_lifetimes(&param, owner.as_deref(), &mut positions);
            }
            let [only] = <[Source; 1]>::try_from(positions).ok()?;
            only
        }
    };
    let (role, name) = match source {
        Source::Named(name) => (Role::ElidedNamed, Some(name)),
        Source::Param(name) if name == "self" => (Role::ElidedFromSelf, None),
        Source::Param(name) => (Role::ElidedFromParam, Some(name)),
        Source::Unnamed => return None,
    };
    Some(Classified {
        role,
        name,
        fallback: Some(Role::RefType),
    })
}

/// Where an elided lifetime in a return type comes from.
enum Source {
    /// A lifetime written out, as `'a` or `'static`, by its name.
    Named(String),
    /// A lifetime left out of the parameter with this name.
    Param(String),
    /// A lifetime left out of a parameter that has no plain name to point to.
    Unnamed,
}

/// Whether `self_param` borrows `self`, as `&self`, `&mut self`, `&'a self` or
/// `self: &Self`. It answers with the lifetime's name when one is written, and `None`
/// inside when it is not. Any other `self` is not a borrow of it.
fn borrowed_self(self_param: &SyntaxNode) -> Option<Option<String>> {
    let reference = self_param
        .children_with_tokens()
        .any(|element| element.kind() == AMP);
    let typed = self_param.children().find(|child| child.kind() == REF_TYPE);
    let holder = match (reference, typed) {
        (true, _) => self_param.clone(),
        (false, Some(typed)) => typed,
        (false, None) => return None,
    };
    Some(written_lifetime(&holder))
}

/// The name of the lifetime written directly in `node`, without its quote. `'_` is not a
/// name: it is a lifetime left to be inferred.
fn written_lifetime(node: &SyntaxNode) -> Option<String> {
    let name = node
        .children()
        .find(|child| child.kind() == LIFETIME)?
        .text()
        .to_string();
    let name = name.strip_prefix('\'')?.to_string();
    (name != "_").then_some(name)
}

/// The plain name a parameter binds, as `x` in `mut x: &u8`.
fn param_name(param: &SyntaxNode) -> Option<String> {
    let pattern = param.children().find(|child| child.kind() == IDENT_PAT)?;
    Some(
        pattern
            .children()
            .find(|child| child.kind() == NAME)?
            .text()
            .to_string(),
    )
}

/// Pushes the lifetime positions in `node`, a parameter: each reference, and each
/// lifetime written elsewhere, as in `Foo<'a>` or `+ 'a`. A function pointer, an `Fn(..)`
/// bound and its return type have a scope of their own, so they are not looked into.
fn collect_lifetimes(node: &SyntaxNode, owner: Option<&str>, positions: &mut Vec<Source>) {
    for child in node.children() {
        match child.kind() {
            FN_PTR_TYPE | PARENTHESIZED_ARG_LIST | RET_TYPE => continue,
            REF_TYPE => positions.push(position(&child, owner)),
            LIFETIME if node.kind() != REF_TYPE => positions.push(position(&child, owner)),
            _ => {}
        }
        collect_lifetimes(&child, owner, positions);
    }
}

/// The position a reference or a lifetime argument holds: the lifetime if it is named,
/// else the parameter it is in.
fn position(holder: &SyntaxNode, owner: Option<&str>) -> Source {
    let written = if holder.kind() == LIFETIME {
        let name = holder.text().to_string();
        name.strip_prefix('\'')
            .filter(|name| *name != "_")
            .map(str::to_string)
    } else {
        written_lifetime(holder)
    };
    match (written, owner) {
        (Some(name), _) => Source::Named(name),
        (None, Some(owner)) => Source::Param(owner.to_string()),
        (None, None) => Source::Unnamed,
    }
}

/// The role of a token in a `macro_rules!` definition: the fragment specifier in a
/// matcher's `$name:spec`, and the `*`, `+` or `?` that ends a repetition `$(...)`.
fn macro_rules_role(token: &SyntaxToken, tree: &SyntaxNode) -> Option<Role> {
    let trees: Vec<SyntaxNode> = tree
        .ancestors()
        .take_while(|node| node.kind() == TOKEN_TREE)
        .collect();
    if trees.last()?.parent()?.kind() != MACRO_RULES {
        return None;
    }
    match token.kind() {
        IDENT if is_fragment_specifier(token) && in_matcher(&trees) => {
            Some(Role::FragmentSpecifier)
        }
        STAR | PLUS | QUESTION if is_repetition_operator(token) => Some(match token.kind() {
            STAR => Role::ZeroOrMore,
            PLUS => Role::OneOrMore,
            _ => Role::ZeroOrOne,
        }),
        _ => None,
    }
}

/// The elements on one side of `element`, nearest first, without whitespace and comments.
fn neighbours(
    element: impl Into<SyntaxElement>,
    direction: Direction,
) -> impl Iterator<Item = SyntaxElement> {
    let step = move |element: &SyntaxElement| match direction {
        Direction::Prev => element.prev_sibling_or_token(),
        Direction::Next => element.next_sibling_or_token(),
    };
    let first = step(&element.into());
    std::iter::successors(first, move |element| step(element))
        .filter(|neighbour| !matches!(neighbour.kind(), WHITESPACE | COMMENT))
}

/// Whether `ident` is the `spec` of a `$name:spec`.
fn is_fragment_specifier(ident: &SyntaxToken) -> bool {
    let kinds: Vec<_> = neighbours(ident.clone(), Direction::Prev)
        .take(3)
        .map(|neighbour| neighbour.kind())
        .collect();
    kinds == [COLON, IDENT, DOLLAR]
}

/// Whether the innermost of `trees` lies in a rule's matcher, the side before its `=>`.
/// `trees` runs from the innermost token tree out to the definition's body, and a rule
/// is the tree directly inside the body. Inside a macro's token trees `=>` is `=`
/// then `>`.
fn in_matcher(trees: &[SyntaxNode]) -> bool {
    let Some(rule) = trees.len().checked_sub(2).map(|rule| &trees[rule]) else {
        return false;
    };
    let after: Vec<_> = neighbours(rule.clone(), Direction::Next)
        .take(2)
        .map(|neighbour| neighbour.kind())
        .collect();
    after == [EQ, R_ANGLE]
}

/// Whether `operator` ends a repetition: it follows `$( ... )`, directly or after one
/// separator token. A separator is never `*`, `+` or `?`, so an operator that follows
/// another is not one.
fn is_repetition_operator(operator: &SyntaxToken) -> bool {
    let mut before = neighbours(operator.clone(), Direction::Prev);
    let group = match before.next() {
        Some(NodeOrToken::Node(group)) => group,
        Some(NodeOrToken::Token(separator))
            if !matches!(separator.kind(), STAR | PLUS | QUESTION) =>
        {
            match before.next() {
                Some(NodeOrToken::Node(group)) => group,
                _ => return false,
            }
        }
        _ => return false,
    };
    let opens_with_paren = group
        .first_token()
        .is_some_and(|first| first.kind() == L_PAREN);
    opens_with_paren && before.next().is_some_and(|dollar| dollar.kind() == DOLLAR)
}

/// The item whose body `block` is, and the role its closing brace takes: a function's
/// body, an `impl` or trait's items, a module's, a struct's fields or an enum's
/// variants. The braces of an `if`, a loop, a closure or an enum variant are not an
/// item's body.
pub(crate) fn item_body(block: &SyntaxNode) -> Option<(Role, SyntaxNode)> {
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
    Some((role, item))
}

/// The role and name of the closing brace of `block`, when `block` is the body of an
/// item long enough that its opening is out of sight: the item's name, or for an `impl`
/// the type it is for, with its trait.
fn closing_brace(block: &SyntaxNode) -> Option<Classified> {
    let (role, item) = item_body(block)?;
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
        fallback: None,
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
