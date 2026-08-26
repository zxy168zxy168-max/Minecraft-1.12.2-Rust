use crate::net::minecraft::util::MovementInput::MovementInput;

/// Backend-neutral pressed-key snapshot. The original class reads
/// `GameSettings.keyBind*`; the desktop backend updates the same logical key
/// states from winit before invoking `updatePlayerMoveState` each client tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MovementKeyState {
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub sneak: bool,
    /// Physical state of MCP `GameSettings.keyBindSprint`. It is not part of
    /// `MovementInput`, but travels with this backend-neutral tick snapshot.
    pub sprint: bool,
}

/// Port of MCP `MovementInputFromOptions.updatePlayerMoveState`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MovementInputFromOptions {
    pub movementInput: MovementInput,
}

impl MovementInputFromOptions {
    pub const fn new() -> Self {
        Self {
            movementInput: MovementInput {
                moveStrafe: 0.0,
                field_192832_b: 0.0,
                forwardKeyDown: false,
                backKeyDown: false,
                leftKeyDown: false,
                rightKeyDown: false,
                jump: false,
                sneak: false,
            },
        }
    }

    pub fn updatePlayerMoveState(&mut self, keys: MovementKeyState) {
        let input = &mut self.movementInput;
        input.moveStrafe = 0.0;
        input.field_192832_b = 0.0;

        if keys.forward {
            input.field_192832_b += 1.0;
            input.forwardKeyDown = true;
        } else {
            input.forwardKeyDown = false;
        }

        if keys.back {
            input.field_192832_b -= 1.0;
            input.backKeyDown = true;
        } else {
            input.backKeyDown = false;
        }

        if keys.left {
            input.moveStrafe += 1.0;
            input.leftKeyDown = true;
        } else {
            input.leftKeyDown = false;
        }

        if keys.right {
            input.moveStrafe -= 1.0;
            input.rightKeyDown = true;
        } else {
            input.rightKeyDown = false;
        }

        input.jump = keys.jump;
        input.sneak = keys.sneak;

        if input.sneak {
            input.moveStrafe = (input.moveStrafe as f64 * 0.3) as f32;
            input.field_192832_b = (input.field_192832_b as f64 * 0.3) as f32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opposing_keys_cancel_and_sneak_scales_like_mcp() {
        let mut movement = MovementInputFromOptions::new();
        movement.updatePlayerMoveState(MovementKeyState {
            forward: true,
            back: true,
            left: true,
            right: false,
            jump: true,
            sneak: true,
            sprint: false,
        });
        assert_eq!(movement.movementInput.field_192832_b, 0.0);
        assert!((movement.movementInput.moveStrafe - 0.3).abs() < f32::EPSILON);
        assert!(movement.movementInput.jump);
    }
}
