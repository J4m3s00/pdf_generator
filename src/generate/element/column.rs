use printpdf::{Mm, Pt};

use crate::generate::element::{Element, element_builder::ColumnWidth};

pub const LEFT_WIDTH: Mm = Mm(50.0);

pub enum LeftWidth {
    Percent(f32),
    Fixed(Mm),
    /// Calculate the width of the left column based on its content, but ensure that the right
    /// column has at least 50mm of space.
    Auto,
}

pub struct Column {
    pub right: Box<dyn Element>,
    pub left: Box<dyn Element>,
    pub left_width: LeftWidth,
}

impl Column {
    pub fn new<Left, Right>(left: Left, right: Right) -> Self
    where
        Left: Element + 'static,
        Right: Element + 'static,
    {
        Column {
            left_width: LeftWidth::Fixed(LEFT_WIDTH),
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn new_with_box(left: Box<dyn Element>, right: Box<dyn Element>) -> Self {
        Self {
            left,
            right,
            left_width: LeftWidth::Fixed(LEFT_WIDTH),
        }
    }

    pub fn with_left_width(mut self, width: LeftWidth) -> Self {
        self.left_width = width;
        self
    }

    fn widht_to_column_width<'a>(
        &self,
        builder: &super::element_builder::ElementBuilder<'a>,
    ) -> ColumnWidth {
        match self.left_width {
            LeftWidth::Percent(p) => ColumnWidth::Percent(p),
            LeftWidth::Fixed(mm) => ColumnWidth::Fixed(mm),
            LeftWidth::Auto => {
                let left_width = self
                    .left
                    .calculate_width(builder)
                    .min(builder.remaining_width_from_cursor() - Pt(150.0));

                ColumnWidth::Fixed(Mm::from(left_width))
            }
        }
    }
}

impl Element for Column {
    fn display_name(&self) -> &str {
        "Column"
    }

    fn calculate_width<'a>(&self, builder: &super::element_builder::ElementBuilder<'a>) -> Pt {
        builder.remaining_width_from_cursor()
    }
    fn calculate_height<'a>(&self, builder: &super::element_builder::ElementBuilder<'a>) -> Pt {
        let column_width = self.widht_to_column_width(builder);
        let (left_builder, right_builder) = builder.generate_column_builder(column_width);

        let left_height = self.left.calculate_height(&left_builder);
        let right_height = self.right.calculate_height(&right_builder);

        left_height.max(right_height)
    }

    fn build<'a>(&self, builder: &mut super::element_builder::ElementBuilder<'a>) {
        let column_width = self.widht_to_column_width(builder);

        let (mut left_builder, mut right_builder) = builder.generate_column_builder(column_width);
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
