//! Strong types for working with the tree-sitter AST

use std::{rc::Rc, str::FromStr};

use anyhow::{Context, Result, anyhow};
use bitflags::bitflags;
use itertools::Itertools;
use ruff_text_size::TextRange;
use strum_macros::{Display, EnumIs, EnumString, IntoStaticStr};

use fortitude_macros::{HasName, HasNode, field, kind, kw};

use crate::Node;
use crate::traits::{HasNode, TextRanged};

#[derive(Clone, Debug)]
pub struct ParameterStatement<'a> {
    pub name: String,
    pub expression: String,
    pub node: Node<'a>,
}

impl<'a> ParameterStatement<'a> {
    pub fn try_from_node(node: Node<'a>) -> Result<Self> {
        Ok(Self {
            name: node
                .child_with_id(kind!("identifier"))
                .context("expected identifier in 'parameter_statement'")?
                .text()
                .to_string(),
            expression: node
                .child(2)
                .context("expected expression in 'parameter_statement'")?
                .text()
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
    pub fn from_node(node: &Node<'a>) -> Self {
        let node = get_name_node_of_declarator(node);
        Self {
            name: node.text().to_string(),
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
    pub fn from_node(node: &Node<'a>) -> Self {
        Self {
            name: Name::from_node(node),
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
        if node.kind_id() == kind!("assumed_size") {
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
        if node.kind_id() != kind!("extent_specifier") {
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
        match node.kind_id() {
            kind!("extent_specifier") => Ok(Self::Extent(Extent::try_from_node(node)?)),
            kind!("assumed_size") => Ok(Self::AssumedSize),
            kind!("assumed_rank") => Ok(Self::AssumedRank),
            kind!("multiple_subscript") => Ok(Self::MultipleSubscript(node)),
            kind!("multiple_subscript_triplet") => {
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
        if !matches!(node.kind_id(), kind!("argument_list") | kind!("size")) {
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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bind<'a> {
    language: &'a str,
    name: Option<&'a str>,
}

impl<'a> Bind<'a> {
    pub fn try_from_node(node: &Node<'a>, src: &'a str) -> Result<Self> {
        // Identifier is a required node in a bind() attribute
        let lang_node = node
            .child_with_id(kind!("identifier"))
            .expect("must have identifier child");
        let lang = lang_node.to_text(src).expect("must have text");

        // The name node is an optional one
        if let Some(name_kw_node) = node.child_with_id(kind!("keyword_argument")) {
            if let Some(name_node) = name_kw_node.child_with_id(kw!("value")) {
                let name = name_node.to_text(src).expect("must have text for name");
                return Ok(Self {
                    language: lang,
                    name: Some(name),
                });
            }
        }

        Ok(Self {
            language: lang,
            name: None,
        })
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
            .map(|child| child.kind_id())
            .collect_vec();
        if children.contains(&kw!("inout"))
            || (children.contains(&kw!("in")) && children.contains(&kw!("out")))
        {
            Self::InOut
        } else if children.contains(&kw!("in")) {
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
    Bind(Bind<'a>),
    Codimension,
    Dimension(Dimension<'a>),
    Constant,
    Contiguous,
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
    pub fn try_from_node(value: &Node<'a>, src: &'a str) -> Result<Self> {
        let first_child = value.child(0).unwrap().kind();
        // TODO: handle codimension properly
        let attr = AttributeKind::from_str(first_child)
            .context(format!("unknown attribute '{first_child}'"))?;

        match attr {
            AttributeKind::Intent(_) => Ok(AttributeKind::Intent(Intent::from_node(value))),
            AttributeKind::Dimension(_) => Ok(AttributeKind::Dimension(Dimension::try_from_node(
                value
                    .child(1)
                    .context("expected more than one child for 'dimension'")?,
            )?)),
            AttributeKind::Bind(_) => Ok(AttributeKind::Bind(Bind::try_from_node(value, src)?)),
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
    pub fn try_from_node(node: Node<'a>, src: &'a str) -> Result<Self> {
        Ok(Self {
            kind: AttributeKind::try_from_node(&node, src)?,
            node,
        })
    }

    pub fn kind(&'_ self) -> &'_ AttributeKind<'_> {
        &self.kind
    }
}

#[derive(Clone, Debug, EnumIs, IntoStaticStr)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum IntrinsicType<'a> {
    Byte(TypeInner<'a>),
    Integer(TypeInner<'a>),
    Real(TypeInner<'a>),
    #[strum(to_string = "double precision")]
    DoublePrecision(TypeInner<'a>),
    Complex(TypeInner<'a>),
    #[strum(to_string = "double complex")]
    DoubleComplex(TypeInner<'a>),
    Logical(TypeInner<'a>),
    Character(TypeInner<'a>),
}

impl<'a> IntrinsicType<'a> {
    pub fn from_node(node: Node<'a>) -> Self {
        if node.kind_id() != kind!("intrinsic_type") {
            panic!(
                "IntrinsicType can only be created from `intrinsic_type`, got {}",
                node.kind()
            );
        }
        let type_node = node.child(0).expect("must have zeroth child");
        let name = type_node.text();
        match type_node.kind_id() {
            kw!("byte") => IntrinsicType::Byte(TypeInner { node, name }),
            kw!("integer") => IntrinsicType::Integer(TypeInner { node, name }),
            kw!("real") => IntrinsicType::Real(TypeInner { node, name }),
            kw!("doubleprecision") => IntrinsicType::DoublePrecision(TypeInner { node, name }),
            kw!("complex") => IntrinsicType::Complex(TypeInner { node, name }),
            kw!("doublecomplex") => IntrinsicType::DoubleComplex(TypeInner { node, name }),
            kw!("logical") => IntrinsicType::Logical(TypeInner { node, name }),
            kw!("character") => IntrinsicType::Character(TypeInner { node, name }),
            kw!("double") => {
                let second = node
                    .child(1)
                    .expect("`double` must be followed by either `complex` or `precision`");
                match second.kind_id() {
                    kw!("complex") => IntrinsicType::DoubleComplex(TypeInner { node, name }),
                    kw!("precision") => IntrinsicType::DoublePrecision(TypeInner { node, name }),
                    _ => unreachable!("unexpected keyword following `double`"),
                }
            }
            _ => unreachable!("Unexpected node kind for `intrinsic_type`"),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Byte(TypeInner { name, .. }) => name,
            Self::Integer(TypeInner { name, .. }) => name,
            Self::Real(TypeInner { name, .. }) => name,
            Self::DoublePrecision(TypeInner { name, .. }) => name,
            Self::Complex(TypeInner { name, .. }) => name,
            Self::DoubleComplex(TypeInner { name, .. }) => name,
            Self::Logical(TypeInner { name, .. }) => name,
            Self::Character(TypeInner { name, .. }) => name,
        }
    }
}

impl<'a> HasNode<'a> for IntrinsicType<'a> {
    fn node(&self) -> &Node<'a> {
        match self {
            Self::Byte(TypeInner { node, .. }) => node,
            Self::Integer(TypeInner { node, .. }) => node,
            Self::Real(TypeInner { node, .. }) => node,
            Self::DoublePrecision(TypeInner { node, .. }) => node,
            Self::Complex(TypeInner { node, .. }) => node,
            Self::DoubleComplex(TypeInner { node, .. }) => node,
            Self::Logical(TypeInner { node, .. }) => node,
            Self::Character(TypeInner { node, .. }) => node,
        }
    }
}

#[derive(Clone, Debug, HasNode)]
pub struct TypeInner<'a> {
    node: Node<'a>,
    name: &'a str,
}

#[derive(Clone, Debug, EnumIs)]
pub enum Type<'a> {
    Intrinsic(IntrinsicType<'a>),
    Derived(TypeInner<'a>),
    Procedure(TypeInner<'a>),
    Declared(TypeInner<'a>),
}

impl<'a> Type<'a> {
    pub fn try_from_node(node: Node<'a>) -> Result<Self> {
        let kind = node.kind_id();
        let name = node.text();
        match kind {
            kind!("intrinsic_type") => Ok(Type::Intrinsic(IntrinsicType::from_node(node))),
            kind!("derived_type") => Ok(Type::Derived(TypeInner { node, name })),
            kind!("procedure") => Ok(Type::Procedure(TypeInner { node, name })),
            kind!("declared_type") => Ok(Type::Declared(TypeInner { node, name })),
            _ => Err(anyhow!("unexpected 'type' kind '{}'", node.kind())),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Intrinsic(inner) => inner.as_str(),
            Self::Derived(TypeInner { name, .. }) => name,
            Self::Procedure(TypeInner { name, .. }) => name,
            Self::Declared(TypeInner { name, .. }) => name,
        }
    }
}

impl<'a> HasNode<'a> for Type<'a> {
    fn node(&self) -> &Node<'a> {
        match self {
            Self::Intrinsic(inner) => inner.node(),
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
    pub fn try_from_node(node: &Node<'a>) -> Result<Self> {
        if node.kind_id() != kind!("variable_declaration") {
            return Err(anyhow!("wrong node type"));
        }

        let type_ = Type::try_from_node(
            node.child_by_field_id(field!("type").into())
                .context("expected type")?,
        )?;

        let attributes: Result<Vec<_>> = node
            .children_by_field_id(field!("attribute"), &mut node.walk())
            .map(|decl| Attribute::try_from_node(decl, src))
            .collect();

        let names = node
            .children_by_field_id(field!("declarator"), &mut node.walk())
            .map(|decl| NameDecl::from_node(&decl))
            .collect_vec();

        let has_colon = node
            .children(&mut node.walk())
            .map(|child| child.text())
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
    pub fn try_from_fn_stmt(node: &Node<'a>) -> Result<Self> {
        if node.kind_id() != kind!("function_statement") {
            return Err(anyhow!("wrong node type"));
        }

        let type_ = Type::try_from_node(
            node.child_by_field_id(field!("type").into())
                .context("expected type")?,
        )?;

        let id = if let Some(result) = node.child_with_id(kind!("function_result")) {
            result
                .child_with_id(kind!("identifier"))
                .expect("`function_result` should have `identifier` child")
        } else {
            node.child_by_field_id(field!("name").into())
                .expect("`function_statement` must have `name` field")
        };
        let name = NameDecl::from_node(&id);

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
    match node.kind_id() {
        kind!("identifier")
        | kind!("method_name")
        | kind!("type_name")
        | kind!("module_name")
        | kind!("local_name")
        | kind!("name") => *node,
        kind!("sized_declarator") => node
            .named_child(0)
            .expect("sized_declarator should have named child"),
        kind!("coarray_declarator") => get_name_node_of_declarator(
            &node
                .named_child(0)
                .expect("coarray_declarator should have named child"),
        ),
        #[allow(clippy::manual_range_patterns)]
        kind!("init_declarator") | kind!("pointer_init_declarator") | kind!("data_declarator") => {
            get_name_node_of_declarator(
                &node
                    .child_by_field_id(field!("left").into())
                    .expect("init/pointer_init/data_declarator should have left-hand side"),
            )
        }
        kind!("use_alias") => node
            .child_with_id(kind!("local_name"))
            .expect("use_alias should have local_name child"),
        _ => unreachable!("unexpected node type in declarator ({node:?})"),
    }
}

/// Returns the tree-sitter node corresponding to the declared size of the
/// declarator node, if there is one
pub fn get_size_node_of_declarator<'a>(node: &'a Node<'a>) -> Option<Node<'a>> {
    node.named_descendants()
        .find(|child| child.kind_id() == kind!("size"))
}

/// Returns the tree-sitter node corresponding to the initialiser of the
/// declarator node, if there is one
pub fn get_init_node_of_declarator<'a>(node: &'a Node<'a>) -> Option<Node<'a>> {
    node.child_by_field_id(field!("right").into())
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
pub enum BlockExit {
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
pub struct ImplicitStatement<'a> {
    node: Node<'a>,
    none_type: ImplicitNoneType,
}

impl<'a> ImplicitStatement<'a> {
    pub fn try_from_node(node: Node<'a>) -> Option<Self> {
        if node.kind_id() != kind!("implicit_statement") {
            return None;
        }
        let mut none_type = ImplicitNoneType::empty();
        let ids = node
            .descendants()
            .map(|child| child.kind_id())
            .collect_vec();
        if !ids.contains(&kind!("none")) {
            return Some(Self { node, none_type });
        }
        if ids.contains(&kw!("type")) {
            none_type |= ImplicitNoneType::TYPE;
        }
        if ids.contains(&kw!("external")) {
            none_type |= ImplicitNoneType::EXTERNAL;
        }
        // If the (type, external) part is missing, then 'type' is implied
        if !ids.contains(&kw!("type")) && !ids.contains(&kw!("external")) {
            none_type = ImplicitNoneType::TYPE;
        }
        Some(Self { node, none_type })
    }

    /// Determine the implicit typing scheme of a
    /// program/module/submodule/function/subroutine node.
    pub fn try_from_scope(node: &'a Node) -> Option<Self> {
        if matches!(
            node.kind_id(),
            kind!("module")
                | kind!("submodule")
                | kind!("program")
                | kind!("function")
                | kind!("subroutine")
        ) {
            if let Some(child) = node.child_with_id(kind!("implicit_statement")) {
                return ImplicitStatement::try_from_node(child);
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
    pub fn try_from_node(node: Node<'a>) -> Result<Self> {
        let kind = ProcedureAttributeKind::from_str(node.kind())
            .context(format!("unknown procedure attribute '{}'", node.kind()))?;
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
    pub fn try_from_node(node: &Node<'a>) -> Result<Self> {
        if !matches!(node.kind_id(), kind!("function") | kind!("subroutine")) {
            return Err(anyhow!("not a procedure"));
        }

        let kind = ProcedureKind::from_str(node.kind())
            .context(format!("unknown procedure kind '{}'", node.kind()))?;

        let stmt = node.child(0).context("expected child")?;

        let type_ = if let Some(child) = stmt.child_by_field_name("type") {
            Some(Type::try_from_node(child)?)
        } else {
            None
        };

        let attributes: Result<Vec<_>> = stmt
            .named_children(&mut stmt.walk())
            .filter(|attr| attr.kind_id() == kind!("procedure_qualifier"))
            .map(|attr| attr.child(0).expect("procedure_qualifier must have child"))
            .map(ProcedureAttribute::try_from_node)
            .collect();
        let attributes = attributes?;

        let name = stmt
            .child_by_field_id(field!("name").into())
            .context("procedure should have `name` field")?;
        let name = Name::from_node(&name);

        let args = stmt
            .child_with_id(kind!("parameters"))
            .map(|params| {
                params
                    .named_children(&mut params.walk())
                    .map(|param| param.text().to_ascii_lowercase())
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
    pub fn try_from_node(node: &Node<'a>) -> Result<Self> {
        if node.kind_id() != kind!("derived_type_definition") {
            return Err(anyhow!("not a derived type"));
        }

        let stmt = node
            .child_with_id(kind!("derived_type_statement"))
            .context("expected dervied_type_statement")?;
        let name_node = stmt
            .child_with_id(kind!("type_name"))
            .context("expected type_name")?;
        let name = Name::from_node(&name_node);

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
    pub fn try_from_node(node: &Node<'a>) -> Result<Self> {
        if node.kind_id() != kind!("module") {
            return Err(anyhow!("not a module"));
        }

        let stmt = node
            .child_with_id(kind!("module_statement"))
            .context("expected module_statement")?;
        let name_node = stmt.child_with_id(kind!("name")).context("expected name")?;
        let name = Name::from_node(&name_node);

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
    pub fn try_from_node(node: &Node<'a>) -> Result<Self> {
        if node.kind_id() != kind!("program") {
            return Err(anyhow!("not a program"));
        }

        let stmt = node
            .child_with_id(kind!("program_statement"))
            .context("expected program_statement")?;
        let name_node = stmt.child_with_id(kind!("name")).context("expected name")?;
        let name = Name::from_node(&name_node);

        Ok(Self { name, node: *node })
    }
}

/// A use statament line
#[derive(Clone, Debug, HasName, HasNode)]
pub struct UseStatement<'a> {
    intrinsic: bool,
    has_only: bool,
    has_colon: bool,
    name: Name<'a>,
    node: Node<'a>,
}

impl<'a> UseStatement<'a> {
    /// Create from `use_statement` node
    pub fn try_from_node(node: &Node<'a>) -> Result<Self> {
        if node.kind_id() != kind!("use_statement") {
            return Err(anyhow!("wrong node type"));
        }

        let name = Name::from_node(
            &node
                .child_with_id(kind!("module_name"))
                .context("expected module_name in 'use_statement'")?,
        );

        let has_only = node.child_with_id(kind!("included_items")).is_some();

        let mut intrinsic = false;
        let mut has_colon = false;
        for child in node.children(&mut node.walk()) {
            if child.kind_id() == kw!("intrinsic") {
                intrinsic = true;
            }
            if child.kind_id() == kw!("::") {
                has_colon = true;
                break;
            }
        }

        Ok(Self {
            intrinsic,
            has_only,
            has_colon,
            name,
            node: *node,
        })
    }

    pub fn included_items(&self) -> Option<Node<'a>> {
        self.node.child_with_id(kind!("included_items"))
    }

    pub fn is_intrinsic(&self) -> bool {
        self.intrinsic
    }

    pub fn has_only(&self) -> bool {
        self.has_only
    }

    pub fn has_colon(&self) -> bool {
        self.has_colon
    }
}

/// A single Fortran variable
#[derive(Clone, Debug, HasName)]
pub struct UsedItem<'a> {
    name: Name<'a>,
    alias_of: Option<Name<'a>>,
    /// Reference to the statement in which the variable is used
    decl: Rc<UseStatement<'a>>,
}

impl<'a> UsedItem<'a> {
    pub fn try_from_node(node: Node<'a>, decl: Rc<UseStatement<'a>>) -> Option<Self> {
        if node.kind_id() == kind!("identifier") {
            let name = Name::from_node(&node);
            Some(Self {
                name,
                alias_of: None,
                decl,
            })
        } else if node.kind_id() == kind!("use_alias") {
            let name = Name::from_node(
                &node
                    .child_with_id(kind!("local_name"))
                    .expect("use_alias should have local_name child"),
            );
            let alias_of = Name::from_node(
                &node
                    .child_with_id(kind!("identifier"))
                    .expect("use_alias should have identifier child"),
            );
            Some(Self {
                name,
                alias_of: Some(alias_of),
                decl,
            })
        } else {
            // Can include comments etc
            None
        }
    }

    pub fn alias_of(&self) -> Option<&Name<'a>> {
        self.alias_of.as_ref()
    }

    pub fn decl_statement(&'a self) -> &'a UseStatement<'a> {
        self.decl.as_ref()
    }

    pub fn module_name(&self) -> &Name<'a> {
        &self.decl.name
    }
}

impl<'a> HasNode<'a> for UsedItem<'a> {
    fn node(&self) -> &Node<'a> {
        self.name.node()
    }
}

/// Returns true if the type passed to it is number-like, and of a kind that can be modified using
/// kinds. 'double precision' and 'double complex' are not included.
pub fn dtype_is_plain_number(dtype: &str) -> bool {
    matches!(
        dtype.to_lowercase().as_str(),
        "integer" | "real" | "logical" | "complex"
    )
}

/// A block of consecutive comments (no blank lines)
#[derive(Debug, Clone)]
pub struct CommentBlock {
    text_range: TextRange,
    start_row: usize,
    end_row: usize,
    text: String,
}

impl CommentBlock {
    pub fn try_from_node_range(nodes: Vec<Node>) -> Result<Self> {
        if let Some(non_comment) = nodes.iter().find(|node| node.kind() != "comment") {
            return Err(anyhow!(
                "Unexpected non-comment '{non_comment:?}' in comment block"
            ));
        }
        if nodes.is_empty() {
            return Err(anyhow!("CommentBlock requires at least one node"));
        }
        // Have at least one, so can get first and last
        let first = nodes.first().unwrap();
        let last = nodes.last().unwrap();

        let start_textsize = first.start_textsize();
        let end_textsize = last.end_textsize();
        let text_range = TextRange::new(start_textsize, end_textsize);

        let start_row = first.start_position().row;
        let end_row = last.end_position().row;

        let text = nodes
            .iter()
            .map(|node| node.text())
            .collect_vec()
            .join("\n");

        Ok(Self {
            text_range,
            start_row,
            end_row,
            text,
        })
    }

    pub fn start_row(&self) -> usize {
        self.start_row
    }

    pub fn end_row(&self) -> usize {
        self.end_row
    }

    pub fn text(&self) -> &str {
        self.text.as_ref()
    }
}

impl TextRanged for CommentBlock {
    fn textrange(&self) -> TextRange {
        self.text_range
    }
}

/// A control flow keyword
#[derive(Clone, Debug, EnumIs)]
pub enum ControlFlow {
    Continue,
    Cycle,
    Exit,
    GoTo(String),
    Return,
    Stop,
}

impl ControlFlow {
    pub fn maybe_from(value: &Node) -> Option<Self> {
        if value.kind_id() != kind!("keyword_statement") {
            return None;
        }
        match value.child(0)?.kind_id() {
            kw!("continue") => Some(Self::Continue),
            kw!("cycle") => Some(Self::Cycle),
            kw!("exit") => Some(Self::Exit),
            kw!("return") => Some(Self::Return),
            kw!("stop") => Some(Self::Stop),
            kw!("error") => Some(Self::Stop),
            keyword => Self::parse_goto(keyword, value),
        }
    }

    fn parse_goto(keyword: u16, value: &Node) -> Option<Self> {
        if !matches!(keyword, kw!("go") | kw!("goto")) {
            return None;
        }

        // We expect either `go to N` or `goto N`.
        // Don't bother with assigned or computed gotos for now
        let expected_ref_index = if keyword == kw!("go") { 2 } else { 1 };
        if value.child_count() > expected_ref_index + 1 {
            return None;
        }

        Some(Self::GoTo(
            value.child(expected_ref_index)?.text().to_string(),
        ))
    }
}

/// A control flow node
#[derive(Clone, Debug)]
pub struct ControlFlowNode<'a> {
    control_flow: ControlFlow,
    node: Node<'a>,
}

impl<'a> ControlFlowNode<'a> {
    pub fn maybe_from(node: Node<'a>) -> Option<Self> {
        ControlFlow::maybe_from(&node).map(|control_flow| Self { control_flow, node })
    }

    pub fn goto_ref(&'a self) -> Option<&'a str> {
        match self.control_flow {
            ControlFlow::GoTo(ref ref_) => Some(ref_),
            _ => None,
        }
    }

    pub fn control_flow(&self) -> ControlFlow {
        self.control_flow.clone()
    }

    pub fn node(&'a self) -> Node<'a> {
        self.node
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    use anyhow::{Context, Result};
    use ruff_text_size::TextSize;
    use test_case::test_case;
    use textwrap::dedent;
    use tree_sitter::Point;

    #[test_case("byte", |result| result.is_byte())]
    #[test_case("integer", |result| result.is_integer())]
    #[test_case("real", |result| result.is_real())]
    #[test_case("doubleprecision", |result| result.is_double_precision())]
    #[test_case("double precision", |result| result.is_double_precision())]
    #[test_case("complex", |result| result.is_complex())]
    #[test_case("doublecomplex", |result| result.is_double_complex())]
    #[test_case("double complex", |result| result.is_double_complex())]
    #[test_case("logical", |result| result.is_logical())]
    #[test_case("character", |result| result.is_character())]
    fn intrinsic_type(type_: &str, check: impl FnOnce(IntrinsicType) -> bool) -> Result<()> {
        let mut parser = Parser::new(&tree_sitter_fortran::LANGUAGE.into())
            .expect("Error loading Fortran grammar");

        let code = format!("{type_} :: foo\nend");

        let tree = parser.parse(&code, None).expect("Failed to parse");
        let root = tree.root_node();
        let type_ = root
            .named_descendants()
            .find(|child| child.kind_id() == kind!("intrinsic_type"))
            .expect("couldn't find intrinsic_type");

        let result = IntrinsicType::from_node(type_);

        assert!(check(result));

        Ok(())
    }

    #[test]
    fn decls() -> Result<()> {
        let mut parser = Parser::new(&tree_sitter_fortran::LANGUAGE.into())
            .expect("Error loading Fortran grammar");

        let code = "real, dimension(2) :: x, y = [4, 2], z(3)\nend";

        let tree = parser.parse(code, None).expect("Failed to parse");
        let root = tree.root_node();
        let type_ = root
            .named_descendants()
            .find(|child| child.kind_id() == kind!("variable_declaration"))
            .expect("couldn't find variable_declaration");

        let result = VariableDeclaration::try_from_node(&type_)?;

        assert_eq!(result.names().len(), 3);

        let mut iter = result.names().iter();
        let x = iter.next().unwrap();
        let y = iter.next().unwrap();
        let z = iter.next().unwrap();

        let x_size = x.size();
        let y_size = y.size();
        let z_size = z.size();
        assert!(x_size.is_none());
        assert!(y_size.is_none());
        assert!(z_size.is_some());

        Ok(())
    }

    #[test]
    fn test_comment_block() -> Result<()> {
        let mut parser = Parser::new(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;

        let code = dedent(
            r#"
          ! one
          ! two
          program foo

          contains

            ! not this

            ! but this
            subroutine bar()
            end subroutine bar
          end program foo

          ! not this either

          module zing
          end module zing

          "#,
        );

        let tree = parser.parse(&code, None).context("Failed to parse")?;

        let program_node = tree
            .root_node()
            .child_with_name("program")
            .context("Missing program node")?;
        let program_comments = program_node
            .prev_attached_comment_block()
            .context("Couldn't find program comment block")?;

        let expected_text = "! one\n! two";
        assert_eq!(
            program_comments.textrange(),
            TextRange::new(
                TextSize::new(1),
                TextSize::new(expected_text.len().saturating_add(1).try_into()?)
            )
        );
        assert_eq!(program_comments.text(), expected_text);

        let subroutine_node = program_node
            .descendants()
            .find(|node| node.kind() == "subroutine")
            .context("Missing subroutine node")?;
        let subroutine_comments = subroutine_node
            .prev_attached_comment_block()
            .context("Couldn't find subroutine comment block")?;
        let expected_text = "! but this";
        assert_eq!(subroutine_comments.text(), expected_text);

        let module_node = tree
            .root_node()
            .child_with_name("module")
            .context("Missing module node")?;
        assert!(module_node.prev_attached_comment_block().is_none());

        Ok(())
    }

    #[test]
    fn prev_line_continuation() -> Result<()> {
        let mut parser = Parser::new(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;

        let code = dedent(
            r#"
          program foo
            do &
              while (.true.)
            end do
          end program foo
          "#,
        );

        let tree = parser.parse(&code, None).context("Failed to parse")?;
        let root = tree.root_node();
        let node = root
            .descendants()
            .find(|node| node.kind() == "while_statement")
            .context("missing 'while'")?;

        let ampersand = node.prev_line_continuation();
        assert!(ampersand.is_some());
        assert_eq!(ampersand.unwrap().start_position(), Point::new(2, 5));

        Ok(())
    }

    #[test]
    fn next_line_continuation() -> Result<()> {
        let mut parser = Parser::new(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;

        let code = dedent(
            r#"
          program foo
            do &
              while (.true.)
            end do
          end program foo
          "#,
        );

        let tree = parser.parse(&code, None).context("Failed to parse")?;
        let root = tree.root_node();
        let node = root
            .descendants()
            .find(|node| node.kind() == "do")
            .context("missing 'do'")?;

        let ampersand = node.next_line_continuation();
        assert!(ampersand.is_some());
        assert_eq!(ampersand.unwrap().start_position(), Point::new(2, 5));

        Ok(())
    }
}
