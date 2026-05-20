use super::PositionEncoding;
use lsp_types as types;
use ruff_source_file::{LineIndex, OneIndexed, SourceLocation};
use ruff_text_size::TextRange;

pub(crate) trait RangeExt {
    fn to_text_range(&self, text: &str, index: &LineIndex, encoding: PositionEncoding)
    -> TextRange;
}

pub(crate) trait ToRangeExt {
    fn to_range(&self, text: &str, index: &LineIndex, encoding: PositionEncoding) -> types::Range;
}

fn u32_index_to_usize(index: u32) -> usize {
    usize::try_from(index).expect("u32 fits in usize")
}

impl RangeExt for lsp_types::Range {
    fn to_text_range(
        &self,
        text: &str,
        index: &LineIndex,
        encoding: PositionEncoding,
    ) -> TextRange {
        let start = index.offset(
            SourceLocation {
                line: OneIndexed::from_zero_indexed(u32_index_to_usize(self.start.line)),
                character_offset: OneIndexed::from_zero_indexed(u32_index_to_usize(
                    self.start.character,
                )),
            },
            text,
            encoding.into(),
        );
        let end = index.offset(
            SourceLocation {
                line: OneIndexed::from_zero_indexed(u32_index_to_usize(self.end.line)),
                character_offset: OneIndexed::from_zero_indexed(u32_index_to_usize(
                    self.end.character,
                )),
            },
            text,
            encoding.into(),
        );

        TextRange::new(start, end)
    }
}

impl ToRangeExt for TextRange {
    fn to_range(&self, text: &str, index: &LineIndex, encoding: PositionEncoding) -> types::Range {
        types::Range {
            start: source_location_to_position(&index.source_location(
                self.start(),
                text,
                encoding.into(),
            )),
            end: source_location_to_position(&index.source_location(
                self.end(),
                text,
                encoding.into(),
            )),
        }
    }
}

fn source_location_to_position(location: &SourceLocation) -> types::Position {
    types::Position {
        line: u32::try_from(location.line.to_zero_indexed()).expect("row usize fits in u32"),
        character: u32::try_from(location.character_offset.to_zero_indexed())
            .expect("character usize fits in u32"),
    }
}
