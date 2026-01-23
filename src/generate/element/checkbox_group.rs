use printpdf::{FontId, Mm, Pt};

use crate::generate::{
    element::{Element, element_builder::ColumnWidth},
    padding::Padding,
    text_gen::{DEFAULT_FONT_LINE_HEIGHT_OFFSET, DEFAULT_FONT_SIZE},
};

pub struct CheckboxGroup {
    pub checkboxes: Vec<String>,

    space_between_checkboxes: Pt,

    font: FontId,
    font_size: Pt,
    font_height_offset: Pt,
}

impl CheckboxGroup {
    pub fn new(checkboxes: Vec<String>, font: FontId) -> Self {
        Self {
            checkboxes,
            font,
            space_between_checkboxes: Pt(7.5), // Default space between checkboxes
            font_size: DEFAULT_FONT_SIZE,
            font_height_offset: DEFAULT_FONT_LINE_HEIGHT_OFFSET,
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
        builder.remaining_width_from_cursor()
    }

    fn calculate_height<'a>(&self, builder: &super::element_builder::ElementBuilder<'a>) -> Mm {
        let (mut left, mut right) = builder
            .generate_column_builder(ColumnWidth::Percent(1.0 / self.checkboxes.len() as f32));

        self.checkboxes
            .iter()
            .enumerate()
            .fold(Mm(0.0), move |v, (index, item)| {
                let (_, text_builder) = left.generate_column_builder(ColumnWidth::Fixed(Mm::from(
                    self.font_size + Pt(4.0),
                )));

                let height = self.font_size.max(
                    text_builder
                        .measure_text(
                            item.as_str(),
                            self.font.clone(),
                            self.font_size,
                            self.font_height_offset,
                        )
                        .1,
                );

                let column_width =
                    ColumnWidth::Percent(1.0 / (self.checkboxes.len() - index - 1) as f32);
                let (new_left, new_right) = right.generate_column_builder(column_width);

                left = new_left;
                right = new_right;

                v.max(Mm::from(height))
            })
    }

    fn build<'a>(&self, builder: &mut super::element_builder::ElementBuilder<'a>) {
        // Create a group builder, to have the checkboxes in a group
        // When the checkbox_group is at the bottom of the page, and one of the text has multiple
        // lines and pushes to the next page, we want the whole checkbox_group to be on the next
        // page if possible
        let mut group_builder =
            builder.generate_group_builder(&Padding::none(), Some(self.calculate_height(builder)));

        let (mut left, mut right) = group_builder
            .generate_column_builder(ColumnWidth::Percent(1.0 / self.checkboxes.len() as f32));

        for (index, item) in self.checkboxes.iter().enumerate() {
            let (mut box_builder, mut text_builder) = left
                .generate_column_builder(ColumnWidth::Fixed(Mm::from(self.font_size + Pt(4.0))));

            box_builder.draw_rect(self.font_size);

            text_builder.push_paragraph(
                item.as_str(),
                self.font.clone(),
                self.font_size,
                self.font_height_offset,
            );

            let column_width =
                ColumnWidth::Percent(1.0 / (self.checkboxes.len() - index - 1) as f32);
            let (new_left, new_right) = right.generate_column_builder(column_width);

            group_builder.merge(box_builder);
            group_builder.merge(text_builder);

            left = new_left;
            right = new_right;
        }

        builder.merge(group_builder);

        builder.advance_cursor(self.calculate_height(builder).into_pt());
    }
}
