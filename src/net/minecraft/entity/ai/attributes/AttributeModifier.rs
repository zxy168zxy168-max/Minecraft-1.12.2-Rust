use uuid::Uuid;

/// Data-only client port of MCP `AttributeModifier`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AttributeModifier {
    id: Uuid,
    amount: f64,
    operation: i8,
}

impl AttributeModifier {
    pub const fn new(id: Uuid, amount: f64, operation: i8) -> Self {
        Self {
            id,
            amount,
            operation,
        }
    }

    pub const fn getID(&self) -> Uuid {
        self.id
    }
    pub const fn getAmount(&self) -> f64 {
        self.amount
    }
    pub const fn getOperation(&self) -> i8 {
        self.operation
    }
}
