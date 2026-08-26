use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuiltInItemVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuiltInItemMesh {
    pub vertices: Vec<BuiltInItemVertex>,
    pub indices: Vec<u32>,
    pub texture: ResourceLocation,
    /// Vertex color applied after the entity texture. Most TEISR textures are
    /// already final-color; banner base colors are composed in the atlas.
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct BoxSpec {
    pub(crate) texture: [i32; 2],
    pub(crate) texture_size: [f32; 2],
    pub(crate) origin: [f32; 3],
    pub(crate) size: [i32; 3],
    pub(crate) delta: f32,
    pub(crate) pivot: [f32; 3],
    pub(crate) rotation: [f32; 3],
    pub(crate) mirror: bool,
}

/// CPU equivalent of Minecraft 1.12.2 `TileEntityItemStackRenderer` for the
/// static built-in/entity item families. The returned model coordinates are
/// after the concrete TileEntitySpecialRenderer transforms and before
/// RenderItem's ItemCameraTransforms GUI/hand transform.
pub struct TileEntityItemStackRenderer;

impl TileEntityItemStackRenderer {
    pub fn buildMesh(stack: &ItemStack) -> Option<BuiltInItemMesh> {
        match stack.itemId {
            54 => Some(chest_mesh("textures/entity/chest/normal.png")),
            130 => Some(chest_mesh("textures/entity/chest/ender.png")),
            146 => Some(chest_mesh("textures/entity/chest/trapped.png")),
            219..=234 => Some(shulker_mesh((stack.itemId - 219) as usize)),
            355 => Some(bed_mesh(stack.itemDamage)),
            397 => Self::buildSkullMesh(stack.itemDamage as i32, 0.0),
            425 => Some(banner_mesh(stack.itemDamage.rem_euclid(16) as usize)),
            _ => None,
        }
    }

    /// Shared `TileEntitySkullRenderer` model mesh used by item and world
    /// rendering. The baseline transform is a floor skull at rotation 180°;
    /// the world renderer applies the facing/rotation delta around its block
    /// center exactly as the TESR translation switch does.
    pub fn buildSkullMesh(skullType: i32, animateTicks: f32) -> Option<BuiltInItemMesh> {
        skull_mesh(skullType, animateTicks)
    }

    pub fn staticTextures() -> Vec<ResourceLocation> {
        let mut textures = vec![
            texture("textures/entity/chest/normal.png"),
            texture("textures/entity/chest/normal_double.png"),
            texture("textures/entity/chest/trapped.png"),
            texture("textures/entity/chest/trapped_double.png"),
            texture("textures/entity/chest/ender.png"),
            texture("textures/entity/skeleton/skeleton.png"),
            texture("textures/entity/skeleton/wither_skeleton.png"),
            texture("textures/entity/zombie/zombie.png"),
            texture("textures/entity/creeper/creeper.png"),
            texture("textures/entity/enderdragon/dragon.png"),
            texture("textures/entity/steve.png"),
            texture("textures/entity/enchanting_table_book.png"),
            texture("textures/entity/banner_base.png"),
            texture("textures/entity/banner/base.png"),
        ];
        for name in DYE_NAMES {
            textures.push(texture(&format!(
                "textures/entity/shulker/shulker_{name}.png"
            )));
        }
        textures.extend(Self::bedTextures());
        textures
    }

    /// `TileEntityBedRenderer.field_193848_a[EnumDyeColor#getMetadata]`.
    /// Keeping the metadata-indexed array explicit prevents a built-in model
    /// lookup from accidentally collapsing all bed variants onto the mesh
    /// model's metadata-0 registration (`minecraft:bed#inventory`).
    pub fn bedTextures() -> [ResourceLocation; 16] {
        std::array::from_fn(|metadata| {
            texture(&format!("textures/entity/bed/{}.png", DYE_NAMES[metadata]))
        })
    }

    pub fn bedTexture(metadata: i16) -> ResourceLocation {
        texture(&format!(
            "textures/entity/bed/{}.png",
            DYE_NAMES[dye_metadata_index(metadata)],
        ))
    }

    /// Exact CPU mesh for one world half of `TileEntityBedRenderer`.
    /// `horizontal_index` is EnumFacing#getHorizontalIndex (N=2/S=0/W=1/E=3
    /// in the 1.12.2 mappings used by BlockBed metadata).
    pub fn buildWorldBedHalf(
        colorMetadata: i16,
        head: bool,
        horizontalIndex: i32,
    ) -> BuiltInItemMesh {
        let mut mesh = empty_mesh(Self::bedTexture(colorMetadata).getPath());
        let (angle, shift_x, shift_z) = match horizontalIndex {
            2 => (0.0, 0.0, 0.0),   // NORTH
            0 => (180.0, 1.0, 1.0), // SOUTH
            1 => (-90.0, 0.0, 1.0), // WEST
            3 => (90.0, 1.0, 0.0),  // EAST
            _ => (0.0, 0.0, 0.0),
        };
        let matrix = multiply(
            multiply(translation([shift_x, 0.5625, shift_z]), rotation_x(90.0)),
            rotation_z(angle),
        );
        if head {
            add_box(
                &mut mesh,
                BoxSpec {
                    texture: [0, 0],
                    texture_size: [64.0, 64.0],
                    origin: [0.0, 0.0, 0.0],
                    size: [16, 16, 6],
                    delta: 0.0,
                    pivot: [0.0; 3],
                    rotation: [0.0; 3],
                    mirror: false,
                },
                matrix,
            );
            add_bed_leg(&mut mesh, 1, matrix);
            add_bed_leg(&mut mesh, 3, matrix);
        } else {
            add_box(
                &mut mesh,
                BoxSpec {
                    texture: [0, 22],
                    texture_size: [64.0, 64.0],
                    origin: [0.0, 0.0, 0.0],
                    size: [16, 16, 6],
                    delta: 0.0,
                    pivot: [0.0; 3],
                    rotation: [0.0; 3],
                    mirror: false,
                },
                matrix,
            );
            add_bed_leg(&mut mesh, 0, matrix);
            add_bed_leg(&mut mesh, 2, matrix);
        }
        mesh
    }

    /// Exact `ModelChest`/`ModelLargeChest` geometry after the transforms in
    /// `TileEntityChestRenderer`. `lidProgress` is the already cubic-eased
    /// value used for chestLid.rotateAngleX.
    pub fn buildWorldChest(
        trapped: bool,
        ender: bool,
        large: bool,
        metadata: i32,
        adjacentXPos: bool,
        adjacentZPos: bool,
        lidProgress: f32,
    ) -> BuiltInItemMesh {
        let path = if ender {
            "textures/entity/chest/ender.png"
        } else if trapped && large {
            "textures/entity/chest/trapped_double.png"
        } else if trapped {
            "textures/entity/chest/trapped.png"
        } else if large {
            "textures/entity/chest/normal_double.png"
        } else {
            "textures/entity/chest/normal.png"
        };
        let mut matrix = multiply(
            multiply(translation([0.0, 1.0, 1.0]), scale([1.0, -1.0, -1.0])),
            translation([0.5, 0.5, 0.5]),
        );
        if metadata == 2 && adjacentXPos {
            matrix = multiply(matrix, translation([1.0, 0.0, 0.0]));
        }
        if metadata == 5 && adjacentZPos {
            matrix = multiply(matrix, translation([0.0, 0.0, -1.0]));
        }
        let yaw = match metadata {
            2 => 180.0,
            4 => 90.0,
            5 => -90.0,
            _ => 0.0,
        };
        matrix = multiply(matrix, rotation_y(yaw));
        matrix = multiply(matrix, translation([-0.5, -0.5, -0.5]));

        let tex_size = if large { [128.0, 64.0] } else { [64.0, 64.0] };
        let width = if large { 30 } else { 14 };
        let knob_x = if large { 16.0 } else { 8.0 };
        let lid_rotation = [
            -(lidProgress.clamp(0.0, 1.0) * std::f32::consts::FRAC_PI_2),
            0.0,
            0.0,
        ];
        let mut mesh = empty_mesh(path);
        add_box(
            &mut mesh,
            BoxSpec {
                texture: [0, 0],
                texture_size: tex_size,
                origin: [0.0, -5.0, -14.0],
                size: [width, 5, 14],
                delta: 0.0,
                pivot: [1.0, 7.0, 15.0],
                rotation: lid_rotation,
                mirror: false,
            },
            matrix,
        );
        add_box(
            &mut mesh,
            BoxSpec {
                texture: [0, 0],
                texture_size: tex_size,
                origin: [-1.0, -2.0, -15.0],
                size: [2, 4, 1],
                delta: 0.0,
                pivot: [knob_x, 7.0, 15.0],
                rotation: lid_rotation,
                mirror: false,
            },
            matrix,
        );
        add_box(
            &mut mesh,
            BoxSpec {
                texture: [0, 19],
                texture_size: tex_size,
                origin: [0.0, 0.0, 0.0],
                size: [width, 10, 14],
                delta: 0.0,
                pivot: [1.0, 6.0, 1.0],
                rotation: [0.0; 3],
                mirror: false,
            },
            matrix,
        );
        mesh
    }

    /// Exact world `TileEntityShulkerBoxRenderer` base/lid geometry and
    /// facing transform. `progress` is `TileEntityShulkerBox#func_190585_a`:
    /// the interpolated 0..1 open amount used for both the half-block lid
    /// translation and the 270-degree lid rotation.
    pub fn buildWorldShulker(
        colorMetadata: i32,
        facing: EnumFacing,
        progress: f32,
    ) -> BuiltInItemMesh {
        let name = DYE_NAMES[colorMetadata.clamp(0, 15) as usize];
        let mut matrix = multiply(
            multiply(
                multiply(
                    multiply(translation([0.5, 1.5, 0.5]), scale([1.0, -1.0, -1.0])),
                    translation([0.0, 1.0, 0.0]),
                ),
                scale([0.9995, 0.9995, 0.9995]),
            ),
            translation([0.0, -1.0, 0.0]),
        );
        matrix = match facing {
            EnumFacing::Down => multiply(
                multiply(matrix, translation([0.0, 2.0, 0.0])),
                rotation_x(180.0),
            ),
            EnumFacing::Up => matrix,
            EnumFacing::North => multiply(
                multiply(
                    multiply(matrix, translation([0.0, 1.0, 1.0])),
                    rotation_x(90.0),
                ),
                rotation_z(180.0),
            ),
            EnumFacing::South => multiply(
                multiply(matrix, translation([0.0, 1.0, -1.0])),
                rotation_x(90.0),
            ),
            EnumFacing::West => multiply(
                multiply(
                    multiply(matrix, translation([-1.0, 1.0, 0.0])),
                    rotation_x(90.0),
                ),
                rotation_z(-90.0),
            ),
            EnumFacing::East => multiply(
                multiply(
                    multiply(matrix, translation([1.0, 1.0, 0.0])),
                    rotation_x(90.0),
                ),
                rotation_z(90.0),
            ),
        };

        let mut mesh = empty_mesh(&format!("textures/entity/shulker/shulker_{name}.png"));
        add_box(
            &mut mesh,
            BoxSpec {
                texture: [0, 28],
                texture_size: [64.0, 64.0],
                origin: [-8.0, -8.0, -8.0],
                size: [16, 8, 16],
                delta: 0.0,
                pivot: [0.0, 24.0, 0.0],
                rotation: [0.0; 3],
                mirror: false,
            },
            matrix,
        );

        let open = progress.clamp(0.0, 1.0);
        let lid_matrix = multiply(
            multiply(matrix, translation([0.0, -open * 0.5, 0.0])),
            rotation_y(270.0 * open),
        );
        add_box(
            &mut mesh,
            BoxSpec {
                texture: [0, 0],
                texture_size: [64.0, 64.0],
                origin: [-8.0, -16.0, -8.0],
                size: [16, 12, 16],
                delta: 0.0,
                pivot: [0.0, 24.0, 0.0],
                rotation: [0.0; 3],
                mirror: false,
            },
            lid_matrix,
        );
        mesh
    }
}

const DYE_NAMES: [&str; 16] = [
    "white",
    "orange",
    "magenta",
    "light_blue",
    "yellow",
    "lime",
    "pink",
    "gray",
    "silver",
    "cyan",
    "purple",
    "blue",
    "brown",
    "green",
    "red",
    "black",
];

fn dye_metadata_index(metadata: i16) -> usize {
    if (0..16).contains(&metadata) {
        metadata as usize
    } else {
        0
    }
}

fn texture(path: &str) -> ResourceLocation {
    ResourceLocation::new("minecraft", path)
}

fn chest_mesh(path: &str) -> BuiltInItemMesh {
    let matrix = multiply(
        multiply(
            multiply(translation([0.0, 1.0, 1.0]), scale([1.0, -1.0, -1.0])),
            translation([0.5, 0.5, 0.5]),
        ),
        translation([-0.5, -0.5, -0.5]),
    );
    let mut mesh = empty_mesh(path);
    add_box(
        &mut mesh,
        BoxSpec {
            texture: [0, 0],
            texture_size: [64.0, 64.0],
            origin: [0.0, -5.0, -14.0],
            size: [14, 5, 14],
            delta: 0.0,
            pivot: [1.0, 7.0, 15.0],
            rotation: [0.0; 3],
            mirror: false,
        },
        matrix,
    );
    add_box(
        &mut mesh,
        BoxSpec {
            texture: [0, 0],
            texture_size: [64.0, 64.0],
            origin: [-1.0, -2.0, -15.0],
            size: [2, 4, 1],
            delta: 0.0,
            pivot: [8.0, 7.0, 15.0],
            rotation: [0.0; 3],
            mirror: false,
        },
        matrix,
    );
    add_box(
        &mut mesh,
        BoxSpec {
            texture: [0, 19],
            texture_size: [64.0, 64.0],
            origin: [0.0, 0.0, 0.0],
            size: [14, 10, 14],
            delta: 0.0,
            pivot: [1.0, 6.0, 1.0],
            rotation: [0.0; 3],
            mirror: false,
        },
        matrix,
    );
    mesh
}

fn shulker_mesh(color: usize) -> BuiltInItemMesh {
    let name = DYE_NAMES[color.min(15)];
    let matrix = multiply(
        multiply(
            multiply(
                multiply(translation([0.5, 1.5, 0.5]), scale([1.0, -1.0, -1.0])),
                translation([0.0, 1.0, 0.0]),
            ),
            scale([0.9995, 0.9995, 0.9995]),
        ),
        translation([0.0, -1.0, 0.0]),
    );
    let mut mesh = empty_mesh(&format!("textures/entity/shulker/shulker_{name}.png"));
    add_box(
        &mut mesh,
        BoxSpec {
            texture: [0, 28],
            texture_size: [64.0, 64.0],
            origin: [-8.0, -8.0, -8.0],
            size: [16, 8, 16],
            delta: 0.0,
            pivot: [0.0, 24.0, 0.0],
            rotation: [0.0; 3],
            mirror: false,
        },
        matrix,
    );
    add_box(
        &mut mesh,
        BoxSpec {
            texture: [0, 0],
            texture_size: [64.0, 64.0],
            origin: [-8.0, -16.0, -8.0],
            size: [16, 12, 16],
            delta: 0.0,
            pivot: [0.0, 24.0, 0.0],
            rotation: [0.0; 3],
            mirror: false,
        },
        matrix,
    );
    mesh
}

fn bed_mesh(metadata: i16) -> BuiltInItemMesh {
    let mut mesh = empty_mesh(TileEntityItemStackRenderer::bedTexture(metadata).getPath());
    // TileEntityBedRenderer passes block metadata 0 when the TileEntity has no
    // world. In EnumFacing 1.12.2 horizontal-index order, 0 is SOUTH (not
    // NORTH): translate by (+1, +0.5625, +1), rotate X by 90 degrees and then
    // rotate Z by 180 degrees. The foot renderer receives z - 1 before the
    // SOUTH offset, exactly matching renderByItem's two func_193847_a calls.
    let head_matrix = bed_item_half_matrix(0.0);
    let foot_matrix = bed_item_half_matrix(-1.0);
    add_box(
        &mut mesh,
        BoxSpec {
            texture: [0, 0],
            texture_size: [64.0, 64.0],
            origin: [0.0, 0.0, 0.0],
            size: [16, 16, 6],
            delta: 0.0,
            pivot: [0.0; 3],
            rotation: [0.0; 3],
            mirror: false,
        },
        head_matrix,
    );
    // head half shows legs 1 and 3
    add_bed_leg(&mut mesh, 1, head_matrix);
    add_bed_leg(&mut mesh, 3, head_matrix);
    add_box(
        &mut mesh,
        BoxSpec {
            texture: [0, 22],
            texture_size: [64.0, 64.0],
            origin: [0.0, 0.0, 0.0],
            size: [16, 16, 6],
            delta: 0.0,
            pivot: [0.0; 3],
            rotation: [0.0; 3],
            mirror: false,
        },
        foot_matrix,
    );
    // foot half shows legs 0 and 2
    add_bed_leg(&mut mesh, 0, foot_matrix);
    add_bed_leg(&mut mesh, 2, foot_matrix);
    mesh
}

fn bed_item_half_matrix(renderer_z: f32) -> [[f32; 4]; 4] {
    multiply(
        multiply(
            translation([1.0, 0.5625, renderer_z + 1.0]),
            rotation_x(90.0),
        ),
        rotation_z(180.0),
    )
}

fn add_bed_leg(mesh: &mut BuiltInItemMesh, index: usize, matrix: [[f32; 4]; 4]) {
    let origins = [
        [0.0, 6.0, -16.0],
        [0.0, 6.0, 0.0],
        [-16.0, 6.0, -16.0],
        [-16.0, 6.0, 0.0],
    ];
    let textures = [[50, 0], [50, 6], [50, 12], [50, 18]];
    let rotations_z = [
        0.0,
        std::f32::consts::FRAC_PI_2,
        std::f32::consts::PI * 1.5,
        std::f32::consts::PI,
    ];
    add_box(
        mesh,
        BoxSpec {
            texture: textures[index],
            texture_size: [64.0, 64.0],
            origin: origins[index],
            size: [3, 3, 3],
            delta: 0.0,
            pivot: [0.0; 3],
            rotation: [std::f32::consts::FRAC_PI_2, 0.0, rotations_z[index]],
            mirror: false,
        },
        matrix,
    );
}

fn banner_mesh(dye_damage: usize) -> BuiltInItemMesh {
    // The atlas owns the exact LayeredColorMaskTexture base composition. Use a
    // synthetic resource path to select the corresponding precomposed sprite.
    let mut mesh = empty_mesh(&format!("textures/generated/banner_base_{dye_damage}.png"));
    let matrix = multiply(
        translation([0.5, 0.5, 0.5]),
        scale([0.6666667, -0.6666667, -0.6666667]),
    );
    let slate_angle = (-0.0125_f32 + 0.01_f32) * std::f32::consts::PI;
    add_box(
        &mut mesh,
        BoxSpec {
            texture: [0, 0],
            texture_size: [64.0, 64.0],
            origin: [-10.0, 0.0, -2.0],
            size: [20, 40, 1],
            delta: 0.0,
            pivot: [0.0, -32.0, 0.0],
            rotation: [slate_angle, 0.0, 0.0],
            mirror: false,
        },
        matrix,
    );
    add_box(
        &mut mesh,
        BoxSpec {
            texture: [44, 0],
            texture_size: [64.0, 64.0],
            origin: [-1.0, -30.0, -1.0],
            size: [2, 42, 2],
            delta: 0.0,
            pivot: [0.0; 3],
            rotation: [0.0; 3],
            mirror: false,
        },
        matrix,
    );
    add_box(
        &mut mesh,
        BoxSpec {
            texture: [0, 42],
            texture_size: [64.0, 64.0],
            origin: [-10.0, -32.0, -1.0],
            size: [20, 2, 2],
            delta: 0.0,
            pivot: [0.0; 3],
            rotation: [0.0; 3],
            mirror: false,
        },
        matrix,
    );
    mesh
}

fn skull_mesh(skull_type: i32, animate_ticks: f32) -> Option<BuiltInItemMesh> {
    if skull_type == 5 {
        return Some(dragon_head_mesh(animate_ticks));
    }
    let (path, humanoid) = match skull_type {
        1 => ("textures/entity/skeleton/wither_skeleton.png", false),
        2 => ("textures/entity/zombie/zombie.png", true),
        3 => ("textures/entity/steve.png", true),
        4 => ("textures/entity/creeper/creeper.png", false),
        _ => ("textures/entity/skeleton/skeleton.png", false),
    };
    let matrix = multiply(translation([0.5, 0.0, 0.5]), scale([-1.0, -1.0, 1.0]));
    let mut mesh = empty_mesh(path);
    add_box(
        &mut mesh,
        BoxSpec {
            // TileEntitySkullRenderer uses ModelSkeletonHead(0, 0, 64, 32)
            // for skeleton, wither skeleton and creeper, and ModelHumanoidHead's
            // 64x64 base/hat pair for zombie and player heads.
            texture: [0, 0],
            texture_size: if humanoid { [64.0, 64.0] } else { [64.0, 32.0] },
            origin: [-4.0, -8.0, -4.0],
            size: [8, 8, 8],
            delta: 0.0,
            pivot: [0.0; 3],
            rotation: [0.0, std::f32::consts::PI, 0.0],
            mirror: false,
        },
        matrix,
    );
    if humanoid {
        add_box(
            &mut mesh,
            BoxSpec {
                texture: [32, 0],
                texture_size: [64.0, 64.0],
                origin: [-4.0, -8.0, -4.0],
                size: [8, 8, 8],
                delta: 0.25,
                pivot: [0.0; 3],
                rotation: [0.0, std::f32::consts::PI, 0.0],
                mirror: false,
            },
            matrix,
        );
    }
    Some(mesh)
}

fn dragon_head_mesh(animate_ticks: f32) -> BuiltInItemMesh {
    // TileEntityItemStackRenderer calls renderSkull(UP, 180, type=5,
    // animateTicks=-1). ModelDragonHead then translates/scales the complete
    // head and renders the jaw as a child ModelRenderer.
    let skull_matrix = multiply(translation([0.5, 0.0, 0.5]), scale([-1.0, -1.0, 1.0]));
    let matrix = multiply(
        multiply(
            multiply(skull_matrix, translation([0.0, -0.374375, 0.0])),
            scale([0.75, 0.75, 0.75]),
        ),
        rotation_y_radians(std::f32::consts::PI),
    );
    let mut mesh = empty_mesh("textures/entity/enderdragon/dragon.png");
    let texture_size = [256.0, 256.0];
    add_box(
        &mut mesh,
        BoxSpec {
            texture: [176, 44],
            texture_size,
            origin: [-6.0, -1.0, -24.0],
            size: [12, 5, 16],
            delta: 0.0,
            pivot: [0.0; 3],
            rotation: [0.0; 3],
            mirror: false,
        },
        matrix,
    );
    add_box(
        &mut mesh,
        BoxSpec {
            texture: [112, 30],
            texture_size,
            origin: [-8.0, -8.0, -10.0],
            size: [16, 16, 16],
            delta: 0.0,
            pivot: [0.0; 3],
            rotation: [0.0; 3],
            mirror: false,
        },
        matrix,
    );
    add_box(
        &mut mesh,
        BoxSpec {
            texture: [0, 0],
            texture_size,
            origin: [-5.0, -12.0, -4.0],
            size: [2, 4, 6],
            delta: 0.0,
            pivot: [0.0; 3],
            rotation: [0.0; 3],
            mirror: true,
        },
        matrix,
    );
    add_box(
        &mut mesh,
        BoxSpec {
            texture: [112, 0],
            texture_size,
            origin: [-5.0, -3.0, -22.0],
            size: [2, 2, 4],
            delta: 0.0,
            pivot: [0.0; 3],
            rotation: [0.0; 3],
            mirror: true,
        },
        matrix,
    );
    add_box(
        &mut mesh,
        BoxSpec {
            texture: [0, 0],
            texture_size,
            origin: [3.0, -12.0, -4.0],
            size: [2, 4, 6],
            delta: 0.0,
            pivot: [0.0; 3],
            rotation: [0.0; 3],
            mirror: false,
        },
        matrix,
    );
    add_box(
        &mut mesh,
        BoxSpec {
            texture: [112, 0],
            texture_size,
            origin: [3.0, -3.0, -22.0],
            size: [2, 2, 4],
            delta: 0.0,
            pivot: [0.0; 3],
            rotation: [0.0; 3],
            mirror: false,
        },
        matrix,
    );
    let jaw_angle = ((animate_ticks * 0.2).sin() + 1.0) * 0.2;
    add_box(
        &mut mesh,
        BoxSpec {
            texture: [176, 65],
            texture_size,
            origin: [-6.0, 0.0, -16.0],
            size: [12, 4, 16],
            delta: 0.0,
            pivot: [0.0, 4.0, -8.0],
            rotation: [jaw_angle, 0.0, 0.0],
            mirror: false,
        },
        matrix,
    );
    mesh
}

pub(crate) fn empty_mesh(path: &str) -> BuiltInItemMesh {
    BuiltInItemMesh {
        vertices: Vec::new(),
        indices: Vec::new(),
        texture: texture(path),
        color: [1.0; 4],
    }
}

pub(crate) fn add_box(mesh: &mut BuiltInItemMesh, spec: BoxSpec, matrix: [[f32; 4]; 4]) {
    let [dx, dy, dz] = spec.size;
    let [x, y, z] = spec.origin;
    let mut x1 = x - spec.delta;
    let y1 = y - spec.delta;
    let z1 = z - spec.delta;
    let mut x2 = x + dx as f32 + spec.delta;
    let y2 = y + dy as f32 + spec.delta;
    let z2 = z + dz as f32 + spec.delta;
    if spec.mirror {
        std::mem::swap(&mut x1, &mut x2);
    }
    let points = [
        [x1, y1, z1],
        [x2, y1, z1],
        [x2, y2, z1],
        [x1, y2, z1],
        [x1, y1, z2],
        [x2, y1, z2],
        [x2, y2, z2],
        [x1, y2, z2],
    ];
    let [u, v] = spec.texture;
    let faces = [
        (
            [5usize, 1, 2, 6],
            [u + dz + dx, v + dz, u + dz + dx + dz, v + dz + dy],
        ),
        ([0usize, 4, 7, 3], [u, v + dz, u + dz, v + dz + dy]),
        ([5usize, 4, 0, 1], [u + dz, v, u + dz + dx, v + dz]),
        (
            [2usize, 3, 7, 6],
            [u + dz + dx, v + dz, u + dz + dx + dx, v],
        ),
        (
            [1usize, 0, 3, 2],
            [u + dz, v + dz, u + dz + dx, v + dz + dy],
        ),
        (
            [4usize, 5, 6, 7],
            [u + dz + dx + dz, v + dz, u + dz + dx + dz + dx, v + dz + dy],
        ),
    ];
    for (order, uv_rect) in faces {
        let base = mesh.vertices.len() as u32;
        let u1 = uv_rect[0] as f32 / spec.texture_size[0];
        let v1 = uv_rect[1] as f32 / spec.texture_size[1];
        let u2 = uv_rect[2] as f32 / spec.texture_size[0];
        let v2 = uv_rect[3] as f32 / spec.texture_size[1];
        let uvs = [[u2, v1], [u1, v1], [u1, v2], [u2, v2]];
        let mut textured = [
            (order[0], uvs[0]),
            (order[1], uvs[1]),
            (order[2], uvs[2]),
            (order[3], uvs[3]),
        ];
        if spec.mirror {
            textured.reverse();
        }
        for (point_index, uv) in textured {
            let local = model_point(points[point_index], spec.pivot, spec.rotation);
            mesh.vertices.push(BuiltInItemVertex {
                position: transform_point(matrix, local),
                uv,
            });
        }
        mesh.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

fn model_point(mut point: [f32; 3], pivot: [f32; 3], rotation: [f32; 3]) -> [f32; 3] {
    point = rotate_x_point(point, rotation[0]);
    point = rotate_y_point(point, rotation[1]);
    point = rotate_z_point(point, rotation[2]);
    [
        (point[0] + pivot[0]) / 16.0,
        (point[1] + pivot[1]) / 16.0,
        (point[2] + pivot[2]) / 16.0,
    ]
}

pub(crate) fn identity() -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}
fn translation(v: [f32; 3]) -> [[f32; 4]; 4] {
    let mut m = identity();
    m[0][3] = v[0];
    m[1][3] = v[1];
    m[2][3] = v[2];
    m
}
fn scale(v: [f32; 3]) -> [[f32; 4]; 4] {
    [
        [v[0], 0.0, 0.0, 0.0],
        [0.0, v[1], 0.0, 0.0],
        [0.0, 0.0, v[2], 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}
fn rotation_x(degrees: f32) -> [[f32; 4]; 4] {
    let radians = degrees.to_radians();
    let (c, s) = (radians.cos(), radians.sin());
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, c, -s, 0.0],
        [0.0, s, c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}
fn rotation_z(degrees: f32) -> [[f32; 4]; 4] {
    let radians = degrees.to_radians();
    let (c, s) = (radians.cos(), radians.sin());
    [
        [c, -s, 0.0, 0.0],
        [s, c, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}
fn rotation_y(degrees: f32) -> [[f32; 4]; 4] {
    rotation_y_radians(degrees.to_radians())
}
fn rotation_y_radians(radians: f32) -> [[f32; 4]; 4] {
    let (c, s) = (radians.cos(), radians.sin());
    [
        [c, 0.0, s, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [-s, 0.0, c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}
fn multiply(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut r = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                r[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    r
}
fn transform_point(m: [[f32; 4]; 4], p: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * p[0] + m[0][1] * p[1] + m[0][2] * p[2] + m[0][3],
        m[1][0] * p[0] + m[1][1] * p[1] + m[1][2] * p[2] + m[1][3],
        m[2][0] * p[0] + m[2][1] * p[1] + m[2][2] * p[2] + m[2][3],
    ]
}
fn rotate_x_point(p: [f32; 3], a: f32) -> [f32; 3] {
    let (c, s) = (a.cos(), a.sin());
    [p[0], p[1] * c - p[2] * s, p[1] * s + p[2] * c]
}
fn rotate_y_point(p: [f32; 3], a: f32) -> [f32; 3] {
    let (c, s) = (a.cos(), a.sin());
    [p[0] * c + p[2] * s, p[1], -p[0] * s + p[2] * c]
}
fn rotate_z_point(p: [f32; 3], a: f32) -> [f32; 3] {
    let (c, s) = (a.cos(), a.sin());
    [p[0] * c - p[1] * s, p[0] * s + p[1] * c, p[2]]
}

#[cfg(test)]
mod tests {
    use super::*;
    fn stack(id: i16, damage: i16) -> ItemStack {
        ItemStack {
            itemId: id,
            count: 1,
            itemDamage: damage,
            tagCompound: None,
        }
    }
    #[test]
    fn all_static_teisr_families_produce_real_faces() {
        for sample in [
            stack(54, 0),
            stack(130, 0),
            stack(146, 0),
            stack(219, 0),
            stack(234, 0),
            stack(355, 14),
            stack(397, 0),
            stack(397, 3),
            stack(425, 15),
        ] {
            let mesh =
                TileEntityItemStackRenderer::buildMesh(&sample).expect("supported TEISR item");
            assert!(!mesh.vertices.is_empty());
            assert_eq!(mesh.indices.len() % 6, 0);
        }
    }
    #[test]
    fn bed_metadata_selects_the_exact_tile_entity_texture() {
        let textures = TileEntityItemStackRenderer::bedTextures();
        assert_eq!(textures.len(), 16);
        assert_eq!(textures[0].getPath(), "textures/entity/bed/white.png");
        assert_eq!(textures[14].getPath(), "textures/entity/bed/red.png");
        assert_eq!(textures[15].getPath(), "textures/entity/bed/black.png");
        assert_eq!(
            textures
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            16
        );
        let black = TileEntityItemStackRenderer::buildMesh(&stack(355, 15)).expect("black bed");
        assert_eq!(black.texture, textures[15]);
    }

    #[test]
    fn bed_item_uses_enumfacing_south_horizontal_index_zero_transform() {
        let head = bed_item_half_matrix(0.0);
        let foot = bed_item_half_matrix(-1.0);
        // Origin is transformed by the SOUTH offset; NORTH would incorrectly
        // leave x/z at zero and omit the 180-degree Z rotation.
        let head_origin = transform_point(head, [0.0, 0.0, 0.0]);
        let foot_origin = transform_point(foot, [0.0, 0.0, 0.0]);
        assert!((head_origin[0] - 1.0).abs() < 1.0e-6);
        assert!((head_origin[1] - 0.5625).abs() < 1.0e-6);
        assert!((head_origin[2] - 1.0).abs() < 1.0e-6);
        assert!((foot_origin[0] - 1.0).abs() < 1.0e-6);
        assert!((foot_origin[1] - 0.5625).abs() < 1.0e-6);
        assert!(foot_origin[2].abs() < 1.0e-6);
        let x_axis = transform_point(head, [1.0, 0.0, 0.0]);
        assert!((x_axis[0] - 0.0).abs() < 1.0e-6);
    }

    #[test]
    fn invalid_bed_metadata_and_skull_type_follow_vanilla_defaults() {
        let bed = TileEntityItemStackRenderer::buildMesh(&stack(355, 17)).expect("bed");
        assert_eq!(bed.texture.getPath(), "textures/entity/bed/white.png");
        let skull = TileEntityItemStackRenderer::buildMesh(&stack(397, 7)).expect("skull");
        assert_eq!(
            skull.texture.getPath(),
            "textures/entity/skeleton/skeleton.png"
        );
    }

    #[test]
    fn dragon_head_uses_the_named_model_boxes_and_child_jaw() {
        let mesh = TileEntityItemStackRenderer::buildMesh(&stack(397, 5)).expect("dragon head");
        assert_eq!(
            mesh.texture.getPath(),
            "textures/entity/enderdragon/dragon.png"
        );
        assert!(mesh.vertices.len() > 6 * 4);
    }

    #[test]
    fn world_shulker_uses_color_facing_and_animated_lid_geometry() {
        let closed = TileEntityItemStackRenderer::buildWorldShulker(14, EnumFacing::North, 0.0);
        let open = TileEntityItemStackRenderer::buildWorldShulker(14, EnumFacing::North, 1.0);
        assert_eq!(
            closed.texture.getPath(),
            "textures/entity/shulker/shulker_red.png"
        );
        assert_eq!(closed.vertices.len(), 12 * 4);
        assert_eq!(closed.indices.len(), 12 * 6);
        assert_ne!(closed.vertices, open.vertices);

        let up = TileEntityItemStackRenderer::buildWorldShulker(0, EnumFacing::Up, 0.0);
        let down = TileEntityItemStackRenderer::buildWorldShulker(0, EnumFacing::Down, 0.0);
        assert_ne!(up.vertices, down.vertices);
    }
}
