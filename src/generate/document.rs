use std::{io, path::Path};

use printpdf::{
    FontId, ImageCompression, ImageOptimizationOptions, Mm, Op, ParsedFont, PdfDocument, PdfPage,
    PdfSaveOptions, PdfWarnMsg, Point, Pt, Px, RawImage, XObjectId, XObjectTransform,
};

use crate::generate::{
    element::{Element, element_builder::ElementBuilder, image::Image},
    font::Font,
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

    pub fn inner_height(&self) -> Mm {
        self.height - self.padding.top - self.padding.bottom
    }
}

struct DocumentImage {
    xobject_id: XObjectId,
    position: Point,
}

#[derive(Default)]
pub struct Page {
    ops: Vec<Op>,
}

impl Page {
    pub fn push(&mut self, op: Op) {
        self.ops.push(op);
    }

    pub fn extend(&mut self, iter: impl Iterator<Item = Op>) {
        self.ops.extend(iter);
    }
}

pub struct Document {
    pdf_document: PdfDocument,

    pub elements: Vec<Box<dyn Element>>,
    style: DocumentStyle,

    footer_img: Option<DocumentImage>,
    header_img: Option<DocumentImage>,

    default_font: Option<FontId>,

    default_font_size: Pt,
    default_font_height_offset: Pt,
}

impl Document {
    pub fn new(
        name: &str,
        width: Mm,
        height: Mm,
        padding: Padding,
        default_font_size: Pt,
        default_font_height_offset: Pt,
    ) -> Self {
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
            default_font_size,
            default_font_height_offset,
        }
    }

    pub fn style(&self) -> &DocumentStyle {
        &self.style
    }

    pub fn pdf_document(&self) -> &PdfDocument {
        &self.pdf_document
    }

    pub fn push<E>(&mut self, element: E)
    where
        E: Element + 'static,
    {
        self.elements.push(Box::new(element));
    }

    pub fn push_boxed(&mut self, element: Box<dyn Element>) {
        self.elements.push(element);
    }

    /// Loads and adds a new font
    ///
    /// If this is the first font added, it will be set as the default font
    pub fn add_font(
        &mut self,
        font_data: &[u8],
        custom_font_size: Option<Pt>,
        custom_font_height_offset: Option<Pt>,
    ) -> io::Result<Font> {
        let mut warnings = Vec::new();

        let Some(parsed_font) = ParsedFont::from_bytes(font_data, 0, &mut warnings) else {
            let message = warnings
                .into_iter()
                .map(|warn| format!("[{:?}] {}", warn.severity, warn.msg))
                .collect::<Vec<_>>()
                .join("\n");

            return Err(io::Error::new(io::ErrorKind::InvalidInput, message));
        };

        let res = self.pdf_document.add_font(&parsed_font);

        if self.default_font.is_none() {
            self.default_font = Some(res.clone());
        }

        Ok(Font::new(
            res,
            custom_font_size.unwrap_or(self.default_font_size),
            custom_font_height_offset.unwrap_or(self.default_font_height_offset),
        ))
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

    /// Saves the document to disk at the specified path.
    ///
    /// If the path is a directory, the document will be saved with its title as the filename.
    pub fn save_to_disk(self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        let output_path = if path.is_dir() {
            path.join(Path::new(&format!(
                "{}.pdf",
                &self.pdf_document.metadata.info.document_title
            )))
        } else {
            path.to_path_buf()
        };

        let (data, warnings) = self.save();

        for warn in warnings {
            println!("[{:?}] {}", warn.severity, warn.msg);
        }

        std::fs::write(output_path, data)
    }

    pub fn save(self) -> (Vec<u8>, Vec<PdfWarnMsg>) {
        let generated = self.generate_document();
        let mut warn_messages = Vec::new();
        let bytes = generated.save(
            &PdfSaveOptions {
                image_optimization: Some(ImageOptimizationOptions {
                    quality: Some(100.0),
                    format: Some(ImageCompression::Auto),
                    max_image_size: Some("10mb".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            &mut warn_messages,
        );

        (bytes, warn_messages)
    }

    pub fn generate_document(mut self) -> PdfDocument {
        // If we have a footer, reserve space for it
        if let Some(footer_img) = &self.footer_img {
            let measure_builder = ElementBuilder::new(&self);
            let img = Image::new(footer_img.xobject_id.clone(), Some(self.style.width));
            let (_, img_height) = measure_builder.measure_image(&img);
            self.style.padding.bottom += Mm::from(img_height);
        }

        let mut current_builder = ElementBuilder::new(&self);
        // Insert header image
        if let Some(header_image) = &self.header_img {
            let img = Image::new(header_image.xobject_id.clone(), Some(self.style.width));
            let (_, img_height) = current_builder.measure_image(&img);

            current_builder.advance_cursor(img_height);
            current_builder
                .pages
                .first_mut()
                .expect("We have at least one page")
                .extend(self.generate_header_ops());
        }
        let footer_ops = self.generate_footer_ops();

        for element in &self.elements {
            element.build(&mut current_builder);
        }

        let pages = current_builder
            .pages
            .into_iter()
            .map(|mut page| {
                page.extend_from_slice(&footer_ops);
                PdfPage::new(self.style.width, self.style.height, page)
            })
            .collect();

        self.pdf_document.with_pages(pages);

        self.pdf_document
    }
    // pub fn generate_document(mut self) -> PdfDocument {
    //     let start_origin = printpdf::Point::new(
    //         self.style.padding.left,
    //         self.style.height - self.style.padding.top,
    //     );
    //
    //     let inner_width = self.style.inner_width();
    //
    //     let mut pages = Vec::new();
    //     let mut current_ops = Vec::new();
    //     current_ops.extend(self.generate_header_ops());
    //     current_ops.extend(self.generate_footer_ops());
    //
    //     let mut current_origin = start_origin;
    //     for element in &self.elements {
    //         let mut element_ops = element.build(
    //             &self.pdf_document,
    //             current_origin,
    //             Some(inner_width),
    //             &self.style,
    //         );
    //
    //         if element_ops.next_cursor.y < self.style.padding.bottom.into_pt() {
    //             pages.push(PdfPage::new(
    //                 self.style.width,
    //                 self.style.height,
    //                 current_ops.clone(),
    //             ));
    //             current_ops.clear();
    //             current_ops.extend(self.generate_footer_ops());
    //
    //             current_origin = start_origin;
    //
    //             element_ops = element.build(
    //                 &self.pdf_document,
    //                 current_origin,
    //                 Some(inner_width),
    //                 &self.style,
    //             );
    //         }
    //
    //         current_ops.extend(element_ops.ops);
    //
    //         current_origin = element_ops.next_cursor;
    //     }
    //
    //     pages.push(PdfPage::new(
    //         self.style.width,
    //         self.style.height,
    //         current_ops,
    //     ));
    //
    //     self.pdf_document.with_pages(pages);
    //     self.pdf_document
    // }

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
