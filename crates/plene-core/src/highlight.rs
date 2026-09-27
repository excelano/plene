//! Highlight classes, derived from the same token kinds and parent nodes that drive expansion.

use ra_ap_syntax::SyntaxKind::{self, *};
use ra_ap_syntax::{Edition, SyntaxNode, SyntaxToken};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HighlightClass {
    Keyword,
    Type,
    Function,
    Macro,
    Identifier,
    Lifetime,
    String,
    Number,
    Comment,
    Attribute,
    Operator,
    Punctuation,
    /// Whitespace and error tokens.
    Plain,
}

pub(crate) fn classify(token: &SyntaxToken, edition: Edition) -> HighlightClass {
    let kind = token.kind();
    if matches!(kind, WHITESPACE | NEWLINE) {
        return HighlightClass::Plain;
    }
    if token.parent_ancestors().any(|node| node.kind() == ATTR) {
        return HighlightClass::Attribute;
    }
    let parent = token.parent();
    let parent_kind = parent.as_ref().map(SyntaxNode::kind);
    match kind {
        ERROR => HighlightClass::Plain,
        COMMENT | INNER_DOC_COMMENT | OUTER_DOC_COMMENT | SHEBANG | FRONTMATTER => {
            HighlightClass::Comment
        }
        LIFETIME_IDENT => HighlightClass::Lifetime,
        STRING | BYTE_STRING | C_STRING | CHAR | BYTE => HighlightClass::String,
        INT_NUMBER | FLOAT_NUMBER => HighlightClass::Number,
        IDENT => parent.map_or(HighlightClass::Identifier, |parent| classify_ident(&parent)),
        BANG if matches!(parent_kind, Some(MACRO_CALL | MACRO_RULES)) => HighlightClass::Macro,
        L_ANGLE | R_ANGLE if parent_kind == Some(BIN_EXPR) => HighlightClass::Operator,
        UNDERSCORE => HighlightClass::Keyword,
        _ if kind.is_keyword(edition) || kind.is_contextual_keyword(edition) => {
            HighlightClass::Keyword
        }
        _ if is_punctuation(kind) => HighlightClass::Punctuation,
        _ if kind.is_punct() => HighlightClass::Operator,
        _ => HighlightClass::Plain,
    }
}

/// Classifies an identifier by the node that holds it: a declared name (`NAME`) by
/// what it declares, a reference (`NAME_REF`) by the path or expression it sits in.
fn classify_ident(parent: &SyntaxNode) -> HighlightClass {
    let grandparent = parent.parent();
    let grandparent_kind = grandparent.as_ref().map(SyntaxNode::kind);
    match (parent.kind(), grandparent_kind) {
        (NAME, Some(STRUCT | ENUM | UNION | TRAIT | TYPE_ALIAS | TYPE_PARAM)) => {
            HighlightClass::Type
        }
        (NAME, Some(FN)) => HighlightClass::Function,
        (NAME, Some(MACRO_RULES | MACRO_DEF)) => HighlightClass::Macro,
        (NAME_REF, Some(METHOD_CALL_EXPR)) => HighlightClass::Function,
        (NAME_REF, Some(PATH_SEGMENT)) => grandparent
            .map_or(HighlightClass::Identifier, |segment| {
                classify_path_segment(&segment)
            }),
        _ => HighlightClass::Identifier,
    }
}

/// Only the final segment of a path takes the path's class: in `std::io::Result`, the
/// final segment's `PATH` sits directly under the type, while `std` and `io` sit in
/// nested qualifier paths.
fn classify_path_segment(segment: &SyntaxNode) -> HighlightClass {
    match segment.parent().and_then(|path| path.parent()) {
        Some(owner) if owner.kind() == PATH_TYPE => HighlightClass::Type,
        Some(owner) if owner.kind() == MACRO_CALL => HighlightClass::Macro,
        Some(owner) if owner.kind() == PATH_EXPR && is_callee(&owner) => HighlightClass::Function,
        _ => HighlightClass::Identifier,
    }
}

fn is_callee(path_expr: &SyntaxNode) -> bool {
    path_expr.parent().is_some_and(|call| {
        call.kind() == CALL_EXPR && call.first_child().as_ref() == Some(path_expr)
    })
}

fn is_punctuation(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        L_PAREN
            | R_PAREN
            | L_BRACK
            | R_BRACK
            | L_CURLY
            | R_CURLY
            | L_ANGLE
            | R_ANGLE
            | COMMA
            | SEMICOLON
            | COLON
            | COLON2
            | DOT
            | POUND
            | DOLLAR
    )
}
