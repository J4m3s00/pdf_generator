use std::path::Path;

use pdf_generator::generate::{
    TOPOL_OTF,
    document_builder::DocumentBuilder,
    element::{column::Column, element_builder::ColumnWidth, paragraph::Paragraph},
};

fn main() {
    let mut doc = DocumentBuilder::new("Test").build();
    let font = doc.add_font(TOPOL_OTF);

    let left = Paragraph::new(include_str!("../../test.txt"), font.clone());
    let right = Paragraph::new(include_str!("../../test.txt"), font.clone());

    let col = Column::new(left, right).with_left_width(ColumnWidth::Percent(0.5));
    doc.push(col);

    let (data, warnings) = doc.save();

    for warn in warnings {
        println!("[{:?}] {}", warn.severity, warn.msg);
    }

    std::fs::write(Path::new("test.pdf"), data).expect("Failed to write file");
}
