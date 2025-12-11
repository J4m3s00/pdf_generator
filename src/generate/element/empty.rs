use printpdf::Mm;

use crate::generate::{document::DocumentStyle, element::Element};

pub struct Empty;

impl Element for Empty {
    fn build(
        &self,
        _document: &printpdf::PdfDocument,
        origin: printpdf::Point,
        _max_width: Option<printpdf::Mm>,
        _page_style: &DocumentStyle,
    ) -> super::BuildResult {
        super::BuildResult {
            ops: Vec::new(),
            next_cursor: origin,
            width: Mm(0.0),
        }
    }
}
