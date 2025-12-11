pub mod document;
pub mod document_builder;
pub mod element;
pub mod outline;
pub mod padding;
pub mod text_gen;

pub static TOPOL_OTF: &[u8] = include_bytes!("./Topol Bold.ttf");
pub static NEUE_HAAS_GROTESK_OTF: &[u8] = include_bytes!("./NHaasGroteskTXPro-55Rg.ttf");
pub static HEADER_BYTES: &[u8] = include_bytes!("./Header.png");
pub static FOOTER_BYTES: &[u8] = include_bytes!("./Footer.png");
