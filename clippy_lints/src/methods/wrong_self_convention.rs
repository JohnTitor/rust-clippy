use crate::methods::SelfKind;
use clippy_utils::diagnostics::span_lint_and_help;
use rustc_lint::LateContext;
use rustc_middle::ty::TyS;
use rustc_span::source_map::Span;
use std::fmt;

use super::WRONG_PUB_SELF_CONVENTION;
use super::WRONG_SELF_CONVENTION;

#[rustfmt::skip]
const CONVENTIONS: [(&[Convention], &[SelfKind]); 8] = [
    (&[Convention::Eq("new")], &[SelfKind::No]),
    (&[Convention::StartsWith("as_")], &[SelfKind::Ref, SelfKind::RefMut]),
    (&[Convention::StartsWith("from_")], &[SelfKind::No]),
    (&[Convention::StartsWith("into_")], &[SelfKind::Value]),
    (&[Convention::StartsWith("is_")], &[SelfKind::Ref, SelfKind::No]),
    (&[Convention::Eq("to_mut")], &[SelfKind::RefMut]),
    (&[Convention::StartsWith("to_"), Convention::EndsWith("_mut")], &[SelfKind::RefMut]),
    (&[Convention::StartsWith("to_"), Convention::NotEndsWith("_mut")], &[SelfKind::Ref]),
];

enum Convention {
    Eq(&'static str),
    StartsWith(&'static str),
    EndsWith(&'static str),
    NotEndsWith(&'static str),
}

impl Convention {
    #[must_use]
    fn check(&self, other: &str) -> bool {
        match *self {
            Self::Eq(this) => this == other,
            Self::StartsWith(this) => other.starts_with(this) && this != other,
            Self::EndsWith(this) => other.ends_with(this) && this != other,
            Self::NotEndsWith(this) => !Self::EndsWith(this).check(other),
        }
    }
}

impl fmt::Display for Convention {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match *self {
            Self::Eq(this) => this.fmt(f),
            Self::StartsWith(this) => this.fmt(f).and_then(|_| '*'.fmt(f)),
            Self::EndsWith(this) => '*'.fmt(f).and_then(|_| this.fmt(f)),
            Self::NotEndsWith(this) => '~'.fmt(f).and_then(|_| this.fmt(f)),
        }
    }
}

pub(super) fn check<'tcx>(
    cx: &LateContext<'tcx>,
    item_name: &str,
    is_pub: bool,
    self_ty: &'tcx TyS<'tcx>,
    first_arg_ty: &'tcx TyS<'tcx>,
    first_arg_span: Span,
) {
    let lint = if is_pub {
        WRONG_PUB_SELF_CONVENTION
    } else {
        WRONG_SELF_CONVENTION
    };
    if let Some((conventions, self_kinds)) = &CONVENTIONS
        .iter()
        .find(|(convs, _)| convs.iter().all(|conv| conv.check(item_name)))
    {
        if !self_kinds.iter().any(|k| k.matches(cx, self_ty, first_arg_ty)) {
            let suggestion = {
                if conventions.len() > 1 {
                    let special_case = {
                        // Don't mention `NotEndsWith` when there is also `StartsWith` convention present
                        if conventions.len() == 2 {
                            match conventions {
                                [Convention::StartsWith(starts_with), Convention::NotEndsWith(_)]
                                | [Convention::NotEndsWith(_), Convention::StartsWith(starts_with)] => {
                                    Some(format!("methods called `{}`", Convention::StartsWith(starts_with)))
                                },
                                _ => None,
                            }
                        } else {
                            None
                        }
                    };

                    if let Some(suggestion) = special_case {
                        suggestion
                    } else {
                        let s = conventions
                            .iter()
                            .map(|c| format!("`{}`", &c.to_string()))
                            .collect::<Vec<_>>()
                            .join(" and ");

                        format!("methods called like this: ({})", &s)
                    }
                } else {
                    format!("methods called `{}`", &conventions[0])
                }
            };

            span_lint_and_help(
                cx,
                lint,
                first_arg_span,
                &format!(
                    "{} usually take {}",
                    suggestion,
                    &self_kinds
                        .iter()
                        .map(|k| k.description())
                        .collect::<Vec<_>>()
                        .join(" or ")
                ),
                None,
                "consider choosing a less ambiguous name",
            );
        }
    }
}
