use printpdf::Mm;

use crate::generate::{
    element::{Element, image::Image},
    padding::Padding,
};

pub struct ImageFlex {
    children: Vec<Image>,
    space_x: Mm,
    space_y: Mm,
}

impl ImageFlex {
    pub fn new() -> Self {
        ImageFlex {
            children: Vec::new(),
            space_x: Mm(0.0),
            space_y: Mm(0.0),
        }
    }

    pub fn with_space_x(mut self, gap: Mm) -> Self {
        self.space_x = gap;
        self
    }

    pub fn with_space_y(mut self, gap: Mm) -> Self {
        self.space_y = gap;
        self
    }

    pub fn push(&mut self, image: Image) {
        self.children.push(image);
    }
}

impl Element for ImageFlex {
    fn display_name(&self) -> &str {
        "Image Flex"
    }

    fn calculate_width<'a>(&self, builder: &super::element_builder::ElementBuilder<'a>) -> Mm {
        self.children
            .iter()
            .fold(Mm(0.0), |width, elem| elem.calculate_width(builder) + width)
            .min(builder.remaining_width_from_cursor())
    }

    fn calculate_height<'a>(
        &self,
        builder: &super::element_builder::ElementBuilder<'a>,
    ) -> printpdf::Mm {
        // Create a group builder to calculate the height of children
        let group_builder = builder.generate_group_builder(&Padding::none(), None);
        group_builder.calculate_flex_height(
            self.children.iter().map(Box::new),
            self.space_x,
            self.space_y,
        )
    }

    fn build<'a>(&self, builder: &mut super::element_builder::ElementBuilder<'a>) {
        builder.push_flex(
            self.children.iter().map(Box::new),
            self.space_x,
            self.space_y,
        );
    }
}
// impl Element for ImageFlex {
//     fn build(
//         &self,
//         document: &printpdf::PdfDocument,
//         origin: printpdf::Point,
//         max_width: Option<printpdf::Mm>,
//         page_style: &DocumentStyle,
//     ) -> super::BuildResult {
//         let mut next_pointer = origin.clone();
//         let max_width = max_width.unwrap_or(page_style.inner_width());
//         let mut ops = vec![];
//         let mut max_height = Pt(0.0);
//
//         for render_image in &self.children {
//             let XObject::Image(image) = document
//                 .resources
//                 .xobjects
//                 .map
//                 .get(&render_image.image)
//                 .expect("Image not found in document resources")
//             else {
//                 panic!("Expected XObject to be an Image");
//             };
//
//             let width = Px(image.width).into_pt(300.0);
//             let height = Px(image.height).into_pt(300.0);
//
//             // Check if we overflow
//
//             let scale = render_image
//                 .desired_width
//                 .map(|desired_width| desired_width.into_pt() / width);
//
//             let final_height = scale.map(|scale| height * scale).unwrap_or(height);
//
//             let transform = XObjectTransform {
//                 translate_x: Some(next_pointer.x),
//                 translate_y: Some(next_pointer.y - final_height),
//                 scale_x: scale,
//                 scale_y: scale,
//                 ..Default::default()
//             };
//
//             ops.push(Op::UseXobject {
//                 id: render_image.image.clone(),
//                 transform,
//             });
//         }
//
//         todo!()
//     }
// }
