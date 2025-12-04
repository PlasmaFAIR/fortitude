use std::collections::HashMap;
use std::io::{self, BufWriter};
use std::process::ExitCode;

use anyhow::Result;
use ruff_source_file::SourceFileBuilder;
use serde::Serializer;
use serde::ser::SerializeSeq;

use crate::cli::{HelpFormat, PreprocessCommand};
use fortitude_linter::fs::read_to_string;
use fortitude_preprocessor::cpp::CPreprocessor;

pub fn preprocess(args: PreprocessCommand) -> Result<ExitCode> {
    // Get the source code from the input file
    let path = args.input_file.as_path();
    let source = match read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            return Err(anyhow::anyhow!(
                "Failed to read input file '{}': {}",
                path.display(),
                err
            ));
        }
    };
    let source_file = SourceFileBuilder::new(path.to_string_lossy(), source.as_str()).finish();

    // Set up user defines
    let defines = args
        .define
        .iter()
        .filter_map(|def| {
            let mut parts = def.splitn(2, '=');
            let key = parts.next()?.to_string();
            let value = parts.next().unwrap_or("").to_string();
            Some((key, value))
        })
        .collect::<HashMap<_, _>>();

    // Preprocess the source code
    let preprocessor = CPreprocessor::new(&source_file.to_source_code(), path, &defines)?;

    // Output results
    match args.output_format {
        HelpFormat::Text => {
            println!("! {}\n{}", path.to_string_lossy(), preprocessor.output());
        }
        HelpFormat::Json => {
            let stdout = BufWriter::new(io::stdout().lock());
            let mut serialiser = serde_json::Serializer::pretty(stdout);
            let mut seq = serialiser.serialize_seq(None)?;
            for snippet in preprocessor {
                seq.serialize_element(&snippet)?;
            }
            seq.end()?;
        }
    }

    Ok(ExitCode::SUCCESS)
}
