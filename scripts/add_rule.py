#!/usr/bin/env python3
# Adapted from Ruff
# Copyright 2025 Charles Marsh
# SPDX-License-Identifier: MIT
"""Generate boilerplate for a new rule.

Example usage:

    python scripts/add_rule.py \
        --name PreferListBuiltin \
        --prefix C \
        --code 807 \
        --category correctness

"""

from __future__ import annotations

import argparse
import subprocess

from _utils import LINTER_DIR, dir_name, get_indent, pascal_case, snake_case


def main(*, name: str, prefix: str, code: str, category: str) -> None:
    """Generate boilerplate for a new rule."""
    # Create a test fixture.
    filestem = f"{prefix}{code}"
    with (
        LINTER_DIR / "resources/test/fixtures" / dir_name(category) / f"{filestem}.f90"
    ).open("a"):
        pass

    plugin_module = LINTER_DIR / "src/rules" / dir_name(category)
    rule_name_snake = snake_case(name)
    new_mod = f"pub mod {rule_name_snake};\n"

    # Add the relevant `#testcase` macro.
    mod_rs = plugin_module / "mod.rs"
    content = mod_rs.read_text()

    with mod_rs.open("w") as fp:
        has_added_testcase = False
        has_added_mod = False
        lines = []
        for line in content.splitlines():
            if not has_added_mod and not line.startswith("pub mod"):
                fp.write(new_mod)
                has_added_mod = True

            if not has_added_testcase and (
                line.strip() == "fn rules(rule_code: Rule, path: &Path) -> Result<()> {"
            ):
                indent = get_indent(line)
                lines.append(
                    f'{indent}#[test_case(Rule::{name}, Path::new("{filestem}.f90"))]',
                )
                fp.write("\n".join(lines))
                fp.write("\n")
                lines.clear()
                has_added_testcase = True

            if has_added_testcase:
                fp.write(line)
                fp.write("\n")
            elif line.strip() == "":
                fp.write("\n".join(lines))
                fp.write("\n\n")
                lines.clear()
            else:
                lines.append(line)

    rules_dir = plugin_module
    rules_mod = rules_dir / "mod.rs"

    # Add the relevant rule function.
    with (rules_dir / f"{rule_name_snake}.rs").open("w") as fp:
        fp.write(
            f"""\
use crate::ast::FortitudeNode;
use crate::settings::CheckSettings;
use crate::{{AstRule, FromAstNode}};
use ruff_diagnostics::{{Diagnostic, Edit, Fix, FixAvailability, Violation}};
use ruff_macros::{{derive_message_formats, ViolationMetadata}};
use ruff_source_file::SourceFile;
use ruff_text_size::TextSize;
use tree_sitter::Node;

/// ## What it does
///
/// ## Why is this bad?
///
/// ## Example
/// ```f90
/// ```
///
/// Use instead:
/// ```f90
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct {name};

impl Violation for {name} {{
    #[derive_message_formats]
    fn message(&self) -> String {{
        format!("TODO: write message: {{}}", todo!("implement message"))
    }}
}}

impl AstRule for {name} {{
    fn check<'a>(
        _settings: &CheckSettings,
        node: &'a Node,
        src: &'a SourceFile,
    ) -> Option<Vec<Diagnostic>> {{
        None
    }}

    fn entrypoints() -> Vec<&'static str> {{
        vec![]
    }}
}}
""",
        )

    text = ""
    with (LINTER_DIR / "src/rules/mod.rs").open("r") as fp:
        while (line := next(fp)).strip() != f"// {category}":
            text += line
        text += line

        lines = []
        while (line := next(fp)).strip() != "":
            lines.append(line)

        variant = pascal_case(category)
        linter_name = category.split(" ")[0].replace("-", "_")
        rule = f"""{linter_name}::{rule_name_snake}::{name}"""
        lines.append(
            " " * 8
            + f"""({variant}, "{code}") => (RuleGroup::Preview, Ast, Optional, {rule}),\n""",
        )
        lines.sort()
        text += "".join(lines)
        text += "\n"
        text += fp.read()
    with (LINTER_DIR / "src/rules/mod.rs").open("w") as fp:
        fp.write(text)

    _rustfmt(rules_mod)


def _rustfmt(path: str) -> None:
    subprocess.run(["rustfmt", path])


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Generate boilerplate for a new rule.",
        epilog=(
            "python scripts/add_rule.py "
            "--name PreferListBuiltin --code C --linter correctness"
        ),
    )
    parser.add_argument(
        "--name",
        type=str,
        required=True,
        help=(
            "The name of the check to generate, in PascalCase "
            "(e.g., 'PreferListBuiltin')."
        ),
    )
    parser.add_argument(
        "--prefix",
        type=str,
        required=True,
        help="Prefix code for the plugin (e.g. 'C').",
    )
    parser.add_argument(
        "--code",
        type=str,
        required=True,
        help="The code of the check to generate (e.g., '807').",
    )
    parser.add_argument(
        "--category",
        type=str,
        required=True,
        help="The category to add the rule to (e.g., 'correctness').",
    )
    args = parser.parse_args()

    main(name=args.name, prefix=args.prefix, code=args.code, category=args.category)
