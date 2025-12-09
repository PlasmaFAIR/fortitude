use std::{collections::HashMap, str::FromStr};

use anyhow::{Context, Result};
use itertools::Itertools;
use ruff_text_size::TextRange;
use strum_macros::{EnumIs, EnumString, IntoStaticStr};
use tree_sitter::Node;

use crate::ast::FortitudeNode;

pub const BEGIN_SCOPE_NODES: &[&str] = &[
    "program",
    "module",
    "subroutine",
    "function",
    "derived_type_definition",
    "block_construct",
];
pub const END_SCOPE_NODES: &[&str] = &[
    "end_program_statement",
    "end_module_statement",
    "end_subroutine_statement",
    "end_function_statement",
    "end_type_statement",
    "end_block_construct_statement",
];

/// A declaration of a single variable
#[derive(Clone, Debug)]
pub struct NameDecl<'a> {
    name: String,
    node: Node<'a>,
}

impl<'a> NameDecl<'a> {
    pub fn from_node(node: &Node<'a>, src: &str) -> Self {
        Self {
            name: get_name_node_of_declarator(node)
                .to_text(src)
                .unwrap_or("<unknown>")
                .to_string(),
            node: *node,
        }
    }
    pub fn textrange(&self) -> TextRange {
        self.node.textrange()
    }
}

#[derive(Clone, Copy, Debug, EnumIs, EnumString, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum AttributeKind {
    Abstract,
    Allocatable,
    Asynchronous,
    Automatic,
    Codimension,
    Dimension,
    Constant,
    Continguous,
    Device,
    External,
    Intent,
    Intrinsic,
    Managed,
    Optional,
    Parameter,
    Pinned,
    Pointer,
    Private,
    Protected,
    Public,
    Rank,
    Save,
    Sequence,
    Shared,
    Static,
    Target,
    Texture,
    Value,
    Volatile,
    Unknown,
}

impl AttributeKind {
    pub fn from_node(value: &Node, src: &str) -> Self {
        let first_child = value.child(0).unwrap().to_text(src).unwrap_or("<unknown>");
        // TODO: handle intent, dimension, codimension properly
        AttributeKind::from_str(first_child).unwrap_or(AttributeKind::Unknown)
    }
}

#[derive(Clone, Debug)]
pub struct Attribute {
    kind: AttributeKind,
    location: TextRange,
}

impl Attribute {
    fn from_node(value: Node, src: &str) -> Self {
        Self {
            kind: AttributeKind::from_node(&value, src),
            location: value.textrange(),
        }
    }
}

/// A variable declaration line
#[derive(Clone, Debug)]
pub struct VariableDeclaration<'a> {
    type_: String,
    attributes: Vec<Attribute>,
    names: Vec<NameDecl<'a>>,
    node: Node<'a>,
}

impl<'a> VariableDeclaration<'a> {
    pub fn from_node(node: &Node<'a>, src: &str) -> Option<Self> {
        let type_ = node.child_by_field_name("type")?.to_text(src)?.to_string();

        let attributes = node
            .children_by_field_name("attribute", &mut node.walk())
            .map(|attr| Attribute::from_node(attr, src))
            .collect_vec();

        let names = node
            .children_by_field_name("declarator", &mut node.walk())
            .map(|decl| NameDecl::from_node(&decl, src))
            .collect_vec();

        Some(Self {
            type_,
            attributes,
            names,
            node: *node,
        })
    }

    pub fn type_(&self) -> &str {
        &self.type_
    }

    pub fn attributes(&self) -> &Vec<Attribute> {
        &self.attributes
    }
    pub fn names(&'_ self) -> &'_ Vec<NameDecl<'_>> {
        &self.names
    }
    pub fn textrange(&self) -> TextRange {
        self.node.textrange()
    }

    pub fn has_attribute(&self, attr: AttributeKind) -> bool {
        self.has_any_attributes(&[attr])
    }

    pub fn has_any_attributes(&self, attrs: &[AttributeKind]) -> bool {
        self.attributes.iter().any(|attr| attrs.contains(&attr.kind))
    }
}

pub fn get_name_node_of_declarator<'a>(node: &Node<'a>) -> Node<'a> {
    match node.kind() {
        "identifier" | "method_name" => *node,
        "sized_declarator" => node
            .named_child(0)
            .expect("sized_declarator should have named child"),
        "coarray_declarator" => {
            let child = node
                .named_child(0)
                .expect("coarray_declarator should have named child");
            match child.kind() {
                "identifier" => child,
                "sized_declarator" => child
                    .named_child(0)
                    .expect("sized_declarator should have named child"),
                _ => unreachable!("unexpected node type in coarray_declarator (found: {child:?})"),
            }
        }
        "init_declarator" | "pointer_init_declarator" | "data_declarator" => node
            .child_by_field_name("left")
            .expect("init/pointer_init/data_declarator should have left-hand side"),
        _ => unreachable!("unexpected node type in declarator ({node:?})"),
    }
}

#[derive(Clone, Debug)]
pub struct Variable<'a> {
    name: String,
    node: Node<'a>,
    decl: &'a VariableDeclaration<'a>,
}

impl<'a> Variable<'a> {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn textrange(&self) -> TextRange {
        self.node.textrange()
    }

    pub fn type_(&self) -> &str {
        self.decl.type_.as_str()
    }

    pub fn attributes(&self) -> &Vec<Attribute> {
        self.decl.attributes()
    }

    pub fn has_attribute(&self, attr: AttributeKind) -> bool {
        self.decl.has_attribute(attr)
    }

    pub fn has_any_attributes(&self, attrs: &[AttributeKind]) -> bool {
        self.decl.has_any_attributes(attrs)
    }
}

#[derive(Clone, Debug)]
pub enum Symbol<'a> {
    Variable(Node<'a>, usize),
}

/// A table of symbols in a given scope
///
/// Variables are not stored directly in the hashmap because we want to be able
/// to link a particular variable to its parent declaration statement, and
/// storing parent-child references is pretty annoying in rust. Instead, we
/// store the variable node + index into a vector of [`VariableDeclaration`],
/// and create a [`Variable`] on demand.
#[derive(Clone, Debug, Default)]
pub struct SymbolTable<'a> {
    inner: HashMap<String, Symbol<'a>>,
    decl_lines: Vec<VariableDeclaration<'a>>,
}

impl<'a> SymbolTable<'a> {
    pub fn new(scope: &Node<'a>, src: &str) -> Self {
        let mut new_table = Self::default();

        scope
            .named_children(&mut scope.walk())
            .filter(|child| child.kind() == "variable_declaration")
            .filter_map(|decl| VariableDeclaration::from_node(&decl, src))
            .for_each(|line| new_table.insert_from_decl_line(line));

        new_table
    }

    pub fn insert_from_decl_line(&mut self, decl: VariableDeclaration<'a>) {
        let index = self.decl_lines.len();
        for name in decl.names.iter() {
            self.inner
                .insert(name.name.clone(), Symbol::Variable(name.node, index));
        }
        self.decl_lines.push(decl);
    }

    pub fn get(&self, name: &str) -> Option<Variable<'_>> {
        // TODO(peter): avoid calling to_ascii_lowercase every time. Strong type?
        match self.inner.get(&name.to_ascii_lowercase()) {
            Some(Symbol::Variable(node, index)) => {
                let decl: &VariableDeclaration = &self.decl_lines[*index];
                Some(Variable {
                    name: name.to_string(),
                    node: *node,
                    decl,
                })
            }
            None => None,
        }
    }
}

/// A stack of [`SymbolTable`]
///
/// Symbols will be looked up starting from the most recent [`SymbolTable`] on
/// the stack.
#[derive(Clone, Debug, Default)]
pub struct SymbolTables<'a> {
    inner: Vec<SymbolTable<'a>>,
}

impl<'a> SymbolTables<'a> {
    pub fn push_table(&mut self, table: SymbolTable<'a>) {
        self.inner.push(table);
    }

    pub fn pop_table(&mut self) {
        self.inner.pop();
    }

    pub fn get(&'_ self, name: &str) -> Option<Variable<'_>> {
        // Check the most recently inserted table first
        for table in self.inner.iter().rev() {
            if let Some(node) = table.get(name) {
                return Some(node);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};
    use ruff_text_size::TextSize;
    use tree_sitter::Parser;

    #[test]
    fn new_symbol_table() -> Result<()> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;

        let code = r#"
program foo
  integer :: x, y(4), z = 5
  real, pointer :: a => null()
end program foo
"#;
        let tree = parser.parse(code, None).context("Failed to parse")?;
        let root = tree.root_node().child(0).context("Missing child")?;

        let first_decl_range = TextRange::new(TextSize::new(15), TextSize::new(40));
        let second_decl_range = TextRange::new(TextSize::new(43), TextSize::new(71));

        let symbol_table = SymbolTable::new(&root, code);

        let x = symbol_table.get("x");
        let y = symbol_table.get("y");
        let z = symbol_table.get("z");
        let a = symbol_table.get("a");
        assert!(x.is_some());
        let x = x.unwrap();
        assert_eq!(
            x.textrange(),
            TextRange::new(TextSize::new(26), TextSize::new(27))
        );
        assert_eq!(x.name, "x");
        assert_eq!(x.type_(), "integer");
        assert_eq!(x.decl.textrange(), first_decl_range);

        assert!(y.is_some());
        let y = y.unwrap();
        assert_eq!(
            y.textrange(),
            TextRange::new(TextSize::new(29), TextSize::new(33))
        );
        assert_eq!(y.name, "y");
        assert_eq!(y.decl.textrange(), first_decl_range);

        assert!(z.is_some());
        let z = z.unwrap();
        assert_eq!(
            z.textrange(),
            TextRange::new(TextSize::new(35), TextSize::new(40))
        );
        assert_eq!(z.name, "z");
        assert_eq!(z.decl.textrange(), first_decl_range);

        assert!(a.is_some());
        let a = a.unwrap();
        assert_eq!(
            a.textrange(),
            TextRange::new(TextSize::new(60), TextSize::new(71))
        );
        assert_eq!(a.name, "a");
        assert_eq!(a.type_(), "real");
        let a_attrs: Vec<&'static str> = a
            .attributes()
            .iter()
            .map(|attr| attr.kind.into())
            .collect_vec();
        assert_eq!(a_attrs, ["pointer"]);
        assert_eq!(a.decl.textrange(), second_decl_range);

        Ok(())
    }

    #[test]
    fn new_symbol_table_outer_scope_only() -> Result<()> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;

        let code = r#"
program foo
  integer :: x
  block
    real, pointer :: a => null()
  end block
end program foo
"#;
        let tree = parser.parse(code, None).context("Failed to parse")?;
        let root = tree.root_node().child(0).context("Missing child")?;

        let symbol_table = SymbolTable::new(&root, code);

        assert!(symbol_table.get("x").is_some());
        assert!(symbol_table.get("a").is_none());

        Ok(())
    }

    #[test]
    fn symbol_table_get_case_insensitive() -> Result<()> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;

        let code = r#"
program foo
  integer :: x, y(4), z = 5
  real, pointer :: a => null()
end program foo
"#;
        let tree = parser.parse(code, None).context("Failed to parse")?;
        let root = tree.root_node().child(0).context("Missing child")?;

        let symbol_table = SymbolTable::new(&root, code);
        let x = symbol_table.get("X");
        assert!(x.is_some());
        assert_eq!(
            x.unwrap().textrange(),
            TextRange::new(TextSize::new(26), TextSize::new(27))
        );

        Ok(())
    }

    #[test]
    fn symbol_table_get_outer_scope() -> Result<()> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;

        let code = r#"
program foo
  integer :: x
  block
    real, pointer :: a => null()
  end block
end program foo
"#;
        let tree = parser.parse(code, None).context("Failed to parse")?;
        let root = tree.root_node().child(0).context("Missing child")?;

        let mut symbol_table = SymbolTables::default();
        symbol_table.push_table(SymbolTable::new(&root, code));

        let block = root
            .child_with_name("block_construct")
            .context("Missing block")?;
        symbol_table.push_table(SymbolTable::new(&block, code));

        let x = symbol_table.get("X");
        assert!(x.is_some());
        assert_eq!(
            x.unwrap().textrange(),
            TextRange::new(TextSize::new(26), TextSize::new(27))
        );

        let a = symbol_table.get("a");
        assert!(a.is_some());
        assert_eq!(
            a.unwrap().textrange(),
            TextRange::new(TextSize::new(57), TextSize::new(68))
        );

        symbol_table.pop_table();
        assert!(symbol_table.get("a").is_none());

        Ok(())
    }

    #[test]
    fn attribute_intent() -> Result<()> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;

        let code = r#"
subroutine foo(x, y)
  integer, dimension(:, :), intent(in) :: x
  integer, dimension(:, :), intent(  in  out) :: y
end subroutine foo
"#;
        let tree = parser.parse(code, None).context("Failed to parse")?;
        let root = tree.root_node().child(0).context("Missing child")?;

        let mut symbol_table = SymbolTables::default();
        symbol_table.push_table(SymbolTable::new(&root, code));

        let x = symbol_table.get("x");
        assert!(x.is_some());
        let x = x.unwrap();
        assert!(x.has_attribute(AttributeKind::Dimension));
        assert!(x.has_attribute(AttributeKind::Intent));

        Ok(())
    }
}
