use std::{
    fs::File,
    io::{BufReader, Write},
    path::PathBuf,
};

use anyhow::Result;
use topiary_core::{formatter, FormatterError, Language, Operation, TopiaryQuery};

fn topiary_query() -> &'static str {
    include_str!("../resources/format/fortran.scm")
}

/// Create the topiary formatter
pub fn create_formatter() -> Language {
    let grammar: topiary_tree_sitter_facade::Language = tree_sitter_fortran::LANGUAGE.into();
    let query = TopiaryQuery::new(&grammar, topiary_query()).expect("building topiary query");
    Language {
        name: "fortran".to_string(),
        query,
        grammar,
        indent: None,
    }
}

/// Format an individual file
pub fn format_file(
    file: PathBuf,
    language: &Language,
    output: &mut impl Write,
) -> Result<(), FormatterError> {
    println!("formatting {file:?}");
    let input = File::open(file)?;

    let mut buf_input = BufReader::new(input);

    formatter(
        &mut buf_input,
        output,
        language,
        // TODO: user args?
        Operation::Format {
            skip_idempotence: true,
            tolerate_parsing_errors: true,
        },
    )?;

    Ok(())
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use anyhow::Result;
    use insta::assert_snapshot;
    use lazy_static::lazy_static;
    use test_case::test_case;
    use topiary_core::{FormatterError, Language};

    use crate::apply_common_filters;

    use super::{create_formatter, format_file};

    lazy_static! {
        pub static ref TEST_FORMATTER: Language = create_formatter();
    }

    #[test_case(Path::new("simple.f90"))]
    #[test_case(Path::new("block-construct.f90"))]
    #[test_case(Path::new("block-data-obsolescent.f90"))]
    #[test_case(Path::new("deferred-binding.f90"))]
    #[test_case(Path::new("derived-type-attributes.f90"))]
    #[test_case(Path::new("derived-type-private-type-bound-procedures.f90"))]
    #[test_case(Path::new("derived-type-procedure-pointer-components.f90"))]
    #[test_case(Path::new("derived-type-variable-declarations.f90"))]
    #[test_case(Path::new("enumeration-type.f90"))]
    #[test_case(Path::new("functions-complex.f90"))]
    #[test_case(Path::new("functions-simple.f90"))]
    #[test_case(Path::new("interface-abstract.f90"))]
    #[test_case(Path::new("interface-assignment.f90"))]
    #[test_case(Path::new("interface-explicit-2.f90"))]
    #[test_case(Path::new("interface-explicit.f90"))]
    #[test_case(Path::new("interface-generic.f90"))]
    #[test_case(Path::new("interface-operator.f90"))]
    #[test_case(Path::new("module-definition-with-internal-module-procedures.f90"))]
    #[test_case(Path::new("procedure-as-argument.f90"))]
    #[test_case(Path::new("program.f90"))]
    #[test_case(Path::new("semicolon-in-interface.f90"))]
    #[test_case(Path::new("submodule-definition-simple.f90"))]
    #[test_case(Path::new("subroutine.f90"))]
    #[test_case(Path::new("use-operator-and-assignment.f90"))]
    fn format(path: &Path) -> Result<(), FormatterError> {
        let snapshot = format!("{}", path.to_string_lossy());

        let path = Path::new("./resources/test/fixtures/format").join(path);

        let mut buf = Vec::new();
        format_file(path, &TEST_FORMATTER, &mut buf)?;
        apply_common_filters!();

        let string = String::from_utf8(buf)?;
        assert_snapshot!(snapshot, string);

        Ok(())
    }
}
