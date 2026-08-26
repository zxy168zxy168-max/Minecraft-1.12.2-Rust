use crate::net::minecraft::util::EnumFacing::EnumFacing;

/// Rust equivalent of MCP 1.12.2 `ModelRotation`'s sixteen X/Y quarter-turns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModelRotation {
    x: i32,
    y: i32,
    quartersX: usize,
    quartersY: usize,
}

impl ModelRotation {
    pub fn new(x: i32, y: i32) -> Self {
        let x = x.rem_euclid(360) / 90 * 90;
        let y = y.rem_euclid(360) / 90 * 90;
        Self {
            x,
            y,
            quartersX: (x / 90).unsigned_abs() as usize,
            quartersY: (y / 90).unsigned_abs() as usize,
        }
    }

    pub const fn x(self) -> i32 {
        self.x
    }
    pub const fn y(self) -> i32 {
        self.y
    }

    /// `ModelRotation.rotateFace` / `rotate(EnumFacing)`.
    pub fn rotateFace(self, mut facing: EnumFacing) -> EnumFacing {
        for _ in 0..self.quartersX {
            facing = rotate_around_x(facing);
        }
        if !is_y_axis(facing) {
            for _ in 0..self.quartersY {
                facing = rotate_around_y(facing);
            }
        }
        facing
    }

    /// `ModelRotation.rotateVertex` / `rotate(EnumFacing, int)`.
    pub fn rotateVertex(self, facing: EnumFacing, vertexIndex: usize) -> usize {
        let mut index = vertexIndex;
        if is_x_axis(facing) {
            index = (index + self.quartersX) % 4;
        }

        let mut rotatedFacing = facing;
        for _ in 0..self.quartersX {
            rotatedFacing = rotate_around_x(rotatedFacing);
        }
        if is_y_axis(rotatedFacing) {
            index = (index + self.quartersY) % 4;
        }
        index
    }

    /// Matrix order is `Y * X`, matching MCP's `Matrix4f.mul(y, x, matrix)`.
    pub fn transformVertex(self, mut position: [f32; 3]) -> [f32; 3] {
        let origin = [0.5, 0.5, 0.5];
        position = subtract(position, origin);
        if self.x != 0 {
            position = rotate_axis(position, 'x', -(self.x as f32).to_radians());
        }
        if self.y != 0 {
            position = rotate_axis(position, 'y', -(self.y as f32).to_radians());
        }
        add(position, origin)
    }
}

fn is_x_axis(facing: EnumFacing) -> bool {
    matches!(facing, EnumFacing::West | EnumFacing::East)
}

fn is_y_axis(facing: EnumFacing) -> bool {
    matches!(facing, EnumFacing::Down | EnumFacing::Up)
}

fn rotate_around_x(facing: EnumFacing) -> EnumFacing {
    match facing {
        EnumFacing::North => EnumFacing::Down,
        EnumFacing::Down => EnumFacing::South,
        EnumFacing::South => EnumFacing::Up,
        EnumFacing::Up => EnumFacing::North,
        EnumFacing::West => EnumFacing::West,
        EnumFacing::East => EnumFacing::East,
    }
}

fn rotate_around_y(facing: EnumFacing) -> EnumFacing {
    match facing {
        EnumFacing::North => EnumFacing::East,
        EnumFacing::East => EnumFacing::South,
        EnumFacing::South => EnumFacing::West,
        EnumFacing::West => EnumFacing::North,
        EnumFacing::Down => EnumFacing::Down,
        EnumFacing::Up => EnumFacing::Up,
    }
}

fn rotate_axis(value: [f32; 3], axis: char, radians: f32) -> [f32; 3] {
    let (sin, cos) = radians.sin_cos();
    match axis {
        'x' => [
            value[0],
            value[1] * cos - value[2] * sin,
            value[1] * sin + value[2] * cos,
        ],
        'y' => [
            value[0] * cos + value[2] * sin,
            value[1],
            -value[0] * sin + value[2] * cos,
        ],
        _ => value,
    }
}

fn add(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] + right[0], left[1] + right[1], left[2] + right[2]]
}

fn subtract(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotations_match_mcp_face_and_vertex_rules() {
        let rotation = ModelRotation::new(90, 90);
        assert_eq!(rotation.rotateFace(EnumFacing::Up), EnumFacing::East);
        assert_eq!(rotation.rotateVertex(EnumFacing::East, 0), 1);
        assert_eq!(
            ModelRotation::new(0, 90).rotateFace(EnumFacing::North),
            EnumFacing::East
        );
    }
}
