use printpdf::Mm;

use crate::generate::{element::Element, outline::LineStyle, padding::Padding};

pub struct Group {
    pub elements: Vec<Box<dyn Element>>,
    pub outline: Option<LineStyle>,
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
        E: Element + 'static,
    {
        self.elements.push(Box::new(element));
    }

    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_outline(mut self, line_style: LineStyle) -> Self {
        self.outline = Some(line_style);
        self
    }
}

impl Element for Group {
    fn display_name(&self) -> &str {
        "Group"
    }

    fn calculate_width<'a>(&self, builder: &super::element_builder::ElementBuilder<'a>) -> Mm {
        let group_builder = builder.generate_group_builder(&self.padding, None);

        self.elements
            .iter()
            .map(|elem| elem.calculate_width(&group_builder))
            .fold(Mm(0.0), |v, h| v + h)
            + self.padding.left
            + self.padding.right
    }

    fn calculate_height<'a>(&self, builder: &super::element_builder::ElementBuilder<'a>) -> Mm {
        // We need to compute the text height with the padding of the group
        let group_builder = builder.generate_group_builder(&self.padding, None);

        self.elements
            .iter()
            .map(|elem| elem.calculate_height(&group_builder))
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
