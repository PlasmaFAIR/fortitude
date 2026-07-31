use crate::diagnostics::{Diagnostic, Violation};
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::{ViolationMetadata, kind};
use ruff_macros::derive_message_formats;
use tree_sitter::Node;

/// ## What it does
/// Checks that `select case` statements have a `case default`.
///
/// ## Why is this bad?
/// Select statements without a default case can lead to incomplete handling of
/// the possible options. If a value isn't handled by any of the cases, the
/// program will continue execution, which may lead to surprising results.  This
/// is a common source of bugs when adding new options, as it's easy to forget
/// to update all `select` statements.  Unfortunately, because Fortran doesn't
/// have proper enums, it's not possible for the compiler to issue warnings for
/// non-exhaustive cases. Having a default case allows for the program to
/// gracefully handle errors.
///
/// ## Examples
///
/// Instead of:
///
/// ```f90
/// select case(ntype)
/// case (1)
///     call routine1()
/// case (2)
///     call routine2()
/// end select
/// ```
///
/// use:
///
/// ```f90
/// select case(ntype)
/// case (1)
///     call routine1()
/// case (2)
///     call routine2()
/// case default
///     call handle_error("Invalid ntype: ", ntype)
/// end select
/// ```
///
/// If you do only intend to handle a subset of cases, you can use a `continue`
/// statement with an explanatory comment:
///
/// ```f90
/// select case(ntype)
/// case (1)
///     call routine1()
/// case (2)
///     call routine2()
/// case default
///     ! Other ntypes handled elsewhere
///     continue
/// end select
/// ```
///
/// You may also consider instead using an `if` statement. This can make your
/// intention more obvious.
#[derive(ViolationMetadata)]
pub(crate) struct MissingDefaultCase {}

impl Violation for MissingDefaultCase {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Missing default case may not handle all values".to_string()
    }

    fn fix_title(&self) -> Option<String> {
        Some("Add 'case default'".to_string())
    }
}

impl AstRule for MissingDefaultCase {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let has_default = node
            .named_children(&mut node.walk())
            .filter(|child| child.kind_id() == kind!("case_statement"))
            .any(|case| {
                case.named_children(&mut case.walk())
                    .any(|child| child.kind_id() == kind!("default"))
            });

        if has_default {
            None
        } else {
            some_vec!(context.create_diagnostic(Self {}, node))
        }
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["select_case_statement"]
    }
}

/// ## What it does
/// Checks that `select type` statements have a `class default`.
///
/// ## Why is this bad?
/// Select statements without a default can lead to incomplete handling of
/// the possible options. If the type isn't handled by any of the classes, the
/// program will continue execution, which may lead to surprising results.  This
/// is a common source of bugs when adding new types or options, as it's easy to forget
/// to update all `select` statements. Having a default allows for the program to
/// gracefully handle errors.
///
/// ## Examples
///
/// Instead of:
///
/// ```f90
/// select type(pet)
/// type is (dog_t)
///     call routine1()
/// class is (animal_t)
///     call routine2()
/// end select
/// ```
///
/// use:
///
/// ```f90
/// select type(pet)
/// type is (dog_t)
///     call routine1()
/// class is (animal_t)
///     call routine2()
/// class default
///    call handle_error("Invalid pet: ", pet)
/// end select
/// ```
///
/// If you do only intend to handle a subset of types, you can use a `continue`
/// statement with an explanatory comment:
///
/// ```f90
/// select type(pet)
/// type is (dog_t)
///     call routine1()
/// class is (animal_t)
///     call routine2()
/// class default
///     ! Other pet types handled elsewhere
///     continue
/// end select
/// ```
///
/// You may also consider instead using an `if` statement. This can make your
/// intention more obvious.
#[derive(ViolationMetadata)]
pub(crate) struct MissingDefaultType {}

impl Violation for MissingDefaultType {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Missing default class may not handle all types".to_string()
    }

    fn fix_title(&self) -> Option<String> {
        Some("Add 'class default'".to_string())
    }
}

impl AstRule for MissingDefaultType {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let has_default = node
            .named_children(&mut node.walk())
            .filter(|child| child.kind_id() == kind!("type_statement"))
            .any(|case| {
                case.named_children(&mut case.walk())
                    .any(|child| child.kind_id() == kind!("default"))
            });

        if has_default {
            None
        } else {
            some_vec!(context.create_diagnostic(Self {}, node))
        }
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["select_type_statement"]
    }
}

/// ## What it does
/// Checks that `select rank` statements have a `rank default`.
///
/// ## Why is this bad?
/// Select statements without a default can lead to incomplete handling of
/// the possible options. If the given rank isn't handled by any of the cases, the
/// program will continue execution, which may lead to surprising results. This
/// is a common source of bugs if the processing is rank-specific, and especially
/// if the variable is part of the arguments to a function/subroutine. Having a
/// default allows for the program to gracefully handle errors.
///
/// ## Examples
///
/// Instead of:
///
/// ```f90
/// select rank(A)
/// rank (0)
///     ! Scalar
///     call scalarVersion(A)
/// rank (1)
///     call vectorVersion(A)
/// end select
/// ```
///
/// use:
///
/// ```f90
/// select rank(A)
/// rank (0)
///     ! Scalar
///     call scalarVersion(A)
/// rank (1)
///     call vectorVersion(A)
/// rank default
///     call handle_error("Unsupported rank: ", rank(A))
/// end select
/// ```
///
/// If you do only intend to handle a subset of ranks, you can use a `continue`
/// statement with an explanatory comment:
///
/// ```f90
/// select rank(A)
/// rank (0)
///     ! Scalar
///     call scalarVersion(A)
/// rank (1)
///     call vectorVersion(A)
/// rank default
///     ! Other ranks handled elsewhere
///     continue
/// end select
/// ```
///
/// You may also consider instead using an `if` statement. This can make your
/// intention more obvious.
#[derive(ViolationMetadata)]
pub(crate) struct MissingDefaultRank {}

impl Violation for MissingDefaultRank {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Missing default rank may not handle all ranks".to_string()
    }

    fn fix_title(&self) -> Option<String> {
        Some("Add 'rank default'".to_string())
    }
}

impl AstRule for MissingDefaultRank {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let has_default = node
            .named_children(&mut node.walk())
            .filter(|child| child.kind_id() == kind!("rank_statement"))
            .any(|case| {
                case.named_children(&mut case.walk())
                    .any(|child| child.kind_id() == kind!("default"))
            });

        if has_default {
            None
        } else {
            some_vec!(context.create_diagnostic(Self {}, node))
        }
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids!["select_rank_statement"]
    }
}
