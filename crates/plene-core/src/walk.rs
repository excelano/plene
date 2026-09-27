//! The tokens `transcribe` renders, in source order, each with its byte offset in the
//! source. The arguments of standard macros that take expressions are parsed as Rust
//! and walked in place of their token trees, so their tokens take roles like any other.
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use ra_ap_syntax::SyntaxKind::*;
use ra_ap_syntax::{AstNode, Edition, NodeOrToken, SourceFile, SyntaxNode, SyntaxToken, ast};

/// Standard macros whose arguments are comma-separated expressions, by the final
/// segment of their path. `vec![x; n]` parses too, as an array repeat.
const EXPRESSION_MACROS: &[&str] = &[
    "addr_of",
    "addr_of_mut",
    "assert",
    "assert_eq",
    "assert_ne",
    "dbg",
    "debug_assert",
    "debug_assert_eq",
    "debug_assert_ne",
    "eprint",
    "eprintln",
    "format",
    "format_args",
    "panic",
    "print",
    "println",
    "todo",
    "unimplemented",
    "unreachable",
    "vec",
    "write",
    "writeln",
];

/// Arguments are parsed as the elements of this array. Its brackets are single-character
/// tokens that never join their neighbours, so the tokens between them are the
/// arguments' own bytes.
const PREFIX: &str = "const _: () = [";
const SUFFIX: &str = "];";

/// Maps offsets in a parsed tree to offsets in the source: an argument text parsed at
/// `from` in its wrapper starts at `to` in the source.
#[derive(Clone, Copy)]
struct Shift {
    from: usize,
    to: usize,
}

impl Shift {
    fn apply(self, offset: usize) -> usize {
        offset - self.from + self.to
    }
}

pub(crate) fn tokens(root: &SyntaxNode, edition: Edition) -> Vec<(SyntaxToken, usize)> {
    let mut tokens = Vec::new();
    walk(root, Shift { from: 0, to: 0 }, edition, &mut tokens);
    tokens
}

fn walk(node: &SyntaxNode, shift: Shift, edition: Edition, tokens: &mut Vec<(SyntaxToken, usize)>) {
    for element in node.children_with_tokens() {
        match element {
            NodeOrToken::Token(token) => {
                let offset = shift.apply(usize::from(token.text_range().start()));
                tokens.push((token, offset));
            }
            NodeOrToken::Node(child) => match parse_arguments(&child, edition) {
                Some(arguments) => walk_arguments(&arguments, shift, edition, tokens),
                None => walk(&child, shift, edition, tokens),
            },
        }
    }
}

/// A listed macro's token tree: its delimiters, and the array expression its
/// arguments parse as.
struct Arguments {
    open: SyntaxToken,
    close: SyntaxToken,
    array: SyntaxNode,
}

/// Walks a macro's token tree as its delimiters around the parsed arguments.
fn walk_arguments(
    arguments: &Arguments,
    shift: Shift,
    edition: Edition,
    tokens: &mut Vec<(SyntaxToken, usize)>,
) {
    let Arguments { open, close, array } = arguments;
    tokens.push((
        open.clone(),
        shift.apply(usize::from(open.text_range().start())),
    ));
    let inner = Shift {
        from: PREFIX.len(),
        to: shift.apply(usize::from(open.text_range().end())),
    };
    let elements: Vec<_> = array.children_with_tokens().collect();
    // The array's own brackets are the wrapper's, not the source's.
    for element in &elements[1..elements.len() - 1] {
        match element {
            NodeOrToken::Token(token) => {
                let offset = inner.apply(usize::from(token.text_range().start()));
                tokens.push((token.clone(), offset));
            }
            NodeOrToken::Node(child) => walk(child, inner, edition, tokens),
        }
    }
    tokens.push((
        close.clone(),
        shift.apply(usize::from(close.text_range().start())),
    ));
}

/// Parses the arguments in `tree` when it is the token tree of a listed macro, closed by
/// its matching delimiter, outside any error node, and the arguments parse without
/// error.
fn parse_arguments(tree: &SyntaxNode, edition: Edition) -> Option<Arguments> {
    if tree.kind() != TOKEN_TREE {
        return None;
    }
    let call = ast::MacroCall::cast(tree.parent()?)?;
    let name = call.path()?.segment()?.name_ref()?;
    if !EXPRESSION_MACROS.contains(&name.text())
        || call.syntax().ancestors().any(|node| node.kind() == ERROR)
    {
        return None;
    }
    let open = tree.first_child_or_token()?.into_token()?;
    let close = tree.last_child_or_token()?.into_token()?;
    if !matches!(
        (open.kind(), close.kind()),
        (L_PAREN, R_PAREN) | (L_BRACK, R_BRACK) | (L_CURLY, R_CURLY)
    ) {
        return None;
    }
    let text = tree.text().to_string();
    let arguments = &text[open.text().len()..text.len() - close.text().len()];
    let parse = SourceFile::parse(&format!("{PREFIX}{arguments}{SUFFIX}"), edition);
    if !parse.errors().is_empty() {
        return None;
    }
    let array = parse
        .syntax_node()
        .descendants()
        .find(|node| node.kind() == ARRAY_EXPR)?;
    Some(Arguments { open, close, array })
}
