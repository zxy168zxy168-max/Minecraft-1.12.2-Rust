use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;

/// Client-visible state owned by MCP 1.12.2 `TileEntitySign`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileEntitySign {
    pub pos: BlockPos,
    pub signText: [ITextComponent; 4],
    /// MCP field used by `GuiEditSign` to draw the `> <` edit markers.
    pub lineBeingEdited: i32,
    isEditable: bool,
}

impl TileEntitySign {
    pub fn new(pos: BlockPos) -> Self {
        Self {
            pos,
            signText: std::array::from_fn(|_| ITextComponent::fromPlainText("")),
            lineBeingEdited: -1,
            isEditable: true,
        }
    }

    /// MCP `TileEntitySign#readFromNBT` subset required by the client renderer.
    /// Command-result stats and command execution remain server responsibilities.
    pub fn fromNbt(tag: &NBTTagCompound) -> Option<Self> {
        let id = tag.getString("id");
        if !id.is_empty() && id != "minecraft:sign" && id != "Sign" {
            return None;
        }
        let mut sign = Self::new(BlockPos::new(
            tag.getInteger("x"),
            tag.getInteger("y"),
            tag.getInteger("z"),
        ));
        sign.isEditable = false;
        for index in 0..4 {
            let raw = tag.getString(&format!("Text{}", index + 1));
            sign.signText[index] = ITextComponent::fromJsonLenient(&raw)
                .unwrap_or_else(|_| ITextComponent::fromPlainText(raw));
        }
        Some(sign)
    }

    pub const fn getIsEditable(&self) -> bool {
        self.isEditable
    }

    pub fn setEditable(&mut self, editable: bool) {
        self.isEditable = editable;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_four_json_text_lines_and_disables_server_loaded_editing() {
        let mut tag = NBTTagCompound::new();
        tag.setString("id", "minecraft:sign");
        tag.setInteger("x", 3);
        tag.setInteger("y", 70);
        tag.setInteger("z", -5);
        tag.setString("Text1", r#"{"text":"Line one"}"#);
        tag.setString("Text2", r#"{"text":"Red","color":"red"}"#);
        let sign = TileEntitySign::fromNbt(&tag).expect("sign");
        assert_eq!(sign.pos, BlockPos::new(3, 70, -5));
        assert_eq!(sign.signText[0].getUnformattedText(), "Line one");
        assert_eq!(sign.signText[1].getUnformattedText(), "Red");
        assert!(!sign.getIsEditable());
        assert_eq!(sign.lineBeingEdited, -1);
    }
}
