use printpdf::{LinePoint, Mm, Op, Point};

use crate::generate::{
    document::DocumentStyle,
    element::{BuildResult, Element, Element2},
    outline::LineStyle,
    padding::Padding,
};

pub struct Line {
    outline: LineStyle,
    padding: Padding,
}

impl Line {
    pub fn new(outline: LineStyle, padding: Padding) -> Self {
        Line { outline, padding }
    }
}

impl Element2 for Line {
    fn calculate_height<'a>(&self, _: &super::element_builder::ElementBuilder<'a>) -> Mm {
        Mm::from(self.outline.thickness) + self.padding.top + self.padding.bottom
    }

    fn build<'a>(&self, builder: &mut super::element_builder::ElementBuilder<'a>) {
        builder.draw_line(&self.padding, &self.outline);
    }
}

// impl Element for Line {
//     fn build(
//         &self,
//         _document: &printpdf::PdfDocument,
//         origin: printpdf::Point,
//         max_width: Option<printpdf::Mm>,
//         _page_style: &DocumentStyle,
//     ) -> super::BuildResult {
//         let width = max_width.unwrap_or(Mm(80.0));
//
//         let mut ops = Vec::new();
//
//         ops.push(Op::SaveGraphicsState);
//         ops.push(Op::SetOutlineColor {
//             col: printpdf::Color::Rgb(self.outline.color.clone()),
//         });
//         ops.push(Op::SetOutlineThickness {
//             pt: self.outline.thickness,
//         });
//
//         ops.push(Op::DrawLine {
//             line: printpdf::Line {
//                 points: vec![
//                     LinePoint {
//                         bezier: false,
//                         p: Point {
//                             x: origin.x + self.outline.padding.left.into_pt(),
//                             y: origin.y - self.outline.padding.top.into_pt(),
//                         },
//                     },
//                     LinePoint {
//                         bezier: false,
//                         p: Point {
//                             x: origin.x + width.into_pt() - self.outline.padding.right.into_pt(),
//                             y: origin.y - self.outline.padding.top.into_pt(),
//                         },
//                     },
//                 ],
//                 is_closed: false,
//             },
//         });
//
//         ops.push(Op::RestoreGraphicsState);
//
//         BuildResult {
//             ops,
//             next_cursor: printpdf::Point {
//                 x: origin.x,
//                 y: origin.y
//                     - (self.outline.padding.bottom.into_pt() + self.outline.padding.top.into_pt()),
//             },
//             width,
//         }
//     }
// }
