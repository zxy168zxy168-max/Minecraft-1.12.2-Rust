use std::sync::Arc;
use crate::net::minecraft::nbt::NBTBase::{NBTBase, TAG_COMPOUND, TAG_LIST};
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::DataFixer::DataFixer;
use crate::net::minecraft::util::datafix::FixTypes::FixTypes;
use crate::net::minecraft::util::datafix::IDataFixer::IDataFixer;
use crate::net::minecraft::util::datafix::IDataWalker::IDataWalker;
use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::math::MathHelper::{cos as minecraft_cos, sin as minecraft_sin};

/// First gameplay-bearing subset of MCP `net.minecraft.entity.Entity`.
/// Field names and the collision/step algorithm follow 1.12.2 so additional
/// entity behavior can be layered onto the same state rather than replaced.
#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    pub ticksExisted: i32,
    pub posX: f64,
    pub posY: f64,
    pub posZ: f64,
    pub prevPosX: f64,
    pub prevPosY: f64,
    pub prevPosZ: f64,
    pub motionX: f64,
    pub motionY: f64,
    pub motionZ: f64,
    pub rotationYaw: f32,
    pub rotationPitch: f32,
    pub prevRotationYaw: f32,
    pub prevRotationPitch: f32,
    pub width: f32,
    pub height: f32,
    pub stepHeight: f32,
    pub onGround: bool,
    pub isCollidedHorizontally: bool,
    pub isCollidedVertically: bool,
    pub isCollided: bool,
    pub noClip: bool,
    pub sneaking: bool,
    /// MCP liquid-contact state, refreshed by `handleWaterMovement`.
    pub inWater: bool,
    pub fallDistance: f32,
    pub firstUpdate: bool,
    /// MCP `Entity#isInWeb`; consumed at the beginning of the next move.
    pub isInWeb: bool,
    /// MCP `field_190534_ay`: remaining fire ticks on the logical entity.
    pub fire: i32,
    pub isDead: bool,
    /// Rust ID-based equivalent of MCP's object references. The world owns the
    /// heterogeneous entity map and maintains both sides of this relation.
    pub ridingEntityId: Option<i32>,
    pub passengerIds: Vec<i32>,
    /// MCP `field_191505_aI`: per-axis displacement already applied by
    /// `MoverType.PISTON` during the current world tick.
    pistonDeltas: [f64; 3],
    /// MCP `field_191506_aJ`: world tick owning `pistonDeltas`.
    pistonDeltaGameTime: i64,
    pub boundingBox: AxisAlignedBB,
}

impl Default for Entity {
    fn default() -> Self {
        let mut entity = Self {
            ticksExisted: 0,
            posX: 0.0,
            posY: 0.0,
            posZ: 0.0,
            prevPosX: 0.0,
            prevPosY: 0.0,
            prevPosZ: 0.0,
            motionX: 0.0,
            motionY: 0.0,
            motionZ: 0.0,
            rotationYaw: 0.0,
            rotationPitch: 0.0,
            prevRotationYaw: 0.0,
            prevRotationPitch: 0.0,
            width: 0.6,
            height: 1.8,
            stepHeight: 0.6,
            onGround: false,
            isCollidedHorizontally: false,
            isCollidedVertically: false,
            isCollided: false,
            noClip: false,
            sneaking: false,
            inWater: false,
            fallDistance: 0.0,
            firstUpdate: true,
            isInWeb: false,
            fire: -1,
            isDead: false,
            ridingEntityId: None,
            passengerIds: Vec::new(),
            pistonDeltas: [0.0; 3],
            pistonDeltaGameTime: i64::MIN,
            boundingBox: AxisAlignedBB::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        };
        entity.setPosition(0.0, 0.0, 0.0);
        entity
    }
}

struct EntityPassengersDataWalker;
impl IDataWalker for EntityPassengersDataWalker {
    fn process(&self, fixer: &dyn IDataFixer, mut compound: NBTTagCompound, versionIn: i32) -> NBTTagCompound {
        if compound.hasKeyWithType("Passengers", TAG_LIST) {
            let mut passengers = compound.getTagList("Passengers", TAG_COMPOUND);
            for index in 0..passengers.tagCount() {
                let fixed = fixer.processVersioned(FixTypes::Entity, passengers.getCompoundTagAt(index), versionIn);
                passengers.set(index, NBTBase::Compound(fixed));
            }
            compound.setTagList("Passengers", passengers);
        }
        compound
    }
}

impl Entity {
    /// MCP `Entity#func_190533_a`: passenger entities are recursively passed
    /// through the ENTITY fixer chain before their parent is constructed.
    pub fn registerFixes(fixer: &mut DataFixer) {
        fixer.registerWalker(FixTypes::Entity, Arc::new(EntityPassengersDataWalker));
    }
    /// Client-world port of MCP `Entity#setSize`. Shrinking recentres the
    /// box on `posX/posZ`; growing preserves the current minimum corner. The
    /// server-only growth nudge is intentionally absent because every entity
    /// represented here belongs to a remote `WorldClient`.
    pub fn setSize(&mut self, width: f32, height: f32) {
        if self.width == width && self.height == height {
            return;
        }
        let previousWidth = self.width;
        self.width = width;
        self.height = height;
        if width < previousWidth {
            let halfWidth = width as f64 / 2.0;
            self.boundingBox = AxisAlignedBB::new(
                self.posX - halfWidth,
                self.posY,
                self.posZ - halfWidth,
                self.posX + halfWidth,
                self.posY + height as f64,
                self.posZ + halfWidth,
            );
        } else {
            let current = self.boundingBox;
            self.boundingBox = AxisAlignedBB::new(
                current.min_x,
                current.min_y,
                current.min_z,
                current.min_x + width as f64,
                current.min_y + height as f64,
                current.min_z + width as f64,
            );
        }
    }

    pub fn setPosition(&mut self, x: f64, y: f64, z: f64) {
        self.posX = x;
        self.posY = y;
        self.posZ = z;
        let half_width = self.width as f64 / 2.0;
        self.boundingBox = AxisAlignedBB::new(
            x - half_width,
            y,
            z - half_width,
            x + half_width,
            y + self.height as f64,
            z + half_width,
        );
    }

    pub fn setPositionAndRotation(&mut self, x: f64, y: f64, z: f64, yaw: f32, pitch: f32) {
        // MCP `Entity.setPositionAndRotation`: position is clamped to the
        // vanilla world limit and both current/previous transforms are reset
        // before rebuilding the bounding box.
        self.posX = x.clamp(-30_000_000.0, 30_000_000.0);
        self.posY = y;
        self.posZ = z.clamp(-30_000_000.0, 30_000_000.0);
        self.prevPosX = self.posX;
        self.prevPosY = self.posY;
        self.prevPosZ = self.posZ;
        self.rotationYaw = yaw;
        self.rotationPitch = pitch.clamp(-90.0, 90.0);
        self.prevRotationYaw = self.rotationYaw;
        self.prevRotationPitch = self.rotationPitch;
        self.setPosition(self.posX, self.posY, self.posZ);
    }

    /// Port of `Entity.turn`. The mouse sensitivity cube is applied by the
    /// desktop input adapter before this method, as in `Minecraft.runTickMouse`.
    pub fn turn(&mut self, yaw: f32, pitch: f32) {
        let old_pitch = self.rotationPitch;
        let old_yaw = self.rotationYaw;
        self.rotationYaw = (self.rotationYaw as f64 + yaw as f64 * 0.15) as f32;
        self.rotationPitch = (self.rotationPitch as f64 - pitch as f64 * 0.15)
            .clamp(-90.0, 90.0) as f32;
        self.prevRotationPitch = (self.prevRotationPitch as f64
            + (self.rotationPitch - old_pitch) as f64) as f32;
        self.prevRotationYaw = (self.prevRotationYaw as f64
            + (self.rotationYaw - old_yaw) as f64) as f32;
    }

    /// Port of MCP `Entity.func_191958_b` (`moveRelative`).
    pub fn func_191958_b(&mut self, mut strafe: f32, mut vertical: f32, mut forward: f32, friction: f32) {
        let mut length = strafe * strafe + vertical * vertical + forward * forward;
        if length < 1.0e-4 {
            return;
        }
        length = length.sqrt();
        if length < 1.0 {
            length = 1.0;
        }
        let scale = friction / length;
        strafe *= scale;
        vertical *= scale;
        forward *= scale;
        let yaw = self.rotationYaw * 0.017453292;
        let sin_yaw = minecraft_sin(yaw);
        let cos_yaw = minecraft_cos(yaw);
        self.motionX += (strafe * cos_yaw - forward * sin_yaw) as f64;
        self.motionY += vertical as f64;
        self.motionZ += (forward * cos_yaw + strafe * sin_yaw) as f64;
    }

    /// Collision-axis ordering and step-height selection are ported from
    /// `Entity.moveEntity(MoverType.SELF, ...)` in MCP 1.12.2.
    pub fn moveEntity(&mut self, world: &WorldClient, x: f64, y: f64, z: f64) {
        self.moveEntityTyped(world, x, y, z, false, false, None);
    }

    /// Entity-aware form of MCP `Entity#moveEntity`. The caller supplies its
    /// world entity ID so `World#getCollisionBoxes(Entity, AABB)` can include
    /// boats and shulkers while excluding the moving entity and its riding
    /// chain. `collidesWithPushableEntities` is true for boats/minecarts,
    /// whose source `getCollisionBox` returns the other pushable entity box.
    pub fn moveEntityWithContext(
        &mut self,
        world: &WorldClient,
        entityId: i32,
        collidesWithPushableEntities: bool,
        x: f64,
        y: f64,
        z: f64,
    ) {
        self.moveEntityTyped(
            world,
            x,
            y,
            z,
            false,
            false,
            Some((entityId, collidesWithPushableEntities)),
        );
    }

    /// Rust equivalent of the source `instanceof EntityLivingBase` branch used
    /// by blocks such as slime while preserving the common Entity move owner.
    pub fn moveEntityLiving(&mut self, world: &WorldClient, x: f64, y: f64, z: f64) {
        self.moveEntityTyped(world, x, y, z, true, false, None);
    }

    pub fn moveEntityLivingWithContext(
        &mut self,
        world: &WorldClient,
        entityId: i32,
        x: f64,
        y: f64,
        z: f64,
    ) {
        self.moveEntityTyped(world, x, y, z, true, false, Some((entityId, false)));
    }

    /// Exact `MoverType.PISTON` entry point. Vanilla accumulates piston
    /// displacement per axis and limits each axis to 0.51 blocks per world
    /// tick, preventing multiple neighbouring moving blocks from over-pushing
    /// one entity in the same tick.
    pub fn moveEntityPiston(&mut self, world: &WorldClient, mut x: f64, mut y: f64, mut z: f64) {
        let game_time = world.getTotalWorldTime();
        if game_time != self.pistonDeltaGameTime {
            self.pistonDeltas = [0.0; 3];
            self.pistonDeltaGameTime = game_time;
        }

        let (axis, requested) = if x != 0.0 {
            (0usize, x)
        } else if y != 0.0 {
            (1usize, y)
        } else if z != 0.0 {
            (2usize, z)
        } else {
            return;
        };
        let clamped = (requested + self.pistonDeltas[axis]).clamp(-0.51, 0.51);
        let delta = clamped - self.pistonDeltas[axis];
        self.pistonDeltas[axis] = clamped;
        if delta.abs() <= 9.999_999_747_378_752e-6 {
            return;
        }
        match axis {
            0 => x = delta,
            1 => y = delta,
            _ => z = delta,
        }
        self.moveEntityTyped(world, x, y, z, false, true, None);
    }

    fn moveEntityTyped(
        &mut self,
        world: &WorldClient,
        mut x: f64,
        mut y: f64,
        mut z: f64,
        livingBase: bool,
        pistonMover: bool,
        collisionContext: Option<(i32, bool)>,
    ) {
        if self.noClip {
            self.boundingBox = self.boundingBox.offset(x, y, z);
            self.resetPositionToBB();
            return;
        }

        if self.isInWeb {
            self.isInWeb = false;
            x *= 0.25;
            y *= 0.05000000074505806;
            z *= 0.25;
            self.motionX = 0.0;
            self.motionY = 0.0;
            self.motionZ = 0.0;
        }

        // MCP `Entity.moveEntity` performs the player-only sneak ledge check
        // before collecting the swept collision list. `isSneaking()` is
        // represented by the concrete player updating this base-state flag.
        if !pistonMover && self.onGround && self.sneaking {
            let ledge_step = 0.05_f64;
            let below = -(self.stepHeight as f64);

            while x != 0.0
                && self.collisionBoxes(world, collisionContext, self.boundingBox.offset(x, below, 0.0)).is_empty()
            {
                x = reduce_towards_zero(x, ledge_step);
            }

            while z != 0.0
                && self.collisionBoxes(world, collisionContext, self.boundingBox.offset(0.0, below, z)).is_empty()
            {
                z = reduce_towards_zero(z, ledge_step);
            }

            while x != 0.0
                && z != 0.0
                && self.collisionBoxes(world, collisionContext, self.boundingBox.offset(x, below, z)).is_empty()
            {
                x = reduce_towards_zero(x, ledge_step);
                z = reduce_towards_zero(z, ledge_step);
            }
        }

        let requested_x = x;
        let requested_y = y;
        let requested_z = z;
        let original_box = self.boundingBox;
        let collisions = self.collisionBoxes(world, collisionContext, self.boundingBox.add_coord(x, y, z));

        for collision in &collisions {
            y = collision.calculate_y_offset(self.boundingBox, y);
        }
        self.boundingBox = self.boundingBox.offset(0.0, y, 0.0);

        for collision in &collisions {
            x = collision.calculate_x_offset(self.boundingBox, x);
        }
        if x != 0.0 {
            self.boundingBox = self.boundingBox.offset(x, 0.0, 0.0);
        }

        for collision in &collisions {
            z = collision.calculate_z_offset(self.boundingBox, z);
        }
        if z != 0.0 {
            self.boundingBox = self.boundingBox.offset(0.0, 0.0, z);
        }

        let can_step = self.onGround || requested_y != y && requested_y < 0.0;
        if self.stepHeight > 0.0 && can_step && (requested_x != x || requested_z != z) {
            let clipped_x = x;
            let clipped_y = y;
            let clipped_z = z;
            let clipped_box = self.boundingBox;
            self.boundingBox = original_box;

            let step_y = self.stepHeight as f64;
            let step_collisions = self.collisionBoxes(
                world,
                collisionContext,
                self.boundingBox.add_coord(requested_x, step_y, requested_z),
            );

            let mut path_a_box = self.boundingBox;
            let path_a_horizontal = path_a_box.add_coord(requested_x, 0.0, requested_z);
            let mut path_a_y = step_y;
            for collision in &step_collisions {
                path_a_y = collision.calculate_y_offset(path_a_horizontal, path_a_y);
            }
            path_a_box = path_a_box.offset(0.0, path_a_y, 0.0);
            let mut path_a_x = requested_x;
            for collision in &step_collisions {
                path_a_x = collision.calculate_x_offset(path_a_box, path_a_x);
            }
            path_a_box = path_a_box.offset(path_a_x, 0.0, 0.0);
            let mut path_a_z = requested_z;
            for collision in &step_collisions {
                path_a_z = collision.calculate_z_offset(path_a_box, path_a_z);
            }
            path_a_box = path_a_box.offset(0.0, 0.0, path_a_z);

            let mut path_b_box = self.boundingBox;
            let mut path_b_y = step_y;
            for collision in &step_collisions {
                path_b_y = collision.calculate_y_offset(path_b_box, path_b_y);
            }
            path_b_box = path_b_box.offset(0.0, path_b_y, 0.0);
            let mut path_b_x = requested_x;
            for collision in &step_collisions {
                path_b_x = collision.calculate_x_offset(path_b_box, path_b_x);
            }
            path_b_box = path_b_box.offset(path_b_x, 0.0, 0.0);
            let mut path_b_z = requested_z;
            for collision in &step_collisions {
                path_b_z = collision.calculate_z_offset(path_b_box, path_b_z);
            }
            path_b_box = path_b_box.offset(0.0, 0.0, path_b_z);

            if path_a_x * path_a_x + path_a_z * path_a_z
                > path_b_x * path_b_x + path_b_z * path_b_z
            {
                x = path_a_x;
                z = path_a_z;
                y = -path_a_y;
                self.boundingBox = path_a_box;
            } else {
                x = path_b_x;
                z = path_b_z;
                y = -path_b_y;
                self.boundingBox = path_b_box;
            }

            for collision in &step_collisions {
                y = collision.calculate_y_offset(self.boundingBox, y);
            }
            self.boundingBox = self.boundingBox.offset(0.0, y, 0.0);

            if clipped_x * clipped_x + clipped_z * clipped_z >= x * x + z * z {
                x = clipped_x;
                y = clipped_y;
                z = clipped_z;
                self.boundingBox = clipped_box;
            }
        }

        self.resetPositionToBB();
        self.isCollidedHorizontally = requested_x != x || requested_z != z;
        self.isCollidedVertically = requested_y != y;
        self.onGround = self.isCollidedVertically && requested_y < 0.0;
        self.isCollided = self.isCollidedHorizontally || self.isCollidedVertically;

        let mut groundPos = crate::net::minecraft::util::math::BlockPos::BlockPos::new(
            self.posX.floor() as i32,
            (self.posY - 0.20000000298023224).floor() as i32,
            self.posZ.floor() as i32,
        );
        let mut groundState = world.getBlockState(groundPos);
        if groundState.isAir() {
            let below = groundPos.down(1);
            let belowState = world.getBlockState(below);
            if matches!(belowState.getBlockId(), 85 | 107 | 139 | 183..=192) {
                groundPos = below;
                groundState = belowState;
            }
        }

        if requested_x != x {
            self.motionX = 0.0;
        }
        if requested_z != z {
            self.motionZ = 0.0;
        }
        if requested_y != y {
            if crate::net::minecraft::block::BlockSlime::isBlockSlime(groundState) {
                crate::net::minecraft::block::BlockSlime::onLanded(self, livingBase);
            } else {
                self.motionY = 0.0;
            }
        }

        if self.onGround
            && crate::net::minecraft::block::BlockSlime::isBlockSlime(groundState)
        {
            crate::net::minecraft::block::BlockSlime::onEntityWalk(self);
        }

        if world.intersectsBlockId(self.boundingBox.expand_xyz(-0.001), 30) {
            crate::net::minecraft::block::BlockWeb::onEntityCollidedWithBlock(self);
        }
        let _ = groundPos;
    }

    /// Direct port of `Entity#handleWaterMovement` for the local logical
    /// entity. Splash particles and sound remain owned by the pending effects
    /// and sound systems; the material volume and flow acceleration are exact.
    pub fn handleWaterMovement(&mut self, world: &WorldClient) -> bool {
        let test_box = self.boundingBox
            .expand(0.0, -0.4000000059604645, 0.0)
            .expand_xyz(-0.001);
        if world.handleMaterialAcceleration(
            test_box,
            crate::net::minecraft::block::BlockLiquid::LiquidMaterial::Water,
            self,
        ) {
            if !self.inWater && !self.firstUpdate {
                // `doWaterSplashEffect` is intentionally deferred until the
                // particle/sound owners exist; no static substitute is emitted.
            }
            self.fallDistance = 0.0;
            self.inWater = true;
        } else {
            self.inWater = false;
        }
        self.inWater
    }

    pub const fn isInWater(&self) -> bool { self.inWater }

    /// MCP `Entity#isPushedByWater`. The base entity implementation returns
    /// true; concrete non-pushable entities can override this contract when
    fn collisionBoxes(
        &self,
        world: &WorldClient,
        collisionContext: Option<(i32, bool)>,
        aabb: AxisAlignedBB,
    ) -> Vec<AxisAlignedBB> {
        match collisionContext {
            Some((entityId, collidesWithPushableEntities)) => world.getCollisionBoxesForEntity(
                entityId,
                self.ridingEntityId,
                collidesWithPushableEntities,
                aabb,
            ),
            None => world.getCollisionBoxes(aabb),
        }
    }

    /// their entity classes are ported.
    pub const fn isPushedByWater(&self) -> bool { true }

    /// Direct port of `Entity#isInLava`.
    pub fn isInLava(&self, world: &WorldClient) -> bool {
        world.isMaterialInBB(
            self.boundingBox.expand(
                -0.10000000149011612,
                -0.4000000059604645,
                -0.10000000149011612,
            ),
            crate::net::minecraft::block::BlockLiquid::LiquidMaterial::Lava,
        )
    }

    /// Port of `Entity#isOffsetPositionInLiquid`, used by liquid wall-exit
    /// assistance in `EntityLivingBase#travel`.
    pub fn isOffsetPositionInLiquid(&self, world: &WorldClient, x: f64, y: f64, z: f64) -> bool {
        let offset = self.boundingBox.offset(x, y, z);
        world.getCollisionBoxes(offset).is_empty() && !world.containsAnyLiquid(offset)
    }

    pub fn setInWeb(&mut self) {
        self.isInWeb = true;
        self.fallDistance = 0.0;
    }

    pub fn setVelocity(&mut self, x: f64, y: f64, z: f64) {
        self.motionX = x;
        self.motionY = y;
        self.motionZ = z;
    }

    pub fn removePassengers(&mut self) -> Vec<i32> {
        std::mem::take(&mut self.passengerIds)
    }

    pub fn setPassengers(&mut self, passengerIds: Vec<i32>) {
        self.passengerIds = passengerIds;
    }

    pub const fn isRiding(&self) -> bool { self.ridingEntityId.is_some() }

    pub fn resetPositionToBB(&mut self) {
        self.posX = (self.boundingBox.min_x + self.boundingBox.max_x) / 2.0;
        self.posY = self.boundingBox.min_y;
        self.posZ = (self.boundingBox.min_z + self.boundingBox.max_z) / 2.0;
    }
}

fn reduce_towards_zero(value: f64, amount: f64) -> f64 {
    if value < amount && value >= -amount {
        0.0
    } else if value > 0.0 {
        value - amount
    } else {
        value + amount
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_box_uses_vanilla_width_and_feet_y() {
        let mut entity = Entity::default();
        entity.setPosition(2.0, 64.0, -3.0);
        assert!((entity.boundingBox.min_x - 1.7).abs() < 1.0e-6);
        assert_eq!(entity.boundingBox.min_y, 64.0);
        assert!((entity.boundingBox.max_y - 65.8).abs() < 1.0e-6);
    }

    #[test]
    fn set_size_matches_client_world_shrink_and_growth_box_anchoring() {
        let mut entity = Entity::default();
        entity.setPosition(10.0, 64.0, -4.0);

        entity.setSize(0.2, 0.2);
        assert!((entity.boundingBox.min_x - 9.9).abs() < 1.0e-6);
        assert!((entity.boundingBox.max_x - 10.1).abs() < 1.0e-6);
        assert!((entity.boundingBox.max_y - 64.2).abs() < 1.0e-6);

        let shrunken_min_x = entity.boundingBox.min_x;
        let shrunken_min_z = entity.boundingBox.min_z;
        entity.setSize(0.6, 1.8);
        assert!((entity.boundingBox.min_x - shrunken_min_x).abs() < 1.0e-12);
        assert!((entity.boundingBox.min_z - shrunken_min_z).abs() < 1.0e-12);
        assert!((entity.boundingBox.max_x - (shrunken_min_x + 0.6)).abs() < 1.0e-6);
        assert!((entity.boundingBox.max_y - 65.8).abs() < 1.0e-6);
    }

    #[test]
    fn move_relative_uses_mcp_mathhelper_lookup_values() {
        let mut entity = Entity::default();
        entity.rotationYaw = 37.0;
        entity.func_191958_b(0.25, 0.0, 1.0, 0.1);
        let yaw = 37.0_f32 * 0.017453292_f32;
        let length = (0.25_f32 * 0.25 + 1.0).sqrt();
        let strafe = 0.25_f32 * 0.1 / length;
        let forward = 1.0_f32 * 0.1 / length;
        let expected_x = strafe * minecraft_cos(yaw) - forward * minecraft_sin(yaw);
        let expected_z = forward * minecraft_cos(yaw) + strafe * minecraft_sin(yaw);
        assert!((entity.motionX - expected_x as f64).abs() < 1.0e-12);
        assert!((entity.motionZ - expected_z as f64).abs() < 1.0e-12);
    }

    #[test]
    fn web_flag_slows_the_next_move_and_clears_motion() {
        let world = WorldClient::new(0);
        let mut entity = Entity::default();
        entity.setPosition(0.5, 64.0, 0.5);
        entity.motionX = 1.0;
        entity.motionY = 1.0;
        entity.motionZ = 1.0;
        entity.setInWeb();
        entity.moveEntity(&world, 1.0, 1.0, 1.0);

        assert!((entity.posX - 0.75).abs() < 1.0e-9);
        assert!((entity.posY - 64.05000000074506).abs() < 1.0e-9);
        assert!((entity.posZ - 0.75).abs() < 1.0e-9);
        assert_eq!((entity.motionX, entity.motionY, entity.motionZ), (0.0, 0.0, 0.0));
        assert!(!entity.isInWeb);
    }

    #[test]
    fn living_entity_bounces_from_real_slime_block() {
        use crate::net::minecraft::block::state::IBlockState::IBlockState;
        use crate::net::minecraft::util::math::BlockPos::BlockPos;

        let mut world = WorldClient::new(0);
        world
            .invalidateRegionAndSetBlock(
                BlockPos::new(0, 63, 0),
                IBlockState::fromGlobalStateId(165 << 4),
            )
            .unwrap();
        let mut entity = Entity::default();
        entity.setPosition(0.5, 64.0, 0.5);
        entity.motionY = -0.5;
        entity.moveEntityLiving(&world, 0.0, -0.5, 0.0);

        assert!(entity.onGround);
        assert!((entity.motionY - 0.5).abs() < 1.0e-9);
    }

    #[test]
    fn downward_motion_is_clipped_by_a_real_world_block() {
        use crate::net::minecraft::block::state::IBlockState::IBlockState;
        use crate::net::minecraft::util::math::BlockPos::BlockPos;

        let mut world = WorldClient::new(0);
        world
            .invalidateRegionAndSetBlock(
                BlockPos::new(0, 63, 0),
                IBlockState::fromGlobalStateId(1 << 4),
            )
            .unwrap();
        let mut entity = Entity::default();
        entity.setPosition(0.5, 64.0, 0.5);
        entity.moveEntity(&world, 0.0, -0.25, 0.0);

        assert_eq!(entity.posY, 64.0);
        assert!(entity.onGround);
        assert!(entity.isCollidedVertically);
    }
}
