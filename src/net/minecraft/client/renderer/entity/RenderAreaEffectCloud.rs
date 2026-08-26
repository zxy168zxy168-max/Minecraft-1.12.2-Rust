/// MCP 1.12.2 `RenderAreaEffectCloud` deliberately has no texture or geometry;
/// all visible cloud output is emitted by `EntityAreaEffectCloud#onUpdate` into ParticleManager.
pub struct RenderAreaEffectCloud;
impl RenderAreaEffectCloud {
    pub const fn hasEntityGeometry() -> bool {
        false
    }
}
