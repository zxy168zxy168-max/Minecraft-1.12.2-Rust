use crate::net::minecraft::client::gui::inventory::GuiContainer::{GuiContainer, GuiSlot};
use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiTextField::{
    GuiTextField, GuiTextFieldKey, GuiTextFieldModifiers, GuiTextFieldRenderState,
};
use crate::net::minecraft::creativetab::CreativeTabs::{
    byIndex, BUILDING_BLOCKS, HOTBAR, INVENTORY, SEARCH,
};
use crate::net::minecraft::item::ItemRegistryData::definition;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// The semantic source represented by a visible `ContainerCreative` slot.
/// This mirrors the two layouts installed by MCP `GuiContainerCreative`:
/// the 45-entry temporary inventory plus the real hotbar, and the wrapped
/// `ContainerPlayer` layout used by the inventory tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreativeSlotKind {
    Catalog { itemIndex: usize },
    Hotbar { playerContainerSlot: i32 },
    Player { playerContainerSlot: i32 },
    Destroy,
}

/// Incremental Rust port of MCP 1.12.2 `GuiContainerCreative` and its
/// `ContainerCreative`. Geometry, tab selection, scrolling and the backing
/// vanilla item lists are kept on the screen object; actual creative click
/// authority remains in `PlayerControllerMP`/the play connection.
#[derive(Debug, Clone)]
pub struct GuiContainerCreative {
    pub container: GuiContainer,
    pub selectedTabIndex: i32,
    pub currentScroll: f32,
    pub isScrolling: bool,
    pub wasClicking: bool,
    pub clearSearch: bool,
    pub searchField: GuiTextField,
    pub itemList: Vec<ItemStack>,
    visibleCatalog: Vec<ItemStack>,
    slotKinds: Vec<CreativeSlotKind>,
}

impl Default for GuiContainerCreative {
    fn default() -> Self {
        Self::new()
    }
}

impl GuiContainerCreative {
    pub const X_SIZE: i32 = 195;
    pub const Y_SIZE: i32 = 136;
    pub const CATALOG_SLOT_COUNT: usize = 45;
    pub const HOTBAR_SLOT_START: i32 = 45;
    pub const DESTROY_SLOT: i32 = 46;

    pub fn new() -> Self {
        let mut searchField = GuiTextField::new(0, 82, 6, 80, 9);
        searchField.setMaxStringLength(50);
        searchField.setEnableBackgroundDrawing(false);
        searchField.setVisible(false);
        searchField.setTextColor(16_777_215);
        searchField.setCanLoseFocus(true);
        searchField.setFocused(false);

        let mut result = Self {
            container: GuiContainer::new(Self::X_SIZE, Self::Y_SIZE, Vec::new()),
            selectedTabIndex: BUILDING_BLOCKS.tabIndex,
            currentScroll: 0.0,
            isScrolling: false,
            wasClicking: false,
            clearSearch: false,
            searchField,
            itemList: Vec::new(),
            visibleCatalog: vec![ItemStack::EMPTY; Self::CATALOG_SLOT_COUNT],
            slotKinds: Vec::new(),
        };
        result.installCatalogSlots();
        result.setCurrentCreativeTab(BUILDING_BLOCKS.tabIndex);
        result
    }

    pub fn initGui(&mut self, width: i32, height: i32) {
        self.container.initGui(width, height);
        self.searchField.xPosition = self.container.guiLeft + 82;
        self.searchField.yPosition = self.container.guiTop + 6;
        // MCP initGui temporarily resets the static selected index and calls
        // setCurrentCreativeTab again. This rebuilds wrapped inventory slots,
        // clears SEARCH text and resets scrolling after opening or resizing.
        let selected = self.selectedTabIndex;
        self.selectedTabIndex = -1;
        self.setCurrentCreativeTab(selected);
    }

    fn installCatalogSlots(&mut self) {
        let mut slots = Vec::with_capacity(54);
        let mut kinds = Vec::with_capacity(54);
        for row in 0..5 {
            for column in 0..9 {
                let slotNumber = row * 9 + column;
                slots.push(GuiSlot {
                    slotNumber,
                    xPos: 9 + column * 18,
                    yPos: 18 + row * 18,
                });
                kinds.push(CreativeSlotKind::Catalog {
                    itemIndex: slotNumber as usize,
                });
            }
        }
        for column in 0..9 {
            slots.push(GuiSlot {
                slotNumber: Self::HOTBAR_SLOT_START + column,
                xPos: 9 + column * 18,
                yPos: 112,
            });
            kinds.push(CreativeSlotKind::Hotbar {
                playerContainerSlot: 36 + column,
            });
        }
        self.container.inventorySlots = slots;
        self.slotKinds = kinds;
    }

    fn installInventorySlots(&mut self) {
        let mut slots = Vec::with_capacity(47);
        let mut kinds = Vec::with_capacity(47);
        for slotNumber in 0..46 {
            let (xPos, yPos) = if (5..9).contains(&slotNumber) {
                let armor = slotNumber - 5;
                (54 + (armor / 2) * 54, 6 + (armor % 2) * 27)
            } else if (0..5).contains(&slotNumber) {
                (-2000, -2000)
            } else if slotNumber == 45 {
                (35, 20)
            } else {
                let inventoryIndex = slotNumber - 9;
                let column = inventoryIndex % 9;
                let row = inventoryIndex / 9;
                (
                    9 + column * 18,
                    if slotNumber >= 36 { 112 } else { 54 + row * 18 },
                )
            };
            slots.push(GuiSlot {
                slotNumber,
                xPos,
                yPos,
            });
            kinds.push(CreativeSlotKind::Player {
                playerContainerSlot: slotNumber,
            });
        }
        slots.push(GuiSlot {
            slotNumber: Self::DESTROY_SLOT,
            xPos: 173,
            yPos: 112,
        });
        kinds.push(CreativeSlotKind::Destroy);
        self.container.inventorySlots = slots;
        self.slotKinds = kinds;
    }

    /// Port of `GuiContainerCreative#setCurrentCreativeTab` for all registry
    /// backed tabs and the two alternate slot layouts. Empty SEARCH uses the
    /// exact compiled MCP SEARCH list. HOTBAR snapshots are intentionally left
    /// to `CreativeSettings` rather than synthesising non-persistent data.
    pub fn setCurrentCreativeTab(&mut self, tabIndex: i32) -> bool {
        let Some(tab) = byIndex(tabIndex) else {
            return false;
        };
        let previous = self.selectedTabIndex;
        self.selectedTabIndex = tabIndex;
        self.container.cancelDragSplitting();
        self.itemList.clear();

        if tabIndex == INVENTORY.tabIndex {
            self.installInventorySlots();
        } else {
            if previous == INVENTORY.tabIndex || self.container.inventorySlots.len() != 54 {
                self.installCatalogSlots();
            }
            if tabIndex == HOTBAR.tabIndex {
                // `CreativeSettings`/`HotbarSnapshot` is persisted in
                // hotbar.nbt. Do not replace it with fabricated toolbar data.
                self.itemList.resize(81, ItemStack::EMPTY);
            } else {
                self.itemList = tab.displayAllRelevantItems();
            }
        }

        if tabIndex == SEARCH.tabIndex {
            self.searchField.setVisible(true);
            self.searchField.setCanLoseFocus(false);
            self.searchField.setFocused(true);
            self.searchField.setText("");
            self.updateCreativeSearch();
        } else {
            self.searchField.setVisible(false);
            self.searchField.setCanLoseFocus(true);
            self.searchField.setFocused(false);
        }

        self.currentScroll = 0.0;
        self.scrollTo(0.0);
        true
    }

    /// Exact `ContainerCreative#scrollTo` row selection and rounding.
    pub fn scrollTo(&mut self, scroll: f32) {
        self.currentScroll = scroll.clamp(0.0, 1.0);
        let rowsBeyondWindow = ((self.itemList.len() + 8) / 9).saturating_sub(5) as f32;
        let row = (self.currentScroll * rowsBeyondWindow + 0.5)
            .floor()
            .max(0.0) as usize;
        self.visibleCatalog.clear();
        self.visibleCatalog.reserve(Self::CATALOG_SLOT_COUNT);
        for visible in 0..Self::CATALOG_SLOT_COUNT {
            let source = (visible % 9) + ((visible / 9) + row) * 9;
            self.visibleCatalog.push(
                self.itemList
                    .get(source)
                    .cloned()
                    .unwrap_or(ItemStack::EMPTY),
            );
        }
    }

    pub fn canScroll(&self) -> bool {
        self.itemList.len() > Self::CATALOG_SLOT_COUNT
    }

    pub fn needsScrollBars(&self) -> bool {
        self.selectedTabIndex != INVENTORY.tabIndex
            && byIndex(self.selectedTabIndex).is_some_and(|tab| tab.shouldHidePlayerInventory())
            && self.canScroll()
    }

    pub fn slotKind(&self, slotNumber: i32) -> Option<CreativeSlotKind> {
        let index = self
            .container
            .inventorySlots
            .iter()
            .position(|slot| slot.slotNumber == slotNumber)?;
        self.slotKinds.get(index).copied()
    }

    pub fn displayStacks(&self, playerContainerSlots: &[ItemStack]) -> Vec<ItemStack> {
        self.slotKinds
            .iter()
            .map(|kind| match *kind {
                CreativeSlotKind::Catalog { itemIndex } => self
                    .visibleCatalog
                    .get(itemIndex)
                    .cloned()
                    .unwrap_or(ItemStack::EMPTY),
                CreativeSlotKind::Hotbar {
                    playerContainerSlot,
                }
                | CreativeSlotKind::Player {
                    playerContainerSlot,
                } => playerContainerSlots
                    .get(playerContainerSlot as usize)
                    .cloned()
                    .unwrap_or(ItemStack::EMPTY),
                CreativeSlotKind::Destroy => ItemStack::EMPTY,
            })
            .collect()
    }

    pub fn stackForSlot(&self, slotNumber: i32, playerContainerSlots: &[ItemStack]) -> ItemStack {
        match self.slotKind(slotNumber) {
            Some(CreativeSlotKind::Catalog { itemIndex }) => self
                .visibleCatalog
                .get(itemIndex)
                .cloned()
                .unwrap_or(ItemStack::EMPTY),
            Some(CreativeSlotKind::Hotbar {
                playerContainerSlot,
            })
            | Some(CreativeSlotKind::Player {
                playerContainerSlot,
            }) => playerContainerSlots
                .get(playerContainerSlot as usize)
                .cloned()
                .unwrap_or(ItemStack::EMPTY),
            Some(CreativeSlotKind::Destroy) | None => ItemStack::EMPTY,
        }
    }

    pub fn tabAt(&self, mouseX: i32, mouseY: i32) -> Option<i32> {
        let relativeX = mouseX - self.container.guiLeft;
        let relativeY = mouseY - self.container.guiTop;
        (0..12).find(|index| self.isMouseOverTab(*index, relativeX, relativeY))
    }

    /// Exact relative tab hit box from MCP `isMouseOverTab`.
    pub fn isMouseOverTab(&self, tabIndex: i32, mouseX: i32, mouseY: i32) -> bool {
        let Some(tab) = byIndex(tabIndex) else {
            return false;
        };
        let column = tab.getTabColumn();
        let mut x = 28 * column;
        let y;
        if tab.rightAligned {
            x = Self::X_SIZE - 28 * (6 - column) + 2;
        } else if column > 0 {
            x += column;
        }
        if tab.isTabInFirstRow() {
            y = -32;
        } else {
            y = Self::Y_SIZE;
        }
        mouseX >= x && mouseX <= x + 28 && mouseY >= y && mouseY <= y + 32
    }

    pub fn scrollbarContains(&self, mouseX: i32, mouseY: i32) -> bool {
        let left = self.container.guiLeft + 175;
        let top = self.container.guiTop + 18;
        mouseX >= left && mouseY >= top && mouseX < left + 14 && mouseY < top + 112
    }

    /// Port of the scroll-drag state machine in `drawScreen`.
    pub fn updateScrollbarDrag(&mut self, mouseX: i32, mouseY: i32, leftButtonDown: bool) -> bool {
        let wasScrolling = self.isScrolling;
        if !self.wasClicking && leftButtonDown && self.scrollbarContains(mouseX, mouseY) {
            self.isScrolling = self.needsScrollBars();
        }
        if !leftButtonDown {
            self.isScrolling = false;
        }
        self.wasClicking = leftButtonDown;
        if self.isScrolling {
            let top = self.container.guiTop + 18;
            let scroll = ((mouseY - top) as f32 - 7.5) / (112.0 - 15.0);
            self.scrollTo(scroll.clamp(0.0, 1.0));
            return true;
        }
        wasScrolling != self.isScrolling
    }

    /// Port of `handleMouseInput`: wheel movement is normalized to one row.
    pub fn handleMouseWheel(&mut self, wheelDelta: i32) -> bool {
        if wheelDelta == 0 || !self.needsScrollBars() {
            return false;
        }
        let rows = ((self.itemList.len() + 8) / 9).saturating_sub(5);
        if rows == 0 {
            return false;
        }
        let direction = if wheelDelta > 0 { 1.0 } else { -1.0 };
        self.scrollTo((self.currentScroll - direction / rows as f32).clamp(0.0, 1.0));
        true
    }

    pub fn updateCursorCounter(&mut self) {
        self.searchField.updateCursorCounter();
    }

    pub fn searchRenderState(&self, font: &FontRenderer) -> Option<GuiTextFieldRenderState> {
        (self.selectedTabIndex == SEARCH.tabIndex).then(|| self.searchField.buildRenderState(font))
    }

    pub fn searchMouseClicked(
        &mut self,
        mouseX: i32,
        mouseY: i32,
        button: i32,
        font: &FontRenderer,
    ) -> bool {
        if self.selectedTabIndex != SEARCH.tabIndex {
            return false;
        }
        self.searchField.mouseClicked(mouseX, mouseY, button, font)
    }

    pub fn searchKeyPressed(
        &mut self,
        key: GuiTextFieldKey,
        modifiers: GuiTextFieldModifiers,
        font: &FontRenderer,
    ) -> bool {
        if self.selectedTabIndex != SEARCH.tabIndex {
            return false;
        }
        if self.clearSearch {
            self.clearSearch = false;
            self.searchField.setText("");
        }
        let changed = self.searchField.keyPressed(key, modifiers, font);
        if changed {
            self.updateCreativeSearch();
        }
        changed
    }

    pub fn searchTypedText(&mut self, text: &str, font: &FontRenderer) -> bool {
        if self.selectedTabIndex != SEARCH.tabIndex {
            return false;
        }
        if self.clearSearch {
            self.clearSearch = false;
            self.searchField.setText("");
        }
        let changed = self.searchField.writeText(text, Some(font));
        if changed {
            self.updateCreativeSearch();
        }
        changed
    }

    /// SEARCH's empty result is the exact MCP registry output. Non-empty
    /// filtering currently implements the registry-name half of MCP SearchTree
    /// (`minecraft:name` suffix search); localized tooltip indexing is kept as
    /// a separately identifiable remaining port rather than approximated with
    /// unrelated display strings.
    pub fn updateCreativeSearch(&mut self) {
        let complete = SEARCH.displayAllRelevantItems();
        let query = self.searchField.getText().to_lowercase();
        self.itemList = if query.is_empty() {
            complete
        } else {
            complete
                .into_iter()
                .filter(|stack| {
                    definition(stack.itemId)
                        .registryName
                        .to_lowercase()
                        .contains(&query)
                })
                .collect()
        };
        self.currentScroll = 0.0;
        self.scrollTo(0.0);
    }

    pub fn creativeTabsTexture() -> ResourceLocation {
        ResourceLocation::parse("textures/gui/container/creative_inventory/tabs.png")
    }

    pub fn backgroundTexture(&self) -> ResourceLocation {
        let background =
            byIndex(self.selectedTabIndex).map_or("items.png", |tab| tab.backgroundImageName);
        ResourceLocation::parse(&format!(
            "textures/gui/container/creative_inventory/tab_{background}"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tab_and_scroll_geometry_match_mcp() {
        let mut gui = GuiContainerCreative::new();
        gui.initGui(320, 240);
        assert_eq!(gui.selectedTabIndex, 0);
        assert_eq!(gui.container.inventorySlots.len(), 54);
        assert_eq!(gui.itemList.len(), 197);
        assert_eq!(
            gui.container.slotPosition(45),
            Some((gui.container.guiLeft + 9, gui.container.guiTop + 112))
        );
        gui.scrollTo(1.0);
        assert!(!gui.visibleCatalog[0].isEmpty());
    }

    #[test]
    fn inventory_tab_replaces_layout_and_adds_destroy_slot() {
        let mut gui = GuiContainerCreative::new();
        assert!(gui.setCurrentCreativeTab(INVENTORY.tabIndex));
        assert_eq!(gui.container.inventorySlots.len(), 47);
        assert_eq!(gui.container.slotPosition(45), Some((35, 20)));
        assert_eq!(
            gui.container
                .slotPosition(GuiContainerCreative::DESTROY_SLOT),
            Some((173, 112))
        );
        assert_eq!(
            gui.slotKind(GuiContainerCreative::DESTROY_SLOT),
            Some(CreativeSlotKind::Destroy)
        );
    }

    #[test]
    fn tab_hitboxes_keep_vanilla_rows_and_alignment() {
        let gui = GuiContainerCreative::new();
        assert!(gui.isMouseOverTab(0, 0, -32));
        assert!(gui.isMouseOverTab(5, 167, -32));
        assert!(gui.isMouseOverTab(11, 167, 136));
    }
}
