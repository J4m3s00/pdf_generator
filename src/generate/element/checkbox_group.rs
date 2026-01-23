use printpdf::{FontId, Mm, PaintMode, Point, Polygon, Pt, Rect};

use crate::generate::{
    document::DocumentStyle,
    element::{Element, Element2, element_builder::ColumnWidth},
    padding::Padding,
    text_gen::{DEFAULT_FONT_LINE_HEIGHT_OFFSET, DEFAULT_FONT_SIZE, shape_text},
};

pub struct CheckboxGroup {
    pub checkboxes: Vec<String>,

    space_between_checkboxes: Pt,

    font: FontId,
    font_size: Pt,
    font_height_offset: Pt,
}

impl CheckboxGroup {
    pub fn new(checkboxes: Vec<String>, font: FontId) -> Self {
        Self {
            checkboxes,
            font,
            space_between_checkboxes: Pt(7.5), // Default space between checkboxes
            font_size: DEFAULT_FONT_SIZE,
            font_height_offset: DEFAULT_FONT_LINE_HEIGHT_OFFSET,
        }
    }

    pub fn with_space_between_checkboxes(mut self, space: Pt) -> Self {
        self.space_between_checkboxes = space;
        self
    }
}

impl Element for CheckboxGroup {
    fn build(
        &self,
        document: &printpdf::PdfDocument,
        origin: printpdf::Point,
        _max_width: Option<printpdf::Mm>,
        _page_style: &DocumentStyle,
    ) -> super::BuildResult {
        let mut max_height = Pt(0.0);
        let mut offset_x = origin.x;
        let mut ops = Vec::new();

        for (index, checkbox) in self.checkboxes.iter().enumerate() {
            let checkbox_rect = Rect {
                x: offset_x,
                y: origin.y,
                width: self.font_size,  // Fixed width for checkbox
                height: self.font_size, // Fixed height for checkbox
            };
            ops.push(printpdf::Op::SaveGraphicsState);
            ops.push(printpdf::Op::SetFillColor {
                col: printpdf::Color::Rgb(printpdf::Rgb::new(0.0, 0.0, 0.0, None)),
            });
            ops.push(printpdf::Op::DrawPolygon {
                polygon: Polygon {
                    mode: PaintMode::Stroke,
                    ..checkbox_rect.to_polygon()
                },
            });
            ops.push(printpdf::Op::RestoreGraphicsState);

            offset_x += self.font_size + Pt(self.font_size.0 / 2.0); // Move to the right for the next checkbox

            let shaped_text = shape_text(
                document,
                self.font.clone(),
                self.font_size,
                self.font_height_offset,
                &checkbox,
                None,
            );

            ops.extend(shaped_text.get_ops(Point {
                x: offset_x,
                y: origin.y,
            }));

            if shaped_text.height > max_height.0 {
                max_height = Pt(shaped_text.height);
            }

            offset_x += Pt(shaped_text.width)
                + if index < self.checkboxes.len() - 1 {
                    self.space_between_checkboxes
                } else {
                    Pt(0.0)
                }; // Add space after checkbox text
        }

        super::BuildResult {
            ops,
            next_cursor: Point {
                x: origin.x,
                y: origin.y - max_height,
            },
            width: Mm::from(offset_x - origin.x),
        }
    }
}

impl Element2 for CheckboxGroup {
    fn display_name(&self) -> &str {
        "Checkbox Group"
    }

    fn calculate_width<'a>(&self, builder: &super::element_builder::ElementBuilder<'a>) -> Mm {
        builder.remaining_width_from_cursor()
    }

    fn calculate_height<'a>(&self, builder: &super::element_builder::ElementBuilder<'a>) -> Mm {
        let (mut left, mut right) = builder
            .generate_column_builder(ColumnWidth::Percent(1.0 / self.checkboxes.len() as f32));

        self.checkboxes
            .iter()
            .enumerate()
            .fold(Mm(0.0), move |v, (index, item)| {
                let (_, text_builder) = left.generate_column_builder(ColumnWidth::Fixed(Mm::from(
                    self.font_size + Pt(4.0),
                )));

                let height = self.font_size.max(
                    text_builder
                        .measure_text(
                            item.as_str(),
                            self.font.clone(),
                            self.font_size,
                            self.font_height_offset,
                        )
                        .1,
                );

                let column_width =
                    ColumnWidth::Percent(1.0 / (self.checkboxes.len() - index - 1) as f32);
                let (new_left, new_right) = right.generate_column_builder(column_width);

                left = new_left;
                right = new_right;

                v.max(Mm::from(height))
            })
    }

    fn build<'a>(&self, builder: &mut super::element_builder::ElementBuilder<'a>) {
        // Create a group builder, to have the checkboxes in a group
        // When the checkbox_group is at the bottom of the page, and one of the text has multiple
        // lines and pushes to the next page, we want the whole checkbox_group to be on the next
        // page if possible
        let mut group_builder =
            builder.generate_group_builder(&Padding::none(), Some(self.calculate_height(builder)));

        let (mut left, mut right) = group_builder
            .generate_column_builder(ColumnWidth::Percent(1.0 / self.checkboxes.len() as f32));

        for (index, item) in self.checkboxes.iter().enumerate() {
            let (mut box_builder, mut text_builder) = left
                .generate_column_builder(ColumnWidth::Fixed(Mm::from(self.font_size + Pt(4.0))));

            box_builder.draw_rect(self.font_size);

            text_builder.push_paragraph(
                item.as_str(),
                self.font.clone(),
                self.font_size,
                self.font_height_offset,
            );

            let column_width =
                ColumnWidth::Percent(1.0 / (self.checkboxes.len() - index - 1) as f32);
            let (new_left, new_right) = right.generate_column_builder(column_width);

            group_builder.merge(box_builder);
            group_builder.merge(text_builder);

            left = new_left;
            right = new_right;
        }

        builder.merge(group_builder);

        builder.advance_cursor(self.calculate_height(builder).into_pt());
    }
}
