#[derive(Debug, Clone, Copy)]
pub struct Damage {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

impl Damage {
    pub fn full(fb_width_words: i32, fb_height: i32) -> Self {
        Self { x1: 0, y1: 0, x2: fb_width_words - 1, y2: fb_height - 1 }
    }

    pub fn cleared(fb_width_words: i32, fb_height: i32) -> Self {
        Self { x1: fb_width_words, y1: fb_height, x2: 0, y2: 0 }
    }

    pub fn update_word_index(&mut self, fb_width_words: i32, fb_height: i32, w_index: i32) {
        let row = w_index / fb_width_words;
        let col = w_index % fb_width_words;
        if row < fb_height {
            self.x1 = self.x1.min(col);
            self.x2 = self.x2.max(col);
            self.y1 = self.y1.min(row);
            self.y2 = self.y2.max(row);
        }
    }
}