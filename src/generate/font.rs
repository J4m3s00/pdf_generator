use printpdf::{FontId, Pt};

#[derive(Clone, Debug)]
pub struct Font {
    font_id: FontId,
    font_size: Pt,
    font_height_offset: Pt,
}

impl Font {
    pub(crate) fn new(font_id: FontId, font_size: Pt, font_height_offset: Pt) -> Self {
        Self {
            font_id,
            font_size,
            font_height_offset,
        }
    }

    pub fn font_id(&self) -> FontId {
        self.font_id.clone()
    }

    pub fn font_size(&self) -> Pt {
        self.font_size
    }

    pub fn font_height_offset(&self) -> Pt {
        self.font_height_offset
    }

    pub fn with_font_size(&self, font_size: Pt) -> Self {
        Self {
            font_id: self.font_id.clone(),
            font_size,
            font_height_offset: self.font_height_offset,
        }
    }

    pub fn with_font_height_offset(&self, font_height_offset: Pt) -> Self {
        Self {
            font_id: self.font_id.clone(),
            font_size: self.font_size,
            font_height_offset,
        }
    }
}
