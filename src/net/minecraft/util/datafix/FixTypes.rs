/// MCP 1.12.2 `FixTypes` / `IFixType` keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FixTypes {
    Level,
    Player,
    Chunk,
    BlockEntity,
    Entity,
    ItemInstance,
    Options,
    Structure,
}
