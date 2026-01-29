use printpdf::{Mm, Pt};

use crate::generate::{document::Document, padding::Padding};

pub enum DocumentFormat {
    A4,
}

pub enum DocumentOrientation {
    Portrait,
    Landscape,
}

pub struct DocumentBuilder {
    name: String,
    format: DocumentFormat,
    orientation: DocumentOrientation,
    padding: Padding,
    default_font_size: Pt,
    default_font_height_offset: Pt,
}

impl DocumentBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            format: DocumentFormat::A4,
            orientation: DocumentOrientation::Portrait,
            padding: Padding::xy(Mm(20.0), Mm(15.0)),
            default_font_size: Pt(9.0),
            default_font_height_offset: Pt(3.9),
        }
    }

    pub fn format(mut self, format: DocumentFormat) -> Self {
        self.format = format;
        self
    }

    pub fn orientation(mut self, orientation: DocumentOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    pub fn default_font_size(mut self, size: Pt) -> Self {
        self.default_font_size = size;
        self
    }

    pub fn default_font_height_offset(mut self, offset: Pt) -> Self {
        self.default_font_height_offset = offset;
        self
    }

    pub fn build(self) -> Document {
        let (width, height) = self.orientation.dimensions(self.format.dimensions());
        Document::new(
            &self.name,
            width,
            height,
            self.padding,
            self.default_font_size,
            self.default_font_height_offset,
        )
    }
}

impl DocumentFormat {
    fn dimensions(&self) -> (Mm, Mm) {
        match self {
            DocumentFormat::A4 => (Mm(210.0), Mm(297.0)),
        }
    }
}

impl DocumentOrientation {
    fn dimensions(&self, size: (Mm, Mm)) -> (Mm, Mm) {
        match self {
            DocumentOrientation::Portrait => (size.0, size.1),
            DocumentOrientation::Landscape => (size.1, size.0),
        }
    }
}
