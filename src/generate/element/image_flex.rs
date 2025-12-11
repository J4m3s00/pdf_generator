use printpdf::{Op, Pt, Px, XObject, XObjectTransform};

use crate::generate::{
    document::DocumentStyle,
    element::{Element, image::Image},
};

pub struct ImageFlex {
    children: Vec<Image>,
}

impl ImageFlex {
    pub fn new() -> Self {
        ImageFlex {
            children: Vec::new(),
        }
    }

    pub fn push(&mut self, image: Image) {
        self.children.push(image);
    }
}

impl Element for ImageFlex {
    fn build(
        &self,
        document: &printpdf::PdfDocument,
        origin: printpdf::Point,
        max_width: Option<printpdf::Mm>,
        page_style: &DocumentStyle,
    ) -> super::BuildResult {
        let mut next_pointer = origin.clone();
        let max_width = max_width.unwrap_or(page_style.inner_width());
        let mut ops = vec![];
        let mut max_height = Pt(0.0);

        for render_image in &self.children {
            let XObject::Image(image) = document
                .resources
                .xobjects
                .map
                .get(&render_image.image)
                .expect("Image not found in document resources")
            else {
                panic!("Expected XObject to be an Image");
            };

            let width = Px(image.width).into_pt(300.0);
            let height = Px(image.height).into_pt(300.0);

            // Check if we overflow

            let scale = render_image
                .desired_width
                .map(|desired_width| desired_width.into_pt() / width);

            let final_height = scale.map(|scale| height * scale).unwrap_or(height);

            let transform = XObjectTransform {
                translate_x: Some(next_pointer.x),
                translate_y: Some(next_pointer.y - final_height),
                scale_x: scale,
                scale_y: scale,
                ..Default::default()
            };

            ops.push(Op::UseXobject {
                id: render_image.image.clone(),
                transform,
            });
        }

        todo!()
    }
}
