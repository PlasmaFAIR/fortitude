use std::collections::HashMap;

use tree_sitter::Node;

use crate::traits::HasNode;

use super::types::{Variable, VariableDeclaration};

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
            .filter_map(|decl| VariableDeclaration::try_from_node(&decl, src).ok())
            .for_each(|line| new_table.insert_from_decl_line(line));

        new_table
    }

    /// Insert all symbols found in a single variable declaration statement
    pub fn insert_from_decl_line(&mut self, decl: VariableDeclaration<'a>) {
        let index = self.decl_lines.len();
        for name in decl.names().iter() {
            self.inner.insert(
                name.name().to_ascii_lowercase(),
                Symbol::Variable(*name.node(), index),
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
                Some(Variable::new(name, *node, decl))
            }
            None => None,
        }
    }

    /// Iterator over the variable declaration lines
    pub fn iter_decl_lines(&self) -> impl Iterator<Item = &VariableDeclaration<'a>> {
        self.decl_lines.iter()
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
    use crate::{
        ast::{
            FortitudeNode,
            types::{AttributeKind, Intent},
        },
        traits::TextRanged,
    };
    use anyhow::{Context, Result};
    use itertools::Itertools;
    use ruff_text_size::{TextRange, TextSize};
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
        assert_eq!(x.name(), "x");
        assert_eq!(x.type_().as_str(), "integer");
        assert_eq!(x.decl_statement().textrange(), first_decl_range);

        assert!(y.is_some());
        let y = y.unwrap();
        assert_eq!(
            y.textrange(),
            TextRange::new(TextSize::new(29), TextSize::new(33))
        );
        assert_eq!(y.name(), "y");
        assert_eq!(y.decl_statement().textrange(), first_decl_range);

        assert!(z.is_some());
        let z = z.unwrap();
        assert_eq!(
            z.textrange(),
            TextRange::new(TextSize::new(35), TextSize::new(40))
        );
        assert_eq!(z.name(), "z");
        assert_eq!(z.decl_statement().textrange(), first_decl_range);

        assert!(a.is_some());
        let a = a.unwrap();
        assert_eq!(
            a.textrange(),
            TextRange::new(TextSize::new(60), TextSize::new(71))
        );
        assert_eq!(a.name(), "a");
        assert_eq!(a.type_().as_str(), "real");
        let a_attrs: Vec<&'static str> = a
            .attributes()
            .iter()
            .map(|attr| attr.kind().into())
            .collect_vec();
        assert_eq!(a_attrs, ["pointer"]);
        assert_eq!(a.decl_statement().textrange(), second_decl_range);

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
