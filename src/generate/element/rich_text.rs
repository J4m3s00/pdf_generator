use printpdf::{Mm, Pt};

use crate::generate::{element::Element, font::Font};

#[derive(Debug)]
pub struct RichTextLinePart {
    pub text: String,
    pub font: Font,
    pub width: Pt,
}

#[derive(Default, Debug)]
pub struct RichTextLine {
    pub parts: Vec<RichTextLinePart>,
    pub height: Pt,
}

pub struct RichText {
    pub(crate) parts: Vec<(String, Font)>,
}

impl RichText {
    pub fn new(parts: impl Into<Vec<(String, Font)>>) -> Self {
        Self {
            parts: parts.into(),
        }
    }
}

impl Element for RichText {
    fn display_name(&self) -> &str {
        "Richt Text"
    }

    fn calculate_width<'a>(
        &self,
        builder: &super::element_builder::ElementBuilder<'a>,
    ) -> printpdf::Mm {
        builder.remaining_width_from_cursor()
    }

    fn calculate_height<'a>(
        &self,
        builder: &super::element_builder::ElementBuilder<'a>,
    ) -> printpdf::Mm {
        let lines = builder.split_rich_text_into_lines(self);
        let height = lines.into_iter().map(|line| line.height.0).sum::<f32>();

        Mm::from(Pt(height))
    }

    fn build<'a>(&self, builder: &mut super::element_builder::ElementBuilder<'a>) {
        builder.push_rich_text(&self);
    }
}
