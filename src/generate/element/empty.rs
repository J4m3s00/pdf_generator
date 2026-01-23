use printpdf::Mm;

use crate::generate::element::Element;

pub struct Empty;

impl Element for Empty {
    fn display_name(&self) -> &str {
        "Empty"
    }

    fn calculate_height<'a>(&self, _: &super::element_builder::ElementBuilder<'a>) -> Mm {
        Mm(0.0)
    }

    fn calculate_width<'a>(&self, _: &super::element_builder::ElementBuilder<'a>) -> Mm {
        Mm(0.0)
    }

    fn build<'a>(&self, _: &mut super::element_builder::ElementBuilder<'a>) {}
}
