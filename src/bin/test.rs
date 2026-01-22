use std::path::Path;

use pdf_generator::generate::{
    TOPOL_OTF,
    document_builder::DocumentBuilder,
    element::{
        checkbox_group::CheckboxGroup,
        column::Column,
        cursor_offset::CursorOffset,
        element_builder::{ColumnWidth, ElementBuilder},
        group::Group,
        line::Line,
        paragraph::Paragraph,
    },
    outline::LineStyle,
    padding::Padding,
    text_gen::{DEFAULT_FONT_LINE_HEIGHT_OFFSET, DEFAULT_FONT_SIZE},
};
use printpdf::Mm;

fn main() {
    let mut doc = DocumentBuilder::new("Test").build();
    let font = doc.add_font(TOPOL_OTF);

    let new_line_paragraph = Paragraph::new(
        "First Line\n\n\nSecond line\nSomething else\nAnother one\nTest\nTest\nTest",
        font.clone(),
    );

    doc.push(new_line_paragraph);

    let mut group = Group::new()
        .with_try_keep_together(true)
        .with_outline(LineStyle::default())
        .with_padding(Padding::new(Mm(46.0), Mm(20.0), Mm(30.0), Mm(40.0)));

    group.push(Paragraph::new(
        include_str!("../../lorem_short.txt"),
        font.clone(),
    ));

    doc.push(group);

    let mut group = Group::new()
        .with_try_keep_together(true)
        .with_outline(LineStyle::default())
        .with_padding(Padding::new(Mm(10.0), Mm(20.0), Mm(30.0), Mm(40.0)));

    let mut inner_group = Group::new()
        .with_try_keep_together(true)
        .with_outline(LineStyle::default())
        .with_padding(Padding::none());

    inner_group.push(Paragraph::new(
        include_str!("../../lorem_short.txt"),
        font.clone(),
    ));

    group.push(inner_group);

    doc.push(group);

    // doc.push(CursorOffset::Relative(DEFAULT_FONT_LINE_HEIGHT_OFFSET));
    // doc.push(CursorOffset::line_breaks(3));

    let checkbox_group = CheckboxGroup::new(
        vec![
            "One asdl alskfjh alksjdhg aldjfgh sldkjgfh sldkjfgh kjshdf".to_string(),
            "Two".to_string(),
            "Three".to_string(),
        ],
        font.clone(),
    );

    doc.push(checkbox_group);

    let mut right = Group::new().with_padding(Padding::left(Mm(5.0)));
    right.push(Paragraph::new(
        include_str!("../../lorem_short.txt"),
        font.clone(),
    ));
    right.push(Line::new(LineStyle::default(), Padding::y(Mm(5.0))));
    let left = Paragraph::new(include_str!("../../lorem_short.txt"), font.clone());

    let col = Column::new(left, right).with_left_width(ColumnWidth::Percent(0.5));
    let mut col_group = Group::new().with_outline(LineStyle::default());
    col_group.push(col);

    doc.push(col_group);

    doc.push(Paragraph::new("Some text", font.clone()));

    let (data, warnings) = doc.save();

    for warn in warnings {
        println!("[{:?}] {}", warn.severity, warn.msg);
    }

    std::fs::write(Path::new("test.pdf"), data).expect("Failed to write file");
}
