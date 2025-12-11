use printpdf::{Mm, PdfDocument, Point};

use crate::generate::{
    document::DocumentStyle,
    element::{BuildResult, Element},
    text_gen::LEFT_WIDTH,
};

pub enum LeftWidth {
    Fixed(Mm),
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
}

impl Element for Column {
    fn build(
        &self,
        document: &PdfDocument,
        origin: printpdf::Point,
        max_width: Option<Mm>,
        page_style: &DocumentStyle,
    ) -> BuildResult {
        let left_width = match self.left_width {
            LeftWidth::Fixed(width) => Some(width),
            LeftWidth::Auto => None,
        };

        let left_text = self.left.build(document, origin, left_width, page_style);

        let left_width = left_width.unwrap_or(left_text.width);

        let right_origin = printpdf::Point {
            x: origin.x + left_width.into_pt(),
            y: origin.y,
        };

        let right_text = self.right.build(
            document,
            right_origin,
            Some(max_width.unwrap_or(page_style.inner_width()) - left_width),
            page_style,
        );

        let mut ops = left_text.ops;
        ops.extend(right_text.ops);

        let next_cursor = Point {
            x: origin.x,
            y: left_text.next_cursor.y.min(right_text.next_cursor.y),
        };

        BuildResult {
            ops,
            next_cursor,
            width: left_width + right_text.width,
        }
    }
}
