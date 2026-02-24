use printpdf::Pt;

use crate::generate::{element::Element, outline::LineStyle, padding::Padding};

pub struct Line {
    outline: LineStyle,
    padding: Padding,
}

impl Line {
    pub fn new(outline: LineStyle, padding: Padding) -> Self {
        Line { outline, padding }
    }
}

impl Element for Line {
    fn display_name(&self) -> &str {
        "Line"
    }

    fn calculate_width<'a>(&self, builder: &super::element_builder::ElementBuilder<'a>) -> Pt {
        builder.remaining_width_from_cursor()
    }
    fn calculate_height<'a>(&self, _: &super::element_builder::ElementBuilder<'a>) -> Pt {
        self.outline.thickness + self.padding.top.into_pt() + self.padding.bottom.into_pt()
    }

    fn build<'a>(&self, builder: &mut super::element_builder::ElementBuilder<'a>) {
        builder.draw_line(&self.padding, &self.outline);
    }
}
