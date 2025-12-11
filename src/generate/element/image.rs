use printpdf::{Mm, Op, Point, Px, XObject, XObjectId, XObjectTransform};

use crate::generate::{document::DocumentStyle, element::Element};

pub struct Image {
    pub image: XObjectId,
    pub desired_width: Option<Mm>,
}

impl Image {
    pub fn new(image: XObjectId, desired_width: Option<Mm>) -> Self {
        Image {
            image,
            desired_width,
        }
    }
}

impl Element for Image {
    fn build(
        &self,
        document: &printpdf::PdfDocument,
        origin: printpdf::Point,
        _max_width: Option<printpdf::Mm>,
        _page_style: &DocumentStyle,
    ) -> super::BuildResult {
        let XObject::Image(image) = document
            .resources
            .xobjects
            .map
            .get(&self.image)
            .expect("Image not found in document resources")
        else {
            panic!("Expected XObject to be an Image");
        };

        let width = Px(image.width).into_pt(300.0);
        let height = Px(image.height).into_pt(300.0);

        let scale = self
            .desired_width
            .map(|desired_width| desired_width.into_pt() / width);

        let final_height = scale.map(|scale| height * scale).unwrap_or(height);

        let transform = XObjectTransform {
            translate_x: Some(origin.x),
            translate_y: Some(origin.y - final_height),
            scale_x: scale,
            scale_y: scale,
            ..Default::default()
        };

        let ops = vec![Op::UseXobject {
            id: self.image.clone(),
            transform,
        }];

        super::BuildResult {
            ops,
            next_cursor: Point {
                x: origin.x,
                y: origin.y - final_height,
            },
            width: printpdf::Mm::from(width),
        }
    }
}
