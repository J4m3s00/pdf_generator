use printpdf::{
    FontId, Mm, Op, ParsedFont, PdfDocument, PdfPage, Point, Pt, Px, RawImage, XObjectId,
    XObjectTransform,
};

use crate::generate::{
    element::{BuildResult, Element},
    padding::Padding,
};

#[derive(Clone, Debug)]
pub struct DocumentStyle {
    pub padding: Padding,
    pub width: Mm,
    pub height: Mm,
}

impl DocumentStyle {
    pub fn inner_width(&self) -> Mm {
        self.width - self.padding.left - self.padding.right
    }
}

struct DocumentImage {
    xobject_id: XObjectId,
    position: Point,
}

pub struct Document {
    pdf_document: PdfDocument,

    pub elements: Vec<Box<dyn Element>>,
    style: DocumentStyle,

    footer_img: Option<DocumentImage>,
    header_img: Option<DocumentImage>,

    default_font: Option<FontId>,
}

impl Document {
    pub fn new(name: &str, width: Mm, height: Mm, padding: Padding) -> Self {
        Document {
            pdf_document: PdfDocument::new(name),
            elements: Vec::new(),
            style: DocumentStyle {
                padding,
                width,
                height,
            },
            footer_img: None,
            header_img: None,
            default_font: None,
        }
    }

    pub fn push<E>(&mut self, element: E)
    where
        E: Element + 'static,
    {
        self.elements.push(Box::new(element));
    }

    pub fn push_box(&mut self, element: Box<dyn Element>) {
        self.elements.push(element);
    }

    pub fn add_font(&mut self, font_data: &[u8]) -> FontId {
        let res = self
            .pdf_document
            .add_font(&ParsedFont::from_bytes(font_data, 0, &mut Vec::new()).unwrap());

        if self.default_font.is_none() {
            self.default_font = Some(res.clone());
        }

        res
    }

    pub fn get_default_font(&self) -> FontId {
        self.default_font
            .clone()
            .expect("Default font not set. Please add a font using `add_font` method.")
    }

    pub fn set_header_image(&mut self, image_data: &[u8]) {
        let raw_image = RawImage::decode_from_bytes(image_data, &mut Vec::new()).unwrap();

        let height_pt = Px(raw_image.height).into_pt(300.0);

        let header = self.pdf_document.add_image(&raw_image);

        self.header_img = Some(DocumentImage {
            xobject_id: header,
            position: Point {
                x: Pt(0.0),
                y: Pt::from(self.style.height) - height_pt,
            },
        });
    }

    pub fn add_image(&mut self, image: RawImage) -> XObjectId {
        self.pdf_document.add_image(&image)
    }

    pub fn set_footer_image(&mut self, image_data: &[u8]) {
        let raw_image = RawImage::decode_from_bytes(image_data, &mut Vec::new()).unwrap();

        let header = self.pdf_document.add_image(&raw_image);

        self.footer_img = Some(DocumentImage {
            xobject_id: header,
            position: Point {
                x: Pt(0.0),
                y: Pt(0.0),
            },
        });
    }

    pub fn generate_document(mut self) -> PdfDocument {
        let start_origin = printpdf::Point::new(
            self.style.padding.left,
            self.style.height - self.style.padding.top,
        );

        let inner_width = self.style.inner_width();

        let mut pages = Vec::new();
        let mut current_ops = Vec::new();
        current_ops.extend(self.generate_header_ops());
        current_ops.extend(self.generate_footer_ops());

        let mut current_origin = start_origin;
        for element in &self.elements {
            let mut element_ops = element.build(
                &self.pdf_document,
                current_origin,
                Some(inner_width),
                &self.style,
            );

            if element_ops.next_cursor.y < self.style.padding.bottom.into_pt() {
                pages.push(PdfPage::new(
                    self.style.width,
                    self.style.height,
                    current_ops.clone(),
                ));
                current_ops.clear();
                current_ops.extend(self.generate_footer_ops());

                current_origin = start_origin;

                element_ops = element.build(
                    &self.pdf_document,
                    current_origin,
                    Some(inner_width),
                    &self.style,
                );
            }

            current_ops.extend(element_ops.ops);

            current_origin = element_ops.next_cursor;
        }

        pages.push(PdfPage::new(
            self.style.width,
            self.style.height,
            current_ops,
        ));

        self.pdf_document.with_pages(pages);
        self.pdf_document
    }

    fn generate_header_ops(&self) -> Vec<Op> {
        if let Some(header) = &self.header_img {
            vec![Op::UseXobject {
                id: header.xobject_id.clone(),
                transform: XObjectTransform {
                    translate_x: Some(header.position.x),
                    translate_y: Some(header.position.y),
                    ..Default::default()
                },
            }]
        } else {
            Vec::new()
        }
    }

    fn generate_footer_ops(&self) -> Vec<Op> {
        if let Some(footer) = &self.footer_img {
            vec![Op::UseXobject {
                id: footer.xobject_id.clone(),
                transform: XObjectTransform {
                    translate_x: Some(footer.position.x),
                    translate_y: Some(footer.position.y),
                    ..Default::default()
                },
            }]
        } else {
            Vec::new()
        }
    }
}

impl Element for Document {
    fn build(
        &self,
        document: &PdfDocument,
        mut origin: Point,
        max_width: Option<Mm>,
        document_style: &DocumentStyle,
    ) -> BuildResult {
        let mut ops = Vec::new();
        for element in &self.elements {
            let element_ops = element.build(document, origin, max_width, document_style);
            ops.extend(element_ops.ops);

            origin = element_ops.next_cursor;
        }
        BuildResult {
            ops,
            next_cursor: origin,
            width: self.style.width,
        }
    }
}
