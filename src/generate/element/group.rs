use printpdf::Mm;

use crate::generate::{element::Element2, outline::TextOutline, padding::Padding};

pub struct Group {
    pub elements: Vec<Box<dyn Element2>>,
    pub outline: Option<TextOutline>,
    pub padding: Padding,
    pub try_keep_together: bool,
}

impl Group {
    pub fn new() -> Self {
        Group {
            elements: Vec::new(),
            outline: None,
            padding: Padding::none(),
            try_keep_together: false,
        }
    }

    pub fn with_try_keep_together(mut self, keep: bool) -> Self {
        self.try_keep_together = keep;
        self
    }

    pub fn push<E>(&mut self, element: E)
    where
        E: Element2 + 'static,
    {
        self.elements.push(Box::new(element));
    }

    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_outline(mut self, outline: TextOutline) -> Self {
        self.outline = Some(outline);
        self
    }
}

// impl Element for Group {
//     fn build(
//         &self,
//         document: &printpdf::PdfDocument,
//         origin: printpdf::Point,
//         max_width: Option<printpdf::Mm>,
//         page_style: &DocumentStyle,
//     ) -> super::BuildResult {
//         let outline_padding_top = self
//             .outline
//             .as_ref()
//             .map_or(Pt(0.0), |o| o.padding.top.into_pt());
//
//         let mut elements_width = Mm(0.0);
//         let mut ops = Vec::new();
//
//         let mut next_cursor = Point {
//             x: origin.x,
//             y: origin.y - outline_padding_top,
//         };
//         for element in &self.elements {
//             let element_ops = element.build(document, next_cursor, max_width, page_style);
//             ops.extend(element_ops.ops);
//             next_cursor = element_ops.next_cursor;
//             if element_ops.width > elements_width {
//                 // Update the width if the current element's width is greater
//                 // than the previously recorded width.
//                 elements_width = element_ops.width;
//             }
//         }
//
//         if let Some(outline) = &self.outline {
//             ops.push(Op::SaveGraphicsState);
//
//             let text_rect = Rect {
//                 x: origin.x - outline.padding.left.into_pt(),
//                 y: origin.y,
//                 width: max_width.unwrap_or(elements_width).into_pt()
//                     + outline.padding.left.into_pt()
//                     + outline.padding.right.into_pt(),
//                 height: (origin.y - next_cursor.y) + outline.padding.bottom.into_pt(),
//             };
//
//             ops.push(Op::SetOutlineColor {
//                 col: printpdf::Color::Rgb(outline.color.clone()),
//             });
//             ops.push(Op::SetOutlineThickness {
//                 pt: outline.thickness,
//             });
//
//             ops.push(Op::DrawPolygon {
//                 polygon: Polygon {
//                     mode: PaintMode::Stroke,
//                     ..text_rect.to_polygon()
//                 },
//             });
//
//             ops.push(Op::RestoreGraphicsState);
//
//             next_cursor.y -= outline.padding.bottom.into_pt();
//         }
//         super::BuildResult {
//             ops,
//             next_cursor,
//             width: max_width.unwrap_or(elements_width),
//         }
//     }
// }

impl Element2 for Group {
    fn calculate_height<'a>(&self, builder: &super::element_builder::ElementBuilder<'a>) -> Mm {
        self.elements
            .iter()
            .map(|elem| elem.calculate_height(builder))
            .fold(Mm(0.0), |v, h| v + h)
            + self.padding.top
            + self.padding.bottom
    }

    fn build<'a>(&self, builder: &mut super::element_builder::ElementBuilder<'a>) {
        let height = self
            .try_keep_together
            .then(|| self.calculate_height(builder));
        let mut group_builder = builder.generate_group_builder(&self.padding, height);

        for child in &self.elements {
            child.build(&mut group_builder);
        }

        if let Some(outline) = &self.outline {
            group_builder.draw_outline(&self.padding, outline);
        }

        let new_y = group_builder.cursor.y - self.padding.bottom.into_pt();

        builder.merge(group_builder);
        builder.update_cursor(new_y);
    }
}
