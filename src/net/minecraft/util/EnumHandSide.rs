use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnumHandSide {
    Left,
    Right,
}

impl EnumHandSide {
    pub const fn getId(self) -> i32 {
        match self {
            Self::Left => 0,
            Self::Right => 1,
        }
    }

    pub const fn byId(id: i32) -> Self {
        if id == 0 {
            Self::Left
        } else {
            Self::Right
        }
    }

    pub const fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}
