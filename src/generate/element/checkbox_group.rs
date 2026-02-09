use printpdf::{Mm, Pt};

use crate::generate::{
    element::{Element, element_builder::ColumnWidth},
    font::Font,
    padding::Padding,
};

pub struct CheckboxGroup {
    pub checkboxes: Vec<String>,

    space_between_checkboxes: Pt,

    font: Font,
}

impl CheckboxGroup {
    pub fn new(checkboxes: Vec<String>, font: Font) -> Self {
        Self {
            checkboxes,
            font,
            space_between_checkboxes: Pt(7.5), // Default space between checkboxes
        }
    }

    pub fn with_space_between_checkboxes(mut self, space: Pt) -> Self {
        self.space_between_checkboxes = space;
        self
    }
}

impl Element for CheckboxGroup {
    fn display_name(&self) -> &str {
        "Checkbox Group"
    }

    fn calculate_width<'a>(&self, builder: &super::element_builder::ElementBuilder<'a>) -> Mm {
        self.checkboxes.iter().fold(Mm(0.0), |t, cb| {
            t + Mm::from(
                builder.measure_text(cb.as_str(), &self.font).0
                    + Pt(4.0)
                    + self.font.font_size()
                    + self.space_between_checkboxes,
            )
        })
    }

    fn calculate_height<'a>(&self, builder: &super::element_builder::ElementBuilder<'a>) -> Mm {
        Mm::from(self.font.font_size() + self.font.font_height_offset())
    }

    fn build<'a>(&self, builder: &mut super::element_builder::ElementBuilder<'a>) {
        // Create a group builder, to have the checkboxes in a group
        // When the checkbox_group is at the bottom of the page, and one of the text has multiple
        // lines and pushes to the next page, we want the whole checkbox_group to be on the next
        // page if possible
        let mut group_builder =
            builder.generate_group_builder(&Padding::none(), Some(self.calculate_height(builder)));

        let mut next_builder = group_builder.clone();

        // let (mut left, mut right) = group_builder
        //     .generate_column_builder(ColumnWidth::Percent(1.0 / self.checkboxes.len() as f32));

        for (index, item) in self.checkboxes.iter().enumerate() {
            let width = next_builder.measure_text(&item, &self.font).0
                + self.font.font_size()
                + Pt(4.0) // Gap between box and text
                + self.space_between_checkboxes;

            let (this_side, next_side) =
                next_builder.generate_column_builder(ColumnWidth::Fixed(Mm::from(width)));

            let (mut box_builder, mut text_builder) = this_side.generate_column_builder(
                ColumnWidth::Fixed(Mm::from(self.font.font_size() + Pt(4.0))),
            );

            box_builder.draw_rect(self.font.font_size());

            text_builder.push_paragraph(item.as_str(), &self.font);

            group_builder.merge(box_builder);
            group_builder.merge(text_builder);

            next_builder = next_side;
        }

        group_builder.advance_cursor(self.calculate_height(builder).into_pt());
        let cursor = group_builder.cursor.y;

        builder.merge(group_builder);
        builder.update_cursor(cursor);
    }
}
