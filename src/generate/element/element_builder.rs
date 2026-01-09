use std::collections::VecDeque;

use printpdf::{FontId, Mm, Op, Point, Pt, ShapedText};

use crate::generate::document::Document;
use crate::generate::text_gen::{shape_text, split_shaped_text};

#[derive(Debug, Default)]
pub enum MoveDirection {
    Right,
    #[default]
    Down,
}

#[derive(Debug, Clone, Copy)]
pub enum ColumnWidth {
    Fixed(Mm),
    Percent(f32),
}

pub struct ElementBuilder<'a> {
    document: &'a Document,
    origin: Point,
    cursor: Point,
    remaining_width: Mm,
    starting_page: usize,
    pub pages: Vec<Vec<Op>>,
}

impl<'a> ElementBuilder<'a> {
    pub fn new(document: &'a Document) -> Self {
        let style = document.style().clone();
        let origin = Point {
            x: style.padding.left.into_pt(),
            y: (style.height - style.padding.top).into_pt(),
        };

        Self {
            document,
            origin,
            cursor: origin,
            remaining_width: style.inner_width(),
            starting_page: 0,
            pages: vec![Vec::new()],
        }
    }
}

impl<'a> ElementBuilder<'a> {
    pub fn push_paragraph(
        &mut self,
        paragraph: &str,
        font: FontId,
        font_size: Pt,
        font_height_offset: Pt,
    ) {
        let shaped_text = shape_text(
            self.document.pdf_document(),
            font,
            font_size,
            font_height_offset,
            paragraph,
            Some(self.remaining_width_from_cursor()),
        );

        self.push_shaped_text(shaped_text, font_size, font_height_offset);
    }

    fn push_shaped_text(&mut self, text: ShapedText, font_size: Pt, font_height_offset: Pt) {
        // Do we need to cut the text?
        let (first, rest) = split_shaped_text(
            text,
            font_size,
            font_height_offset,
            self.remaining_height_from_cursor(),
        );

        let ops = first.get_ops(self.cursor);
        self.pages
            .last_mut()
            .expect("We always have one page")
            .extend(ops.into_iter());
        self.cursor.y -= Pt(first.height);

        if let Some(rest) = rest {
            self.next_page();
            self.push_shaped_text(rest, font_size, font_height_offset);
        }
    }

    pub fn push_column(&mut self, width: ColumnWidth) -> (ElementBuilder<'a>, ElementBuilder<'a>) {
        let left_width = match width {
            ColumnWidth::Fixed(mm) => mm,
            ColumnWidth::Percent(fr) => self.remaining_width_from_cursor() * fr,
        };
        let right_width = self.remaining_width_from_cursor() - left_width;
        let left_builder = ElementBuilder {
            document: self.document,
            origin: self.cursor,
            cursor: self.cursor,
            remaining_width: left_width,
            starting_page: self.pages.len() - 1,
            pages: vec![Vec::new()],
        };
        let right_origin = Point {
            x: self.cursor.x + left_width.into_pt(),
            y: self.cursor.y,
        };

        let right_builder = ElementBuilder {
            document: self.document,
            origin: right_origin,
            cursor: right_origin,
            remaining_width: right_width,
            starting_page: self.pages.len() - 1,
            pages: vec![Vec::new()],
        };

        (left_builder, right_builder)
    }

    fn remaining_height_from_cursor(&self) -> Mm {
        Mm::from(self.cursor.y) - self.document.style().padding.bottom
    }

    fn remaining_width_from_cursor(&self) -> Mm {
        let x_offset = Mm::from(self.cursor.x - self.origin.x);
        self.remaining_width - x_offset
    }

    /// Updates the origin and cursor to the initial position
    /// Recalculates remaining_height
    /// remaining_width stays the same
    fn next_page(&mut self) {
        let style = self.document.style().clone();
        let origin = Point {
            x: self.origin.x,
            y: (style.height - style.padding.top).into_pt(),
        };

        self.origin = origin;
        self.cursor = origin;
        self.pages.push(Vec::new());
    }

    pub fn merge(&mut self, other: ElementBuilder) {
        let mut dequeue = VecDeque::from(other.pages);

        for p in self.pages[other.starting_page..].iter_mut() {
            if let Some(extend) = dequeue.pop_front() {
                p.extend(extend);
            } else {
                break;
            }
        }

        while let Some(next) = dequeue.pop_front() {
            self.pages.push(next);
        }
    }
}
