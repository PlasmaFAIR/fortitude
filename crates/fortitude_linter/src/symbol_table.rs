use std::{collections::HashMap, str::FromStr};

use anyhow::{Result, anyhow};
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

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn node(&self) -> Node<'_> {
        self.node
    }

    pub fn textrange(&self) -> TextRange {
        self.node.textrange()
    }
}

#[derive(Clone, Copy, Debug, EnumIs, PartialEq)]
pub enum ExtentSize<'a> {
    Expression(Node<'a>),
    AssumedSize,
}

impl<'a> ExtentSize<'a> {
    pub fn from_node(node: Node<'a>) -> Self {
        if node.kind() == "assumed_size" {
            Self::AssumedSize
        } else {
            Self::Expression(node)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Extent<'a> {
    start: Option<Node<'a>>,
    stop: Option<ExtentSize<'a>>,
    stride: Option<Node<'a>>,
}

impl<'a> Extent<'a> {
    pub fn try_from_node(node: Node<'a>) -> Result<Self> {
        if node.kind() != "extent_specifier" {
            return Err(anyhow!(
                "expected 'extent_specifier', got '{}'",
                node.kind()
            ));
        }

        let cursor = &mut node.walk();
        let mut iter = node.named_children(cursor);

        Ok(Self {
            start: iter.next(),
            stop: iter.next().map(ExtentSize::from_node),
            stride: iter.next(),
        })
    }
}

/// One rank of a dimension's array-spec
#[derive(Clone, Copy, Debug, EnumIs, PartialEq)]
pub enum DimensionArraySpec<'a> {
    Expression(Node<'a>),
    Extent(Extent<'a>),
    AssumedSize,
    AssumedRank,
    MultipleSubscript(Node<'a>),
    MultipleSubscriptTriplet(Extent<'a>),
}

impl<'a> DimensionArraySpec<'a> {
    pub fn try_from_node(node: Node<'a>) -> Result<Self> {
        match node.kind() {
            "extent_specifier" => Ok(Self::Extent(Extent::try_from_node(node)?)),
            "assumed_size" => Ok(Self::AssumedSize),
            "assumed_rank" => Ok(Self::AssumedRank),
            "multiple_subscript" => Ok(Self::MultipleSubscript(node)),
            "multiple_subscript_triplet" => {
                Ok(Self::MultipleSubscriptTriplet(Extent::try_from_node(node)?))
            }
            _ => Ok(Self::Expression(node)),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Dimension<'a> {
    pub ranks: Vec<DimensionArraySpec<'a>>,
}

impl<'a> Dimension<'a> {
    pub fn try_from_node(node: Node<'a>) -> Result<Self> {
        if !matches!(node.kind(), "argument_list" | "size") {
            return Err(anyhow!(
                "Dimension::try_from_node called with wrong node kind (expected 'argument_list/size', got '{}'",
                node.kind()
            ));
        }

        let ranks: Result<Vec<_>> = node
            .named_children(&mut node.walk())
            .map(DimensionArraySpec::try_from_node)
            .collect();

        Ok(Self { ranks: ranks? })
    }
}

#[derive(Clone, Copy, Debug, Default, EnumIs, IntoStaticStr, PartialEq)]
pub enum Intent {
    In,
    Out,
    #[default]
    InOut,
}

impl Intent {
    pub fn from_node(node: &Node) -> Self {
        let children = node
            .children(&mut node.walk())
            .map(|child| child.kind())
            .collect_vec();
        if children.contains(&"inout") || (children.contains(&"in") && children.contains(&"out")) {
            Self::InOut
        } else if children.contains(&"in") {
            Self::In
        } else {
            Self::Out
        }
    }
}

#[derive(Clone, Debug, EnumIs, EnumString, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum AttributeKind<'a> {
    Abstract,
    Allocatable,
    Asynchronous,
    Automatic,
    Codimension,
    Dimension(Dimension<'a>),
    Constant,
    Continguous,
    Device,
    External,
    Intent(Intent),
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
    // We shouldn't actually need this, it indicates there was a syntax error
    Unknown,
}

impl<'a> AttributeKind<'a> {
    pub fn from_node(value: &Node<'a>) -> Self {
        let first_child = value.child(0).unwrap().kind();
        // TODO: handle codimension properly
        let attr = AttributeKind::from_str(first_child).unwrap_or(AttributeKind::Unknown);

        match attr {
            AttributeKind::Intent(_) => AttributeKind::Intent(Intent::from_node(value)),
            AttributeKind::Dimension(_) => {
                AttributeKind::Dimension(Dimension::try_from_node(value.child(1).unwrap()).unwrap())
            }
            _ => attr,
        }
    }
}

/// A variable attribute and where it is
#[derive(Clone, Debug)]
pub struct Attribute<'a> {
    kind: AttributeKind<'a>,
    #[allow(dead_code)]
    location: TextRange,
}

impl<'a> Attribute<'a> {
    pub fn from_node(value: Node<'a>) -> Self {
        Self {
            kind: AttributeKind::from_node(&value),
            location: value.textrange(),
        }
    }

    pub fn kind(&'_ self) -> &'_ AttributeKind<'_> {
        &self.kind
    }
}

#[derive(Clone, Debug, EnumIs)]
pub enum Type {
    Intrinsic(String),
    Derived(String),
    Procedure(String),
    Declared(String),
}

impl Type {
    pub fn from_node(node: &Node, src: &str) -> Option<Self> {
        let kind = node.kind();
        let name = node.to_text(src)?.to_string();
        match kind {
            "intrinsic_type" => Some(Type::Intrinsic(name)),
            "derived_type" => Some(Type::Derived(name)),
            "procedure" => Some(Type::Procedure(name)),
            "declared_type" => Some(Type::Declared(name)),
            _ => unreachable!("unexpected 'type' kind '{kind}'"),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Intrinsic(name) => name.as_str(),
            Self::Derived(name) => name.as_str(),
            Self::Procedure(name) => name.as_str(),
            Self::Declared(name) => name.as_str(),
        }
    }
}

/// A variable declaration line
#[derive(Clone, Debug)]
pub struct VariableDeclaration<'a> {
    type_: Type,
    attributes: Vec<Attribute<'a>>,
    names: Vec<NameDecl<'a>>,
    node: Node<'a>,
}

impl<'a> VariableDeclaration<'a> {
    pub fn from_node(node: &Node<'a>, src: &str) -> Option<Self> {
        let type_ = Type::from_node(&node.child_by_field_name("type")?, src)?;

        let attributes = node
            .children_by_field_name("attribute", &mut node.walk())
            .map(Attribute::from_node)
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

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn attributes(&self) -> &Vec<Attribute<'_>> {
        &self.attributes
    }

    pub fn names(&'_ self) -> &'_ Vec<NameDecl<'_>> {
        &self.names
    }

    pub fn node(&self) -> Node<'_> {
        self.node
    }

    pub fn textrange(&self) -> TextRange {
        self.node.textrange()
    }

    pub fn has_attribute(&self, attr: AttributeKind) -> bool {
        self.has_any_attributes(&[attr])
    }

    pub fn has_any_attributes(&self, attrs: &[AttributeKind]) -> bool {
        self.attributes
            .iter()
            .any(|attr| attrs.contains(&attr.kind))
    }
}

/// Returns the tree-sitter node corresponding to the actual name of a
/// declarator node, and not, say, the initialiser
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

/// A single Fortran variable
#[derive(Clone, Debug)]
pub struct Variable<'a> {
    name: String,
    node: Node<'a>,
    /// Reference to the statement in which the variable is declared
    decl: &'a VariableDeclaration<'a>,
}

impl<'a> Variable<'a> {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn node(&self) -> Node<'_> {
        self.node
    }

    pub fn decl_statement(&'a self) -> &'a VariableDeclaration<'a> {
        self.decl
    }

    pub fn textrange(&self) -> TextRange {
        self.node.textrange()
    }

    pub fn type_(&self) -> &Type {
        self.decl.type_()
    }

    pub fn attributes(&self) -> &Vec<Attribute<'_>> {
        self.decl.attributes()
    }

    pub fn has_attribute(&self, attr: AttributeKind) -> bool {
        self.decl.has_attribute(attr)
    }

    pub fn has_any_attributes(&self, attrs: &[AttributeKind]) -> bool {
        self.decl.has_any_attributes(attrs)
    }
}

/// A named symbol
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
    /// Create a new [`SymbolTable`] for a node which is a scope (that is,
    /// contains variable declarations)
    pub fn new(scope: &Node<'a>, src: &str) -> Self {
        let mut new_table = Self::default();

        scope
            .named_children(&mut scope.walk())
            .filter(|child| child.kind() == "variable_declaration")
            .filter_map(|decl| VariableDeclaration::from_node(&decl, src))
            .for_each(|line| new_table.insert_from_decl_line(line));

        new_table
    }

    /// Insert all symbols found in a single variable declaration statement
    pub fn insert_from_decl_line(&mut self, decl: VariableDeclaration<'a>) {
        let index = self.decl_lines.len();
        for name in decl.names.iter() {
            self.inner.insert(
                name.name.to_ascii_lowercase(),
                Symbol::Variable(name.node, index),
            );
        }
        self.decl_lines.push(decl);
    }

    /// Return the symbol with the given name if it exists
    pub fn get(&self, name: &str) -> Option<Variable<'_>> {
        // TODO(peter): avoid calling to_ascii_lowercase every time. Strong type?
        let name = name.to_ascii_lowercase();
        match self.inner.get(&name) {
            Some(Symbol::Variable(node, index)) => {
                let decl: &VariableDeclaration = &self.decl_lines[*index];
                Some(Variable {
                    name,
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

    /// Return the symbol with the given name if it exists
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
  integer :: x, Y(4), z = 5
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
        let z = symbol_table.get("Z");
        let a = symbol_table.get("a");
        assert!(x.is_some());
        let x = x.unwrap();
        assert_eq!(
            x.textrange(),
            TextRange::new(TextSize::new(26), TextSize::new(27))
        );
        assert_eq!(x.name, "x");
        assert_eq!(x.type_().as_str(), "integer");
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
        assert_eq!(a.type_().as_str(), "real");
        let a_attrs: Vec<&'static str> = a
            .attributes()
            .iter()
            .map(|attr| attr.kind().into())
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
  integer, dimension(0:, *), intent(  in  out) :: y
end subroutine foo
"#;
        let tree = parser.parse(code, None).context("Failed to parse")?;
        let root = tree.root_node().child(0).context("Missing child")?;

        let mut symbol_table = SymbolTables::default();
        symbol_table.push_table(SymbolTable::new(&root, code));

        let x = symbol_table.get("x");
        assert!(x.is_some());
        let x = x.unwrap();
        assert!(x.attributes().iter().any(|attr| attr.kind().is_dimension()));
        assert!(x.has_attribute(AttributeKind::Intent(Intent::In)));

        let y = symbol_table.get("y");
        assert!(y.is_some());
        let y = y.unwrap();
        let y_dim = y
            .attributes()
            .iter()
            .find(|attr| attr.kind().is_dimension());
        assert!(y_dim.is_some());
        if let AttributeKind::Dimension(dim) = y_dim.unwrap().kind() {
            assert_eq!(dim.ranks.len(), 2);
            assert!(dim.ranks[0].is_extent());
            assert!(dim.ranks[1].is_assumed_size());
        }
        assert!(y.has_attribute(AttributeKind::Intent(Intent::InOut)));

        Ok(())
    }
}
