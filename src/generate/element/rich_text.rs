use crate::generate::{element::Element, font::Font};

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
        _builder: &super::element_builder::ElementBuilder<'a>,
    ) -> printpdf::Mm {
        unimplemented!()
    }

    fn calculate_height<'a>(
        &self,
        _builder: &super::element_builder::ElementBuilder<'a>,
    ) -> printpdf::Mm {
        unimplemented!()
    }

    fn build<'a>(&self, builder: &mut super::element_builder::ElementBuilder<'a>) {
        builder.push_rich_text(&self);
    }
}
