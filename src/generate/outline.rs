use printpdf::{Pt, Rgb};

pub struct TextOutline {
    pub color: Rgb,
    pub thickness: Pt,
}

impl Default for TextOutline {
    fn default() -> Self {
        Self {
            color: Rgb {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                icc_profile: None,
            },
            thickness: Pt(1.0),
        }
    }
}
