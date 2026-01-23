use printpdf::{Mm, Pt};

use crate::generate::{
    element::Element,
    text_gen::{DEFAULT_FONT_LINE_HEIGHT_OFFSET, DEFAULT_FONT_SIZE},
};

pub enum CursorOffset {
    Relative(Pt),
    LineBreaks {
        lines: u8,
        font_size: Pt,
        font_height_offset: Pt,
    },
}

impl CursorOffset {
    /// Adds a line break offset with the default font size and font line height offset.
    pub fn line_breaks(lines: u8) -> Self {
        Self::LineBreaks {
            lines,
            font_size: DEFAULT_FONT_SIZE,
            font_height_offset: DEFAULT_FONT_LINE_HEIGHT_OFFSET,
        }
    }
}

impl Element for CursorOffset {
    fn display_name(&self) -> &str {
        "Cursor Offset"
    }

    fn calculate_width<'a>(&self, _: &super::element_builder::ElementBuilder<'a>) -> Mm {
        Mm(0.0)
    }

    fn calculate_height<'a>(&self, _: &super::element_builder::ElementBuilder<'a>) -> Mm {
        match self {
            Self::Relative(rel) => Mm::from(*rel),
            Self::LineBreaks {
                lines,
                font_size,
                font_height_offset,
            } => Mm::from(Pt(*lines as f32 * (font_size.0 + font_height_offset.0))),
        }
    }

    fn build<'a>(&self, builder: &mut super::element_builder::ElementBuilder<'a>) {
        builder.advance_cursor(match self {
            Self::Relative(rel) => *rel,
            Self::LineBreaks {
                lines,
                font_size,
                font_height_offset,
            } => Pt(*lines as f32 * (font_size.0 + font_height_offset.0)),
        });

        // TODO: Maybe this should be handled on every case
        if let Self::LineBreaks { .. } = self {
            builder.reset_cursor_x();
        }
    }
}
