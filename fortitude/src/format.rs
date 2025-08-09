use std::{
    cmp::Ordering,
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
    process::ExitCode,
};

use crate::{
    cli::FormatArgs,
    configuration::{self, resolve_bool_arg},
    fs::{get_files, FilePatternSet, FORTRAN_EXTS},
    settings::{FileResolverSettings, PreviewMode},
};

use anyhow::Result;
use itertools::Itertools;
use topiary_core::{FormatterError, Language, TopiaryQuery};
use tree_sitter::{Point, Query, QueryCursor, StreamingIterator, Tree, TreeCursor};

fn topiary_query() -> &'static str {
    include_str!("../resources/format/fortran.scm")
}

fn wrap_query() -> &'static str {
    include_str!("../resources/format/wrap.scm")
}

/// Create the topiary formatter
fn create_formatter() -> Language {
    let grammar: topiary_tree_sitter_facade::Language = tree_sitter_fortran::LANGUAGE.into();
    let query = TopiaryQuery::new(&grammar, topiary_query()).expect("building topiary query");
    Language {
        name: "fortran".to_string(),
        query,
        grammar,
        indent: None,
    }
}

/// Run the formatter over a whole project
///
/// TODO: Proper options like ``check``
pub fn format(args: FormatArgs) -> Result<ExitCode> {
    if !args.i_understand_the_risks.unwrap_or_default() {
        println!(
            "The format command is still in development and may break your code (although it
_probably_ won't), and future changes may give different results. To use the
format command, you must set `--i-understand-the-risks` on the command line to acknowledge the
risks."
        );
        return Ok(ExitCode::FAILURE);
    }

    let preview = resolve_bool_arg(args.preview, args.no_preview)
        .map(PreviewMode::from)
        .unwrap_or_default();

    if preview == PreviewMode::Disabled {
        println!("Format mode is currently in preview; nothing to do");
        return Ok(ExitCode::SUCCESS);
    }

    let files = args.files.unwrap_or_default();
    let file_extensions = args
        .file_extensions
        .unwrap_or(FORTRAN_EXTS.iter().map(|ext| ext.to_string()).collect_vec());
    let project_root = configuration::project_root(path_absolutize::path_dedot::CWD.as_path())?;

    let language = create_formatter();

    let file_resolver = FileResolverSettings {
        excludes: FilePatternSet::default(),
        force_exclude: true,
        files,
        file_extensions,
        respect_gitignore: true,
        project_root,
    };

    for file in get_files(&file_resolver, false)? {
        let output = std::io::stdout();
        let mut buf_output = BufWriter::new(output);

        match format_file(file, &language, &mut buf_output) {
            Ok(_) => continue,
            Err(err) => {
                println!("Formatter error: {err}");
                return Ok(ExitCode::FAILURE);
            }
        };
    }

    Ok(ExitCode::SUCCESS)
}

// An actual computed line break
#[derive(Debug, Clone)]
struct Break {
    point: Point,
    next_indent: usize,
}

#[derive(Debug, Clone)]
struct BreakCollection {
    breaks: Vec<Break>,
    pattern_id: usize,
}

// Repeately break lines until no long lines remain
fn break_lines(lines: &mut Vec<String>) {
    for _ in 1..10 {
        if dbg!(break_lines_once(lines)) == 0 {
            break;
        };
    }
}

fn get_indent(line: &String) -> usize {
    line.find(|c: char| !c.is_whitespace()).unwrap_or(0)
}

fn break_lines_once(lines: &mut Vec<String>) -> usize {
    let language = tree_sitter_fortran::LANGUAGE;
    let mut parser = tree_sitter::Parser::new();

    let long_lines: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, x)| x.len() > LINE_LENGTH)
        .map(|(i, _)| i)
        .collect();

    if long_lines.len() == 0 {
        return 0;
    }

    parser
        .set_language(&language.into())
        .expect("Error loading Fortran parser");

    let tree = parser
        .parse_with_options(
            &mut |_byte: usize, position: Point| -> &[u8] {
                let row = position.row as usize;
                let column = position.column as usize;
                if row < lines.len() {
                    if column < lines[row].as_bytes().len() {
                        &lines[row].as_bytes()[column..]
                    } else {
                        b"\n"
                    }
                } else {
                    &[]
                }
            },
            None,
            None,
        )
        .unwrap();

    let query = Query::new(&language.into(), wrap_query()).unwrap();

    let i_root = query.capture_index_for_name("root").unwrap();
    let i_break_after = query.capture_index_for_name("break-after").unwrap();

    let mut cursor = QueryCursor::new();

    // I couldn't figure out of to get text_provider to work with Vec<String> as input
    let text = lines.join("\n");

    let matches = cursor.matches(&query, tree.root_node(), text.as_bytes());

    let mut possible_breaks = HashMap::<usize, BreakCollection>::new();

    matches.for_each(|x| {
        let mut break_node_id = None;
        let mut breaks = vec![];
        x.captures.iter().for_each(|qc| {
            let range = &qc.node.range();
            if qc.index == i_root {
                assert!(
                    break_node_id.replace(qc.node.id()) == None,
                    "Should only be one root node"
                );
            } else if qc.index == i_break_after {
                let row = qc.node.start_position().row;
                breaks.push(Break {
                    point: range.end_point,
                    next_indent: get_indent(&lines[row]) + INDENT,
                })
            }
            // Can there be more than on match per capture?
        });

        assert!(breaks.len() > 0, "There should be some breaks found");
        let break_node_id = break_node_id.expect("There should be some breaks found");

        match possible_breaks.get_mut(&break_node_id) {
            None => {
                possible_breaks.insert(
                    break_node_id,
                    BreakCollection {
                        breaks: breaks,
                        pattern_id: x.pattern_index,
                    },
                );
            }
            Some(b) => {
                assert!(
                    b.pattern_id == x.pattern_index,
                    "We should only match each node with the same pattern?"
                );
                b.breaks.extend(breaks)
            }
        };
    });

    let mut breaks: Vec<Break> = get_breaks(&tree, &long_lines, &mut possible_breaks)
        .iter()
        .flat_map(|x| x.breaks.clone())
        .collect();

    breaks.sort_by_key(|x| (x.point.row, x.point.column));

    let mut num_breaks = 0;

    breaks.iter().rev().for_each(|br| {
        num_breaks += 1;

        let line = lines.get_mut(br.point.row).unwrap();
        let next_line = line.split_off(br.point.column);
        line.push_str(" &");

        lines.insert(
            br.point.row + 1,
            format!("{}{}", " ".repeat(br.next_indent), next_line.trim_start()),
        );
    });

    num_breaks
}

fn get_breaks(
    tree: &Tree,
    long_lines: &Vec<usize>,
    possible_breaks: &mut HashMap<usize, BreakCollection>,
) -> Vec<BreakCollection> {
    let mut cursor = tree.walk();
    let mut actual_breaks = vec![];

    get_breaks_dfs(
        &mut cursor,
        long_lines,
        possible_breaks,
        &mut actual_breaks,
        false,
    );

    actual_breaks
}

const LINE_LENGTH: usize = 60;
const INDENT: usize = 3;

fn get_breaks_dfs(
    cursor: &mut TreeCursor,
    long_lines: &Vec<usize>,
    possible_breaks: &mut HashMap<usize, BreakCollection>,
    actual_breaks: &mut Vec<BreakCollection>,
    ancestor_is_breakable: bool,
) -> Option<BreakCollection> {
    let node = cursor.node();

    if !node_contains_long_line(node, long_lines) {
        // No reason to continue if subtree contains no long lines
        return None;
    }

    let mut breaks = possible_breaks.remove(&node.id());

    let this_is_breakable =
        breaks.is_some() && node.start_position().row == node.end_position().row;

    if !this_is_breakable && ancestor_is_breakable {
        // No reason to continue if parent is breakable, and this
        // node has no breaks for parent to absorb
        return None;
    }

    // The following implements a DFS search, searching for the
    // outer-most breakable scope of the syntax tree.
    if cursor.goto_first_child() {
        loop {
            let child_breaks = get_breaks_dfs(
                cursor,
                long_lines,
                possible_breaks,
                actual_breaks,
                ancestor_is_breakable || this_is_breakable,
            );

            // We need to absorb similar breaks into this the parent node to deal with eg. math expressions.
            match (&mut breaks, &child_breaks) {
                (Some(br), Some(ch)) if br.pattern_id == ch.pattern_id => {
                    br.breaks.extend(ch.breaks.clone());
                }
                _ => {}
            }

            if !cursor.goto_next_sibling() {
                break;
            };
        }
        assert!(cursor.goto_parent());
    }

    if (!ancestor_is_breakable) && this_is_breakable {
        // If we get to here, then this node is the outer-most breakable syntax node,
        // and should therefore be line-broken.
        actual_breaks.push(breaks.take().unwrap());
    }

    breaks
}

fn node_contains_long_line(node: tree_sitter::Node<'_>, long_lines: &Vec<usize>) -> bool {
    let a = node.start_position().row;
    let b = node.end_position().row;

    long_lines
        .binary_search_by(|&i| {
            if a <= i && i <= b {
                Ordering::Equal
            } else if i < a {
                Ordering::Less
            } else if i > b {
                Ordering::Greater
            } else {
                panic!("Should be unreachable")
            }
        })
        .is_ok()
}

/// Format an individual file
pub(crate) fn format_file(
    file: PathBuf,
    _language: &Language,
    output: &mut impl Write,
) -> Result<(), FormatterError> {
    println!("formatting {file:?}");
    let input = File::open(file)?;

    let buf_input = BufReader::new(input);

    let mut lines: Vec<String> = buf_input.lines().map(|x| x.unwrap().to_owned()).collect();

    break_lines(&mut lines);

    for ele in lines {
        output.write(ele.as_bytes())?;
        output.write("\n".as_bytes())?;
    }

    // TODOs:
    //  - line_length / indent as parameters
    //  - Remove existing line-continuations prior to wrapping?
    //  - Integrate in formatting pipeline
    //  - Error handling?
    //  - Tests?
    //  - Optimisations?
    //      - Query and look up nodes to insert line breaks via descendant_for_point_range()?
    //      - Implement TextProvider for Vec<String>?
    //      - Reuse parse / parse tree

    // formatter(
    //     &mut buf_input,
    //     output,
    //     language,
    //     // TODO: user args?
    //     Operation::Format {
    //         skip_idempotence: true,
    //         tolerate_parsing_errors: true,
    //     },
    // )?;

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
