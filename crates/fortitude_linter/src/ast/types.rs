//! Strong types for working with the tree-sitter AST

use std::{rc::Rc, str::FromStr};

use anyhow::{Context, Result, anyhow};
use bitflags::bitflags;
use fortitude_macros::{HasName, HasNode};
use itertools::Itertools;
use ruff_source_file::SourceFile;
use strum_macros::{Display, EnumIs, EnumString, IntoStaticStr};
use tree_sitter::Node;

use crate::{ast::FortitudeNode, traits::HasNode};

#[derive(Clone, Debug)]
pub struct ParameterStatement<'a> {
    pub name: String,
    pub expression: String,
    pub node: Node<'a>,
}

impl<'a> ParameterStatement<'a> {
    pub fn try_from_node(node: Node<'a>, src: &str) -> Result<Self> {
        Ok(Self {
            name: node
                .child_with_name("identifier")
                .context("expected identifier in 'parameter_statement'")?
                .to_text(src)
                .context("expected text")?
                .to_string(),
            expression: node
                .child(2)
                .context("expected expression in 'parameter_statement'")?
                .to_text(src)
                .context("expected text")?
                .to_string(),
            node,
        })
    }
}

/// The name node of a variable declaration, procedure declaration, type
/// definition, etc.
#[derive(Clone, Debug, HasNode)]
pub struct Name<'a> {
    name: String,
    node: Node<'a>,
}

impl<'a> Name<'a> {
    pub fn from_node(node: &Node<'a>, src: &str) -> Self {
        let node = get_name_node_of_declarator(node);
        Self {
            name: node.to_text(src).unwrap_or("<unknown>").to_string(),
            node,
        }
    }

    pub fn as_str(&self) -> &str {
        self.name.as_str()
    }
}

impl std::fmt::Display for Name<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub trait HasName<'a> {
    fn name(&self) -> &Name<'a>;
}

impl<'a, T> HasName<'a> for &'a T
where
    T: HasName<'a>,
{
    fn name(&self) -> &Name<'a> {
        T::name(self)
    }
}

impl<'a> HasName<'a> for &'a Name<'a> {
    fn name(&self) -> &Name<'a> {
        self
    }
}

/// A declaration of a single variable, including sizes, assignments, etc.
#[derive(Clone, Debug, HasName, HasNode)]
pub struct NameDecl<'a> {
    name: Name<'a>,
    node: Node<'a>,
}

impl<'a> NameDecl<'a> {
    pub fn from_node(node: &Node<'a>, src: &str) -> Self {
        Self {
            name: Name::from_node(node, src),
            node: *node,
        }
    }

    /// Get size node, if there is one
    pub fn size(&'a self) -> Option<Node<'a>> {
        get_size_node_of_declarator(&self.node)
    }

    /// Get initialiser node, if there is one
    pub fn init(&'a self) -> Option<Node<'a>> {
        get_init_node_of_declarator(&self.node)
    }
}

#[derive(Clone, Copy, Debug, EnumIs, PartialEq)]
pub enum ExtentSize<'a> {
    Expression(Node<'a>),
    AssumedSize(Node<'a>),
}

impl<'a> ExtentSize<'a> {
    pub fn from_node(node: Node<'a>) -> Self {
        if node.kind() == "assumed_size" {
            Self::AssumedSize(node)
        } else {
            Self::Expression(node)
        }
    }
}

impl<'a> HasNode<'a> for ExtentSize<'a> {
    fn node(&self) -> &Node<'a> {
        match self {
            Self::Expression(node) => node,
            Self::AssumedSize(node) => node,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, HasNode)]
pub struct Extent<'a> {
    start: Option<Node<'a>>,
    stop: Option<ExtentSize<'a>>,
    stride: Option<Node<'a>>,
    node: Node<'a>,
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
            node,
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
}

impl<'a> AttributeKind<'a> {
    pub fn try_from_node(value: &Node<'a>) -> Result<Self> {
        let first_child = value.child(0).unwrap().kind();
        // TODO: handle codimension properly
        let attr = AttributeKind::from_str(first_child)?;

        match attr {
            AttributeKind::Intent(_) => Ok(AttributeKind::Intent(Intent::from_node(value))),
            AttributeKind::Dimension(_) => Ok(AttributeKind::Dimension(Dimension::try_from_node(
                value
                    .child(1)
                    .context("expected more than one child for 'dimension'")?,
            )?)),
            _ => Ok(attr),
        }
    }
}

/// A variable attribute and where it is
#[derive(Clone, Debug, HasNode)]
pub struct Attribute<'a> {
    kind: AttributeKind<'a>,
    node: Node<'a>,
}

impl<'a> Attribute<'a> {
    pub fn try_from_node(node: Node<'a>) -> Result<Self> {
        Ok(Self {
            kind: AttributeKind::try_from_node(&node)?,
            node,
        })
    }

    pub fn kind(&'_ self) -> &'_ AttributeKind<'_> {
        &self.kind
    }
}

#[derive(Clone, Debug)]
pub struct TypeInner<'a> {
    node: Node<'a>,
    name: String,
}

#[derive(Clone, Debug, EnumIs)]
pub enum Type<'a> {
    Intrinsic(TypeInner<'a>),
    Derived(TypeInner<'a>),
    Procedure(TypeInner<'a>),
    Declared(TypeInner<'a>),
}

impl<'a> Type<'a> {
    pub fn try_from_node(node: Node<'a>, src: &str) -> Result<Self> {
        let kind = node.kind();
        let name = node.to_text(src).context("expected text")?.to_string();
        match kind {
            "intrinsic_type" => Ok(Type::Intrinsic(TypeInner { node, name })),
            "derived_type" => Ok(Type::Derived(TypeInner { node, name })),
            "procedure" => Ok(Type::Procedure(TypeInner { node, name })),
            "declared_type" => Ok(Type::Declared(TypeInner { node, name })),
            _ => Err(anyhow!("unexpected 'type' kind '{kind}'")),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Intrinsic(TypeInner { name, .. }) => name.as_str(),
            Self::Derived(TypeInner { name, .. }) => name.as_str(),
            Self::Procedure(TypeInner { name, .. }) => name.as_str(),
            Self::Declared(TypeInner { name, .. }) => name.as_str(),
        }
    }
}

impl<'a> HasNode<'a> for Type<'a> {
    fn node(&self) -> &Node<'a> {
        match self {
            Self::Intrinsic(TypeInner { node, .. }) => node,
            Self::Derived(TypeInner { node, .. }) => node,
            Self::Procedure(TypeInner { node, .. }) => node,
            Self::Declared(TypeInner { node, .. }) => node,
        }
    }
}

/// A variable declaration line
#[derive(Clone, Debug, HasNode)]
pub struct VariableDeclaration<'a> {
    type_: Type<'a>,
    attributes: Vec<Attribute<'a>>,
    names: Vec<NameDecl<'a>>,
    node: Node<'a>,
    has_colon: bool,
    is_function: bool,
}

impl<'a> VariableDeclaration<'a> {
    /// Create from `variable_declaration` node
    pub fn try_from_node(node: &Node<'a>, src: &str) -> Result<Self> {
        if node.kind() != "variable_declaration" {
            return Err(anyhow!("wrong node type"));
        }

        let type_ = Type::try_from_node(
            node.child_by_field_name("type").context("expected type")?,
            src,
        )?;

        let attributes: Result<Vec<_>> = node
            .children_by_field_name("attribute", &mut node.walk())
            .map(Attribute::try_from_node)
            .collect();

        let names = node
            .children_by_field_name("declarator", &mut node.walk())
            .map(|decl| NameDecl::from_node(&decl, src))
            .collect_vec();

        let has_colon = node
            .children(&mut node.walk())
            .filter_map(|child| child.to_text(src))
            .any(|child| child == "::");

        Ok(Self {
            type_,
            attributes: attributes?,
            names,
            node: *node,
            has_colon,
            is_function: false,
        })
    }

    /// Create from `function_statement`. Will fail if the statement has no `type`
    pub fn try_from_fn_stmt(node: &Node<'a>, src: &str) -> Result<Self> {
        if node.kind() != "function_statement" {
            return Err(anyhow!("wrong node type"));
        }

        let type_ = Type::try_from_node(
            node.child_by_field_name("type").context("expected type")?,
            src,
        )?;

        let id = if let Some(result) = node.child_with_name("function_result") {
            result
                .child_with_name("identifier")
                .expect("`function_result` should have `identifier` child")
        } else {
            node.child_by_field_name("name")
                .expect("`function_statement` must have `name` field")
        };
        let name = NameDecl::from_node(&id, src);

        Ok(Self {
            type_,
            attributes: vec![],
            names: vec![name],
            node: *node,
            has_colon: false,
            is_function: true,
        })
    }

    pub fn type_(&self) -> &Type<'_> {
        &self.type_
    }

    pub fn attributes(&self) -> &[Attribute<'_>] {
        &self.attributes
    }

    pub fn names(&self) -> &[NameDecl<'a>] {
        &self.names
    }

    pub fn has_attribute(&self, attr: AttributeKind) -> bool {
        self.has_any_attributes(&[attr])
    }

    pub fn has_any_attributes(&self, attrs: &[AttributeKind]) -> bool {
        self.attributes
            .iter()
            .any(|attr| attrs.contains(&attr.kind))
    }

    pub const fn has_colon(&self) -> bool {
        self.has_colon
    }

    /// Is this variable declaration actually a function statement?
    pub const fn is_function(&self) -> bool {
        self.is_function
    }
}

/// Returns the tree-sitter node corresponding to the actual name of a
/// declarator node, and not, say, the initialiser
pub fn get_name_node_of_declarator<'a>(node: &Node<'a>) -> Node<'a> {
    match node.kind() {
        "identifier" | "method_name" | "type_name" | "name" => *node,
        "sized_declarator" => node
            .named_child(0)
            .expect("sized_declarator should have named child"),
        "coarray_declarator" => get_name_node_of_declarator(
            &node
                .named_child(0)
                .expect("coarray_declarator should have named child"),
        ),
        "init_declarator" | "pointer_init_declarator" | "data_declarator" => {
            get_name_node_of_declarator(
                &node
                    .child_by_field_name("left")
                    .expect("init/pointer_init/data_declarator should have left-hand side"),
            )
        }
        _ => unreachable!("unexpected node type in declarator ({node:?})"),
    }
}

/// Returns the tree-sitter node corresponding to the declared size of the
/// declarator node, if there is one
pub fn get_size_node_of_declarator<'a>(node: &'a Node<'a>) -> Option<Node<'a>> {
    node.named_descendants()
        .find(|child| child.kind() == "size")
}

/// Returns the tree-sitter node corresponding to the initialiser of the
/// declarator node, if there is one
pub fn get_init_node_of_declarator<'a>(node: &'a Node<'a>) -> Option<Node<'a>> {
    node.child_by_field_name("right")
}

/// A single Fortran variable
#[derive(Clone, Debug)]
pub struct Variable<'a> {
    name: NameDecl<'a>,
    is_dummy_var: bool,
    /// Reference to the statement in which the variable is declared
    decl: Rc<VariableDeclaration<'a>>,
}

impl<'a> Variable<'a> {
    pub fn new(name: NameDecl<'a>, is_dummy_var: bool, decl: Rc<VariableDeclaration<'a>>) -> Self {
        Self {
            name,
            is_dummy_var,
            decl,
        }
    }

    pub fn is_dummy_var(&self) -> bool {
        self.is_dummy_var
    }

    pub fn decl(&self) -> &NameDecl<'a> {
        &self.name
    }

    pub fn decl_statement(&'a self) -> &'a VariableDeclaration<'a> {
        self.decl.as_ref()
    }

    pub fn type_(&self) -> &Type<'_> {
        self.decl.type_()
    }

    pub fn attributes(&self) -> &[Attribute<'_>] {
        self.decl.attributes()
    }

    pub fn has_attribute(&self, attr: AttributeKind) -> bool {
        self.decl.has_attribute(attr)
    }

    pub fn has_any_attributes(&self, attrs: &[AttributeKind]) -> bool {
        self.decl.has_any_attributes(attrs)
    }
}

impl<'a> HasName<'a> for Variable<'a> {
    fn name(&self) -> &Name<'a> {
        self.name.name()
    }
}

impl<'a> HasNode<'a> for Variable<'a> {
    fn node(&self) -> &Node<'a> {
        self.name.node()
    }
}

#[derive(EnumString, Display)]
#[strum(ascii_case_insensitive)]
pub(crate) enum BlockExit {
    Return,
    Cycle,
    Exit,
    Stop,
    Error,
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct ImplicitNoneType: u8 {
        const TYPE = 0b0001;
        const EXTERNAL = 0b0010;
    }
}

#[derive(Clone, Debug, HasNode)]
pub(crate) struct ImplicitStatement<'a> {
    node: Node<'a>,
    none_type: ImplicitNoneType,
}

impl<'a> ImplicitStatement<'a> {
    pub fn try_from_node(node: Node<'a>, src: &SourceFile) -> Option<Self> {
        if node.kind() != "implicit_statement" {
            return None;
        }
        let text = node.to_text(src.source_text()).map(|t| t.to_lowercase())?;
        let mut none_type = ImplicitNoneType::empty();
        if !text.contains("none") {
            return Some(Self { node, none_type });
        }
        if text.contains("type") {
            none_type |= ImplicitNoneType::TYPE;
        }
        if text.contains("external") {
            none_type |= ImplicitNoneType::EXTERNAL;
        }
        // If the (type, external) part is missing, then 'type' is implied
        if !text.contains("type") && !text.contains("external") {
            none_type = ImplicitNoneType::TYPE;
        }
        Some(Self { node, none_type })
    }

    /// Determine the implicit typing scheme of a
    /// program/module/submodule/function/subroutine node.
    pub fn try_from_scope(node: &'a Node, src: &SourceFile) -> Option<Self> {
        if matches!(
            node.kind(),
            "module" | "submodule" | "program" | "function" | "subroutine"
        ) {
            if let Some(child) = node.child_with_name("implicit_statement") {
                return ImplicitStatement::try_from_node(child, src);
            }
            return None;
        }
        None
    }

    pub fn is_equivalent_to(&self, other: &Self) -> bool {
        self.none_type == other.none_type
    }

    pub fn is_not_implicit_none(&self) -> bool {
        self.none_type.is_empty()
    }

    pub fn is_implicit_none_type(&self) -> bool {
        self.none_type.contains(ImplicitNoneType::TYPE)
    }

    pub fn is_implicit_none_external(&self) -> bool {
        self.none_type.contains(ImplicitNoneType::EXTERNAL)
    }
}

#[derive(Clone, Debug, EnumIs, EnumString, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "snake_case", ascii_case_insensitive)]
pub enum ProcedureAttributeKind {
    Elemental,
    Impure,
    Module,
    NonRecursive,
    Pure,
    Recursive,
    Simple,
}

/// A procedure attribute and where it is
#[derive(Clone, Debug, HasNode)]
pub struct ProcedureAttribute<'a> {
    kind: ProcedureAttributeKind,
    node: Node<'a>,
}

impl<'a> ProcedureAttribute<'a> {
    pub fn try_from_node(node: Node<'a>, src: &str) -> Result<Self> {
        let kind = ProcedureAttributeKind::from_str(node.to_text(src).context("expected text")?)?;
        Ok(Self { kind, node })
    }

    pub fn kind(&self) -> &ProcedureAttributeKind {
        &self.kind
    }
}

#[derive(Copy, Clone, Debug, EnumIs, EnumString, PartialEq)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum ProcedureKind {
    Function,
    Subroutine,
}

#[derive(Clone, Debug, HasName, HasNode)]
pub struct Procedure<'a> {
    type_: Option<Type<'a>>,
    attributes: Vec<ProcedureAttribute<'a>>,
    name: Name<'a>,
    args: Vec<String>,
    kind: ProcedureKind,
    node: Node<'a>,
}

impl<'a> Procedure<'a> {
    pub fn try_from_node(node: &Node<'a>, src: &str) -> Result<Self> {
        if !node.is_named() || !matches!(node.kind(), "function" | "subroutine") {
            return Err(anyhow!("not a procedure"));
        }

        let kind = ProcedureKind::from_str(node.kind()).unwrap();

        let stmt = node.child(0).context("expected child")?;

        let type_ = if let Some(child) = stmt.child_by_field_name("type") {
            Some(Type::try_from_node(child, src)?)
        } else {
            None
        };

        let attributes: Result<Vec<_>> = stmt
            .named_children(&mut stmt.walk())
            .filter(|attr| attr.kind() == "procedure_qualifier")
            .map(|attr| ProcedureAttribute::try_from_node(attr, src))
            .collect();
        let attributes = attributes?;

        let name = stmt
            .child_by_field_name("name")
            .context("procedure should have `name` field")?;
        let name = Name::from_node(&name, src);

        let args = stmt
            .child_with_name("parameters")
            .map(|params| {
                params
                    .named_children(&mut params.walk())
                    .flat_map(|param| param.to_text(src))
                    .map(|param| param.to_ascii_lowercase())
                    .collect_vec()
            })
            .unwrap_or_default();

        Ok(Self {
            type_,
            attributes,
            name,
            args,
            kind,
            node: *node,
        })
    }

    pub const fn type_(&self) -> &Option<Type<'_>> {
        &self.type_
    }

    pub const fn attributes(&self) -> &Vec<ProcedureAttribute<'_>> {
        &self.attributes
    }

    pub const fn args(&self) -> &Vec<String> {
        &self.args
    }

    pub const fn kind(&self) -> ProcedureKind {
        self.kind
    }

    pub const fn is_function(&self) -> bool {
        self.kind.is_function()
    }

    pub const fn is_subroutine(&self) -> bool {
        self.kind.is_subroutine()
    }
}

/// Type representing a derived type definition.
/// Not yet fleshed out! Should add type attributes,
/// list of type-bound procedures, etc.
#[derive(Clone, Debug, HasName, HasNode)]
pub struct TypeDefinition<'a> {
    name: Name<'a>,
    node: Node<'a>,
}

impl<'a> TypeDefinition<'a> {
    pub fn try_from_node(node: &Node<'a>, src: &str) -> Result<Self> {
        if !node.is_named() || node.kind() != "derived_type_definition" {
            return Err(anyhow!("not a derived type"));
        }

        let stmt = node
            .child_with_name("derived_type_statement")
            .context("expected dervied_type_statement")?;
        let name_node = stmt
            .child_with_name("type_name")
            .context("expected type_name")?;
        let name = Name::from_node(&name_node, src);

        Ok(Self { name, node: *node })
    }
}

/// Type representing a module.
/// Not yet fleshed out! Should add implicit statement, list of used modules,
/// default accessibility, etc.
#[derive(Clone, Debug, HasName, HasNode)]
pub struct Module<'a> {
    name: Name<'a>,
    node: Node<'a>,
}

impl<'a> Module<'a> {
    pub fn try_from_node(node: &Node<'a>, src: &str) -> Result<Self> {
        if !node.is_named() || node.kind() != "module" {
            return Err(anyhow!("not a module"));
        }

        let stmt = node
            .child_with_name("module_statement")
            .context("expected module_statement")?;
        let name_node = stmt.child_with_name("name").context("expected name")?;
        let name = Name::from_node(&name_node, src);

        Ok(Self { name, node: *node })
    }
}

/// Type representing a program.
/// Not yet fleshed out! Should add implicit statement, list of used modules,
/// etc.
#[derive(Clone, Debug, HasName, HasNode)]
pub struct Program<'a> {
    name: Name<'a>,
    node: Node<'a>,
}

impl<'a> Program<'a> {
    pub fn try_from_node(node: &Node<'a>, src: &str) -> Result<Self> {
        if !node.is_named() || node.kind() != "program" {
            return Err(anyhow!("not a program"));
        }

        let stmt = node
            .child_with_name("program_statement")
            .context("expected program_statement")?;
        let name_node = stmt.child_with_name("name").context("expected name")?;
        let name = Name::from_node(&name_node, src);

        Ok(Self { name, node: *node })
    }
}
