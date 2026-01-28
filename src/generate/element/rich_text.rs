use printpdf::{FontId, Pt};

use crate::generate::{element::Element, text_gen::shape_text};

pub struct RichText {
    pub(crate) parts: Vec<(String, FontId)>,

    pub(crate) font_size: Pt,
    pub(crate) font_height_offset: Pt,
}

impl RichText {}

impl Element for RichText {
    fn display_name(&self) -> &str {
        "Richt Text"
    }

    fn calculate_width<'a>(
        &self,
        builder: &super::element_builder::ElementBuilder<'a>,
    ) -> printpdf::Mm {
        unimplemented!()
    }

    fn calculate_height<'a>(
        &self,
        builder: &super::element_builder::ElementBuilder<'a>,
    ) -> printpdf::Mm {
        unimplemented!()
    }

    fn build<'a>(&self, builder: &mut super::element_builder::ElementBuilder<'a>) {
        builder.push_rich_text(&self);
    }
}
