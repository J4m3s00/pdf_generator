use printpdf::Mm;

#[derive(Clone, Debug)]
pub struct Padding {
    pub top: Mm,
    pub bottom: Mm,
    pub left: Mm,
    pub right: Mm,
}

impl Padding {
    pub fn new(top: Mm, bottom: Mm, left: Mm, right: Mm) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }

    pub fn none() -> Self {
        Self {
            top: Mm(0.0),
            bottom: Mm(0.0),
            left: Mm(0.0),
            right: Mm(0.0),
        }
    }

    #[allow(dead_code)]
    pub fn all(padding: Mm) -> Self {
        Self {
            top: padding,
            bottom: padding,
            left: padding,
            right: padding,
        }
    }

    pub fn xy(x: Mm, y: Mm) -> Self {
        Self {
            top: y,
            bottom: y,
            left: x,
            right: x,
        }
    }

    #[allow(dead_code)]
    pub fn x(x: Mm) -> Self {
        Self {
            top: Mm(0.0),
            bottom: Mm(0.0),
            left: x,
            right: x,
        }
    }

    #[allow(dead_code)]
    pub fn y(y: Mm) -> Self {
        Self {
            top: y,
            bottom: y,
            left: Mm(0.0),
            right: Mm(0.0),
        }
    }

    #[allow(dead_code)]
    pub fn top(val: Mm) -> Self {
        Self {
            top: val,
            bottom: Mm(0.0),
            left: Mm(0.0),
            right: Mm(0.0),
        }
    }

    #[allow(dead_code)]
    pub fn bottom(val: Mm) -> Self {
        Self {
            top: Mm(0.0),
            bottom: val,
            left: Mm(0.0),
            right: Mm(0.0),
        }
    }

    #[allow(dead_code)]
    pub fn left(val: Mm) -> Self {
        Self {
            top: Mm(0.0),
            bottom: Mm(0.0),
            left: val,
            right: Mm(0.0),
        }
    }

    #[allow(dead_code)]
    pub fn right(val: Mm) -> Self {
        Self {
            top: Mm(0.0),
            bottom: Mm(0.0),
            left: Mm(0.0),
            right: val,
        }
    }
}
