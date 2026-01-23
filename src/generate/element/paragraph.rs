use printpdf::{FontId, Mm, PdfDocument, Point, Pt};

use crate::generate::{
    element::{Element2, element_builder::ElementBuilder},
    text_gen::{DEFAULT_FONT_LINE_HEIGHT_OFFSET, DEFAULT_FONT_SIZE},
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

impl Element2 for Paragraph {
    fn display_name(&self) -> &str {
        "Paragraph"
    }

    fn calculate_width<'a>(&self, builder: &ElementBuilder<'a>) -> Mm {
        Mm::from(
            builder
                .measure_text(
                    self.text.as_str(),
                    self.font.clone(),
                    self.font_size,
                    self.font_height_offset,
                )
                .0,
        )
    }

    fn calculate_height<'a>(&self, builder: &ElementBuilder<'a>) -> Mm {
        Mm::from(
            builder
                .measure_text(
                    self.text.as_str(),
                    self.font.clone(),
                    self.font_size,
                    self.font_height_offset,
                )
                .1,
        )
    }

    fn build<'a>(&self, builder: &mut ElementBuilder<'a>) {
        builder.push_paragraph(
            self.text.as_str(),
            self.font.clone(),
            self.font_size,
            self.font_height_offset,
        );
    }
}
