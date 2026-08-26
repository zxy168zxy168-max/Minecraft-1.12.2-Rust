use crate::net::minecraft::creativetab::CreativeTabData::itemsForTab;
use crate::net::minecraft::item::ItemStack::ItemStack;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreativeTab {
    pub tabIndex: i32,
    pub tabLabel: &'static str,
    pub backgroundImageName: &'static str,
    pub hasScrollbar: bool,
    pub drawTitle: bool,
    pub rightAligned: bool,
    iconBytes: &'static [u8],
}

impl CreativeTab {
    pub const fn getTabIndex(self) -> i32 {
        self.tabIndex
    }
    pub const fn getTabColumn(self) -> i32 {
        self.tabIndex % 6
    }
    pub const fn isTabInFirstRow(self) -> bool {
        self.tabIndex < 6
    }
    pub const fn shouldHidePlayerInventory(self) -> bool {
        self.hasScrollbar
    }
    pub const fn drawInForegroundOfTab(self) -> bool {
        self.drawTitle
    }
    pub const fn getTranslatedTabLabel(self) -> &'static str {
        match self.tabIndex {
            0 => "itemGroup.buildingBlocks",
            1 => "itemGroup.decorations",
            2 => "itemGroup.redstone",
            3 => "itemGroup.transportation",
            4 => "itemGroup.hotbar",
            5 => "itemGroup.search",
            6 => "itemGroup.misc",
            7 => "itemGroup.food",
            8 => "itemGroup.tools",
            9 => "itemGroup.combat",
            10 => "itemGroup.brewing",
            11 => "itemGroup.inventory",
            _ => "",
        }
    }

    pub fn getIconItemStack(self) -> ItemStack {
        let mut input = self.iconBytes;
        ItemStack::readFromBuffer(&mut input).expect("compiled MCP creative-tab icon must decode")
    }

    pub fn displayAllRelevantItems(self) -> Vec<ItemStack> {
        itemsForTab(self.tabIndex)
    }
}

pub const BUILDING_BLOCKS: CreativeTab = CreativeTab {
    tabIndex: 0,
    tabLabel: "buildingBlocks",
    backgroundImageName: "items.png",
    hasScrollbar: true,
    drawTitle: true,
    rightAligned: false,
    iconBytes: &[0x00, 0x2d, 0x01, 0x00, 0x00, 0x00],
};
pub const DECORATIONS: CreativeTab = CreativeTab {
    tabIndex: 1,
    tabLabel: "decorations",
    backgroundImageName: "items.png",
    hasScrollbar: true,
    drawTitle: true,
    rightAligned: false,
    iconBytes: &[0x00, 0xaf, 0x01, 0x00, 0x05, 0x00],
};
pub const REDSTONE: CreativeTab = CreativeTab {
    tabIndex: 2,
    tabLabel: "redstone",
    backgroundImageName: "items.png",
    hasScrollbar: true,
    drawTitle: true,
    rightAligned: false,
    iconBytes: &[0x01, 0x4b, 0x01, 0x00, 0x00, 0x00],
};
pub const TRANSPORTATION: CreativeTab = CreativeTab {
    tabIndex: 3,
    tabLabel: "transportation",
    backgroundImageName: "items.png",
    hasScrollbar: true,
    drawTitle: true,
    rightAligned: false,
    iconBytes: &[0x00, 0x1b, 0x01, 0x00, 0x00, 0x00],
};
pub const HOTBAR: CreativeTab = CreativeTab {
    tabIndex: 4,
    tabLabel: "hotbar",
    backgroundImageName: "items.png",
    hasScrollbar: true,
    drawTitle: true,
    rightAligned: true,
    iconBytes: &[0x00, 0x2f, 0x01, 0x00, 0x00, 0x00],
};
pub const SEARCH: CreativeTab = CreativeTab {
    tabIndex: 5,
    tabLabel: "search",
    backgroundImageName: "item_search.png",
    hasScrollbar: true,
    drawTitle: true,
    rightAligned: true,
    iconBytes: &[0x01, 0x59, 0x01, 0x00, 0x00, 0x00],
};
pub const MISC: CreativeTab = CreativeTab {
    tabIndex: 6,
    tabLabel: "misc",
    backgroundImageName: "items.png",
    hasScrollbar: true,
    drawTitle: true,
    rightAligned: false,
    iconBytes: &[0x01, 0x47, 0x01, 0x00, 0x00, 0x00],
};
pub const FOOD: CreativeTab = CreativeTab {
    tabIndex: 7,
    tabLabel: "food",
    backgroundImageName: "items.png",
    hasScrollbar: true,
    drawTitle: true,
    rightAligned: false,
    iconBytes: &[0x01, 0x04, 0x01, 0x00, 0x00, 0x00],
};
pub const TOOLS: CreativeTab = CreativeTab {
    tabIndex: 8,
    tabLabel: "tools",
    backgroundImageName: "items.png",
    hasScrollbar: true,
    drawTitle: true,
    rightAligned: false,
    iconBytes: &[0x01, 0x02, 0x01, 0x00, 0x00, 0x00],
};
pub const COMBAT: CreativeTab = CreativeTab {
    tabIndex: 9,
    tabLabel: "combat",
    backgroundImageName: "items.png",
    hasScrollbar: true,
    drawTitle: true,
    rightAligned: false,
    iconBytes: &[0x01, 0x1b, 0x01, 0x00, 0x00, 0x00],
};
pub const BREWING: CreativeTab = CreativeTab {
    tabIndex: 10,
    tabLabel: "brewing",
    backgroundImageName: "items.png",
    hasScrollbar: true,
    drawTitle: true,
    rightAligned: false,
    iconBytes: &[
        0x01, 0x75, 0x01, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x08, 0x00, 0x06, 0x50, 0x6f, 0x74, 0x69,
        0x6f, 0x6e, 0x00, 0x0f, 0x6d, 0x69, 0x6e, 0x65, 0x63, 0x72, 0x61, 0x66, 0x74, 0x3a, 0x77,
        0x61, 0x74, 0x65, 0x72, 0x00,
    ],
};
pub const INVENTORY: CreativeTab = CreativeTab {
    tabIndex: 11,
    tabLabel: "inventory",
    backgroundImageName: "inventory.png",
    hasScrollbar: false,
    drawTitle: false,
    rightAligned: true,
    iconBytes: &[0x00, 0x36, 0x01, 0x00, 0x00, 0x00],
};

pub const CREATIVE_TAB_ARRAY: [CreativeTab; 12] = [
    BUILDING_BLOCKS,
    DECORATIONS,
    REDSTONE,
    TRANSPORTATION,
    HOTBAR,
    SEARCH,
    MISC,
    FOOD,
    TOOLS,
    COMBAT,
    BREWING,
    INVENTORY,
];

pub fn byIndex(index: i32) -> Option<CreativeTab> {
    CREATIVE_TAB_ARRAY.get(index as usize).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_geometry_and_icons_match_mcp() {
        assert_eq!(SEARCH.getTabColumn(), 5);
        assert!(SEARCH.isTabInFirstRow());
        assert!(SEARCH.rightAligned);
        assert_eq!(INVENTORY.backgroundImageName, "inventory.png");
        assert!(!INVENTORY.shouldHidePlayerInventory());
        assert_eq!(BREWING.getIconItemStack().itemId, 373);
        assert_eq!(DECORATIONS.getIconItemStack().itemDamage, 5);
    }
}
