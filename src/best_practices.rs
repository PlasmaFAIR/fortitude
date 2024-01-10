use crate::rules::{register_rule, Category, Code, Method, Rule, Status};
use std::collections::HashMap;

pub mod use_implicit_none;
pub mod use_modules;

pub fn add_best_practices_rules(registry: &mut HashMap<String, Rule>) {
    register_rule(
        registry,
        Rule::new(
            Code::new(Category::BestPractices, 1),
            Method::Tree(use_modules::use_modules),
            use_modules::USE_MODULES,
            Status::Standard,
        ),
    );
    register_rule(
        registry,
        Rule::new(
            Code::new(Category::BestPractices, 10),
            Method::Tree(use_implicit_none::use_implicit_none),
            use_implicit_none::USE_IMPLICIT_NONE,
            Status::Standard,
        ),
    );
    register_rule(
        registry,
        Rule::new(
            Code::new(Category::BestPractices, 11),
            Method::Tree(use_implicit_none::use_interface_implicit_none),
            use_implicit_none::USE_INTERFACE_IMPLICIT_NONE,
            Status::Standard,
        ),
    );
    register_rule(
        registry,
        Rule::new(
            Code::new(Category::BestPractices, 12),
            Method::Tree(use_implicit_none::avoid_superfluous_implicit_none),
            use_implicit_none::AVOID_SUPERFLUOUS_IMPLICIT_NONE,
            Status::Optional,
        ),
    );
}
