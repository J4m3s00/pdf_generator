use pdf_generator::generate::{TOPOL_OTF, document_builder::DocumentBuilder};

fn main() {
    let mut doc = DocumentBuilder::new("Test").build();

    let font = doc.add_font(TOPOL_OTF);
}
