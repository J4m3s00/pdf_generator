use printpdf::{FontId, Mm, PdfDocument, Pt, ShapedText, TextShapingOptions};

pub const DEFAULT_FONT_SIZE: Pt = Pt(9.0);
pub const DEFAULT_FONT_LINE_HEIGHT_OFFSET: Pt = Pt(3.9);

pub const LEFT_WIDTH: Mm = Mm(50.0);

fn space_between_newlines(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut prev_was_nl = false;

    for ch in input.chars() {
        if ch == '\n' && prev_was_nl {
            result.push(' ');
        }
        result.push(ch);
        prev_was_nl = ch == '\n';
    }

    result
}

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

    // We need to add a space between two new lines to avoid not showing the second new line.
    let formated_text = space_between_newlines(text);

    let shaping_options = TextShapingOptions {
        font_size,
        //line_height: Some(Pt(font_size.0 + font_height_offset)),
        line_height: Some(font_size + font_height_offset),
        max_width: max_width.map(Pt::from),
        ..Default::default()
    };

    let parsed_font = doc.resources.fonts.map.get(&font).unwrap();

    let space_width = parsed_font.get_space_width().unwrap_or_default() as f32
        / parsed_font.font_metrics.units_per_em as f32
        * font_size.0;

    let spaces_at_end = formated_text
        .chars()
        .rev()
        .take_while(|&c| c == ' ')
        .count() as f32;

    let mut shaped_text = doc
        .shape_text(&formated_text, &font, &shaping_options)
        .unwrap();

    shaped_text.height = shaped_text.lines.len() as f32 * (font_size + font_height_offset).0;
    // Dividing by two makes it better. Cant figure out how to calculate it properly.
    shaped_text.width += (space_width * spaces_at_end) / 2.0;

    shaped_text
}

/// This will cut the shaped text to the given max height.
/// We will cut only once, since the next max_height could be different.
pub fn split_shaped_text(
    mut text: ShapedText,
    font_size: Pt,
    font_height_offset: Pt,
    max_height: Mm,
) -> (ShapedText, Option<ShapedText>) {
    let max_height_pt = max_height.into_pt();
    let fit_lines = (max_height_pt / (font_size + font_height_offset)) as usize;

    if fit_lines >= text.lines.len() {
        return (text, None);
    }

    let first_height = fit_lines as f32 * (font_size + font_height_offset).0;
    let rest_height = text.height - first_height;

    let rest_lines = text.lines.split_off(fit_lines);
    text.height = first_height;

    let rest = ShapedText {
        font_id: text.font_id.clone(),
        options: text.options.clone(),
        lines: rest_lines,
        width: text.width,
        height: rest_height,
    };

    (text, Some(rest))
}
