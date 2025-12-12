use ruff_text_size::{TextRange, TextSize};
use tree_sitter::Node;

/// Something with a [`tree_sitter::Node`]
pub trait HasNode<'a> {
    fn node(&self) -> &Node<'a>;
}

impl<'a, T> HasNode<'a> for &'a T
where
    T: HasNode<'a>,
{
    fn node(&self) -> &Node<'a> {
        T::node(self)
    }
}

#[macro_export]
macro_rules! impl_has_node {
    ($ty: ty) => {
        impl<'a> HasNode<'a> for $ty {
            #[inline]
            fn node(&self) -> &Node<'a> {
                &self.node
            }
        }
    };
}

/// A ranged item in the source text.
///
/// A kind of shim for [`ruff_text_size::Ranged`] for compatibility with
/// [`tree_sitter::Node::range`]
pub trait TextRanged {
    /// The range of this item in the source text.
    fn textrange(&self) -> TextRange;

    /// The start offset of this item in the source text.
    fn start_textsize(&self) -> TextSize {
        self.textrange().start()
    }

    /// The end offset of this item in the source text.
    fn end_textsize(&self) -> TextSize {
        self.textrange().end()
    }
}

impl<'a, T> TextRanged for T
where
    T: HasNode<'a>,
{
    fn textrange(&self) -> TextRange {
        self.node().textrange()
    }
}

impl<'a> TextRanged for Node<'a> {
    #[inline]
    fn start_textsize(&self) -> TextSize {
        TextSize::try_from(self.start_byte()).unwrap()
    }
    #[inline]
    fn end_textsize(&self) -> TextSize {
        TextSize::try_from(self.end_byte()).unwrap()
    }
    #[inline]
    fn textrange(&self) -> TextRange {
        TextRange::new(self.start_textsize(), self.end_textsize())
    }
}
