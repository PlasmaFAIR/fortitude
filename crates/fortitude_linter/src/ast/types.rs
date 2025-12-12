//! Strong types for working with the tree-sitter AST

use std::str::FromStr;

use anyhow::{Context, Result, anyhow};
use itertools::Itertools;
use strum_macros::{Display, EnumIs, EnumString, IntoStaticStr};
use tree_sitter::Node;

use crate::{ast::FortitudeNode, impl_has_node, traits::HasNode};

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

    /// Variable name
    pub fn name(&self) -> &str {
        self.name.as_str()
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

impl_has_node!(NameDecl<'a>);

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

#[derive(Clone, Copy, Debug, PartialEq)]
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

impl_has_node!(Extent<'a>);

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
#[derive(Clone, Debug)]
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

impl_has_node!(Attribute<'a>);

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
#[derive(Clone, Debug)]
pub struct VariableDeclaration<'a> {
    type_: Type<'a>,
    attributes: Vec<Attribute<'a>>,
    names: Vec<NameDecl<'a>>,
    node: Node<'a>,
    has_colon: bool,
}

impl<'a> VariableDeclaration<'a> {
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
        })
    }

    pub fn type_(&self) -> &Type<'_> {
        &self.type_
    }

    pub fn attributes(&self) -> &Vec<Attribute<'_>> {
        &self.attributes
    }

    pub fn names(&self) -> &Vec<NameDecl<'a>> {
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

    pub fn has_colon(&self) -> bool {
        self.has_colon
    }
}

impl_has_node!(VariableDeclaration<'a>);

/// Returns the tree-sitter node corresponding to the actual name of a
/// declarator node, and not, say, the initialiser
pub fn get_name_node_of_declarator<'a>(node: &Node<'a>) -> Node<'a> {
    match node.kind() {
        "identifier" | "method_name" => *node,
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
    name: String,
    node: Node<'a>,
    /// Reference to the statement in which the variable is declared
    decl: &'a VariableDeclaration<'a>,
}

impl<'a> Variable<'a> {
    pub fn new(name: String, node: Node<'a>, decl: &'a VariableDeclaration<'a>) -> Self {
        Self { name, node, decl }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn decl_statement(&'a self) -> &'a VariableDeclaration<'a> {
        self.decl
    }

    pub fn type_(&self) -> &Type<'_> {
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

impl_has_node!(Variable<'a>);

#[derive(EnumString, Display)]
#[strum(ascii_case_insensitive)]
pub(crate) enum BlockExit {
    Return,
    Cycle,
    Exit,
    Stop,
    Error,
}
