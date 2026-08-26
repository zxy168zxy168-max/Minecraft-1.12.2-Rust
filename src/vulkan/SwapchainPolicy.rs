use ash::vk;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwapchainChoice {
    pub surface_format: vk::SurfaceFormatKHR,
    pub present_mode: vk::PresentModeKHR,
    pub extent: vk::Extent2D,
    pub image_count: u32,
    pub pre_transform: vk::SurfaceTransformFlagsKHR,
    pub composite_alpha: vk::CompositeAlphaFlagsKHR,
}

/// Selects Vulkan swapchain properties without changing any Minecraft-visible
/// setting. `enable_vsync` maps the original option to FIFO; when disabled,
/// IMMEDIATE is preferred because it is the closest Vulkan equivalent of
/// LWJGL with VSync disabled. MAILBOX is the non-tearing fallback.
pub fn choose_swapchain(
    capabilities: &vk::SurfaceCapabilitiesKHR,
    formats: &[vk::SurfaceFormatKHR],
    present_modes: &[vk::PresentModeKHR],
    requested_width: u32,
    requested_height: u32,
    enable_vsync: bool,
) -> Option<SwapchainChoice> {
    let surface_format = choose_surface_format(formats)?;
    let present_mode = choose_present_mode(present_modes, enable_vsync);
    let extent = choose_extent(capabilities, requested_width, requested_height);
    let image_count = choose_image_count(capabilities);
    let pre_transform = if capabilities
        .supported_transforms
        .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
    {
        vk::SurfaceTransformFlagsKHR::IDENTITY
    } else {
        capabilities.current_transform
    };
    let composite_alpha = choose_composite_alpha(capabilities.supported_composite_alpha);
    Some(SwapchainChoice {
        surface_format,
        present_mode,
        extent,
        image_count,
        pre_transform,
        composite_alpha,
    })
}

pub fn choose_surface_format(formats: &[vk::SurfaceFormatKHR]) -> Option<vk::SurfaceFormatKHR> {
    if formats.len() == 1 && formats[0].format == vk::Format::UNDEFINED {
        return Some(vk::SurfaceFormatKHR {
            format: vk::Format::B8G8R8A8_SRGB,
            color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
        });
    }
    const PREFERRED: [vk::Format; 4] = [
        vk::Format::B8G8R8A8_SRGB,
        vk::Format::R8G8B8A8_SRGB,
        vk::Format::B8G8R8A8_UNORM,
        vk::Format::R8G8B8A8_UNORM,
    ];
    for format in PREFERRED {
        if let Some(candidate) = formats.iter().copied().find(|candidate| {
            candidate.format == format && candidate.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        }) {
            return Some(candidate);
        }
    }
    formats.first().copied()
}

pub fn choose_present_mode(
    present_modes: &[vk::PresentModeKHR],
    enable_vsync: bool,
) -> vk::PresentModeKHR {
    if enable_vsync {
        return vk::PresentModeKHR::FIFO;
    }
    if present_modes.contains(&vk::PresentModeKHR::IMMEDIATE) {
        vk::PresentModeKHR::IMMEDIATE
    } else if present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
        vk::PresentModeKHR::MAILBOX
    } else {
        vk::PresentModeKHR::FIFO
    }
}

pub fn choose_extent(
    capabilities: &vk::SurfaceCapabilitiesKHR,
    requested_width: u32,
    requested_height: u32,
) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        return capabilities.current_extent;
    }
    vk::Extent2D {
        width: requested_width.clamp(
            capabilities.min_image_extent.width,
            capabilities.max_image_extent.width,
        ),
        height: requested_height.clamp(
            capabilities.min_image_extent.height,
            capabilities.max_image_extent.height,
        ),
    }
}

pub fn choose_image_count(capabilities: &vk::SurfaceCapabilitiesKHR) -> u32 {
    let preferred = capabilities.min_image_count.saturating_add(1);
    if capabilities.max_image_count > 0 {
        preferred.min(capabilities.max_image_count)
    } else {
        preferred
    }
}

fn choose_composite_alpha(supported: vk::CompositeAlphaFlagsKHR) -> vk::CompositeAlphaFlagsKHR {
    for candidate in [
        vk::CompositeAlphaFlagsKHR::OPAQUE,
        vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED,
        vk::CompositeAlphaFlagsKHR::POST_MULTIPLIED,
        vk::CompositeAlphaFlagsKHR::INHERIT,
    ] {
        if supported.contains(candidate) {
            return candidate;
        }
    }
    vk::CompositeAlphaFlagsKHR::OPAQUE
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capabilities() -> vk::SurfaceCapabilitiesKHR {
        vk::SurfaceCapabilitiesKHR {
            min_image_count: 2,
            max_image_count: 3,
            current_extent: vk::Extent2D {
                width: u32::MAX,
                height: u32::MAX,
            },
            min_image_extent: vk::Extent2D {
                width: 320,
                height: 240,
            },
            max_image_extent: vk::Extent2D {
                width: 3840,
                height: 2160,
            },
            supported_transforms: vk::SurfaceTransformFlagsKHR::IDENTITY,
            current_transform: vk::SurfaceTransformFlagsKHR::IDENTITY,
            supported_composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            ..Default::default()
        }
    }

    #[test]
    fn vsync_always_uses_fifo() {
        assert_eq!(
            choose_present_mode(&[vk::PresentModeKHR::MAILBOX], true),
            vk::PresentModeKHR::FIFO
        );
    }

    #[test]
    fn non_vsync_prefers_immediate_then_mailbox() {
        assert_eq!(
            choose_present_mode(
                &[
                    vk::PresentModeKHR::FIFO,
                    vk::PresentModeKHR::IMMEDIATE,
                    vk::PresentModeKHR::MAILBOX
                ],
                false,
            ),
            vk::PresentModeKHR::IMMEDIATE
        );
        assert_eq!(
            choose_present_mode(
                &[vk::PresentModeKHR::FIFO, vk::PresentModeKHR::MAILBOX],
                false
            ),
            vk::PresentModeKHR::MAILBOX
        );
    }

    #[test]
    fn extent_is_clamped_when_surface_does_not_fix_it() {
        let value = choose_extent(&capabilities(), 200, 3000);
        assert_eq!(
            value,
            vk::Extent2D {
                width: 320,
                height: 2160
            }
        );
    }

    #[test]
    fn image_count_uses_minimum_plus_one_with_maximum_cap() {
        assert_eq!(choose_image_count(&capabilities()), 3);
        let mut uncapped = capabilities();
        uncapped.max_image_count = 0;
        assert_eq!(choose_image_count(&uncapped), 3);
    }
}
