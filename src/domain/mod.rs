pub mod color;
pub mod config;
pub mod cursor;
pub mod editor;
pub mod geometry;
pub mod layer;
pub mod parser;
pub mod undo;

pub use color::{ColorDef, ColorKind, ColorPalette, RgbColor};
pub use config::Config;
pub use layer::Layer;

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
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RgbPos {
    pub row: usize,
    pub col: usize,
    pub half: Half,
}
