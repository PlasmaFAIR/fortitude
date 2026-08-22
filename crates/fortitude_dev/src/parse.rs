/// Adapted from tree-sitter
/// Copyright 2026 Max Brunsfeld, Amaan Qureshi
/// SPDX-License-Identifier: MIT
use std::{
    env,
    io::{self, Write},
    path::{Path, PathBuf},
};

use anstyle::{Color, Style};
use anyhow::{Context, Result};
use fortitude_linter::fs::read_to_string;
use fortitude_macros::kind;
use fortitude_sitter::{
    Node, TreeCursor,
    ast::types::{
        Attribute, AttributeKind, HasName, NameDecl, Type, UseStatement, VariableDeclaration,
    },
    traits::HasNode,
};
use tree_sitter::{Parser, Range, ffi};
use tree_sitter_cli::{
    parse::{ParseFileOptions, ParseOutput, ParseStats, ParseTheme, parse_file_at_path},
    util,
};

#[derive(clap::Args)]
pub(crate) struct Args {
    /// The Fortran file to parse
    pub path: PathBuf,
    /// Output the parse data in a pretty-printed CST format
    #[arg(long = "cst", short = 'c', conflicts_with = "output_fortitude")]
    pub output_cst: bool,
    /// Omit ranges in the output
    #[arg(long)]
    pub no_ranges: bool,
    /// Parse into fortitude's Rust types
    #[arg(long = "fortitude", conflicts_with = "output_cst")]
    pub output_fortitude: bool,
}

pub(crate) fn main(args: &Args) -> Result<()> {
    let colour = env::var("NO_COLOR").map_or(true, |v| v != "1");
    let output = if args.output_cst {
        ParseOutput::Cst
    } else {
        ParseOutput::Normal
    };
    let parse_theme = if colour {
        ParseTheme::default()
    } else {
        ParseTheme::empty()
    };

    let mut parser = Parser::new();

    let mut stats = ParseStats::default();
    let edits: Vec<String> = vec![];
    let cancellation_flag = util::cancel_on_signal();

    let mut options = ParseFileOptions {
        edits: &edits
            .iter()
            .map(std::string::String::as_str)
            .collect::<Vec<&str>>(),
        output,
        print_time: false,
        timeout: 0,
        stats: &mut stats,
        debug: tree_sitter_cli::parse::ParseDebugType::Quiet,
        debug_graph: false,
        cancellation_flag: Some(&cancellation_flag),
        encoding: Some(ffi::TSInputEncodingUTF8),
        open_log: false,
        no_ranges: args.no_ranges,
        parse_theme: &parse_theme,
    };

    let max_path_length = args.path.to_string_lossy().chars().count();

    if args.output_fortitude {
        output_fortitude(&args.path, &options)?;
    } else {
        parse_file_at_path(
            &mut parser,
            &tree_sitter_fortran::LANGUAGE.into(),
            &args.path,
            &args.path.display().to_string(),
            max_path_length,
            &mut options,
        )?;
    }

    Ok(())
}

fn output_fortitude(path: &Path, opts: &ParseFileOptions) -> Result<()> {
    let stdout = io::stdout();
    let mut stdout = io::BufWriter::with_capacity(64 * 1024, stdout.lock());

    let mut parser = fortitude_sitter::Parser::new(&tree_sitter_fortran::LANGUAGE.into())
        .context("Error loading Fortran grammar")?;

    let source_code = read_to_string(path)?;

    let tree = parser.parse(&source_code, None)?;
    let mut cursor = tree.walk();

    let total_width = source_code
        .lines()
        .enumerate()
        .map(|(row, col)| {
            row.checked_ilog10().unwrap_or(0) as usize
                + col.len().checked_ilog10().unwrap_or(0) as usize
                + 1
        })
        .max()
        .unwrap_or(1);
    let mut indent_level = usize::from(!opts.no_ranges);
    let mut did_visit_children = false;
    let mut in_error = false;
    loop {
        if did_visit_children {
            if cursor.goto_next_sibling() {
                did_visit_children = false;
            } else if cursor.goto_parent() {
                did_visit_children = true;
                indent_level -= 1;
                if !cursor.node().has_error() {
                    in_error = false;
                }
            } else {
                break;
            }
        } else {
            if fortitude_render_node(
                opts,
                &mut cursor,
                &mut stdout,
                total_width,
                indent_level,
                in_error,
            )? {
                did_visit_children = true;
            } else if cursor.goto_first_child() {
                did_visit_children = false;
                indent_level += 1;
                if cursor.node().has_error() {
                    in_error = true;
                }
            } else {
                did_visit_children = true;
            }
        }
    }

    Ok(())
}

pub fn paint(color: Option<impl Into<Color>>, text: &str) -> String {
    let style = Style::new().fg_color(color.map(Into::into));
    format!("{style}{text}{style:#}")
}

fn render_node_range(
    opts: &ParseFileOptions,
    cursor: &TreeCursor,
    is_named: bool,
    is_multiline: bool,
    total_width: usize,
    range: Range,
) -> String {
    let has_field_name = cursor.field_name().is_some();
    let start = range.start_point;
    let end = range.end_point;
    let range_color = if is_named && !is_multiline && !has_field_name {
        opts.parse_theme.row_color_named
    } else {
        opts.parse_theme.row_color
    };

    let remaining_width = |row: usize, col: usize| {
        (total_width
            .saturating_sub(row.checked_ilog10().unwrap_or(0) as usize)
            .saturating_sub(col.checked_ilog10().unwrap_or(0) as usize))
        .max(1)
    };
    let remaining_width_start = remaining_width(start.row, start.column);
    let remaining_width_end = remaining_width(end.row, end.column);
    paint(
        range_color,
        &format!(
            "{}:{}{:remaining_width_start$}- {}:{}{:remaining_width_end$}",
            start.row, start.column, ' ', end.row, end.column, ' ',
        ),
    )
}

fn write_node_text(
    opts: &ParseFileOptions,
    out: &mut impl Write,
    cursor: &TreeCursor,
    is_named: bool,
    source: &str,
    color: Option<impl Into<Color> + Copy>,
    text_info: (usize, usize),
) -> Result<()> {
    let (total_width, indent_level) = text_info;
    let (quote, quote_color) = if is_named {
        ('`', opts.parse_theme.backtick)
    } else {
        ('\"', color.map(|c| c.into()))
    };

    if !is_named {
        write!(
            out,
            "{}{}{}",
            paint(quote_color, &String::from(quote)),
            paint(color, &render_node_text(source)),
            paint(quote_color, &String::from(quote)),
        )?;
    } else {
        let multiline = source.contains('\n');
        for (i, line) in source.split_inclusive('\n').enumerate() {
            if line.is_empty() {
                break;
            }
            let mut node_range = cursor.node().range();
            // For each line of text, adjust the row by shifting it down `i` rows,
            // and adjust the column by setting it to the length of *this* line.
            node_range.start_point.row += i;
            node_range.end_point.row = node_range.start_point.row;
            node_range.end_point.column = line.len()
                + if i == 0 {
                    node_range.start_point.column
                } else {
                    0
                };
            let formatted_line = render_line_feed(line, opts);
            write!(
                out,
                "{}{}{}{}{}{}",
                if multiline { "\n" } else { " " },
                if multiline && !opts.no_ranges {
                    render_node_range(opts, cursor, is_named, true, total_width, node_range)
                } else {
                    String::new()
                },
                if multiline {
                    "  ".repeat(indent_level + 1)
                } else {
                    String::new()
                },
                paint(quote_color, &String::from(quote)),
                paint(color, &render_node_text(&formatted_line)),
                paint(quote_color, &String::from(quote)),
            )?;
        }
    }

    Ok(())
}

fn render_node_text(source: &str) -> String {
    source
        .chars()
        .fold(String::with_capacity(source.len()), |mut acc, c| {
            if let Some(esc) = escape_invisible(c) {
                acc.push_str(esc);
            } else if let Some(esc) = escape_delimiter(c) {
                acc.push_str(esc);
            } else {
                acc.push(c);
            }
            acc
        })
}

fn render_line_feed(source: &str, opts: &ParseFileOptions) -> String {
    if cfg!(windows) {
        source.replace("\r\n", &paint(opts.parse_theme.line_feed, "\r\n"))
    } else {
        source.replace('\n', &paint(opts.parse_theme.line_feed, "\n"))
    }
}

const fn escape_invisible(c: char) -> Option<&'static str> {
    Some(match c {
        '\n' => "\\n",
        '\r' => "\\r",
        '\t' => "\\t",
        '\0' => "\\0",
        '\\' => "\\\\",
        '\x0b' => "\\v",
        '\x0c' => "\\f",
        _ => return None,
    })
}

const fn escape_delimiter(c: char) -> Option<&'static str> {
    Some(match c {
        '`' => "\\`",
        '\"' => "\\\"",
        _ => return None,
    })
}

/// The (optional) range and indentation
struct Preamble<'a> {
    opts: &'a ParseFileOptions<'a>,
    cursor: &'a TreeCursor<'a>,
    total_width: usize,
    indent_level: usize,
    in_error: bool,
}

impl<'a> Preamble<'a> {
    fn write(&self, out: &mut impl Write, node: &Node) -> Result<()> {
        write_preamble(
            self.opts,
            self.cursor,
            out,
            node,
            self.total_width,
            self.indent_level,
            self.in_error,
        )
    }
    fn writeln(&self, out: &mut impl Write, node: &Node) -> Result<()> {
        writeln!(out)?;
        self.write(out, node)
    }

    /// Range and indentation when we don't have a node
    fn write_nodeless(&self, out: &mut impl Write) -> Result<()> {
        if !self.opts.no_ranges {
            write!(
                out,
                "{}",
                paint(
                    self.opts.parse_theme.row_color_named,
                    &".".repeat(self.total_width * 4 + 2),
                )
            )?;
        }
        write!(out, "{}", "  ".repeat(self.indent_level),)?;
        Ok(())
    }
    fn writeln_nodeless(&self, out: &mut impl Write) -> Result<()> {
        writeln!(out)?;
        self.write_nodeless(out)
    }

    fn with_indent(&self) -> Self {
        Self {
            indent_level: self.indent_level + 1,
            ..*self
        }
    }
}

/// Range and indentation
fn write_preamble(
    opts: &ParseFileOptions,
    cursor: &TreeCursor,
    out: &mut impl Write,
    node: &Node,
    total_width: usize,
    indent_level: usize,
    in_error: bool,
) -> Result<()> {
    let is_named = node.is_named();
    if !opts.no_ranges {
        write!(
            out,
            "{}",
            render_node_range(opts, cursor, is_named, false, total_width, node.range())
        )?;
    }
    write!(
        out,
        "{}{}",
        "  ".repeat(indent_level),
        if in_error && !node.has_error() {
            " "
        } else {
            ""
        }
    )?;

    Ok(())
}

/// Returns `true` if we handled a Fortitude Rust AST type
fn fortitude_render_node(
    opts: &ParseFileOptions,
    cursor: &mut TreeCursor,
    out: &mut impl Write,
    total_width: usize,
    indent_level: usize,
    in_error: bool,
) -> Result<bool> {
    let node = cursor.node();
    let preamble = Preamble {
        opts,
        cursor,
        total_width,
        indent_level,
        in_error,
    };
    preamble.write(out, &node)?;
    let is_named = node.is_named();
    let handled = if is_named {
        if let Some(field_name) = cursor.field_name() {
            write!(
                out,
                "{}",
                paint(opts.parse_theme.field, &format!("{field_name}: "))
            )?;
        }

        if node.has_error() || node.is_error() {
            write!(out, "{}", paint(opts.parse_theme.error, "•"))?;
        }

        let handled = write_fortitude_type(&node, &preamble, opts, out)?;

        if handled {
            writeln!(out)?;
            preamble.with_indent().write(out, &node)?;
            write_node_text(
                opts,
                out,
                cursor,
                false,
                node.text(),
                opts.parse_theme.literal,
                (total_width, indent_level + 1),
            )?;
        }

        if node.child_count() == 0 {
            // Node text from a pattern or external scanner
            write_node_text(
                opts,
                out,
                cursor,
                is_named,
                node.text(),
                opts.parse_theme.literal,
                (total_width, indent_level),
            )?;
        }
        handled
    } else if node.is_missing() {
        write!(out, "{}: ", paint(opts.parse_theme.missing, "MISSING"))?;
        write!(out, "\"{}\"", paint(opts.parse_theme.missing, node.kind()))?;
        false
    } else {
        // Terminal literals, like "fn"
        write_node_text(
            opts,
            out,
            cursor,
            is_named,
            node.kind(),
            opts.parse_theme.literal,
            (total_width, indent_level),
        )?;
        false
    };
    writeln!(out)?;

    Ok(handled)
}

/// Returns `true` if we handled a Fortitude Rust AST type
fn write_fortitude_type(
    node: &Node,
    preamble: &Preamble,
    opts: &ParseFileOptions,
    out: &mut impl Write,
) -> Result<bool> {
    let kind_color = if node.is_error() {
        opts.parse_theme.error
    } else if node.is_extra() || node.parent().is_some_and(|p| p.is_extra() && !p.is_error()) {
        opts.parse_theme.extra
    } else {
        opts.parse_theme.node_kind
    };

    match node.kind_id() {
        kind!("use_statement") => {
            write!(out, "{}", paint(opts.parse_theme.missing, "UseStatement"))?;
            match UseStatement::try_from_node(node) {
                Ok(stmt) => {
                    write_use_stmt(stmt, &preamble.with_indent(), opts, out)?;
                }
                Err(err) => {
                    let msg = format!("INTERAL FORTITUDE ERROR: {err}");
                    write!(out, " -- {}", paint(opts.parse_theme.error, &msg))?;
                }
            }

            Ok(true)
        }
        kind!("variable_declaration") => {
            write!(
                out,
                "{}",
                paint(opts.parse_theme.missing, "VariableDeclaration")
            )?;
            match VariableDeclaration::try_from_node(node) {
                Ok(decl) => {
                    write_variable_declaration(decl, &preamble.with_indent(), opts, out)?;
                }
                Err(err) => {
                    let msg = format!("INTERAL FORTITUDE ERROR: {err}");
                    write!(out, " -- {}", paint(opts.parse_theme.error, &msg))?;
                }
            };
            Ok(true)
        }
        _ => {
            write!(out, "{}", paint(kind_color, node.kind()))?;
            Ok(false)
        }
    }
}

fn write_key_value(
    opts: &ParseFileOptions,
    out: &mut impl Write,
    key: &str,
    value: &str,
) -> Result<()> {
    write!(
        out,
        "{}: {}",
        paint(opts.parse_theme.missing, key),
        paint(opts.parse_theme.field, value)
    )?;
    Ok(())
}

fn write_variable_declaration(
    decl: VariableDeclaration,
    preamble: &Preamble,
    opts: &ParseFileOptions,
    out: &mut impl Write,
) -> Result<()> {
    write_variable_type(decl.type_(), preamble, opts, out)?;

    for attr in decl.attributes() {
        write_variable_attribute(attr, preamble, opts, out)?;
    }

    for name in decl.names() {
        write_variable_name(name, preamble, opts, out)?;
    }

    preamble.writeln_nodeless(out)?;
    write_key_value(opts, out, "has_colon", &decl.has_colon().to_string())?;
    preamble.writeln_nodeless(out)?;
    write_key_value(opts, out, "is_function", &decl.is_function().to_string())?;

    Ok(())
}

fn write_variable_type(
    type_: &Type,
    preamble: &Preamble,
    opts: &ParseFileOptions,
    out: &mut impl Write,
) -> Result<()> {
    let node = type_.node();
    preamble.writeln(out, node)?;
    write_key_value(opts, out, "Type", &type_.to_string())?;

    Ok(())
}

fn write_variable_attribute(
    attr: &Attribute,
    preamble: &Preamble,
    opts: &ParseFileOptions,
    out: &mut impl Write,
) -> Result<()> {
    let node = attr.node();
    preamble.writeln(out, node)?;
    write_key_value(opts, out, "Attribute", attr.into())?;
    match attr.kind() {
        AttributeKind::Dimension(dim) => {
            write!(out, "{}", paint(opts.parse_theme.field, &dim.to_string()))?
        }
        AttributeKind::Intent(intent) => write!(
            out,
            "({})",
            paint(opts.parse_theme.field, &intent.to_string())
        )?,
        _ => (),
    }

    Ok(())
}

fn write_variable_name(
    name: &NameDecl,
    preamble: &Preamble,
    opts: &ParseFileOptions,
    out: &mut impl Write,
) -> Result<()> {
    let node = name.node();
    preamble.writeln(out, node)?;
    write_key_value(opts, out, "Name", name.name().as_str())?;

    if let Some(size) = name.size() {
        preamble.with_indent().writeln(out, node)?;
        write_key_value(opts, out, "Size", size.text())?;
    }
    if let Some(init) = name.init() {
        preamble.with_indent().writeln(out, node)?;
        write_key_value(opts, out, "Initialiser", init.text())?;
    }

    Ok(())
}

fn write_use_stmt(
    stmt: UseStatement,
    preamble: &Preamble,
    opts: &ParseFileOptions,
    out: &mut impl Write,
) -> Result<()> {
    preamble.writeln(out, stmt.node())?;
    write_key_value(opts, out, "Name", stmt.name().as_str())?;

    preamble.writeln_nodeless(out)?;
    write_key_value(opts, out, "intrinsic", &stmt.is_intrinsic().to_string())?;
    preamble.writeln_nodeless(out)?;
    write_key_value(opts, out, "has_only", &stmt.has_only().to_string())?;
    preamble.writeln_nodeless(out)?;
    write_key_value(opts, out, "has_colon", &stmt.has_colon().to_string())?;

    Ok(())
}
