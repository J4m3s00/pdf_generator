use printpdf::{Mm, Rgb};

use crate::generate::font::Font;

/// Where to place the page number on the page.
///
/// Each variant carries an `(x, y)` offset (in `Mm`) measured inward from the
/// named corner. For example `PageNumberPosition::BottomRight(Mm(10.0), Mm(10.0))`
/// places the page number 10mm from the right edge and 10mm from the bottom edge.
#[derive(Clone, Debug)]
pub enum PageNumberPosition {
    TopLeft(Mm, Mm),
    TopRight(Mm, Mm),
    BottomLeft(Mm, Mm),
    BottomRight(Mm, Mm),
}

/// Configuration for rendering page numbers (e.g. `3/10`) on every page.
#[derive(Clone, Debug)]
pub struct PageNumberOptions {
    pub position: PageNumberPosition,
    pub font: Font,
    pub color: Rgb,
    /// If `true`, the first page will not have a page number rendered on it.
    pub skip_first_page: bool,
}

impl PageNumberOptions {
    pub fn new(position: PageNumberPosition, font: Font) -> Self {
        Self {
            position,
            font,
            color: Rgb {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                icc_profile: None,
            },
            skip_first_page: false,
        }
    }

    pub fn with_color(mut self, color: Rgb) -> Self {
        self.color = color;
        self
    }

    pub fn with_skip_first_page(mut self, skip_first_page: bool) -> Self {
        self.skip_first_page = skip_first_page;
        self
    }
}
