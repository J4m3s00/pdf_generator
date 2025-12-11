use printpdf::{Mm, Op, PdfDocument, Point};

use crate::generate::document::DocumentStyle;

pub mod checkbox_group;
pub mod column;
pub mod cursor_offset;
pub mod custom;
pub mod empty;
pub mod group;
pub mod image;
pub mod image_flex;
pub mod line;
pub mod paragraph;

pub struct BuildResult {
    pub ops: Vec<Op>,
    pub next_cursor: Point,
    pub width: Mm,
}

pub trait Element {
    fn build(
        &self,
        document: &PdfDocument,
        origin: Point,
        max_width: Option<Mm>,
        page_style: &DocumentStyle,
    ) -> BuildResult;
}
