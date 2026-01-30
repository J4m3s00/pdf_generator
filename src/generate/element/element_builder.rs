use std::collections::VecDeque;

use printpdf::{
    Line, LinePoint, Mm, Op, PaintMode, Point, Polygon, Pt, Px, Rect, ShapedText, TextItem,
    XObject, XObjectTransform,
};

use crate::generate::document::Document;
use crate::generate::element::Element;
use crate::generate::element::image::Image;
use crate::generate::element::rich_text::{RichText, RichTextLine, RichTextLinePart};
use crate::generate::font::Font;
use crate::generate::outline::LineStyle;
use crate::generate::padding::Padding;
use crate::generate::text_gen::{shape_text, split_shaped_text};

#[derive(Debug, Default)]
pub enum MoveDirection {
    Right,
    #[default]
    Down,
}

pub enum TextListStyle {
    // Bulleted,
    // Numbered,
    Checkbox,
}

#[derive(Debug, Clone, Copy)]
pub enum ColumnWidth {
    Fixed(Mm),
    Percent(f32),
}

pub struct ElementBuilder<'a> {
    pub(crate) document: &'a Document,
    origin: Point,
    pub cursor: Point,
    remaining_width: Mm,
    starting_page: usize,
    pub pages: Vec<Vec<Op>>,
    added_padding_bottom: Mm,
    errors: Vec<String>,
}

impl<'a> ElementBuilder<'a> {
    pub fn new(document: &'a Document) -> Self {
        let style = document.style().clone();
        let origin = Point {
            x: style.padding.left.into_pt(),
            y: (style.height - style.padding.top).into_pt(),
        };

        Self {
            document,
            origin,
            cursor: origin,
            remaining_width: style.inner_width(),
            starting_page: 0,
            pages: vec![Vec::new()],
            added_padding_bottom: Mm(0.0),
            errors: Vec::new(),
        }
    }
}

impl<'a> ElementBuilder<'a> {
    pub fn measure_text(&self, text: &str, font: &Font) -> (Pt, Pt) {
        let shaped_text = shape_text(
            self.document.pdf_document(),
            font.font_id(),
            font.font_size(),
            font.font_height_offset(),
            text,
            Some(self.remaining_width_from_cursor()),
        );

        (Pt(shaped_text.width), Pt(shaped_text.height))
    }

    pub fn measure_image(&self, image: &Image) -> (Pt, Pt) {
        let XObject::Image(raw_image) = self
            .document
            .pdf_document()
            .resources
            .xobjects
            .map
            .get(&image.image)
            .expect("Image not found in document resources")
        else {
            panic!("Expected XObject to be an Image");
        };

        let width = Px(raw_image.width).into_pt(300.0);
        let height = Px(raw_image.height).into_pt(300.0);

        let scale = image
            .desired_width
            .map(|desired_width| desired_width.into_pt() / width);

        let final_width = scale.map(|scale| width * scale).unwrap_or(width);
        let final_height = scale.map(|scale| height * scale).unwrap_or(height);

        (final_width, final_height)
    }

    pub fn push_paragraph(&mut self, paragraph: &str, font: &Font) {
        let shaped_text = shape_text(
            self.document.pdf_document(),
            font.font_id(),
            font.font_size(),
            font.font_height_offset(),
            paragraph,
            Some(self.remaining_width_from_cursor()),
        );

        self.push_shaped_text(shaped_text, font.font_size(), font.font_height_offset());
    }

    /// Returning the last shaped text that didn't fit
    fn push_shaped_text(&mut self, text: ShapedText, font_size: Pt, font_height_offset: Pt) {
        // Do we need to cut the text?
        let (first, rest) = split_shaped_text(
            text,
            font_size,
            font_height_offset,
            self.remaining_height_from_cursor(),
        );

        let ops = first.get_ops(Point {
            x: self.cursor.x,
            y: self.cursor.y,
        });
        self.pages
            .last_mut()
            .expect("We always have one page")
            .extend(ops.into_iter());
        self.cursor.y -= Pt(first.height);

        if let Some(rest) = rest {
            self.next_page();
            self.push_shaped_text(rest, font_size, font_height_offset);
        }
    }

    pub fn draw_rect(&mut self, size: Pt) {
        if self.remaining_height_from_cursor().into_pt() < size {
            self.next_page();
        }

        let mut ops = Vec::new();

        let checkbox_rect = Rect {
            x: self.cursor.x,
            y: self.cursor.y,
            width: size,  // Fixed width for checkbox
            height: size, // Fixed height for checkbox
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

        self.advance_cursor(size);

        self.pages
            .last_mut()
            .expect("Always at least one page")
            .extend(ops);
    }

    pub fn draw_line(&mut self, padding: &Padding, outline: &LineStyle) {
        self.advance_cursor(padding.top.into_pt());

        let width = self.remaining_width_from_cursor() - padding.left - padding.right;
        println!("Width of line: {:?}", width);

        let mut ops = Vec::new();
        ops.push(Op::SaveGraphicsState);
        ops.push(Op::SetOutlineColor {
            col: printpdf::Color::Rgb(outline.color.clone()),
        });
        ops.push(Op::SetOutlineThickness {
            pt: outline.thickness,
        });
        ops.push(Op::DrawLine {
            line: printpdf::Line {
                points: vec![
                    LinePoint {
                        bezier: false,
                        p: Point {
                            x: self.cursor.x + padding.left.into_pt(),
                            y: self.cursor.y,
                        },
                    },
                    LinePoint {
                        bezier: false,
                        p: Point {
                            x: self.cursor.x + width.into_pt() - padding.right.into_pt(),
                            y: self.cursor.y,
                        },
                    },
                ],
                is_closed: false,
            },
        });

        ops.push(Op::RestoreGraphicsState);

        self.pages
            .last_mut()
            .expect("Always at least one page")
            .extend(ops);

        self.advance_cursor(padding.bottom.into_pt());
    }

    pub fn calculate_flex_height<'element>(
        &self,
        elements: impl IntoIterator<Item = Box<&'element (impl Element + 'element)>>,
        space_x: Mm,
        space_y: Mm,
    ) -> Mm {
        let remaining_width = self.remaining_width_from_cursor();

        let mut x_cursor = self.cursor.x;
        let mut current_measured_height = Mm(0.0);
        let mut current_line_height = Mm(0.0);

        for element in elements.into_iter() {
            let width = element.calculate_width(self);
            if width > remaining_width {
                // The element wont fit at all. We skip it. In the rendering, we will generate an
                // error
                continue;
            }

            let x_offset = Mm::from(x_cursor - self.origin.x);
            let current_remaining_width = remaining_width - x_offset;
            if width > current_remaining_width {
                // Push to next line
                x_cursor = self.origin.x;
                current_measured_height += current_line_height + space_y;
                current_line_height = Mm(0.0);
            }
            x_cursor += (width + space_x).into_pt();

            current_line_height = current_line_height.max(element.calculate_height(self));
        }

        current_line_height + current_measured_height
    }

    /// Flex will try and order elements on the x axis first, before going to the next line.
    pub fn push_flex<'e>(
        &mut self,
        elements: impl Iterator<Item = Box<&'e (impl Element + 'e)>>,
        space_x: Mm,
        space_y: Mm,
    ) {
        let remaining_width = self.remaining_width_from_cursor();

        let mut current_line_height = Mm(0.0);
        for element in elements {
            let width = element.calculate_width(self);
            let height = element.calculate_height(self);
            if width > remaining_width {
                // This element wont fit at all. We skip it. Also we generate an error
                self.errors.push(format!(
                    "Failed to generate flex item \"{}\"",
                    element.display_name()
                ));
                continue;
            }

            if width > self.remaining_width_from_cursor() {
                // Go to the next line
                self.advance_cursor((current_line_height + space_y).into_pt());
                self.reset_cursor_x();
                current_line_height = Mm(0.0);
            }

            if height > self.remaining_height_from_cursor() {
                self.reset_cursor_x();
                self.next_page();
                current_line_height = Mm(0.0);
            }

            current_line_height = current_line_height.max(height);

            element.build(self);

            self.cursor.x += space_x.into_pt();
        }

        self.advance_cursor(current_line_height.into_pt());
        self.reset_cursor_x();
    }

    pub fn push_image(&mut self, image: &Image) {
        let XObject::Image(raw_image) = self
            .document
            .pdf_document()
            .resources
            .xobjects
            .map
            .get(&image.image)
            .expect("Image not found in document resources")
        else {
            panic!("Expected XObject to be an Image");
        };

        let width = Px(raw_image.width).into_pt(300.0);
        let height = Px(raw_image.height).into_pt(300.0);

        let scale = image
            .desired_width
            .map(|desired_width| desired_width.into_pt() / width);

        let final_width = scale.map(|scale| width * scale).unwrap_or(width);
        let final_height = scale.map(|scale| height * scale).unwrap_or(height);

        let transform = XObjectTransform {
            translate_x: Some(self.cursor.x),
            translate_y: Some(self.cursor.y - final_height),
            scale_x: scale,
            scale_y: scale,
            ..Default::default()
        };

        let ops = vec![Op::UseXobject {
            id: image.image.clone(),
            transform,
        }];

        self.pages
            .last_mut()
            .expect("We always have one page")
            .extend(ops.into_iter());

        self.cursor.x += final_width;
    }

    pub fn generate_column_builder(
        &self,
        width: ColumnWidth,
    ) -> (ElementBuilder<'a>, ElementBuilder<'a>) {
        let left_width = match width {
            ColumnWidth::Fixed(mm) => mm,
            ColumnWidth::Percent(fr) => self.remaining_width_from_cursor() * fr,
        };
        let right_width = self.remaining_width_from_cursor() - left_width;
        let left_builder = ElementBuilder {
            document: self.document,
            origin: self.cursor,
            cursor: self.cursor,
            remaining_width: left_width,
            starting_page: self.pages.len() - 1,
            pages: vec![Vec::new()],
            added_padding_bottom: Mm(0.0),
            errors: Vec::new(),
        };
        let right_origin = Point {
            x: self.cursor.x + left_width.into_pt(),
            y: self.cursor.y,
        };

        let right_builder = ElementBuilder {
            document: self.document,
            origin: right_origin,
            cursor: right_origin,
            remaining_width: right_width,
            starting_page: self.pages.len() - 1,
            pages: vec![Vec::new()],
            added_padding_bottom: Mm(0.0),
            errors: Vec::new(),
        };

        (left_builder, right_builder)
    }

    /// Generate a new ElementBuilder for a group element.
    ///
    /// The padding will be applied to the new builder
    ///
    /// If you want to try to fit the group on the same page, set try_same_page to Some(Mm). The
    /// value should be the calculated height of the group element. It will be ignored, if the
    /// height is bigger than the whole page. Should be height be bigger than the remaining height
    /// of the current page, a new page will be started.
    pub fn generate_group_builder(
        &self,
        padding: &Padding,
        try_same_page: Option<Mm>,
    ) -> ElementBuilder<'a> {
        let (origin, new_page) = match try_same_page {
            Some(height)
                if height <= self.document.style().inner_height()
                    && self.remaining_height_from_cursor() < height =>
            {
                // We can fit the group on a single page, but need to go to the next
                let origin = Point {
                    x: self.origin.x + padding.left.into_pt(),
                    y: (self.document.style().height
                        - self.document.style().padding.top
                        - padding.top)
                        .into_pt(),
                };
                (origin, true)
            }
            _ => (
                Point {
                    x: self.cursor.x + padding.left.into_pt(),
                    y: self.cursor.y - padding.top.into_pt(),
                },
                false,
            ),
        };

        ElementBuilder {
            document: self.document,
            origin,
            cursor: origin,
            remaining_width: self.remaining_width - (padding.left + padding.right),
            starting_page: self.pages.len() - if new_page { 0 } else { 1 },
            pages: vec![Vec::new()],
            added_padding_bottom: padding.bottom,
            errors: Vec::new(),
        }
    }

    pub fn draw_outline(&mut self, padding: &Padding, outline: &LineStyle) {
        let width = self.remaining_width + padding.left + padding.right;

        {
            // Draw the top

            let mut ops = vec![];
            ops.push(Op::SaveGraphicsState);

            ops.push(Op::SetOutlineColor {
                col: printpdf::Color::Rgb(outline.color.clone()),
            });
            ops.push(Op::SetOutlineThickness {
                pt: outline.thickness,
            });
            ops.push(Op::DrawLine {
                line: Line {
                    points: vec![
                        LinePoint {
                            p: Point {
                                x: self.origin.x - padding.left.into_pt(),
                                y: self.origin.y + padding.top.into_pt(),
                            },
                            bezier: false,
                        },
                        LinePoint {
                            p: Point {
                                x: self.origin.x - padding.left.into_pt() + width.into_pt(),
                                y: self.origin.y + padding.top.into_pt(),
                            },
                            bezier: false,
                        },
                    ],
                    is_closed: false,
                },
            });

            ops.push(Op::RestoreGraphicsState);

            self.pages.first_mut().expect("Always one page").extend(ops);
        }

        {
            // Draw bottom

            let mut ops = vec![];
            ops.push(Op::SaveGraphicsState);

            ops.push(Op::SetOutlineColor {
                col: printpdf::Color::Rgb(outline.color.clone()),
            });
            ops.push(Op::SetOutlineThickness {
                pt: outline.thickness,
            });
            ops.push(Op::DrawLine {
                line: Line {
                    points: vec![
                        LinePoint {
                            p: Point {
                                x: self.cursor.x - padding.left.into_pt(),
                                y: self.cursor.y - padding.bottom.into_pt(),
                            },
                            bezier: false,
                        },
                        LinePoint {
                            p: Point {
                                x: self.cursor.x - padding.left.into_pt() + width.into_pt(),
                                y: self.cursor.y - padding.bottom.into_pt(),
                            },
                            bezier: false,
                        },
                    ],
                    is_closed: false,
                },
            });

            ops.push(Op::RestoreGraphicsState);

            self.pages
                .last_mut()
                .expect("Always have at least one page")
                .extend(ops);
        }

        // Draw left and right lines
        // First if the whole builder is a single page we can just connect the lines
        if self.pages.len() == 1 {
            let mut ops = vec![];
            ops.push(Op::SaveGraphicsState);

            ops.push(Op::SetOutlineColor {
                col: printpdf::Color::Rgb(outline.color.clone()),
            });
            ops.push(Op::SetOutlineThickness {
                pt: outline.thickness,
            });
            ops.push(Op::DrawLine {
                line: Line {
                    points: vec![
                        LinePoint {
                            p: Point {
                                x: self.origin.x - padding.left.into_pt(),
                                y: self.origin.y + padding.top.into_pt(),
                            },
                            bezier: false,
                        },
                        LinePoint {
                            p: Point {
                                x: self.cursor.x - padding.left.into_pt(),
                                y: self.cursor.y - padding.bottom.into_pt(),
                            },
                            bezier: false,
                        },
                    ],
                    is_closed: false,
                },
            });
            ops.push(Op::DrawLine {
                line: Line {
                    points: vec![
                        LinePoint {
                            p: Point {
                                x: self.origin.x - padding.left.into_pt() + width.into_pt(),
                                y: self.origin.y + padding.top.into_pt(),
                            },
                            bezier: false,
                        },
                        LinePoint {
                            p: Point {
                                x: self.cursor.x - padding.left.into_pt() + width.into_pt(),
                                y: self.cursor.y - padding.bottom.into_pt(),
                            },
                            bezier: false,
                        },
                    ],
                    is_closed: false,
                },
            });

            ops.push(Op::RestoreGraphicsState);

            self.pages[0].extend(ops);
        } else {
            // Draw till the end on the first page
            {
                let mut ops = vec![];
                ops.push(Op::SaveGraphicsState);

                ops.push(Op::SetOutlineColor {
                    col: printpdf::Color::Rgb(outline.color.clone()),
                });
                ops.push(Op::SetOutlineThickness {
                    pt: outline.thickness,
                });
                ops.push(Op::DrawLine {
                    line: Line {
                        points: vec![
                            LinePoint {
                                p: Point {
                                    x: self.origin.x - padding.left.into_pt(),
                                    y: self.origin.y + padding.top.into_pt(),
                                },
                                bezier: false,
                            },
                            LinePoint {
                                p: Point {
                                    x: self.cursor.x - padding.left.into_pt(),
                                    y: self.document.style().padding.bottom.into_pt(),
                                },
                                bezier: false,
                            },
                        ],
                        is_closed: false,
                    },
                });
                ops.push(Op::DrawLine {
                    line: Line {
                        points: vec![
                            LinePoint {
                                p: Point {
                                    x: self.origin.x - padding.left.into_pt() + width.into_pt(),
                                    y: self.origin.y + padding.top.into_pt(),
                                },
                                bezier: false,
                            },
                            LinePoint {
                                p: Point {
                                    x: self.cursor.x - padding.left.into_pt() + width.into_pt(),
                                    y: self.document.style().padding.bottom.into_pt(),
                                },
                                bezier: false,
                            },
                        ],
                        is_closed: false,
                    },
                });

                ops.push(Op::RestoreGraphicsState);

                self.pages[0].extend(ops);
            }

            {
                // Draw last to beginning of the page
                let mut ops = vec![];
                ops.push(Op::SaveGraphicsState);

                ops.push(Op::SetOutlineColor {
                    col: printpdf::Color::Rgb(outline.color.clone()),
                });
                ops.push(Op::SetOutlineThickness {
                    pt: outline.thickness,
                });
                ops.push(Op::DrawLine {
                    line: Line {
                        points: vec![
                            LinePoint {
                                p: Point {
                                    x: self.origin.x - padding.left.into_pt(),
                                    y: self.document.style().height.into_pt()
                                        - self.document.style().padding.top.into_pt(),
                                },
                                bezier: false,
                            },
                            LinePoint {
                                p: Point {
                                    x: self.cursor.x - padding.left.into_pt(),
                                    y: self.cursor.y - padding.bottom.into_pt(),
                                },
                                bezier: false,
                            },
                        ],
                        is_closed: false,
                    },
                });
                ops.push(Op::DrawLine {
                    line: Line {
                        points: vec![
                            LinePoint {
                                p: Point {
                                    x: self.origin.x - padding.left.into_pt() + width.into_pt(),
                                    y: self.document.style().height.into_pt()
                                        - self.document.style().padding.top.into_pt(),
                                },
                                bezier: false,
                            },
                            LinePoint {
                                p: Point {
                                    x: self.cursor.x - padding.left.into_pt() + width.into_pt(),
                                    y: self.cursor.y - padding.bottom.into_pt(),
                                },
                                bezier: false,
                            },
                        ],
                        is_closed: false,
                    },
                });

                ops.push(Op::RestoreGraphicsState);

                self.pages
                    .last_mut()
                    .expect("At least one page")
                    .extend(ops);
            }

            if self.pages.len() > 2 {
                // Draw the middle pages
                let mut ops = vec![];
                ops.push(Op::SaveGraphicsState);

                ops.push(Op::SetOutlineColor {
                    col: printpdf::Color::Rgb(outline.color.clone()),
                });
                ops.push(Op::SetOutlineThickness {
                    pt: outline.thickness,
                });
                ops.push(Op::DrawLine {
                    line: Line {
                        points: vec![
                            LinePoint {
                                p: Point {
                                    x: self.origin.x - padding.left.into_pt(),
                                    y: self.document.style().height.into_pt()
                                        - self.document.style().padding.top.into_pt(),
                                },
                                bezier: false,
                            },
                            LinePoint {
                                p: Point {
                                    x: self.cursor.x - padding.left.into_pt(),
                                    y: self.document.style().padding.bottom.into_pt(),
                                },
                                bezier: false,
                            },
                        ],
                        is_closed: false,
                    },
                });
                ops.push(Op::DrawLine {
                    line: Line {
                        points: vec![
                            LinePoint {
                                p: Point {
                                    x: self.origin.x - padding.left.into_pt() + width.into_pt(),
                                    y: self.document.style().height.into_pt()
                                        - self.document.style().padding.top.into_pt(),
                                },
                                bezier: false,
                            },
                            LinePoint {
                                p: Point {
                                    x: self.cursor.x - padding.left.into_pt() + width.into_pt(),
                                    y: self.document.style().padding.bottom.into_pt(),
                                },
                                bezier: false,
                            },
                        ],
                        is_closed: false,
                    },
                });

                ops.push(Op::RestoreGraphicsState);

                let num_pages = self.pages.len();

                for p in self.pages.iter_mut().skip(1).take(num_pages - 2) {
                    p.extend(ops.clone());
                }
            }
        }
    }

    /// Advances the cursor on the y axis
    pub fn advance_cursor(&mut self, dy: Pt) {
        let remaining_height = self.remaining_height_from_cursor().into_pt();
        if dy > remaining_height {
            // We need to go to the next page
            self.next_page();
            let rest = dy - remaining_height;
            self.cursor.y -= rest;
        } else {
            self.cursor.y -= dy;
        }
    }

    /// Sets the cursor y position
    pub fn update_cursor(&mut self, y: Pt) {
        self.cursor.y = y;
    }

    /// Sets the cursor x back to the origin
    pub fn reset_cursor_x(&mut self) {
        self.cursor.x = self.origin.x;
    }

    fn remaining_height_from_cursor(&self) -> Mm {
        Mm::from(self.cursor.y) - self.document.style().padding.bottom - self.added_padding_bottom
    }

    pub fn remaining_width_from_cursor(&self) -> Mm {
        let x_offset = Mm::from(self.cursor.x - self.origin.x);
        self.remaining_width - x_offset
    }

    /// Updates the origin and cursor to the initial position
    /// Recalculates remaining_height
    /// remaining_width stays the same
    fn next_page(&mut self) {
        let style = self.document.style().clone();
        let origin = Point {
            x: self.origin.x,
            y: (style.height - style.padding.top).into_pt(),
        };

        self.cursor = origin;
        self.pages.push(Vec::new());
    }

    pub fn merge(&mut self, other: ElementBuilder) {
        let mut dequeue = VecDeque::from(other.pages);

        for p in self.pages[other.starting_page..].iter_mut() {
            if let Some(extend) = dequeue.pop_front() {
                p.extend(extend);
            } else {
                break;
            }
        }

        while let Some(next) = dequeue.pop_front() {
            self.pages.push(next);
        }

        self.errors.extend(other.errors);
    }

    pub fn split_rich_text_into_lines(&self, rich_text: &RichText) -> Vec<RichTextLine> {
        let mut current_line_height = Pt(0.0);
        let mut current_line_width = Pt(0.0);
        let mut lines: Vec<RichTextLine> = vec![RichTextLine::default()];

        for (text, font) in rich_text.parts.iter() {
            if text.is_empty() {
                continue;
            }

            let current_line = lines.last_mut().expect("We always have one line");

            let shaped_text = shape_text(
                self.document.pdf_document(),
                font.font_id(),
                font.font_size(),
                font.font_height_offset(),
                &text,
                Some(self.remaining_width - Mm::from(current_line_width)),
            );

            let width = Pt(if shaped_text.lines.len() == 1 {
                shape_text(
                    self.document.pdf_document(),
                    font.font_id(),
                    font.font_size(),
                    font.font_height_offset(),
                    &text,
                    None,
                )
                .width
            } else {
                // let space_advance_px =
                //     shaped_words.get_space_advance_px(text_layout_options.font_size_px);
                // let word_spacing_px = space_advance_px
                //     * text_layout_options
                //         .word_spacing
                //         .as_ref()
                //         .copied()
                //         .unwrap_or(DEFAULT_WORD_SPACING);

                shaped_text
                    .lines
                    .first()
                    .expect("For now")
                    .words
                    .iter()
                    .map(|w| w.width)
                    .sum()
            });

            let height = font.font_size() + font.font_height_offset();

            let line_text = if shaped_text.lines.len() == 1 {
                text.clone()
            } else {
                shaped_text
                    .lines
                    .first()
                    .expect("For now")
                    .words
                    .iter()
                    .map(|w| w.text.as_str())
                    .collect::<Vec<_>>()
                    .join("")
            };

            let rest_text = &text[line_text.len()..text.len()];

            current_line.parts.push(RichTextLinePart {
                text: line_text,
                font: font.clone(),
                width,
            });

            current_line_height = current_line_height.max(height);
            current_line_width += width;

            current_line.height = current_line_height;

            if shaped_text.lines.len() > 1 {
                current_line_width = Pt(0.0);
                // Check the rest of the lines
                let shaped_rest = shape_text(
                    self.document.pdf_document(),
                    font.font_id(),
                    font.font_size(),
                    font.font_height_offset(),
                    rest_text,
                    Some(self.remaining_width),
                );
                for line in shaped_rest.lines.iter() {
                    let line_text = line
                        .words
                        .iter()
                        .map(|w| w.text.as_str())
                        .collect::<Vec<_>>()
                        .join("");

                    let width = line.words.iter().map(|w| w.width).sum::<f32>();

                    lines.push(RichTextLine {
                        parts: vec![RichTextLinePart {
                            text: line_text,
                            font: font.clone(),
                            width: Pt(width),
                        }],
                        height: Pt(font.font_size().0 + font.font_height_offset().0),
                    });
                    current_line_width = Pt(width);
                }
            }
        }

        lines
    }

    pub fn push_rich_text(&mut self, rich_text: &crate::generate::element::rich_text::RichText) {
        // We first need to cut into lines, so we can calculate the height properly
        let lines = self.split_rich_text_into_lines(rich_text);

        for line in lines.into_iter() {
            self.advance_cursor(line.height);
            for part in line.parts {
                let ops = Self::get_ops(&part.text, &part.font, self.cursor);
                self.pages
                    .last_mut()
                    .expect("Always have one page")
                    .extend(ops);
                self.cursor.x += part.width;
            }
            self.reset_cursor_x();
        }
    }

    fn get_ops(text: &str, font: &Font, origin: Point) -> Vec<Op> {
        let line_height = font.font_height_offset() + font.font_size();
        let font_size = font.font_size();

        let mut ops = Vec::new();

        ops.push(Op::SaveGraphicsState);

        // Start text section
        ops.push(Op::StartTextSection);

        // The origin_TOP_LEFT is the top left origin of the entire text block being layouted
        // However, in PDF, the "set text cursor" sets the baseline of the first line...
        ops.push(Op::SetTextCursor { pos: origin });

        ops.push(Op::SetFontSize {
            size: font_size,
            font: font.font_id(),
        });

        ops.push(Op::SetLineHeight { lh: line_height });

        ops.push(Op::WriteText {
            items: vec![TextItem::Text(text.to_string())],
            font: font.font_id(),
        });

        // End text section
        ops.push(Op::EndTextSection);

        ops.push(Op::RestoreGraphicsState);

        ops
    }
}
