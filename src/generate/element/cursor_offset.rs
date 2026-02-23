use printpdf::{Mm, Pt};

use crate::generate::{element::Element, font::Font};

pub enum CursorOffset {
    Relative(Pt),
    LineBreaks {
        lines: u8,
        font_size: Pt,
        font_height_offset: Pt,
    },
    PageBreaks {
        pages: u8,
    },
}

impl CursorOffset {
    /// Adds a line break offset with the default font size and font line height offset.
    pub fn line_breaks(lines: u8, font: &Font) -> Self {
        Self::LineBreaks {
            lines,
            font_size: font.font_size(),
            font_height_offset: font.font_height_offset(),
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

    fn calculate_height<'a>(&self, builder: &super::element_builder::ElementBuilder<'a>) -> Mm {
        match self {
            Self::Relative(rel) => Mm::from(*rel),
            Self::LineBreaks {
                lines,
                font_size,
                font_height_offset,
            } => Mm::from(Pt(*lines as f32 * (font_size.0 + font_height_offset.0))),
            Self::PageBreaks { pages } => {
                builder.remaining_height_from_cursor()
                    + Mm((pages - 1) as f32 * builder.document.style().inner_height().0)
            }
        }
    }

    fn build<'a>(&self, builder: &mut super::element_builder::ElementBuilder<'a>) {
        match self {
            Self::Relative(rel) => {
                builder.advance_cursor(*rel);
            }
            Self::LineBreaks {
                lines,
                font_size,
                font_height_offset,
            } => {
                builder.advance_cursor(Pt(*lines as f32 * (font_size.0 + font_height_offset.0)));
            }
            Self::PageBreaks { pages } => {
                for _ in 0..*pages {
                    builder.next_page();
                }
            }
        }

        builder.reset_cursor_x();
    }
}
