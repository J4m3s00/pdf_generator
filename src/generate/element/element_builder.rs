use std::collections::VecDeque;

use printpdf::{FontId, Line, LinePoint, Mm, Op, PaintMode, Point, Polygon, Pt, Rect, ShapedText};

use crate::generate::document::Document;
use crate::generate::outline::TextOutline;
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
    document: &'a Document,
    origin: Point,
    pub cursor: Point,
    remaining_width: Mm,
    starting_page: usize,
    pub pages: Vec<Vec<Op>>,
    added_padding_bottom: Mm,
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
        }
    }
}

impl<'a> ElementBuilder<'a> {
    pub fn measure_text(
        &self,
        text: &str,
        font: FontId,
        font_size: Pt,
        font_height_offset: Pt,
    ) -> (Pt, Pt) {
        let shaped_text = shape_text(
            self.document.pdf_document(),
            font,
            font_size,
            font_height_offset,
            text,
            Some(self.remaining_width_from_cursor()),
        );

        (Pt(shaped_text.width), Pt(shaped_text.height))
    }

    pub fn push_paragraph(
        &mut self,
        paragraph: &str,
        font: FontId,
        font_size: Pt,
        font_height_offset: Pt,
    ) {
        let shaped_text = shape_text(
            self.document.pdf_document(),
            font,
            font_size,
            font_height_offset,
            paragraph,
            Some(self.remaining_width_from_cursor()),
        );

        self.push_shaped_text(shaped_text, font_size, font_height_offset);
    }

    fn push_shaped_text(&mut self, text: ShapedText, font_size: Pt, font_height_offset: Pt) {
        // Do we need to cut the text?
        let (first, rest) = split_shaped_text(
            text,
            font_size,
            font_height_offset,
            self.remaining_height_from_cursor(),
        );

        println!("Pushing text with height: {:?}", Mm::from(Pt(first.height)));

        println!(
            "Pushing text at {:?} {:?}",
            Mm::from(self.cursor.x),
            self.document.style().height - Mm::from(self.cursor.y)
        );

        let ops = first.get_ops(self.cursor);
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

    pub fn push_square(&mut self, size: Pt) {
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

        self.pages
            .last_mut()
            .expect("Always at least one page")
            .extend(ops);
    }

    /// Currently the prefix is just a box.
    pub fn push_text_with_prefix(
        &mut self,
        text: &str,
        font: FontId,
        font_size: Pt,
        font_height_offset: Pt,
    ) {
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
        println!(
            "Generating group builder at {:?}",
            self.document.style().height - Mm::from(self.cursor.y)
        );
        println!(
            "Remaining height from cursor: {:?}",
            self.remaining_height_from_cursor()
        );
        println!("Try same page: {:?}", try_same_page);

        let (origin, new_page) = match try_same_page {
            Some(height)
                if height <= self.document.style().inner_height()
                    && self.remaining_height_from_cursor() < height =>
            {
                println!("Cant fit");
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
        }
    }

    pub fn draw_outline(&mut self, padding: &Padding, outline: &TextOutline) {
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

    pub fn update_cursor(&mut self, y: Pt) {
        self.cursor.y = y;
    }

    fn remaining_height_from_cursor(&self) -> Mm {
        Mm::from(self.cursor.y) - self.document.style().padding.bottom - self.added_padding_bottom
    }

    fn remaining_width_from_cursor(&self) -> Mm {
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
    }
}
