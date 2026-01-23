use printpdf::Mm;

use crate::generate::{element::Element2, outline::LineStyle, padding::Padding};

pub struct Line {
    outline: LineStyle,
    padding: Padding,
}

impl Line {
    pub fn new(outline: LineStyle, padding: Padding) -> Self {
        Line { outline, padding }
    }
}

impl Element2 for Line {
    fn display_name(&self) -> &str {
        "Line"
    }

    fn calculate_width<'a>(&self, builder: &super::element_builder::ElementBuilder<'a>) -> Mm {
        builder.remaining_width_from_cursor()
    }
    fn calculate_height<'a>(&self, _: &super::element_builder::ElementBuilder<'a>) -> Mm {
        Mm::from(self.outline.thickness) + self.padding.top + self.padding.bottom
    }

    fn build<'a>(&self, builder: &mut super::element_builder::ElementBuilder<'a>) {
        builder.draw_line(&self.padding, &self.outline);
    }
}
