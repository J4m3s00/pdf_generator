use std::collections::HashMap;

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
    /// Template used to render the page number text. Supports the
    /// placeholders `{current}` (current page number) and `{total}` (total
    /// number of pages), e.g. `"Page {current}/{total}"`.
    ///
    /// Defaults to `"{current}/{total}"`.
    pub template: String,
    /// Per-page position overrides, keyed by 0-based page index. Pages not
    /// present in this map use `position` instead.
    pub position_overrides: HashMap<usize, PageNumberPosition>,
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
            template: "{current}/{total}".to_string(),
            position_overrides: HashMap::new(),
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

    /// Sets the template used to render the page number text. Supports the
    /// placeholders `{current}` and `{total}`, e.g. `"Page {current}/{total}"`.
    pub fn with_template(mut self, template: impl Into<String>) -> Self {
        self.template = template.into();
        self
    }

    /// Overrides the position for a single page, identified by its 0-based
    /// index. Can be called multiple times to override multiple pages.
    pub fn with_position_override(mut self, page_index: usize, position: PageNumberPosition) -> Self {
        self.position_overrides.insert(page_index, position);
        self
    }

    /// Resolves the position to use for the given 0-based page index, taking
    /// any override into account.
    pub fn position_for_page(&self, page_index: usize) -> &PageNumberPosition {
        self.position_overrides
            .get(&page_index)
            .unwrap_or(&self.position)
    }

    /// Renders the page number text for the given 1-based current page and total page count.
    pub fn render_text(&self, current: usize, total: usize) -> String {
        self.template
            .replace("{current}", &current.to_string())
            .replace("{total}", &total.to_string())
    }
}
