use printpdf::{FontId, Mm, PdfDocument, Pt, ShapedText, TextShapingOptions};

pub const DEFAULT_FONT_SIZE: Pt = Pt(9.0);
pub const DEFAULT_FONT_LINE_HEIGHT_OFFSET: Pt = Pt(3.9);

pub const LEFT_WIDTH: Mm = Mm(50.0);

pub fn shape_text(
    doc: &PdfDocument,
    font: FontId,
    font_size: Pt,
    font_height_offset: Pt,
    text: &str,
    max_width: Option<Mm>,
) -> ShapedText {
    if !doc.resources.fonts.map.contains_key(&font) {
        panic!("Font resource not found for font ID: {:?}", font);
    }

    let shaping_options = TextShapingOptions {
        font_size,
        //line_height: Some(Pt(font_size.0 + font_height_offset)),
        line_height: Some(font_size + font_height_offset),
        max_width: max_width.map(Pt::from),
        ..Default::default()
    };
    let mut shaped_text = doc.shape_text(text, &font, &shaping_options).unwrap();
    shaped_text.height = shaped_text.lines.len() as f32 * (font_size + font_height_offset).0;
    shaped_text
}
