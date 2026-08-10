use std::collections::{HashMap, VecDeque};
use rustc_hash::FxHashSet;

use crate::net::minecraft::client::renderer::chunk::CompiledChunk::CompiledChunk;
use crate::net::minecraft::client::renderer::chunk::RenderChunk::RenderChunkKey;
use crate::net::minecraft::client::renderer::ViewFrustum::containsRenderChunk;
use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::math::RayTraceResult::{RayTraceResult, Type as RayTraceType};


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectionBoxRenderState {
    pub boundingBox: AxisAlignedBB,
    pub color: [f32; 4],
    pub lineWidth: f32,
}

/// MCP 1.12.2 `RenderGlobal.drawSelectionBox` block branch. Vulkan owns the
/// actual blend/depth/line state; this class preserves the selected AABB,
/// expansion epsilon, black alpha and requested two-pixel line width.
pub fn drawSelectionBox(
    world: &WorldClient,
    movingObjectPositionIn: Option<RayTraceResult>,
    execute: i32,
) -> Option<SelectionBoxRenderState> {
    if execute != 0 {
        return None;
    }
    let hit = movingObjectPositionIn?;
    if hit.typeOfHit != RayTraceType::Block {
        return None;
    }
    let selected = world.getSelectedBoundingBox(hit.getBlockPos())?;
    Some(selectionBoxForBounds(selected))
}

pub fn selectionBoxForBounds(bounds: AxisAlignedBB) -> SelectionBoxRenderState {
    SelectionBoxRenderState {
        boundingBox: bounds.expand_xyz(0.0020000000949949026),
        color: [0.0, 0.0, 0.0, 0.4],
        lineWidth: 2.0,
    }
}

#[derive(Debug, Clone, Copy)]
struct ContainerLocalRenderInformation {
    renderChunk: RenderChunkKey,
    facing: Option<EnumFacing>,
    setFacing: u8,
}

impl ContainerLocalRenderInformation {
    const fn new(
        renderChunk: RenderChunkKey,
        facing: Option<EnumFacing>,
        setFacing: u8,
    ) -> Self {
        Self {
            renderChunk,
            facing,
            setFacing,
        }
    }

    const fn hasDirection(self, facing: EnumFacing) -> bool {
        (self.setFacing & (1_u8 << facing.index())) != 0
    }
}

/// Reusable storage for the allocation-sensitive `RenderGlobal.setupTerrain`
/// breadth-first traversal. Keeping ownership here preserves the exact vanilla
/// queue/visited algorithm while allowing native renderers to retain capacity
/// across frames instead of rebuilding three large containers every frame.
#[derive(Debug, Default)]
pub struct TerrainTraversalScratch {
    result: Vec<RenderChunkKey>,
    visited: FxHashSet<RenderChunkKey>,
    queue: VecDeque<ContainerLocalRenderInformation>,
}

impl TerrainTraversalScratch {
    pub fn clear(&mut self) {
        self.result.clear();
        self.visited.clear();
        self.queue.clear();
    }

    pub fn result(&self) -> &[RenderChunkKey] {
        &self.result
    }
}

/// Visibility traversal corresponding to MCP 1.12.2
/// `RenderGlobal.setupTerrain` and `ContainerLocalRenderInformation`.
///
/// The caller supplies the frustum test because the Vulkan backend owns the
/// current clip matrix. Empty RenderChunks remain in `compiledChunks` so the
/// traversal can pass through air without issuing a draw.
pub fn setupTerrain<F>(
    start: RenderChunkKey,
    renderDistanceChunks: i32,
    compiledChunks: &HashMap<RenderChunkKey, CompiledChunk>,
    isInFrustum: F,
) -> Vec<RenderChunkKey>
where
    F: FnMut(RenderChunkKey) -> bool,
{
    let start = if compiledChunks.contains_key(&start) {
        start
    } else if let Some(nearest) = compiledChunks
        .keys()
        .filter(|key| key.x == start.x && key.z == start.z)
        .min_by_key(|key| (key.y - start.y).abs())
        .copied()
    {
        nearest
    } else {
        return Vec::new();
    };
    setupTerrainWithLookup(
        start,
        renderDistanceChunks,
        |key| compiledChunks.get(&key).copied(),
        isInFrustum,
    )
}

/// Allocation-free lookup variant used by native world backends. It preserves
/// the exact `RenderGlobal.setupTerrain` queue, facing mask and VisGraph rules,
/// while allowing the caller to query its resident chunk cache directly rather
/// than rebuilding a second `HashMap<RenderChunkKey, CompiledChunk>` every
/// rendered frame.
pub fn setupTerrainWithLookup<G, F>(
    start: RenderChunkKey,
    renderDistanceChunks: i32,
    compiledChunk: G,
    isInFrustum: F,
) -> Vec<RenderChunkKey>
where
    G: FnMut(RenderChunkKey) -> Option<CompiledChunk>,
    F: FnMut(RenderChunkKey) -> bool,
{
    let mut scratch = TerrainTraversalScratch::default();
    setupTerrainWithLookupScratch(
        start,
        renderDistanceChunks,
        compiledChunk,
        isInFrustum,
        &mut scratch,
    );
    std::mem::take(&mut scratch.result)
}

/// Capacity-reusing variant for render backends that execute setupTerrain on
/// every frame. The traversal itself is deliberately identical to the public
/// Vec-returning helper above and to MCP's `RenderGlobal#setupTerrain`.
pub fn setupTerrainWithLookupScratch<G, F>(
    start: RenderChunkKey,
    renderDistanceChunks: i32,
    mut compiledChunk: G,
    mut isInFrustum: F,
    scratch: &mut TerrainTraversalScratch,
) where
    G: FnMut(RenderChunkKey) -> Option<CompiledChunk>,
    F: FnMut(RenderChunkKey) -> bool,
{
    scratch.clear();
    if compiledChunk(start).is_none() {
        return;
    }

    scratch.visited.insert(start);
    scratch
        .queue
        .push_back(ContainerLocalRenderInformation::new(start, None, 0));

    while let Some(info) = scratch.queue.pop_front() {
        let Some(compiled) = compiledChunk(info.renderChunk) else {
            continue;
        };
        scratch.result.push(info.renderChunk);

        for facing in EnumFacing::VALUES {
            if info.hasDirection(facing.opposite()) {
                continue;
            }
            if let Some(entryFacing) = info.facing {
                if !compiled.isVisible(entryFacing.opposite(), facing) {
                    continue;
                }
            }

            let neighbour = info.renderChunk.offset(facing);
            if !containsRenderChunk(start, neighbour, renderDistanceChunks)
                || scratch.visited.contains(&neighbour)
                || compiledChunk(neighbour).is_none()
                || !isInFrustum(neighbour)
            {
                continue;
            }

            scratch.visited.insert(neighbour);
            scratch.queue.push_back(ContainerLocalRenderInformation::new(
                neighbour,
                Some(facing),
                info.setFacing | (1_u8 << facing.index()),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::client::renderer::chunk::SetVisibility::SetVisibility;

    #[test]
    fn selection_box_preserves_vanilla_epsilon_color_and_width() {
        let state = selectionBoxForBounds(AxisAlignedBB::new(1.0, 2.0, 3.0, 2.0, 3.0, 4.0));
        let epsilon = 0.0020000000949949026;
        assert!((state.boundingBox.min_x - (1.0 - epsilon)).abs() < 1.0e-12);
        assert!((state.boundingBox.max_z - (4.0 + epsilon)).abs() < 1.0e-12);
        assert_eq!(state.color, [0.0, 0.0, 0.0, 0.4]);
        assert_eq!(state.lineWidth, 2.0);
    }

    #[test]
    fn traversal_passes_through_empty_render_chunks() {
        let mut chunks = HashMap::new();
        for x in 0..3 {
            chunks.insert(RenderChunkKey::new(x, 4, 0), CompiledChunk::emptyVisible());
        }
        let visible = setupTerrain(RenderChunkKey::new(0, 4, 0), 4, &chunks, |_| true);
        assert_eq!(visible.len(), 3);
    }

    #[test]
    fn direct_lookup_variant_matches_hash_map_wrapper() {
        let mut chunks = HashMap::new();
        for x in -2..=2 {
            for y in 3..=5 {
                chunks.insert(RenderChunkKey::new(x, y, 0), CompiledChunk::emptyVisible());
            }
        }
        let start = RenderChunkKey::new(0, 4, 0);
        let wrapped = setupTerrain(start, 4, &chunks, |_| true);
        let direct = setupTerrainWithLookup(
            start,
            4,
            |key| chunks.get(&key).copied(),
            |_| true,
        );
        assert_eq!(direct, wrapped);
    }

    #[test]
    fn occluded_face_pair_stops_traversal() {
        let mut chunks = HashMap::new();
        let mut middle = CompiledChunk::emptyVisible();
        let mut visibility = SetVisibility::allVisible();
        visibility.setVisible(EnumFacing::West, EnumFacing::East, false);
        middle.setVisibility(visibility);
        chunks.insert(RenderChunkKey::new(0, 4, 0), CompiledChunk::emptyVisible());
        chunks.insert(RenderChunkKey::new(1, 4, 0), middle);
        chunks.insert(RenderChunkKey::new(2, 4, 0), CompiledChunk::emptyVisible());
        let visible = setupTerrain(RenderChunkKey::new(0, 4, 0), 4, &chunks, |_| true);
        assert_eq!(visible, vec![RenderChunkKey::new(0, 4, 0), RenderChunkKey::new(1, 4, 0)]);
    }
}
