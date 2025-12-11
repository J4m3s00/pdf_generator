use printpdf::{FontId, Mm, Op, Point, Pt, TextItem};

use crate::generate::{
    document::DocumentStyle,
    element::{BuildResult, Element},
};

pub struct Custom {
    ops: Vec<Op>,
    position: Point,
}

impl Custom {
    pub fn new(position: Point) -> Self {
        Custom {
            ops: Vec::new(),
            position,
        }
    }

    pub fn write_text(&mut self, text: &str, font: FontId, font_size: Pt) {
        self.ops.push(Op::SetFontSize {
            size: font_size,
            font: font.clone(),
        });
        self.ops.push(Op::WriteText {
            items: vec![TextItem::Text(text.to_string())],
            font: font.clone(),
        });
    }
}

impl Element for Custom {
    fn build(
        &self,
        _document: &printpdf::PdfDocument,
        origin: printpdf::Point,
        _max_width: Option<printpdf::Mm>,
        _page_style: &DocumentStyle,
    ) -> super::BuildResult {
        let mut ops = Vec::new();

        ops.push(Op::StartTextSection);
        ops.push(Op::SetTextCursor { pos: self.position });
        ops.extend(self.ops.clone());
        ops.push(Op::EndTextSection);

        BuildResult {
            ops,
            next_cursor: origin,
            width: Mm(0.0),
        }
    }
}
