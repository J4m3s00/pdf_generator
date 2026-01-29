use std::io;

use pdf_generator::generate::{
    document_builder::{DocumentBuilder, DocumentFormat, DocumentOrientation},
    element::paragraph::Paragraph,
    padding::Padding,
};
use printpdf::Mm;

const ROBOTO_FILE: &[u8] = include_bytes!("../Roboto/Roboto-VariableFont_wdth,wght.ttf");

fn main() -> io::Result<()> {
    let mut doc = DocumentBuilder::new("Basic")
        .format(DocumentFormat::A4)
        .orientation(DocumentOrientation::Portrait)
        .padding(Padding::xy(Mm(20.0), Mm(15.0)))
        .build();

    let font = doc.add_font(ROBOTO_FILE, None, None)?;

    doc.push(Paragraph::new("This is some text", font.clone()));

    doc.save_to_disk("./")?;

    Ok(())
}
