use printpdf::Pt;

use crate::generate::{
    element::{Element, element_builder::ElementBuilder},
    font::Font,
};

pub struct Paragraph {
    text: String,

    font: Font,
}

impl Paragraph {
    pub fn new(text: impl Into<String>, font: Font) -> Self {
        Paragraph {
            text: text.into(),
            font,
        }
    }
}

impl Element for Paragraph {
    fn display_name(&self) -> &str {
        "Paragraph"
    }

    fn calculate_width<'a>(&self, builder: &ElementBuilder<'a>) -> Pt {
        builder.measure_text(self.text.as_str(), &self.font).0
    }

    fn calculate_height<'a>(&self, builder: &ElementBuilder<'a>) -> Pt {
        builder.measure_text(self.text.as_str(), &self.font).1
    }

    fn build<'a>(&self, builder: &mut ElementBuilder<'a>) {
        builder.push_paragraph(self.text.as_str(), &self.font);
    }
}
