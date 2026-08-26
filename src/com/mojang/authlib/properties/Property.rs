#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property {
    name: String,
    value: String,
    signature: Option<String>,
}

impl Property {
    pub fn new(
        name: impl Into<String>,
        value: impl Into<String>,
        signature: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            signature,
        }
    }

    pub fn getName(&self) -> &str {
        &self.name
    }
    pub fn getValue(&self) -> &str {
        &self.value
    }
    pub fn getSignature(&self) -> Option<&str> {
        self.signature.as_deref()
    }
    pub const fn hasSignature(&self) -> bool {
        self.signature.is_some()
    }
}
