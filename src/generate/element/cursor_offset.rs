use printpdf::{Mm, Point, Pt};

use crate::generate::{
    document::DocumentStyle,
    element::Element,
    text_gen::{DEFAULT_FONT_LINE_HEIGHT_OFFSET, DEFAULT_FONT_SIZE},
};

pub enum CursorOffset {
    Absolute(Pt),
    Relative(Pt),
    LineBreaks {
        lines: u8,
        font_size: Pt,
        font_height_offset: Pt,
    },
}

impl CursorOffset {
    pub fn line_breaks(lines: u8) -> Self {
        Self::LineBreaks {
            lines,
            font_size: DEFAULT_FONT_SIZE,
            font_height_offset: DEFAULT_FONT_LINE_HEIGHT_OFFSET,
        }
    }
}

impl Element for CursorOffset {
    fn build(
        &self,
        _document: &printpdf::PdfDocument,
        origin: Point,
        _max_width: Option<printpdf::Mm>,
        _page_style: &DocumentStyle,
    ) -> super::BuildResult {
        let next_cursor = match self {
            CursorOffset::Absolute(p) => Point { x: origin.x, y: *p },
            CursorOffset::Relative(y) => Point {
                x: origin.x,
                y: origin.y - *y,
            },
            CursorOffset::LineBreaks {
                lines,
                font_size,
                font_height_offset,
            } => Point {
                x: origin.x,
                y: origin.y - Pt(*lines as f32 * (*font_size + *font_height_offset).0), // Assuming 12pt line height
            },
        };

        super::BuildResult {
            ops: vec![],
            next_cursor,
            width: Mm(0.0),
        }
    }
}
