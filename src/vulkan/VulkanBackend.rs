use crate::net::minecraft::client::settings::GameSettings::VulkanBackendSettings;
use crate::vulkan::DeviceSelection::{select_physical_device, PhysicalDeviceCandidate};
use ash::{vk, Device, Entry, Instance};
use std::ffi::{CStr, CString};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VulkanBackendError {
    #[error("failed to create Vulkan instance: {0:?}")]
    Instance(vk::Result),
    #[error("no Vulkan physical device with a graphics queue was found")]
    NoPhysicalDevice,
    #[error("failed to create Vulkan logical device: {0:?}")]
    Device(vk::Result),
}

pub struct VulkanBackend {
    pub entry: Entry,
    pub instance: Instance,
    pub physical_device: PhysicalDeviceCandidate,
    pub device: Device,
    pub graphics_queue: vk::Queue,
}

impl VulkanBackend {
    /// Creates the device-only portion of the backend. Surface and swapchain
    /// creation are intentionally separate because the client window lifecycle
    /// is driven by Winit 0.30's application handler.
    pub fn probe(settings: &VulkanBackendSettings) -> Result<Self, VulkanBackendError> {
        let entry = Entry::linked();
        let application_name = CString::new("Minecraft 1.12.2 Rust Client").expect("static string");
        let engine_name =
            CString::new("MC112 Vulkan Compatibility Renderer").expect("static string");
        let application_info = vk::ApplicationInfo::default()
            .application_name(&application_name)
            .application_version(vk::make_api_version(0, 0, 1, 0))
            .engine_name(&engine_name)
            .engine_version(vk::make_api_version(0, 0, 1, 0))
            .api_version(vk::API_VERSION_1_1);

        let available_layers =
            unsafe { entry.enumerate_instance_layer_properties() }.unwrap_or_default();
        let validation_name = CString::new("VK_LAYER_KHRONOS_validation").expect("static string");
        let validation_available = available_layers.iter().any(|layer| {
            let name = unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) };
            name == validation_name.as_c_str()
        });
        let enabled_layer_names = if settings.enable_validation && validation_available {
            vec![validation_name.as_ptr()]
        } else {
            Vec::new()
        };

        let instance_info = vk::InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_layer_names(&enabled_layer_names);
        let instance = unsafe { entry.create_instance(&instance_info, None) }
            .map_err(VulkanBackendError::Instance)?;

        let physical_device =
            unsafe { select_physical_device(&instance, settings.prefer_discrete_gpu) }
                .ok_or(VulkanBackendError::NoPhysicalDevice)?;
        let priority = [1.0_f32];
        let queue_info = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(physical_device.graphics_queue_family)
            .queue_priorities(&priority)];
        let device_info = vk::DeviceCreateInfo::default().queue_create_infos(&queue_info);
        let device = unsafe { instance.create_device(physical_device.handle, &device_info, None) }
            .map_err(VulkanBackendError::Device)?;
        let graphics_queue =
            unsafe { device.get_device_queue(physical_device.graphics_queue_family, 0) };

        Ok(Self {
            entry,
            instance,
            physical_device,
            device,
            graphics_queue,
        })
    }
}

impl Drop for VulkanBackend {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}
