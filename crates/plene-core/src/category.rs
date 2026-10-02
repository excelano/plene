//! Categories of expansion: the groups a reader switches on and off together.
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use std::fmt;
use std::str::FromStr;

use crate::role::Role;

/// Declares `Category` with each variant's identifier, used on the command line, and
/// its name for a menu.
macro_rules! categories {
    ($($variant:ident => ($id:literal, $label:literal),)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum Category {
            $($variant,)*
        }

        impl Category {
            pub const ALL: &[Category] = &[$(Category::$variant,)*];

            pub fn as_str(self) -> &'static str {
                match self {
                    $(Category::$variant => $id,)*
                }
            }

            pub fn label(self) -> &'static str {
                match self {
                    $(Category::$variant => $label,)*
                }
            }
        }

        impl FromStr for Category {
            type Err = UnknownCategory;

            fn from_str(id: &str) -> Result<Category, UnknownCategory> {
                match id {
                    $($id => Ok(Category::$variant),)*
                    _ => Err(UnknownCategory(id.to_string())),
                }
            }
        }
    };
}

categories! {
    Keywords => ("keywords", "Keywords"),
    Visibility => ("visibility", "Visibility"),
    Mutability => ("mutability", "Mutability"),
    References => ("references", "References and pointers"),
    Operators => ("operators", "Operators"),
    Ranges => ("ranges", "Ranges"),
    Patterns => ("patterns", "Patterns and wildcards"),
    Lifetimes => ("lifetimes", "Lifetimes and labels"),
    Bounds => ("bounds", "Trait bounds"),
    Flow => ("flow", "Match arms, returns, closures and ?"),
    Ends => ("ends", "Labels on the closing braces of long items"),
    Macros => ("macros", "macro_rules! fragments and repetitions"),
    Elision => ("elision", "Elided lifetimes in return types"),
}

impl Category {
    /// Whether the category starts off: its expansions add what the source leaves out,
    /// so a reader asks for them. The others expand until the reader keeps them as
    /// written.
    pub fn opt_in(self) -> bool {
        matches!(self, Category::Elision)
    }

    /// `id` as a category that starts off, for the lists of those a reader turns on.
    pub fn from_opt_in_str(id: &str) -> Result<Category, String> {
        let category = id.parse::<Category>().map_err(|error| error.to_string())?;
        if category.opt_in() {
            Ok(category)
        } else {
            let starting_off: Vec<&str> = Category::ALL
                .iter()
                .filter(|category| category.opt_in())
                .map(|category| category.as_str())
                .collect();
            Err(format!(
                "`{id}` already expands by default; the categories that start off are: {}",
                starting_off.join(", ")
            ))
        }
    }

    /// The categories to leave as written: those of `keep`, and the ones that start off
    /// unless `expand` names them. A category that starts off is not in `keep`'s hands:
    /// keeping others does not turn it on.
    pub fn kept(keep: &[Category], expand: &[Category]) -> Vec<Category> {
        Category::ALL
            .iter()
            .copied()
            .filter(|category| {
                if category.opt_in() {
                    !expand.contains(category)
                } else {
                    keep.contains(category)
                }
            })
            .collect()
    }

    /// The category of the glossary entry for `token` in `role`. Every role belongs to
    /// one, so a new role is not accepted until it is placed here.
    pub fn of(token: &str, role: Role) -> Category {
        match role {
            Role::Keyword => match token {
                "pub" => Category::Visibility,
                "mut" => Category::Mutability,
                "ref" => Category::References,
                _ => Category::Keywords,
            },
            Role::ImplBlock | Role::ImplTraitType => Category::Keywords,
            Role::RefExpr
            | Role::RefType
            | Role::RefPattern
            | Role::SelfParam
            | Role::Deref
            | Role::PtrType => Category::References,
            Role::Binary | Role::CompoundAssign | Role::Not => Category::Operators,
            Role::Range | Role::RangeFrom | Role::RangeFull | Role::RangeInclusive => {
                Category::Ranges
            }
            Role::PatternOr
            | Role::PatternBinding
            | Role::Wildcard
            | Role::Discard
            | Role::InferredType
            | Role::RestPattern
            | Role::RestBinding
            | Role::StructUpdate => Category::Patterns,
            Role::Lifetime | Role::LifetimeAnonymous | Role::Label | Role::LifetimeBound => {
                Category::Lifetimes
            }
            Role::TraitBound | Role::BoundSeparator | Role::MaybeBound | Role::NegativeImpl => {
                Category::Bounds
            }
            Role::MatchArm
            | Role::Try
            | Role::TryChained
            | Role::RetType
            | Role::ClosureOpen
            | Role::ClosureClose => Category::Flow,
            Role::FnEnd
            | Role::ImplEnd
            | Role::ModEnd
            | Role::TraitEnd
            | Role::StructEnd
            | Role::EnumEnd => Category::Ends,
            Role::FragmentSpecifier | Role::ZeroOrMore | Role::OneOrMore | Role::ZeroOrOne => {
                Category::Macros
            }
            Role::ElidedFromSelf | Role::ElidedFromParam | Role::ElidedNamed => Category::Elision,
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownCategory(pub String);

impl fmt::Display for UnknownCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown category `{}`; one of: ", self.0)?;
        for (index, category) in Category::ALL.iter().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }
            f.write_str(category.as_str())?;
        }
        Ok(())
    }
}

impl std::error::Error for UnknownCategory {}
