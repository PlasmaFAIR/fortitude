use anyhow::{Context, Result};
use itertools::Itertools;
use lazy_regex::regex;
use std::process::ExitCode;
use tree_sitter::Parser;

use crate::{
    check::read_to_string,
    cli::ConvertFixedArgs,
    configuration::{self},
    fs::{get_files, FilePatternSet, EXCLUDE_BUILTINS, FIXED_FORTRAN_EXTS},
    settings::FileResolverSettings,
    warn_user,
};

pub fn convert_fixed(args: ConvertFixedArgs) -> Result<ExitCode> {
    let project_root = configuration::project_root(path_absolutize::path_dedot::CWD.as_path())?;

    let file_extensions = args.file_extensions.unwrap_or(
        FIXED_FORTRAN_EXTS
            .iter()
            .map(|ext| ext.to_string())
            .collect_vec(),
    );

    let excludes = FilePatternSet::try_from_iter(
        EXCLUDE_BUILTINS
            .iter()
            .cloned()
            .chain(args.exclude.unwrap_or_default().into_iter()),
    )?;

    let resolver = FileResolverSettings {
        excludes,
        force_exclude: true,
        files: args.files.unwrap_or_default(),
        file_extensions,
        fixed_extensions: vec![],
        respect_gitignore: true,
        project_root,
    };

    for path in get_files(&resolver, false)? {
        let text = read_to_string(&path)?;

        if has_syntax_errors(&text, FortranForm::Fixed)? {
            warn_user!("file '{}' has syntax errors, skipping", &path.display());
            continue;
        }

        let converted_text = convert_contents(text);

        if has_syntax_errors(&converted_text, FortranForm::Free)? {
            warn_user!("error converting file '{}', skipping", &path.display());
            continue;
        }

        std::fs::write(path, converted_text)?;
    }

    Ok(ExitCode::SUCCESS)
}

enum FortranForm {
    Fixed,
    Free,
}

fn has_syntax_errors<S: AsRef<str>>(text: S, form: FortranForm) -> anyhow::Result<bool> {
    let language = match form {
        FortranForm::Fixed => tree_sitter_fixed_form_fortran::LANGUAGE,
        FortranForm::Free => tree_sitter_fortran::LANGUAGE,
    };
    let mut parser = Parser::new();
    parser
        .set_language(&language.into())
        .context("Error loading Fortran grammar")?;

    let tree = parser
        .parse(text.as_ref(), None)
        .context("Failed to parse")?;

    Ok(tree.root_node().has_error())
}

fn convert_contents<S: AsRef<str>>(fixed: S) -> String {
    let comments_re = regex!(r#"^[^\s]"#im);
    let new_string = comments_re.replace_all(fixed.as_ref(), "!");

    let continue_re = regex!(r#"\n^     [^\s]"#m);
    let new_string = continue_re.replace_all(&new_string, " &\n     &");

    new_string.to_string()
}

#[cfg(test)]
mod tests {
    use similar_asserts::assert_eq;

    use super::*;

    #[test]
    fn basic_convert() -> Result<()> {
        let fixed_file = r#"
c Example fixed form program
      PROGRAM TEST
      PRINT*,'start
     +        stop'
* another comment
      END
"#;

        let free_file = r#"
! Example fixed form program
      PROGRAM TEST
      PRINT*,'start &
     &        stop'
! another comment
      END
"#;

        let result = convert_contents(fixed_file);

        assert_eq!(result, free_file);

        Ok(())
    }
}
