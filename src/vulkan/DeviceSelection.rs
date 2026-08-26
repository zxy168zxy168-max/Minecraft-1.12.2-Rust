use ash::{vk, Instance};

#[derive(Debug, Clone)]
pub struct PhysicalDeviceCandidate {
    pub handle: vk::PhysicalDevice,
    pub properties: vk::PhysicalDeviceProperties,
    pub graphics_queue_family: u32,
}

pub unsafe fn select_physical_device(
    instance: &Instance,
    prefer_discrete: bool,
) -> Option<PhysicalDeviceCandidate> {
    let mut candidates = instance
        .enumerate_physical_devices()
        .ok()?
        .into_iter()
        .filter_map(|handle| {
            let properties = instance.get_physical_device_properties(handle);
            let queue_families = instance.get_physical_device_queue_family_properties(handle);
            let graphics_queue_family = queue_families
                .iter()
                .position(|family| family.queue_flags.contains(vk::QueueFlags::GRAPHICS))?
                as u32;
            Some(PhysicalDeviceCandidate {
                handle,
                properties,
                graphics_queue_family,
            })
        })
        .collect::<Vec<_>>();

    candidates.sort_by_key(|candidate| {
        let rank = match candidate.properties.device_type {
            vk::PhysicalDeviceType::DISCRETE_GPU if prefer_discrete => 0,
            vk::PhysicalDeviceType::INTEGRATED_GPU => 1,
            vk::PhysicalDeviceType::DISCRETE_GPU => 2,
            vk::PhysicalDeviceType::VIRTUAL_GPU => 3,
            vk::PhysicalDeviceType::CPU => 4,
            _ => 5,
        };
        (
            rank,
            std::cmp::Reverse(candidate.properties.limits.max_image_dimension2_d),
        )
    });
    candidates.into_iter().next()
}
