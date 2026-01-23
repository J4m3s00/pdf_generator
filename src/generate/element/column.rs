use printpdf::Mm;

use crate::generate::{
    element::{Element2, element_builder::ColumnWidth},
    text_gen::LEFT_WIDTH,
};

pub enum LeftWidth {
    Fixed(Mm),
    Auto,
}

pub struct Column {
    pub right: Box<dyn Element2>,
    pub left: Box<dyn Element2>,
    pub left_width: ColumnWidth,
}

impl Column {
    pub fn new<Left, Right>(left: Left, right: Right) -> Self
    where
        Left: Element2 + 'static,
        Right: Element2 + 'static,
    {
        Column {
            left_width: ColumnWidth::Fixed(LEFT_WIDTH),
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn new_with_box(left: Box<dyn Element2>, right: Box<dyn Element2>) -> Self {
        Self {
            left,
            right,
            left_width: ColumnWidth::Fixed(LEFT_WIDTH),
        }
    }

    pub fn with_left_width(mut self, width: ColumnWidth) -> Self {
        self.left_width = width;
        self
    }
}

impl Element2 for Column {
    fn display_name(&self) -> &str {
        "Column"
    }

    fn calculate_width<'a>(&self, builder: &super::element_builder::ElementBuilder<'a>) -> Mm {
        builder.remaining_width_from_cursor()
    }
    fn calculate_height<'a>(&self, builder: &super::element_builder::ElementBuilder<'a>) -> Mm {
        let (left_builder, right_builder) = builder.generate_column_builder(self.left_width);

        let left_height = self.left.calculate_height(&left_builder);
        let right_height = self.right.calculate_height(&right_builder);

        left_height.max(right_height)
    }

    fn build<'a>(&self, builder: &mut super::element_builder::ElementBuilder<'a>) {
        let (mut left_builder, mut right_builder) =
            builder.generate_column_builder(self.left_width);
        self.left.build(&mut left_builder);
        self.right.build(&mut right_builder);

        let new_y = if left_builder.pages.len() == right_builder.pages.len() {
            left_builder.cursor.y.min(right_builder.cursor.y)
        } else if left_builder.pages.len() > right_builder.pages.len() {
            left_builder.cursor.y
        } else {
            right_builder.cursor.y
        };

        builder.merge(left_builder);
        builder.merge(right_builder);
        builder.update_cursor(new_y);
    }
}
