use crate::net::minecraft::client::gui::inventory::GuiContainer::{GuiContainer, GuiSlot};
use crate::net::minecraft::client::gui::recipebook::GuiRecipeBook::GuiRecipeBook;
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::entity::player::InventoryPlayer::InventoryPlayer;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::stats::RecipeBook::RecipeBook;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `GuiCrafting`/`ContainerWorkbench` fixed slot geometry and its
/// owned `GuiRecipeBook`. The recipe book, rather than the renderer, controls
/// the wide-screen horizontal offset and the narrow-screen overlay branch.
#[derive(Debug, Clone)]
pub struct GuiCrafting {
    pub container: GuiContainer,
    pub recipeBook: GuiRecipeBook,
    pub widthTooNarrow: bool,
}

impl GuiCrafting {
    pub const X_SIZE: i32 = 176;
    pub const Y_SIZE: i32 = 166;

    pub fn new() -> Self {
        let mut slots = Vec::with_capacity(46);
        slots.push(GuiSlot {
            slotNumber: 0,
            xPos: 124,
            yPos: 35,
        });
        for row in 0..3 {
            for column in 0..3 {
                slots.push(GuiSlot {
                    slotNumber: 1 + column + row * 3,
                    xPos: 30 + column * 18,
                    yPos: 17 + row * 18,
                });
            }
        }
        append_player_slots(&mut slots, 10);
        Self {
            container: GuiContainer::new(Self::X_SIZE, Self::Y_SIZE, slots),
            recipeBook: GuiRecipeBook::new(),
            widthTooNarrow: false,
        }
    }

    pub fn initGui(&mut self, width: i32, height: i32) {
        self.initGuiWithRecipeBook(width, height, &RecipeBook::default());
    }

    pub fn initGuiWithRecipeBook(&mut self, width: i32, height: i32, book: &RecipeBook) {
        self.container.initGui(width, height);
        self.widthTooNarrow = width < 379;
        self.recipeBook
            .init(width, height, self.widthTooNarrow, book);
        self.container.guiLeft = self.recipeBook.containerLeft(Self::X_SIZE);
    }

    pub fn rebuildRecipeBook(
        &mut self,
        book: &RecipeBook,
        inventory: &InventoryPlayer,
        craftingStacks: &[ItemStack],
        resetPage: bool,
        locale: &Locale,
    ) {
        self.recipeBook
            .rebuild(book, inventory, craftingStacks, 3, 3, resetPage, locale);
        self.container.guiLeft = self.recipeBook.containerLeft(Self::X_SIZE);
    }

    pub fn craftingSlotPositions(&self) -> Vec<(i32, i32)> {
        (0..=9)
            .filter_map(|slot| self.container.slotPosition(slot))
            .collect()
    }

    pub fn craftingBackground() -> ResourceLocation {
        ResourceLocation::parse("textures/gui/container/crafting_table.png")
    }
}

impl Default for GuiCrafting {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn append_player_slots(slots: &mut Vec<GuiSlot>, lowerSlotCount: i32) {
    for row in 0..3 {
        for column in 0..9 {
            slots.push(GuiSlot {
                slotNumber: lowerSlotCount + column + row * 9,
                xPos: 8 + column * 18,
                yPos: 84 + row * 18,
            });
        }
    }
    for column in 0..9 {
        slots.push(GuiSlot {
            slotNumber: lowerSlotCount + 27 + column,
            xPos: 8 + column * 18,
            yPos: 142,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_match_container_workbench() {
        let mut gui = GuiCrafting::new();
        gui.initGui(320, 240);
        assert_eq!(gui.container.inventorySlots.len(), 46);
        assert_eq!(gui.container.slotPosition(0), Some((196, 72)));
        assert_eq!(gui.container.slotPosition(1), Some((102, 54)));
        assert_eq!(gui.container.slotPosition(10), Some((80, 121)));
        assert_eq!(gui.container.slotPosition(37), Some((80, 179)));
    }

    #[test]
    fn narrow_recipe_book_keeps_centered_container() {
        let mut book = RecipeBook::default();
        book.setGuiOpen(true);
        let mut gui = GuiCrafting::new();
        gui.initGuiWithRecipeBook(320, 240, &book);
        assert!(gui.widthTooNarrow);
        assert_eq!(gui.container.guiLeft, 72);
    }
}
