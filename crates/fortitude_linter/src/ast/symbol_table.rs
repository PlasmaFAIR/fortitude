use std::{collections::HashMap, rc::Rc};

use itertools::Itertools;
use strum_macros::EnumIs;
use tree_sitter::Node;

use crate::{ast::types::ProcedureKind, traits::HasNode};

use super::{
    FortitudeNode,
    types::{
        HasName, Module, Name, Procedure, Program, TypeDefinition, UseStatement, UsedItem,
        Variable, VariableDeclaration,
    },
};

pub const BEGIN_SCOPE_NODES: &[&str] = &[
    "translation_unit",
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
#[derive(Clone, Debug, EnumIs)]
pub enum Symbol<'a> {
    Variable(Variable<'a>),
    Function(Procedure<'a>),
    Subroutine(Procedure<'a>),
    Type(TypeDefinition<'a>),
    Module(Module<'a>),
    Program(Program<'a>),
    UsedItem(UsedItem<'a>),
}

impl<'a> Symbol<'a> {
    pub fn name(&self) -> &Name<'a> {
        match self {
            Self::Variable(var) => var.name(),
            Self::Function(proc) | Self::Subroutine(proc) => proc.name(),
            Self::Type(typedef) => typedef.name(),
            Self::Module(module) => module.name(),
            Self::Program(program) => program.name(),
            Self::UsedItem(item) => item.name(),
        }
    }
}

impl<'a> HasNode<'a> for Symbol<'a> {
    fn node(&self) -> &Node<'a> {
        match self {
            Self::Variable(var) => var.node(),
            Self::Function(proc) | Self::Subroutine(proc) => proc.node(),
            Self::Type(typedef) => typedef.node(),
            Self::Module(module) => module.node(),
            Self::Program(program) => program.node(),
            Self::UsedItem(item) => item.node(),
        }
    }
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
    decl_lines: Vec<Rc<VariableDeclaration<'a>>>,
    use_statements: Vec<Rc<UseStatement<'a>>>,
}

impl<'a> SymbolTable<'a> {
    /// Create a new [`SymbolTable`] for a node which is a scope (that is,
    /// contains variable declarations)
    pub fn new(scope: &Node<'a>, src: &str) -> Self {
        let mut new_table = Self::default();

        // If this is a procedure, collect a list of dummy arg names
        let dummy_vars = if matches!(scope.kind(), "function" | "subroutine") {
            scope
                .named_child(0)
                .expect("First child must be function/subroutine statement")
                .child_by_field_name("parameters")
                .map_or(vec![], |params| {
                    params
                        .named_children(&mut params.walk())
                        .filter(|child| child.kind() == "identifier")
                        .map(|child| {
                            child
                                .to_text(src)
                                .unwrap_or("<unknown>")
                                .to_ascii_lowercase()
                        })
                        .collect_vec()
                })
        } else {
            vec![]
        };

        scope
            .named_children(&mut scope.walk())
            .filter(|child| child.kind() == "use_statement")
            .filter_map(|stmt| UseStatement::try_from_node(&stmt, src).ok())
            .for_each(|stmt| new_table.insert_from_use_statement(stmt, src));

        scope
            .named_children(&mut scope.walk())
            .filter(|child| child.kind() == "variable_declaration")
            .filter_map(|decl| VariableDeclaration::try_from_node(&decl, src).ok())
            .for_each(|line| new_table.insert_from_decl_line(line, &dummy_vars));

        // The `function` statement itself _may_ also be the declaration line if
        // it has a type as a procedure attribute. If it doesn't, then it will
        // either have an explicit decl line, which is handled above, or it's
        // implicitly typed, which we don't currently handle here at all
        if scope.kind() == "function" {
            let stmt = scope
                .child(0)
                .expect("`function` must have `function_statement` as zeroth child");
            if let Ok(decl) = VariableDeclaration::try_from_fn_stmt(&stmt, src) {
                let name = decl
                    .names()
                    .first()
                    .expect("Function must have a name")
                    .name()
                    .as_str()
                    .to_ascii_lowercase();
                new_table.insert_from_decl_line(decl, &[name]);
            }
        }

        // Add procedure definitions
        if let Some(procs) = scope.child_with_name("internal_procedures") {
            procs
                .named_children(&mut procs.walk())
                .filter_map(|proc| match proc.kind() {
                    "function" | "subroutine" => Procedure::try_from_node(&proc, src).ok(),
                    _ => None,
                })
                .for_each(|proc| {
                    let name = proc.name();
                    match proc.kind() {
                        ProcedureKind::Function => new_table
                            .inner
                            .insert(name.to_string(), Symbol::Function(proc)),
                        ProcedureKind::Subroutine => new_table
                            .inner
                            .insert(name.to_string(), Symbol::Subroutine(proc)),
                    };
                })
        }

        // Add modules, programs, and derived type definitions
        scope
            .named_children(&mut scope.walk())
            .filter_map(|child| match child.kind() {
                "derived_type_definition" => TypeDefinition::try_from_node(&child, src)
                    .ok()
                    .map(Symbol::Type),
                "module" => Module::try_from_node(&child, src).ok().map(Symbol::Module),
                "program" => Program::try_from_node(&child, src)
                    .ok()
                    .map(Symbol::Program),
                _ => None,
            })
            .for_each(|symbol| {
                let name = symbol.name();
                new_table.inner.insert(name.to_string(), symbol);
            });

        new_table
    }

    /// Insert all symbols found in a single variable declaration statement
    pub fn insert_from_decl_line(&mut self, decl: VariableDeclaration<'a>, dummy_vars: &[String]) {
        let decl = Rc::new(decl);
        for name in decl.names().iter() {
            let name_lower = name.name().as_str().to_ascii_lowercase();
            let is_dummy_var = dummy_vars.contains(&name_lower);
            self.inner.insert(
                name_lower,
                Symbol::Variable(Variable::new(name.clone(), is_dummy_var, decl.clone())),
            );
        }
        self.decl_lines.push(decl);
    }

    /// Insert all symbols found in a single use statement
    pub fn insert_from_use_statement(&mut self, stmt: UseStatement<'a>, src: &str) {
        let stmt = Rc::new(stmt);
        if let Some(items) = stmt.included_items() {
            for item in items.named_children(&mut items.walk()) {
                // Other nodes such as comments can be found in the list
                if let Some(item) = UsedItem::try_from_node(item, src, stmt.clone()) {
                    let symbol = Symbol::UsedItem(item);
                    let name = symbol.name().as_str().to_ascii_lowercase();
                    self.inner.insert(name, symbol);
                }
            }
        }
        self.use_statements.push(stmt);
    }

    /// Return the symbol with the given name if it exists
    pub fn get(&self, name: &str) -> Option<&Symbol<'_>> {
        self.inner.get(name)
    }

    /// Iterator over the variable declaration lines
    pub fn iter_decl_lines(&self) -> impl Iterator<Item = &Rc<VariableDeclaration<'a>>> {
        self.decl_lines.iter()
    }

    /// Iterator over the use statements
    pub fn iter_use_statements(&self) -> impl Iterator<Item = &Rc<UseStatement<'a>>> {
        self.use_statements.iter()
    }

    /// Iterator over symbols in this scope
    pub fn iter_symbols(&self) -> impl Iterator<Item = &Symbol<'_>> {
        self.inner.values()
    }

    /// Iterator over names in this scope
    pub fn iter_names(&self) -> impl Iterator<Item = &String> {
        self.inner.keys()
    }

    /// Iterator over symbol names and values
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Symbol<'_>)> {
        self.inner.iter()
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

    pub fn pop_table(&mut self) -> Option<SymbolTable<'a>> {
        self.inner.pop()
    }

    /// Return the symbol with the given name if it exists and is a variable
    pub fn get_var(&'_ self, name: &str) -> Option<&Variable<'_>> {
        let name = name.to_ascii_lowercase();

        // Check the most recently inserted table first
        for table in self.inner.iter().rev() {
            match table.get(&name) {
                Some(Symbol::Variable(var)) => {
                    return Some(var);
                }
                // We found a name, but it isn't a variable and will shadow any
                // variables with the same name further up the stack, so quit
                // now so we don't find them
                Some(_) => {
                    return None;
                }
                // Nothing in this scope, keep looking
                None => (),
            }
        }
        None
    }

    /// Return the symbol with the given name if it exists
    pub fn get(&'_ self, name: &str) -> Option<&Symbol<'_>> {
        let name = name.to_ascii_lowercase();

        // Check the most recently inserted table first
        for table in self.inner.iter().rev() {
            if let Some(var) = table.get(&name) {
                return Some(var);
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

        let mut symbol_table = SymbolTables::default();
        symbol_table.push_table(SymbolTable::new(&root, code));

        let x = symbol_table.get_var("x");
        let y = symbol_table.get_var("y");
        let z = symbol_table.get_var("Z");
        let a = symbol_table.get_var("a");
        assert!(x.is_some());
        let x = x.unwrap();
        assert_eq!(
            x.textrange(),
            TextRange::new(TextSize::new(26), TextSize::new(27))
        );
        assert_eq!(x.name().as_str(), "x");
        assert_eq!(x.type_().as_str(), "integer");
        assert_eq!(x.decl_statement().textrange(), first_decl_range);

        assert!(y.is_some());
        let y = y.unwrap();
        assert_eq!(
            y.textrange(),
            TextRange::new(TextSize::new(29), TextSize::new(33))
        );
        assert_eq!(y.name().as_str(), "Y");
        assert_eq!(y.decl_statement().textrange(), first_decl_range);

        assert!(z.is_some());
        let z = z.unwrap();
        assert_eq!(
            z.textrange(),
            TextRange::new(TextSize::new(35), TextSize::new(40))
        );
        assert_eq!(z.name().as_str(), "z");
        assert_eq!(z.decl_statement().textrange(), first_decl_range);

        assert!(a.is_some());
        let a = a.unwrap();
        assert_eq!(
            a.textrange(),
            TextRange::new(TextSize::new(60), TextSize::new(71))
        );
        assert_eq!(a.name().as_str(), "a");
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

        let mut symbol_table = SymbolTables::default();
        symbol_table.push_table(SymbolTable::new(&root, code));

        let x = symbol_table.get_var("X");
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

        let x = symbol_table.get_var("X");
        assert!(x.is_some());
        assert_eq!(
            x.unwrap().textrange(),
            TextRange::new(TextSize::new(26), TextSize::new(27))
        );

        let a = symbol_table.get_var("a");
        assert!(a.is_some());
        assert_eq!(
            a.unwrap().textrange(),
            TextRange::new(TextSize::new(57), TextSize::new(68))
        );

        symbol_table.pop_table();
        assert!(symbol_table.get_var("a").is_none());

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

        let x = symbol_table.get_var("x");
        assert!(x.is_some());
        let x = x.unwrap();
        assert!(x.is_dummy_var());
        assert!(x.attributes().iter().any(|attr| attr.kind().is_dimension()));
        assert!(x.has_attribute(AttributeKind::Intent(Intent::In)));

        let y = symbol_table.get_var("y");
        assert!(y.is_some());
        let y = y.unwrap();
        assert!(y.is_dummy_var());
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

    #[test]
    fn function_variable_with_attributes() -> Result<()> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;

        let code = r#"
function foo(x)
  integer, intent(in) :: x
  integer, allocatable, dimension(:) :: foo
end function foo
"#;
        let tree = parser.parse(code, None).context("Failed to parse")?;
        let root = tree.root_node().child(0).context("Missing child")?;

        let mut symbol_table = SymbolTables::default();
        symbol_table.push_table(SymbolTable::new(&root, code));

        let foo = symbol_table.get_var("foo");
        assert!(foo.is_some());
        let foo = foo.unwrap();
        assert!(!foo.is_dummy_var());
        assert!(foo.has_attribute(AttributeKind::Allocatable));
        assert!(
            foo.attributes()
                .iter()
                .any(|attr| attr.kind().is_dimension())
        );

        Ok(())
    }

    #[test]
    fn function_variable() -> Result<()> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;

        let code = r#"
integer function foo(x)
  integer, intent(in) :: x
end function foo
"#;
        let tree = parser.parse(code, None).context("Failed to parse")?;
        let root = tree.root_node().child(0).context("Missing child")?;

        let mut symbol_table = SymbolTables::default();
        symbol_table.push_table(SymbolTable::new(&root, code));

        let foo = symbol_table.get_var("foo");
        assert!(foo.is_some());
        let foo = foo.unwrap();
        assert!(foo.is_dummy_var());
        assert!(foo.type_().is_intrinsic());

        Ok(())
    }

    #[test]
    fn function_result() -> Result<()> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;

        let code = r#"
integer function foo(x) result(y)
  integer, intent(in) :: x
end function foo
"#;
        let tree = parser.parse(code, None).context("Failed to parse")?;
        let root = tree.root_node().child(0).context("Missing child")?;

        let mut symbol_table = SymbolTables::default();
        symbol_table.push_table(SymbolTable::new(&root, code));

        let y = symbol_table.get_var("y");
        assert!(y.is_some());
        let y = y.unwrap();
        assert!(y.is_dummy_var());
        assert!(y.type_().is_intrinsic());

        Ok(())
    }

    #[test]
    fn internal_procedures() -> Result<()> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;

        let code = r#"
program foo
contains
  integer function bar(x) result(y)
    integer, intent(in) :: x
  end function bar

  subroutine zing()
  end subroutine zing
end program foo
"#;
        let tree = parser.parse(code, None).context("Failed to parse")?;
        let root = tree.root_node().child(0).context("Missing child")?;

        let mut symbol_table = SymbolTables::default();
        symbol_table.push_table(SymbolTable::new(&root, code));

        let bar = symbol_table.get("bar");
        assert!(bar.is_some());
        let bar = bar.unwrap();
        assert!(bar.is_function());

        let zing = symbol_table.get("zing");
        assert!(zing.is_some());
        let zing = zing.unwrap();
        assert!(zing.is_subroutine());

        Ok(())
    }

    #[test]
    fn used_items() -> Result<()> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;

        let code = r#"
program foo
  use, intrinsic :: iso_fortran_env, only: i8 => int8, real32
  use :: my_module, only: my_subroutine
  use another_module
end program foo
"#;
        let tree = parser.parse(code, None).context("Failed to parse")?;
        let root = tree.root_node().child(0).context("Missing child")?;

        let mut symbol_table = SymbolTables::default();
        symbol_table.push_table(SymbolTable::new(&root, code));

        let i8 = symbol_table.get("i8");
        assert!(i8.is_some());
        let i8 = i8.unwrap();
        assert!(i8.is_used_item());
        assert!(i8.name().as_str() == "i8");
        let Symbol::UsedItem(i8) = i8 else {
            panic!("Expected i8 to be a UsedItem");
        };
        assert!(i8.alias_of().is_some());
        let i8_alias_of = i8.alias_of().unwrap();
        assert!(i8_alias_of.as_str() == "int8");
        let i8_module = i8.module_name();
        assert!(i8_module.as_str() == "iso_fortran_env");

        let real32 = symbol_table.get("real32");
        assert!(real32.is_some());
        let real32 = real32.unwrap();
        assert!(real32.is_used_item());
        assert!(real32.name().as_str() == "real32");
        let Symbol::UsedItem(real32) = real32 else {
            panic!("Expected real32 to be a UsedItem");
        };
        assert!(real32.alias_of().is_none());
        let real32_module = real32.module_name();
        assert!(real32_module.as_str() == "iso_fortran_env");

        let iso_fortran_env = real32.decl_statement();
        assert!(iso_fortran_env.name().as_str() == "iso_fortran_env");
        assert!(iso_fortran_env.is_intrinsic());
        assert!(iso_fortran_env.has_colon());
        assert!(iso_fortran_env.has_only());

        let my_subroutine = symbol_table.get("my_subroutine");
        assert!(my_subroutine.is_some());
        let my_subroutine = my_subroutine.unwrap();
        assert!(my_subroutine.is_used_item());
        let Symbol::UsedItem(my_subroutine) = my_subroutine else {
            panic!("Expected my_subroutine to be a UsedItem");
        };
        assert!(my_subroutine.alias_of().is_none());
        let my_subroutine_module = my_subroutine.module_name();
        assert!(my_subroutine_module.as_str() == "my_module");

        let my_module = my_subroutine.decl_statement();
        assert!(my_module.name().as_str() == "my_module");
        assert!(!my_module.is_intrinsic());
        assert!(my_module.has_colon());
        assert!(my_module.has_only());

        let symbol_table = symbol_table.pop_table().unwrap();
        let another_module = symbol_table.iter_use_statements().last().unwrap();
        assert!(another_module.name().as_str() == "another_module");
        assert!(!another_module.is_intrinsic());
        assert!(!another_module.has_colon());
        assert!(!another_module.has_only());

        Ok(())
    }

    #[test]
    fn all_symbols() -> Result<()> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;

        let code = r#"
program foo
  use :: some_module, only: p, q => r
  integer :: a, b
contains
  integer function bar(x) result(y)
    integer, intent(in) :: x
  end function bar

  subroutine zing()
  end subroutine zing
end program foo
"#;
        let tree = parser.parse(code, None).context("Failed to parse")?;
        let root = tree.root_node().child(0).context("Missing child")?;

        let symbol_table = SymbolTable::new(&root, code);

        let count = symbol_table.iter_symbols().count();
        assert_eq!(count, 6);

        Ok(())
    }
}
