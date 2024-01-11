use crate::rules::{register_rule, Category, Code, Method, Rule, Status};
use std::collections::HashMap;

pub mod floating_point;
pub mod implicit_none;
pub mod modules_and_programs;

pub fn add_best_practices_rules(registry: &mut HashMap<String, Rule>) {
    for rule in [
        Rule::new(
            Code::new(Category::BestPractices, 1),
            Method::Tree(modules_and_programs::use_modules_and_programs),
            modules_and_programs::USE_MODULES_AND_PROGRAMS,
            Status::Standard,
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
            Method::Tree(floating_point::avoid_double_precision),
            floating_point::AVOID_DOUBLE_PRECISION,
            Status::Standard,
        ),
    ] {
        register_rule(registry, rule);
    }
}
