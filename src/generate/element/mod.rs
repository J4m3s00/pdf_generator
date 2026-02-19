use printpdf::{Mm, Op, Point};

use crate::generate::element::element_builder::ElementBuilder;

pub mod checkbox_group;
pub mod column;
pub mod cursor_offset;
pub mod element_builder;
pub mod empty;
pub mod group;
pub mod image;
pub mod image_flex;
pub mod line;
pub mod paragraph;
pub mod rich_text;
pub mod table;

pub struct BuildResult {
    pub ops: Vec<Op>,
    pub next_cursor: Point,
    pub width: Mm,
}

pub trait Element {
    fn display_name(&self) -> &str;

    fn calculate_width<'a>(&self, builder: &ElementBuilder<'a>) -> Mm;
    fn calculate_height<'a>(&self, builder: &ElementBuilder<'a>) -> Mm;
    fn build<'a>(&self, builder: &mut ElementBuilder<'a>);
}
