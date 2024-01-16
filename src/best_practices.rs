use crate::rules::{register_rule, Category, Code, Method, Registry, Rule, Status};

mod implicit_none;
mod kinds;
mod modules_and_programs;

pub fn add_best_practices_rules(registry: &mut Registry) {
    for rule in [
        Rule::new(
            Code::new(Category::BestPractices, 1),
            Method::Tree(modules_and_programs::use_modules_and_programs),
            modules_and_programs::USE_MODULES_AND_PROGRAMS,
            Status::Standard,
        ),
        Rule::new(
            Code::new(Category::BestPractices, 2),
            Method::Tree(modules_and_programs::use_only_clause),
            modules_and_programs::USE_ONLY_CLAUSE,
            Status::Optional,
        ),
        Rule::new(
            Code::new(Category::BestPractices, 10),
            Method::Tree(implicit_none::use_implicit_none_modules_and_programs),
            implicit_none::USE_IMPLICIT_NONE_MODULES_AND_PROGRAMS,
            Status::Standard,
        ),
        Rule::new(
            Code::new(Category::BestPractices, 11),
            Method::Tree(implicit_none::use_implicit_none_interfaces),
            implicit_none::USE_IMPLICIT_NONE_INTERFACES,
            Status::Standard,
        ),
        Rule::new(
            Code::new(Category::BestPractices, 12),
            Method::Tree(implicit_none::avoid_superfluous_implicit_none),
            implicit_none::AVOID_SUPERFLUOUS_IMPLICIT_NONE,
            Status::Optional,
        ),
        Rule::new(
            Code::new(Category::BestPractices, 20),
            Method::Tree(kinds::avoid_number_literal_kinds),
            kinds::AVOID_NUMBER_LITERAL_KINDS,
            Status::Standard,
        ),
        Rule::new(
            Code::new(Category::BestPractices, 21),
            Method::Tree(kinds::avoid_non_standard_byte_specifier),
            kinds::AVOID_NON_STANDARD_BYTE_SPECIFIER,
            Status::Standard,
        ),
        Rule::new(
            Code::new(Category::BestPractices, 22),
            Method::Tree(kinds::avoid_double_precision),
            kinds::AVOID_DOUBLE_PRECISION,
            Status::Standard,
        ),
        Rule::new(
            Code::new(Category::BestPractices, 23),
            Method::Tree(kinds::use_floating_point_suffixes),
            kinds::USE_FLOATING_POINT_SUFFIXES,
            Status::Standard,
        ),
        Rule::new(
            Code::new(Category::BestPractices, 24),
            Method::Tree(kinds::avoid_numbered_kind_suffixes),
            kinds::AVOID_NUMBERED_KIND_SUFFIXES,
            Status::Standard,
        ),
    ] {
        register_rule(registry, rule);
    }
}
