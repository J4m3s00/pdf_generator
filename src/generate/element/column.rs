use printpdf::{Mm, PdfDocument, Point};

use crate::generate::{
    document::DocumentStyle,
    element::{BuildResult, Element2, element_builder::ColumnWidth},
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
    fn build<'a>(&self, builder: &mut super::element_builder::ElementBuilder<'a>) {
        let (mut left_builder, mut right_builder) = builder.push_column(self.left_width);
        self.left.build(&mut left_builder);
        self.right.build(&mut right_builder);
        builder.merge(left_builder);
        builder.merge(right_builder);
    }
}
