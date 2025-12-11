use printpdf::{Mm, Op, PaintMode, Point, Polygon, Pt, Rect};

use crate::generate::{document::DocumentStyle, element::Element, outline::TextOutline};

pub struct Group {
    pub elements: Vec<Box<dyn Element>>,
    pub outline: Option<TextOutline>,
}

impl Group {
    pub fn new() -> Self {
        Group {
            elements: Vec::new(),
            outline: None,
        }
    }

    pub fn push<E>(&mut self, element: E)
    where
        E: Element + 'static,
    {
        self.elements.push(Box::new(element));
    }

    pub fn draw_outline(&mut self, outline: TextOutline) {
        self.outline = Some(outline);
    }
}

impl Element for Group {
    fn build(
        &self,
        document: &printpdf::PdfDocument,
        origin: printpdf::Point,
        max_width: Option<printpdf::Mm>,
        page_style: &DocumentStyle,
    ) -> super::BuildResult {
        let outline_padding_top = self
            .outline
            .as_ref()
            .map_or(Pt(0.0), |o| o.padding.top.into_pt());

        let mut elements_width = Mm(0.0);
        let mut ops = Vec::new();

        let mut next_cursor = Point {
            x: origin.x,
            y: origin.y - outline_padding_top,
        };
        for element in &self.elements {
            let element_ops = element.build(document, next_cursor, max_width, page_style);
            ops.extend(element_ops.ops);
            next_cursor = element_ops.next_cursor;
            if element_ops.width > elements_width {
                // Update the width if the current element's width is greater
                // than the previously recorded width.
                elements_width = element_ops.width;
            }
        }

        if let Some(outline) = &self.outline {
            ops.push(Op::SaveGraphicsState);

            let text_rect = Rect {
                x: origin.x - outline.padding.left.into_pt(),
                y: origin.y,
                width: max_width.unwrap_or(elements_width).into_pt()
                    + outline.padding.left.into_pt()
                    + outline.padding.right.into_pt(),
                height: (origin.y - next_cursor.y) + outline.padding.bottom.into_pt(),
            };

            ops.push(Op::SetOutlineColor {
                col: printpdf::Color::Rgb(outline.color.clone()),
            });
            ops.push(Op::SetOutlineThickness {
                pt: outline.thickness,
            });

            ops.push(Op::DrawPolygon {
                polygon: Polygon {
                    mode: PaintMode::Stroke,
                    ..text_rect.to_polygon()
                },
            });

            ops.push(Op::RestoreGraphicsState);

            next_cursor.y -= outline.padding.bottom.into_pt();
        }
        super::BuildResult {
            ops,
            next_cursor,
            width: max_width.unwrap_or(elements_width),
        }
    }
}
