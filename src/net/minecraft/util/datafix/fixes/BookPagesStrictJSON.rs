use crate::net::minecraft::nbt::NBTBase::{NBTBase, TAG_LIST, TAG_STRING};
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;
use crate::net::minecraft::util::datafix::fixes::SignStrictJSON::SignStrictJSON;

/// MCP 1.12.2 `BookPagesStrictJSON` (DataVersion 165).
pub struct BookPagesStrictJSON;
impl IFixableData for BookPagesStrictJSON {
    fn getFixVersion(&self) -> i32 { 165 }
    fn fixTagCompound(&self, mut compound: NBTTagCompound) -> NBTTagCompound {
        if compound.getString("id") != "minecraft:written_book" { return compound; }
        let mut tag = compound.getCompoundTag("tag");
        if tag.hasKeyWithType("pages", TAG_LIST) {
            let mut pages = tag.getTagList("pages", TAG_STRING);
            for index in 0..pages.tagCount() {
                let fixed = SignStrictJSON::normalizeTextComponentJson(&pages.getStringTagAt(index));
                pages.set(index, NBTBase::String(fixed));
            }
            tag.setTagList("pages", pages);
            compound.setCompoundTag("tag", tag);
        }
        compound
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::nbt::NBTTagList::NBTTagList;
    #[test]
    fn written_book_pages_use_same_strict_component_migration_as_signs() {
        let mut pages = NBTTagList::new(); pages.appendTag(NBTBase::String("hello".into()));
        let mut tag = NBTTagCompound::new(); tag.setTagList("pages", pages);
        let mut book = NBTTagCompound::new(); book.setString("id", "minecraft:written_book"); book.setCompoundTag("tag", tag);
        let fixed = BookPagesStrictJSON.fixTagCompound(book);
        assert_eq!(fixed.getCompoundTag("tag").getTagList("pages", TAG_STRING).getStringTagAt(0), r#"{"text":"hello"}"#);
    }
}
