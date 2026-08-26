use serde::{Deserialize, Serialize};

/// User-selected desktop graphics API. This is a Rust-port launcher choice,
/// not a Minecraft 1.12.2 gameplay option. The active backend is chosen before
/// the window surface is created and therefore changes only after restart.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RenderBackend {
    Vulkan,
    OpenGl,
}

impl RenderBackend {
    pub const fn optionValue(self) -> &'static str {
        match self {
            Self::Vulkan => "vulkan",
            Self::OpenGl => "opengl",
        }
    }

    pub const fn displayName(self) -> &'static str {
        match self {
            Self::Vulkan => "Vulkan",
            Self::OpenGl => "OpenGL",
        }
    }

    pub fn parse(value: &str) -> Self {
        if value.eq_ignore_ascii_case("opengl") || value.eq_ignore_ascii_case("gl") {
            Self::OpenGl
        } else {
            Self::Vulkan
        }
    }

    pub const fn toggled(self) -> Self {
        match self {
            Self::Vulkan => Self::OpenGl,
            Self::OpenGl => Self::Vulkan,
        }
    }
}

impl Default for RenderBackend {
    fn default() -> Self {
        Self::Vulkan
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_values_round_trip_and_unknown_values_remain_vulkan_safe() {
        assert_eq!(RenderBackend::parse("vulkan"), RenderBackend::Vulkan);
        assert_eq!(RenderBackend::parse("OpenGL"), RenderBackend::OpenGl);
        assert_eq!(RenderBackend::parse("unknown"), RenderBackend::Vulkan);
        assert_eq!(RenderBackend::OpenGl.optionValue(), "opengl");
    }
}
