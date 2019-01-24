pub enum Position {
    Center,
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone)]
pub struct Rect {
    pub x: u32, pub y: u32,
    pub width: u32, pub height: u32,
}

impl Rect {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Rect { x, y, width, height, }
    }

    pub fn relative(&self, pos: Position, width: u32, height: u32) -> Self {
        match pos {
            Position::Center => {
                self.at_center(width, height)
            }
            Position::Left => {
                self.at_left(width, height)
            }
            Position::Right => {
                self.at_right(width, height)
            }
            Position::Top => {
                self.at_top(width, height)
            }
            Position::Bottom => {
                self.at_bottom(width, height)
            }
        }
    }

    pub fn clamp(&self, mut other: Rect) -> Self {

        if !self.contains_point(other.x, other.y) {
            return Rect::new(0, 0, 0, 0);
        }

        if other.bottom() > self.bottom() {
            other.height = self.bottom().saturating_sub(other.y);
        }

        if other.right() > self.right() {
            other.width = self.right().saturating_sub(other.x);
        }

        other
    }

    pub fn contains_width(&self, other: &Rect) -> bool {
        self.x <= other.x && self.width >= other.width
    }

    pub fn contains_height(&self, other: &Rect) -> bool {
        self.y <= other.y && self.height >= other.height
    }

    pub fn contains_point(&self, x: u32, y: u32) -> bool {
        self.x <= x && x <= self.right() && self.y <= y  && y <= self.bottom()
    }
}

impl Rect {
    fn at_center(&self, width: u32, height: u32) -> Self {
        let (cx,cy) = self.center();
        let (hw,hh) = (width/2, height/2);

        let x = if hw > cx { 0 } else { cx - hw };
        let y = if hh > cy { 0 } else { cy - hh };

        Rect::new(x, y, width, height)
    }

    fn at_left(&self, width: u32, height: u32) -> Self {
        let width = if width > self.x { self.x } else { width };
        let x = self.x - width;

        Rect::new(x, self.y, width, height)
    }

    fn at_right(&self, width: u32, height: u32) -> Self {
        Rect::new(self.right(), self.y, width, height)
    }

    fn at_top(&self, width: u32, height: u32) -> Self {
        let height = if height > self.y { self.y } else { height };
        let y = self.y - height;

        Rect::new(self.x, y, width, height)
    }

    fn at_bottom(&self, width: u32, height: u32) -> Self {
        Rect::new(self.x, self.bottom(), width, height)
    }
}

impl Rect {
    pub fn bottom(&self) -> u32 {
        self.y + self.height
    }

    pub fn right(&self) -> u32 {
        self.x + self.width
    }

    pub fn center(&self) -> (u32, u32) {
        ((self.width/2) + self.x, self.height/2 + self.y)
    }
}
