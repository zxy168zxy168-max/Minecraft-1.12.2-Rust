use crate::net::minecraft::util::math::MathHelper::sin as minecraft_sin;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::vulkan::GuiDrawList::PanoramaCommand;

/// The fixed off-screen target used by `GuiMainMenu.renderSkybox` in 1.12.2.
pub const PANORAMA_TARGET_SIZE: u32 = 256;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerspectiveSpec {
    pub vertical_fov_degrees: f32,
    pub aspect_ratio: f32,
    pub near_plane: f32,
    pub far_plane: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EulerRotation {
    pub x_degrees: f32,
    pub y_degrees: f32,
    pub z_degrees: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PanoramaFaceDraw {
    pub texture: ResourceLocation,
    pub orientation: EulerRotation,
    pub alpha_u8: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PanoramaSample {
    /// The sub-pixel translation applied before the animated rotations.
    pub translate_x: f32,
    pub translate_y: f32,
    pub pitch_degrees: f32,
    pub yaw_degrees: f32,
    pub faces: [PanoramaFaceDraw; 6],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlurLayer {
    pub horizontal_uv_offset: f32,
    pub alpha: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlurInvocation {
    pub layers: Vec<BlurLayer>,
    /// MCP writes RGB but masks alpha for the panorama accumulation target.
    pub preserve_destination_alpha: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanoramaCompositeVertex {
    pub x: f32,
    pub y: f32,
    pub u: f32,
    pub v: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PanoramaPassPlan {
    pub target_width: u32,
    pub target_height: u32,
    pub perspective: PerspectiveSpec,
    pub base_rotation: EulerRotation,
    pub depth_write: bool,
    pub culling: bool,
    pub alpha_test: bool,
    pub samples: Vec<PanoramaSample>,
    pub blur_invocations: Vec<BlurInvocation>,
    pub composite: [PanoramaCompositeVertex; 4],
}

impl PanoramaPassPlan {
    /// Converts the renderer-independent GUI command into the exact logical
    /// pass sequence of MCP `drawPanorama`, `rotateAndBlurSkybox`, and
    /// `renderSkybox`. Vulkan render passes may implement these operations
    /// differently, but must consume this plan without changing its order or
    /// constants.
    pub fn from_command(command: &PanoramaCommand) -> Self {
        let sample_count = command.first_blur_passes.max(0) as usize;
        let pitch_degrees = minecraft_sin(command.panorama_timer / 400.0) * 25.0 + 20.0;
        let yaw_degrees = -command.panorama_timer * 0.1;

        let samples = (0..sample_count)
            .map(|sample_index| {
                let k = sample_index as i32;
                let translate_x = ((k % 8) as f32 / 8.0 - 0.5) / 64.0;
                let translate_y = ((k / 8) as f32 / 8.0 - 0.5) / 64.0;
                // Java integer division is intentional here.
                let alpha_u8 = (255 / (k + 1)) as u8;
                PanoramaSample {
                    translate_x,
                    translate_y,
                    pitch_degrees,
                    yaw_degrees,
                    faces: std::array::from_fn(|face| PanoramaFaceDraw {
                        texture: command.textures[face].clone(),
                        orientation: face_orientation(face),
                        alpha_u8,
                    }),
                }
            })
            .collect();

        let invocation_count = 1 + command.final_blur_pairs.max(0) as usize * 2;
        let blur_invocations = (0..invocation_count)
            .map(|_| BlurInvocation {
                layers: (0..command.second_blur_passes.max(0))
                    .map(|layer| BlurLayer {
                        horizontal_uv_offset: (layer as f32 - 1.0) / 256.0,
                        alpha: 1.0 / (layer as f32 + 1.0),
                    })
                    .collect(),
                preserve_destination_alpha: true,
            })
            .collect();

        let max_dimension = command.screen_width.max(command.screen_height) as f32;
        let scale = if max_dimension == 0.0 {
            0.0
        } else {
            120.0 / max_dimension
        };
        let vertical_extent = command.screen_height as f32 * scale / 256.0;
        let horizontal_extent = command.screen_width as f32 * scale / 256.0;
        let width = command.screen_width as f32;
        let height = command.screen_height as f32;
        let composite = [
            PanoramaCompositeVertex {
                x: 0.0,
                y: height,
                u: 0.5 - vertical_extent,
                v: 0.5 + horizontal_extent,
            },
            PanoramaCompositeVertex {
                x: width,
                y: height,
                u: 0.5 - vertical_extent,
                v: 0.5 - horizontal_extent,
            },
            PanoramaCompositeVertex {
                x: width,
                y: 0.0,
                u: 0.5 + vertical_extent,
                v: 0.5 - horizontal_extent,
            },
            PanoramaCompositeVertex {
                x: 0.0,
                y: 0.0,
                u: 0.5 + vertical_extent,
                v: 0.5 + horizontal_extent,
            },
        ];

        Self {
            target_width: PANORAMA_TARGET_SIZE,
            target_height: PANORAMA_TARGET_SIZE,
            perspective: PerspectiveSpec {
                vertical_fov_degrees: 120.0,
                aspect_ratio: 1.0,
                near_plane: 0.05,
                far_plane: 10.0,
            },
            base_rotation: EulerRotation {
                x_degrees: 180.0,
                y_degrees: 0.0,
                z_degrees: 90.0,
            },
            depth_write: false,
            culling: false,
            alpha_test: false,
            samples,
            blur_invocations,
            composite,
        }
    }
}

fn face_orientation(face: usize) -> EulerRotation {
    match face {
        0 => EulerRotation {
            x_degrees: 0.0,
            y_degrees: 0.0,
            z_degrees: 0.0,
        },
        1 => EulerRotation {
            x_degrees: 0.0,
            y_degrees: 90.0,
            z_degrees: 0.0,
        },
        2 => EulerRotation {
            x_degrees: 0.0,
            y_degrees: 180.0,
            z_degrees: 0.0,
        },
        3 => EulerRotation {
            x_degrees: 0.0,
            y_degrees: -90.0,
            z_degrees: 0.0,
        },
        4 => EulerRotation {
            x_degrees: 90.0,
            y_degrees: 0.0,
            z_degrees: 0.0,
        },
        5 => EulerRotation {
            x_degrees: -90.0,
            y_degrees: 0.0,
            z_degrees: 0.0,
        },
        _ => panic!("Minecraft panorama has exactly six faces"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command() -> PanoramaCommand {
        PanoramaCommand {
            textures: std::array::from_fn(|i| {
                ResourceLocation::parse(format!("textures/gui/title/background/panorama_{i}.png"))
            }),
            panorama_timer: 80.0,
            first_blur_passes: 64,
            second_blur_passes: 3,
            final_blur_pairs: 3,
            screen_width: 854,
            screen_height: 480,
        }
    }

    #[test]
    fn vanilla_plan_preserves_mcp_iteration_counts() {
        let plan = PanoramaPassPlan::from_command(&command());
        assert_eq!(plan.samples.len(), 64);
        assert_eq!(plan.samples[0].faces.len(), 6);
        assert_eq!(plan.blur_invocations.len(), 7); // initial + 3 pairs
        assert!(plan
            .blur_invocations
            .iter()
            .all(|pass| pass.layers.len() == 3));
    }

    #[test]
    fn first_sample_matches_java_integer_and_offset_semantics() {
        let plan = PanoramaPassPlan::from_command(&command());
        assert_eq!(plan.samples[0].translate_x, -0.5 / 64.0);
        assert_eq!(plan.samples[0].translate_y, -0.5 / 64.0);
        assert_eq!(plan.samples[0].faces[0].alpha_u8, 255);
        assert_eq!(plan.samples[1].faces[0].alpha_u8, 127);
        assert_eq!(plan.samples[63].faces[0].alpha_u8, 3);
    }

    #[test]
    fn face_rotations_match_draw_panorama_switches() {
        let plan = PanoramaPassPlan::from_command(&command());
        let faces = &plan.samples[0].faces;
        assert_eq!(faces[1].orientation.y_degrees, 90.0);
        assert_eq!(faces[2].orientation.y_degrees, 180.0);
        assert_eq!(faces[3].orientation.y_degrees, -90.0);
        assert_eq!(faces[4].orientation.x_degrees, 90.0);
        assert_eq!(faces[5].orientation.x_degrees, -90.0);
    }

    #[test]
    fn blur_offsets_and_alpha_match_rotate_and_blur_skybox() {
        let plan = PanoramaPassPlan::from_command(&command());
        let layers = &plan.blur_invocations[0].layers;
        assert_eq!(layers[0].horizontal_uv_offset, -1.0 / 256.0);
        assert_eq!(layers[1].horizontal_uv_offset, 0.0);
        assert_eq!(layers[2].horizontal_uv_offset, 1.0 / 256.0);
        assert_eq!(layers[0].alpha, 1.0);
        assert_eq!(layers[1].alpha, 0.5);
        assert!((layers[2].alpha - 1.0 / 3.0).abs() < f32::EPSILON);
    }
}
