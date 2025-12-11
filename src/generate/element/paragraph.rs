use printpdf::{FontId, Mm, PdfDocument, Point, Pt};

use crate::generate::{
    document::DocumentStyle,
    element::{BuildResult, Element},
    text_gen::{DEFAULT_FONT_LINE_HEIGHT_OFFSET, DEFAULT_FONT_SIZE, shape_text},
};

pub struct Paragraph {
    text: String,

    font: FontId,
    font_size: Pt,
    font_height_offset: Pt,
}

impl Paragraph {
    pub fn new(text: impl Into<String>, font: FontId) -> Self {
        Paragraph {
            text: text.into(),
            font,
            font_size: DEFAULT_FONT_SIZE,
            font_height_offset: DEFAULT_FONT_LINE_HEIGHT_OFFSET,
        }
    }

    pub fn with_font_size(mut self, font_size: Pt) -> Self {
        self.font_size = font_size;
        self
    }
}

impl Element for Paragraph {
    fn build(
        &self,
        document: &PdfDocument,
        origin: Point,
        max_width: Option<Mm>,
        _page_style: &DocumentStyle,
    ) -> BuildResult {
        let shaped_text = shape_text(
            document,
            self.font.clone(),
            self.font_size,
            self.font_height_offset,
            &self.text,
            max_width,
        );

        let ops = shaped_text.get_ops(origin);
        let next_cursor = Point {
            x: origin.x,
            y: origin.y - Pt(shaped_text.height),
        };

        BuildResult {
            ops,
            next_cursor,
            width: Mm::from(Pt(shaped_text.width)),
        }
    }
}
