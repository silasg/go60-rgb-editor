pub mod color;
pub mod layer;
pub mod config;

pub use color::{RgbColor, ColorDef, ColorPalette, ColorKind};
pub use layer::{Layer, ROW_COUNT, MAIN_ROW_COLS, THUMB_ROW_COLS, COLORS_PER_PICKER_ROW};
pub use config::Config;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Half {
    #[default]
    Left,
    Right,
}

impl Half {
    pub fn opposite(self) -> Self {
        match self {
            Half::Left => Half::Right,
            Half::Right => Half::Left,
        }
    }

    pub fn is_left(self) -> bool {
        self == Half::Left
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RgbPos {
    pub row: usize,
    pub col: usize,
    pub half: Half,
}
