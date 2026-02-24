use printpdf::{Greyscale, Mm, Point, Pt};
use taffy::{
    AvailableSpace, Display, NodeId, Overflow, Rect, Size, Style, TaffyTree,
    prelude::{auto, length},
};

use crate::generate::{element::Element, font::Font};

pub struct Table {
    font: Font,
    num_cols: usize,
    header: Option<(Vec<String>, Font)>,
    content: Vec<Vec<String>>,
}

impl Table {
    pub fn new(num_cols: usize, font: Font) -> Self {
        Self {
            num_cols,
            header: None,
            content: Vec::default(),
            font,
        }
    }

    pub fn set_header(&mut self, header: Vec<String>, font: Option<Font>) {
        assert_eq!(header.len(), self.num_cols);
        self.header = Some((header, font.unwrap_or(self.font.clone())));
    }

    pub fn add_row(&mut self, row: impl IntoIterator<Item = String>) {
        let vec = row.into_iter().collect::<Vec<_>>();

        assert_eq!(vec.len(), self.num_cols);
        self.content.push(vec);
    }

    fn build_headers<'a>(
        &self,
        builder: &mut super::element_builder::ElementBuilder<'a>,
        built: &BuiltTable,
    ) {
        if let Some(header_cells) = built.header_cells() {
            for (i, node) in header_cells.iter().enumerate() {
                self.build_cell(builder, built, i, *node, true);
            }
        }
    }

    fn build_cell<'a>(
        &self,
        builder: &mut super::element_builder::ElementBuilder<'a>,
        built: &BuiltTable,
        index: usize,
        node: NodeId,
        is_header: bool,
    ) {
        let layout = built.taffy.layout(node).unwrap();
        let row = index / self.num_cols;
        let col = index % self.num_cols;

        // Fill rect
        let color = if is_header {
            printpdf::Color::Greyscale(Greyscale::new(0.9, None))
        } else {
            match row % 2 {
                0 => printpdf::Color::Greyscale(Greyscale::new(0.95, None)),
                1 => printpdf::Color::Greyscale(Greyscale::new(1.0, None)),
                _ => unreachable!(),
            }
        };

        builder.fill_rect_dont_change_cursor(Pt(layout.size.width), Pt(layout.size.height), color);

        if let Some(content) = built.taffy.get_node_context(node) {
            builder.push_text_dont_change_cursor(
                &content.content,
                &content.font,
                Point {
                    x: Pt(layout.padding.left),
                    y: Pt(layout.padding.top),
                },
                Some(Pt(layout.content_box_width() + 3.0)),
            );
            println!(
                "Building cell: {}, width: {:?}, height: {:?}",
                content.content,
                Mm::from(Pt(layout.content_box_width())),
                Mm::from(Pt(layout.content_box_height()))
            );
            builder.cursor.x += Pt(layout.size.width);
        }
        if col == self.num_cols - 1 {
            builder.reset_cursor_x();
            if builder.advance_cursor(Pt(layout.size.height)) {
                self.build_headers(builder, built);
            }
        }
    }

    fn row_height(built: &BuiltTable, nodes: &[NodeId]) -> printpdf::Pt {
        let mut res = Pt(0.0);
        for node in nodes.iter() {
            let layout = built.taffy.layout(*node).unwrap();
            res = res.max(Pt(layout.size.height))
        }
        res
    }
}

impl Element for Table {
    fn display_name(&self) -> &str {
        "Table"
    }

    fn calculate_width<'a>(
        &self,
        builder: &super::element_builder::ElementBuilder<'a>,
    ) -> printpdf::Pt {
        let built = BuiltTable::build(self, builder);
        let root_layout = built.taffy.layout(built.root).unwrap();

        Pt(root_layout.size.width)
    }

    fn calculate_height<'a>(
        &self,
        builder: &super::element_builder::ElementBuilder<'a>,
    ) -> printpdf::Pt {
        let built = BuiltTable::build(self, builder);
        let mut mut_builder = builder.clone();
        let mut height = Pt(0.0);
        if let Some(header_cells) = built.header_cells() {
            let header_height = Self::row_height(&built, header_cells);
            height += header_height;
            mut_builder.advance_cursor(header_height);
        }

        for row in 0..self.content.len() {
            let cells = built.row_cells(row);
            let row_height = Self::row_height(&built, cells);
            height += row_height;
            if mut_builder.advance_cursor(row_height) {
                if let Some(header_cells) = built.header_cells() {
                    let header_height = Self::row_height(&built, header_cells);
                    height += header_height;
                    mut_builder.advance_cursor(header_height);
                }
            }
        }

        height
    }

    fn build<'a>(&self, builder: &mut super::element_builder::ElementBuilder<'a>) {
        let built = BuiltTable::build(self, builder);

        self.build_headers(builder, &built);
        for (i, node) in built.content_cells().iter().enumerate() {
            self.build_cell(builder, &built, i, *node, false);
        }
    }
}

struct CellContent {
    content: String,
    font: Font,
}

struct BuiltTable {
    taffy: TaffyTree<CellContent>,
    root: NodeId,
    has_header: bool,
    cells: Vec<NodeId>,
    num_cols: usize,
}

impl BuiltTable {
    fn header_cells(&self) -> Option<&[NodeId]> {
        if self.has_header {
            Some(&self.cells[0..self.num_cols])
        } else {
            None
        }
    }

    fn content_cells(&self) -> &[NodeId] {
        if self.has_header {
            &self.cells[self.num_cols..]
        } else {
            &self.cells
        }
    }

    fn row_cells(&self, row: usize) -> &[NodeId] {
        let start = row * self.num_cols;
        let end = start + self.num_cols;
        if start >= self.cells.len() {
            return &[];
        }

        &self.cells[start..end]
    }

    fn content_to_cell(taffy: &mut TaffyTree<CellContent>, cell: CellContent) -> taffy::NodeId {
        taffy
            .new_leaf_with_context(
                Style {
                    size: Size {
                        width: auto(),
                        height: auto(),
                    },
                    padding: Rect::length(5.0),
                    overflow: taffy::Point {
                        x: Overflow::Clip,
                        y: Overflow::Visible,
                    },
                    ..Default::default()
                },
                cell,
            )
            .unwrap()
    }

    fn build(value: &Table, builder: &super::element_builder::ElementBuilder) -> Self {
        let mut taffy = TaffyTree::<CellContent>::new();
        taffy.disable_rounding();

        let grid_style = Style {
            display: Display::Grid,
            size: Size {
                width: length(builder.remaining_width_from_cursor().0),
                height: auto(),
            },
            min_size: Size {
                width: length(builder.remaining_width_from_cursor().0),
                height: auto(),
            },
            max_size: Size {
                width: length(builder.remaining_width_from_cursor().0),
                height: auto(),
            },
            grid_template_columns: vec![
                // minmax(
                //     MinTrackSizingFunction::min_content(),
                //     MaxTrackSizingFunction::fr(1.0)
                // );
                auto();
                value.num_cols
            ],
            ..Default::default()
        };

        let cells = value
            .header
            .clone()
            .map(|header| {
                header
                    .0
                    .iter()
                    .map(|cell| CellContent {
                        content: cell.clone(),
                        font: header.1.clone(),
                    })
                    .collect()
            })
            .unwrap_or(Vec::new())
            .into_iter()
            .chain(value.content.iter().flat_map(|row| {
                row.iter().map(|cell| CellContent {
                    content: cell.clone(),
                    font: value.font.clone(),
                })
            }))
            .map(|cell| Self::content_to_cell(&mut taffy, cell))
            .collect::<Vec<_>>();

        let root = taffy
            .new_with_children(grid_style, cells.as_slice())
            .unwrap();

        taffy
            .compute_layout_with_measure(
                root,
                length(builder.remaining_width_from_cursor().0),
                |known_dimensions, available_space, _, node_context, _| {
                    if let Size {
                        width: Some(width),
                        height: Some(height),
                    } = known_dimensions
                    {
                        return Size { width, height };
                    }

                    match node_context {
                        None => Size::ZERO,
                        Some(content) => {
                            let min_text_width = builder
                                .measure_text_min_content(&content.content, &content.font)
                                .0;
                            let max_text_width = builder
                                .measure_text_manuel(&content.content, &content.font, None)
                                .0
                                .0;

                            let width = match available_space.width {
                                AvailableSpace::Definite(w) => {
                                    // w.min(max_text_width).max(min_text_width)
                                    w
                                }
                                AvailableSpace::MinContent => min_text_width,
                                AvailableSpace::MaxContent => max_text_width,
                            };

                            let measured = builder.measure_text_manuel(
                                &content.content,
                                &content.font,
                                Some(Pt(width)),
                            );

                            println!("Measure text: {}, available_space: {:?}, width: {:?}, height: {:?}", content.content, available_space, Mm::from(Pt(width)), Mm::from(measured.1));

                            Size {
                                width,
                                height: measured.1.0,
                            }
                        }
                    }
                },
            )
            .unwrap();

        Self {
            taffy,
            root,
            has_header: value.header.is_some(),
            cells,
            num_cols: value.num_cols,
        }
    }
}
