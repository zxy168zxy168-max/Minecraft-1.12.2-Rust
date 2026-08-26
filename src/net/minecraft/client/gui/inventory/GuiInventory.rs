use crate::net::minecraft::client::gui::inventory::GuiContainer::{GuiContainer, GuiSlot};
use crate::net::minecraft::client::gui::recipebook::GuiRecipeBook::GuiRecipeBook;
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::entity::player::InventoryPlayer::InventoryPlayer;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::stats::RecipeBook::RecipeBook;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `GuiInventory` layout for the normal survival player container.
#[derive(Debug, Clone)]
pub struct GuiInventory {
    pub container: GuiContainer,
    pub recipeBook: GuiRecipeBook,
    pub widthTooNarrow: bool,
}

impl Default for GuiInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl GuiInventory {
    pub const X_SIZE: i32 = 176;
    pub const Y_SIZE: i32 = 166;

    pub fn new() -> Self {
        let mut slots = Vec::with_capacity(46);
        // ContainerPlayer constructor order is the network slot order.
        slots.push(GuiSlot {
            slotNumber: 0,
            xPos: 154,
            yPos: 28,
        });
        for row in 0..2 {
            for column in 0..2 {
                slots.push(GuiSlot {
                    slotNumber: 1 + column + row * 2,
                    xPos: 98 + column * 18,
                    yPos: 18 + row * 18,
                });
            }
        }
        for armor in 0..4 {
            slots.push(GuiSlot {
                slotNumber: 5 + armor,
                xPos: 8,
                yPos: 8 + armor * 18,
            });
        }
        for row in 0..3 {
            for column in 0..9 {
                slots.push(GuiSlot {
                    slotNumber: 9 + column + row * 9,
                    xPos: 8 + column * 18,
                    yPos: 84 + row * 18,
                });
            }
        }
        for column in 0..9 {
            slots.push(GuiSlot {
                slotNumber: 36 + column,
                xPos: 8 + column * 18,
                yPos: 142,
            });
        }
        slots.push(GuiSlot {
            slotNumber: 45,
            xPos: 77,
            yPos: 62,
        });
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
            .rebuild(book, inventory, craftingStacks, 2, 2, resetPage, locale);
        self.container.guiLeft = self.recipeBook.containerLeft(Self::X_SIZE);
    }

    pub fn craftingSlotPositions(&self) -> Vec<(i32, i32)> {
        (0..=4)
            .filter_map(|slot| self.container.slotPosition(slot))
            .collect()
    }

    pub fn slotAt(&self, mouseX: i32, mouseY: i32) -> Option<i32> {
        self.container.slotAt(mouseX, mouseY)
    }
    pub fn slotPosition(&self, slotNumber: i32) -> Option<(i32, i32)> {
        self.container.slotPosition(slotNumber)
    }

    pub fn inventoryBackground() -> ResourceLocation {
        ResourceLocation::parse("textures/gui/container/inventory.png")
    }
}
