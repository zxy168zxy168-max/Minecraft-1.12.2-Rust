use crate::net::minecraft::client::gui::recipebook::GhostRecipe::GhostRecipe;
use crate::net::minecraft::client::gui::recipebook::GuiRecipeOverlay::{
    GuiRecipeOverlay, RecipeOverlayRenderState,
};
use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiTextField::{
    GuiTextField, GuiTextFieldKey, GuiTextFieldModifiers, GuiTextFieldRenderState,
};
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::client::util::RecipeBookClient::RecipeBookClient;
use crate::net::minecraft::client::util::RecipeItemHelper::RecipeItemHelper;
use crate::net::minecraft::entity::player::InventoryPlayer::InventoryPlayer;
use crate::net::minecraft::item::crafting::CraftingManager::CraftingManager;
use crate::net::minecraft::item::crafting::Ingredient::Ingredient;
use crate::net::minecraft::item::crafting::RecipeRegistryData::{RecipeCategory, RecipeKind};
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::item::ItemTooltip::getTooltip;
use crate::net::minecraft::stats::RecipeBook::RecipeBook;
use std::array;

pub const RECIPE_BOOK_WIDTH: i32 = 147;
pub const RECIPE_BOOK_HEIGHT: i32 = 166;
const PAGE_SIZE: usize = 20;
const TAB_ORDER: [RecipeCategory; 5] = [
    RecipeCategory::Search,
    RecipeCategory::Tools,
    RecipeCategory::BuildingBlocks,
    RecipeCategory::Misc,
    RecipeCategory::Redstone,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuiRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl GuiRect {
    pub const fn contains(self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.width && y < self.y + self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RecipeButtonState {
    pub listIndex: usize,
    pub rect: GuiRect,
    pub recipeId: i32,
    pub craftable: bool,
    pub multiple: bool,
    pub allOutputsEqual: bool,
    /// MCP `GuiButtonRecipe.field_191778_t` scale about `(x + 8, y + 12)`.
    pub animationScale: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecipeTabState {
    pub category: RecipeCategory,
    pub rect: GuiRect,
    pub selected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeBookClick {
    None,
    Consumed,
    PlaceRecipe {
        recipeId: i32,
        placeAll: bool,
        closeBook: bool,
    },
    SettingsChanged {
        open: bool,
        filtering: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct GhostIngredientRenderState {
    pub stack: ItemStack,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecipeBookRenderState {
    pub open: bool,
    pub widthTooNarrow: bool,
    pub inventoryScreen: bool,
    pub panelLeft: i32,
    pub panelTop: i32,
    pub containerLeft: i32,
    pub toggle: GuiRect,
    pub filter: GuiRect,
    pub search: GuiRect,
    pub previous: GuiRect,
    pub next: GuiRect,
    pub filteringCraftable: bool,
    pub searchField: GuiTextFieldRenderState,
    pub currentPage: usize,
    pub pageCount: usize,
    pub tabs: Vec<RecipeTabState>,
    pub buttons: Vec<RecipeButtonState>,
    pub ghost: Vec<GhostIngredientRenderState>,
    pub overlay: RecipeOverlayRenderState,
}

/// Direct state/geometry port of MCP 1.12.2 `GuiRecipeBook` and
/// `RecipeBookPage`. Rendering is emitted by the Vulkan GUI pass, while this
/// class owns search-field focus, book settings, list filtering, pagination,
/// click routing and ghost-recipe state just as the Java class does.
#[derive(Debug, Clone)]
pub struct GuiRecipeBook {
    screenWidth: i32,
    screenHeight: i32,
    widthTooNarrow: bool,
    open: bool,
    filteringCraftable: bool,
    selectedCategory: RecipeCategory,
    searchField: GuiTextField,
    lastSearchText: String,
    page: usize,
    client: RecipeBookClient,
    visibleLists: Vec<usize>,
    ghost: GhostRecipe,
    overlay: GuiRecipeOverlay,
    /// MCP owns one `GuiButtonRecipe` instance for each of the twenty page slots.
    /// Their cycle and bounce timers persist across list reassignment.
    buttonCycleTicks: [f32; PAGE_SIZE],
    buttonBounceTicks: [f32; PAGE_SIZE],
    buttonListIndices: [Option<usize>; PAGE_SIZE],
}

impl Default for GuiRecipeBook {
    fn default() -> Self {
        Self::new()
    }
}

impl GuiRecipeBook {
    pub fn new() -> Self {
        Self {
            screenWidth: 0,
            screenHeight: 0,
            widthTooNarrow: false,
            open: false,
            filteringCraftable: false,
            selectedCategory: RecipeCategory::Search,
            searchField: createSearchField(0, 0),
            lastSearchText: String::new(),
            page: 0,
            client: RecipeBookClient::new(),
            visibleLists: Vec::new(),
            ghost: GhostRecipe::default(),
            overlay: GuiRecipeOverlay::default(),
            buttonCycleTicks: [0.0; PAGE_SIZE],
            buttonBounceTicks: [0.0; PAGE_SIZE],
            buttonListIndices: array::from_fn(|_| None),
        }
    }

    pub fn init(&mut self, width: i32, height: i32, widthTooNarrow: bool, book: &RecipeBook) {
        self.screenWidth = width;
        self.screenHeight = height;
        self.widthTooNarrow = widthTooNarrow;
        self.open = book.isGuiOpen();
        self.filteringCraftable = book.isFilteringCraftable();
        self.selectedCategory = RecipeCategory::Search;
        self.page = 0;
        self.searchField = createSearchField(self.panelLeft() + 25, self.panelTop() + 14);
        self.lastSearchText.clear();
        self.overlay.close();
        self.buttonListIndices = array::from_fn(|_| None);
        self.buttonBounceTicks = [0.0; PAGE_SIZE];
    }

    pub const fn isOpen(&self) -> bool {
        self.open
    }
    pub const fn isWidthTooNarrow(&self) -> bool {
        self.widthTooNarrow
    }
    pub const fn isFilteringCraftable(&self) -> bool {
        self.filteringCraftable
    }
    pub const fn selectedCategory(&self) -> RecipeCategory {
        self.selectedCategory
    }
    pub fn searchText(&self) -> String {
        self.searchField.getText()
    }
    pub const fn searchFocused(&self) -> bool {
        self.searchField.isFocused()
    }

    pub fn setSearchText(&mut self, text: &str) {
        self.searchField.setText(text);
    }

    pub fn panelLeft(&self) -> i32 {
        (self.screenWidth - RECIPE_BOOK_WIDTH) / 2 - if self.widthTooNarrow { 0 } else { 86 }
    }

    pub fn panelTop(&self) -> i32 {
        (self.screenHeight - RECIPE_BOOK_HEIGHT) / 2
    }

    pub fn containerLeft(&self, containerWidth: i32) -> i32 {
        if self.open && !self.widthTooNarrow {
            177 + (self.screenWidth - containerWidth - 200) / 2
        } else {
            (self.screenWidth - containerWidth) / 2
        }
    }

    pub fn toggleRect(&self, inventory: bool) -> GuiRect {
        let x = self.containerLeft(176) + if inventory { 104 } else { 5 };
        let y = self.screenHeight / 2 + if inventory { -22 } else { -49 };
        GuiRect {
            x,
            y,
            width: 20,
            height: 18,
        }
    }

    pub fn filterRect(&self) -> GuiRect {
        GuiRect {
            x: self.panelLeft() + 110,
            y: self.panelTop() + 12,
            width: 26,
            height: 16,
        }
    }

    pub fn searchRect(&self) -> GuiRect {
        GuiRect {
            x: self.panelLeft() + 25,
            y: self.panelTop() + 14,
            width: 80,
            height: 14,
        }
    }

    pub fn previousRect(&self) -> GuiRect {
        GuiRect {
            x: self.panelLeft() + 38,
            y: self.panelTop() + 137,
            width: 12,
            height: 17,
        }
    }

    pub fn nextRect(&self) -> GuiRect {
        GuiRect {
            x: self.panelLeft() + 93,
            y: self.panelTop() + 137,
            width: 12,
            height: 17,
        }
    }

    pub fn pageCount(&self) -> usize {
        (self.visibleLists.len() + PAGE_SIZE - 1) / PAGE_SIZE
    }
    pub const fn currentPage(&self) -> usize {
        self.page
    }

    pub fn tabs(&self) -> Vec<RecipeTabState> {
        let mut result = Vec::new();
        let mut row = 0;
        for category in TAB_ORDER {
            let visible = category == RecipeCategory::Search
                || self.client.listIndices(category).iter().any(|&index| {
                    self.client
                        .list(index)
                        .is_some_and(|list| list.hasUnlocked() && list.hasFitting())
                });
            if visible {
                result.push(RecipeTabState {
                    category,
                    rect: GuiRect {
                        x: self.panelLeft() - 30,
                        y: self.panelTop() + 3 + 27 * row,
                        width: 35,
                        height: 27,
                    },
                    selected: category == self.selectedCategory,
                });
                row += 1;
            }
        }
        result
    }

    pub fn rebuild(
        &mut self,
        book: &RecipeBook,
        inventory: &InventoryPlayer,
        craftingStacks: &[ItemStack],
        gridWidth: usize,
        gridHeight: usize,
        resetPage: bool,
        locale: &Locale,
    ) {
        self.open = book.isGuiOpen();
        self.filteringCraftable = book.isFilteringCraftable();
        let mut helper = RecipeItemHelper::default();
        for stack in &inventory.mainInventory {
            helper.accountStack(stack);
        }
        for stack in craftingStacks {
            helper.accountStack(stack);
        }

        let indices = self.client.listIndices(self.selectedCategory).to_vec();
        for &index in &indices {
            if let Some(list) = self.client.listMut(index) {
                list.updateUnlocked(book);
                list.updateCraftable(&helper, gridWidth, gridHeight, book);
            }
        }

        let search = self.searchField.getText().trim().to_lowercase();
        self.visibleLists = indices
            .into_iter()
            .filter(|&index| {
                self.client.list(index).is_some_and(|list| {
                    if !list.hasUnlocked()
                        || !list.hasFitting()
                        || (self.filteringCraftable && !list.hasCraftable())
                    {
                        return false;
                    }
                    if search.is_empty() {
                        return true;
                    }
                    list.recipes().iter().any(|&id| {
                        CraftingManager::getRecipe(id).is_some_and(|recipe| {
                            let registry = recipe.getRegistryName().to_lowercase();
                            if search.contains(':') && registry.contains(&search) {
                                return true;
                            }
                            getTooltip(&recipe.getRecipeOutput(), locale, false)
                                .into_iter()
                                .map(|line| stripFormatting(&line).trim().to_lowercase())
                                .any(|line| !line.is_empty() && line.contains(&search))
                        })
                    })
                })
            })
            .collect();
        if resetPage || self.page >= self.pageCount().max(1) {
            self.page = 0;
        }
        self.lastSearchText = search;

        // `RecipeBookPage#func_194198_d` assigns the current page's lists to
        // twenty persistent GuiButtonRecipe instances. `func_193928_a` starts
        // the 15-tick bounce whenever any currently visible recipe is new.
        let start = self.page * PAGE_SIZE;
        for button in 0..PAGE_SIZE {
            let assigned = self.visibleLists.get(start + button).copied();
            self.buttonListIndices[button] = assigned;
            let containsNew = assigned
                .and_then(|index| self.client.list(index))
                .is_some_and(|list| {
                    list.visibleRecipes(self.filteringCraftable)
                        .into_iter()
                        .any(|recipeId| {
                            CraftingManager::getRecipe(recipeId)
                                .is_some_and(|recipe| book.isNew(recipe))
                        })
                });
            if containsNew {
                self.buttonBounceTicks[button] = 15.0;
            }
        }
    }

    pub fn recipeButtons(&self, _book: &RecipeBook) -> Vec<RecipeButtonState> {
        let start = self.page * PAGE_SIZE;
        self.visibleLists
            .iter()
            .skip(start)
            .take(PAGE_SIZE)
            .enumerate()
            .filter_map(|(button, &listIndex)| {
                let list = self.client.list(listIndex)?;
                let mut recipes = list.recipesByCraftability(true);
                if !self.filteringCraftable {
                    recipes.extend(list.recipesByCraftability(false));
                }
                if recipes.is_empty() {
                    return None;
                }
                let recipeId = recipes
                    [((self.buttonCycleTicks[button] / 30.0).floor() as usize) % recipes.len()];
                let bounce = self.buttonBounceTicks[button];
                let animationScale = if bounce > 0.0 {
                    1.0 + 0.1 * (bounce / 15.0 * std::f32::consts::PI).sin()
                } else {
                    1.0
                };
                Some(RecipeButtonState {
                    listIndex,
                    rect: GuiRect {
                        x: self.panelLeft() + 11 + 25 * (button as i32 % 5),
                        y: self.panelTop() + 31 + 25 * (button as i32 / 5),
                        width: 25,
                        height: 25,
                    },
                    recipeId,
                    craftable: list.isCraftable(recipeId),
                    multiple: list.visibleRecipes(self.filteringCraftable).len() > 1,
                    allOutputsEqual: list.allOutputsEqual(),
                    animationScale,
                })
            })
            .collect()
    }

    pub fn tick(&mut self, partialTicks: f32, controlHeld: bool) {
        if self.open {
            self.searchField.updateCursorCounter();
            for button in 0..PAGE_SIZE {
                if self.buttonListIndices[button].is_none() {
                    continue;
                }
                if !controlHeld {
                    self.buttonCycleTicks[button] += partialTicks;
                }
                if self.buttonBounceTicks[button] > 0.0 {
                    self.buttonBounceTicks[button] =
                        (self.buttonBounceTicks[button] - partialTicks).max(0.0);
                }
            }
        }
        self.ghost.tick(partialTicks, controlHeld);
        self.overlay.tick(partialTicks);
    }

    pub fn click(
        &mut self,
        inventoryScreen: bool,
        x: i32,
        y: i32,
        button: i32,
        shiftHeld: bool,
        book: &mut RecipeBook,
        font: &FontRenderer,
    ) -> RecipeBookClick {
        // The normal GuiButtonImage is not drawn in the narrow/open branch.
        if !(self.open && self.widthTooNarrow) && self.toggleRect(inventoryScreen).contains(x, y) {
            self.open = !self.open;
            book.setGuiOpen(self.open);
            if !self.open {
                self.ghost.clear();
            }
            return RecipeBookClick::SettingsChanged {
                open: self.open,
                filtering: self.filteringCraftable,
            };
        }
        if !self.open {
            return RecipeBookClick::None;
        }

        // `RecipeBookPage` gives an active GuiRecipeOverlay first refusal for
        // every mouse button. A primary-button hit selects that exact recipe;
        // every other click closes the overlay and is still consumed.
        if self.overlay.isVisible() {
            let listIndex = self.overlay.listIndex();
            let selected = self.overlay.click(x, y, button);
            if let Some(recipeId) = selected {
                if listIndex
                    .and_then(|index| self.client.list(index))
                    .is_some_and(|list| {
                        !list.isCraftable(recipeId) && self.ghost.recipeId() == Some(recipeId)
                    })
                {
                    return RecipeBookClick::None;
                }
                self.ghost.clear();
                let closeBook = self.widthTooNarrow;
                if closeBook {
                    self.open = false;
                    book.setGuiOpen(false);
                    self.overlay.close();
                }
                return RecipeBookClick::PlaceRecipe {
                    recipeId,
                    placeAll: shiftHeld,
                    closeBook,
                };
            }
            self.overlay.close();
            return RecipeBookClick::Consumed;
        }

        if button == 0 && self.nextRect().contains(x, y) && self.page + 1 < self.pageCount() {
            self.page += 1;
            self.overlay.close();
            return RecipeBookClick::Consumed;
        }
        if button == 0 && self.previousRect().contains(x, y) && self.page > 0 {
            self.page -= 1;
            self.overlay.close();
            return RecipeBookClick::Consumed;
        }
        for state in self.recipeButtons(book) {
            if !state.rect.contains(x, y) {
                continue;
            }
            if button == 0 {
                if !state.craftable && self.ghost.recipeId() == Some(state.recipeId) {
                    return RecipeBookClick::None;
                }
                self.ghost.clear();
                let closeBook = self.widthTooNarrow;
                if closeBook {
                    self.open = false;
                    book.setGuiOpen(false);
                    self.overlay.close();
                }
                return RecipeBookClick::PlaceRecipe {
                    recipeId: state.recipeId,
                    placeAll: shiftHeld,
                    closeBook,
                };
            }
            if state.multiple {
                if let Some(list) = self.client.list(state.listIndex) {
                    self.overlay.open(
                        state.listIndex,
                        list,
                        state.rect.x,
                        state.rect.y,
                        self.panelLeft() + RECIPE_BOOK_WIDTH / 2,
                        self.panelTop() + 13 + RECIPE_BOOK_HEIGHT / 2,
                        state.rect.width as f32,
                        book,
                    );
                }
            }
            return RecipeBookClick::Consumed;
        }
        if button != 0 {
            return RecipeBookClick::None;
        }

        if self.searchField.mouseClicked(x, y, button, font) {
            return RecipeBookClick::Consumed;
        }
        if self.filterRect().contains(x, y) {
            self.overlay.close();
            self.filteringCraftable = !self.filteringCraftable;
            book.setFilteringCraftable(self.filteringCraftable);
            return RecipeBookClick::SettingsChanged {
                open: self.open,
                filtering: self.filteringCraftable,
            };
        }
        for tab in self.tabs() {
            if tab.rect.contains(x, y) {
                if self.selectedCategory != tab.category {
                    self.selectedCategory = tab.category;
                    self.page = 0;
                    self.overlay.close();
                }
                return RecipeBookClick::Consumed;
            }
        }
        RecipeBookClick::None
    }

    pub fn focusSearchFromChatKey(&mut self) -> bool {
        if !self.open || self.searchField.isFocused() {
            return false;
        }
        self.searchField.setFocused(true);
        true
    }

    pub fn keyPressed(
        &mut self,
        key: GuiTextFieldKey,
        modifiers: GuiTextFieldModifiers,
        font: &FontRenderer,
    ) -> (bool, bool) {
        if !self.open {
            return (false, false);
        }
        let before = self.searchField.getText();
        let handled = self.searchField.keyPressed(key, modifiers, font);
        let textChanged = handled && before != self.searchField.getText();
        (handled, textChanged)
    }

    pub fn selectAllSearch(&mut self, font: &FontRenderer) -> bool {
        if !self.open || !self.searchField.isFocused() {
            return false;
        }
        self.searchField.selectAll(font);
        true
    }

    pub fn typedText(&mut self, text: &str, font: &FontRenderer) -> bool {
        if !self.open || !self.searchField.isFocused() {
            return false;
        }
        let before = self.searchField.getText();
        let handled = self.searchField.writeText(text, Some(font));
        handled && before != self.searchField.getText()
    }

    pub fn closeOnEscape(&mut self, book: &mut RecipeBook) -> Option<RecipeBookClick> {
        if !self.open || !self.widthTooNarrow {
            return None;
        }
        self.open = false;
        self.ghost.clear();
        self.overlay.close();
        book.setGuiOpen(false);
        Some(RecipeBookClick::SettingsChanged {
            open: false,
            filtering: self.filteringCraftable,
        })
    }

    pub fn placeGhostRecipe(
        &mut self,
        recipeId: i32,
        slotPositions: &[(i32, i32)],
        gridWidth: usize,
        gridHeight: usize,
    ) {
        let Some(recipe) = CraftingManager::getRecipe(recipeId) else {
            return;
        };
        if slotPositions.is_empty() {
            return;
        }
        self.ghost.clear();
        self.ghost.setRecipe(recipeId);
        self.ghost.addIngredient(
            Ingredient::fromStacks(vec![recipe.getRecipeOutput()]),
            slotPositions[0].0,
            slotPositions[0].1,
        );
        let width = if recipe.getKind() == RecipeKind::Shaped {
            recipe.definition().width as usize
        } else {
            gridWidth
        };
        let ingredients = recipe.getIngredients();
        let mut iterator = ingredients.into_iter();
        let mut slot = 1_usize;
        for _row in 0..gridHeight {
            for _column in 0..width {
                let Some(ingredient) = iterator.next() else {
                    return;
                };
                if !ingredient.getMatchingStacks().is_empty() {
                    if let Some(&(x, y)) = slotPositions.get(slot) {
                        self.ghost.addIngredient(ingredient, x, y);
                    }
                }
                slot += 1;
            }
            if width < gridWidth {
                slot += gridWidth - width;
            }
        }
    }

    /// Returns recipe IDs that vanilla would acknowledge through
    /// `EntityPlayerSP#func_193103_a` when the current page first displays a
    /// recipe-list button. Only the twenty lists assigned to the active page
    /// participate, matching `RecipeBookPage#func_194198_d`.
    pub fn newlyDisplayedRecipeIds(&self, book: &RecipeBook) -> Vec<i32> {
        if !self.open {
            return Vec::new();
        }
        let mut result = Vec::new();
        let start = self.page * PAGE_SIZE;
        for &listIndex in self.visibleLists.iter().skip(start).take(PAGE_SIZE) {
            let Some(list) = self.client.list(listIndex) else {
                continue;
            };
            let visible = list.visibleRecipes(self.filteringCraftable);
            let containsNew = visible.iter().any(|&recipeId| {
                CraftingManager::getRecipe(recipeId).is_some_and(|recipe| book.isNew(recipe))
            });
            if !containsNew {
                continue;
            }
            for recipeId in visible {
                let isNew =
                    CraftingManager::getRecipe(recipeId).is_some_and(|recipe| book.isNew(recipe));
                if isNew && !result.contains(&recipeId) {
                    result.push(recipeId);
                }
            }
        }
        result
    }

    pub fn renderState(
        &self,
        inventoryScreen: bool,
        book: &RecipeBook,
        containerWidth: i32,
        font: &FontRenderer,
    ) -> RecipeBookRenderState {
        let ghost = self
            .ghost
            .ingredients()
            .iter()
            .enumerate()
            .map(|(index, ingredient)| GhostIngredientRenderState {
                stack: self.ghost.displayedStack(index),
                x: ingredient.x,
                y: ingredient.y,
            })
            .collect();
        RecipeBookRenderState {
            open: self.open,
            widthTooNarrow: self.widthTooNarrow,
            inventoryScreen,
            panelLeft: self.panelLeft(),
            panelTop: self.panelTop(),
            containerLeft: self.containerLeft(containerWidth),
            toggle: self.toggleRect(inventoryScreen),
            filter: self.filterRect(),
            search: self.searchRect(),
            previous: self.previousRect(),
            next: self.nextRect(),
            filteringCraftable: self.filteringCraftable,
            searchField: self.searchField.buildRenderState(font),
            currentPage: self.page,
            pageCount: self.pageCount(),
            tabs: self.tabs(),
            buttons: self.recipeButtons(book),
            ghost,
            overlay: self.overlay.renderState(),
        }
    }

    pub fn ghost(&self) -> &GhostRecipe {
        &self.ghost
    }
    pub fn clearGhost(&mut self) {
        self.ghost.clear();
    }

    pub fn isPointOutside(
        &self,
        mouseX: i32,
        mouseY: i32,
        guiLeft: i32,
        guiTop: i32,
        xSize: i32,
        ySize: i32,
    ) -> bool {
        if !self.open {
            return true;
        }
        let outside = mouseX < guiLeft
            || mouseY < guiTop
            || mouseX >= guiLeft + xSize
            || mouseY >= guiTop + ySize;
        let between = guiLeft - 147 < mouseX
            && mouseX < guiLeft
            && guiTop < mouseY
            && mouseY < guiTop + ySize;
        let selectedTab = self
            .tabs()
            .into_iter()
            .find(|tab| tab.selected)
            .is_some_and(|tab| tab.rect.contains(mouseX, mouseY));
        outside && !between && !selectedTab
    }
}

fn createSearchField(x: i32, y: i32) -> GuiTextField {
    let mut field = GuiTextField::new(0, x, y, 80, 14);
    field.setMaxStringLength(50);
    field.setEnableBackgroundDrawing(false);
    field.setVisible(true);
    field.setTextColor(16_777_215);
    field
}

fn stripFormatting(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut skip = false;
    for character in value.chars() {
        if skip {
            skip = false;
            continue;
        }
        if character == '§' {
            skip = true;
        } else {
            output.push(character);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_panel_and_button_geometry() {
        let mut gui = GuiRecipeBook::new();
        let book = RecipeBook::default();
        gui.init(400, 240, false, &book);
        assert_eq!(gui.panelLeft(), 40);
        assert_eq!(gui.panelTop(), 37);
        assert_eq!(
            gui.filterRect(),
            GuiRect {
                x: 150,
                y: 49,
                width: 26,
                height: 16
            }
        );
        assert_eq!(gui.containerLeft(176), 112);
    }

    #[test]
    fn narrow_open_hides_toggle_and_closes_after_recipe_selection() {
        let mut gui = GuiRecipeBook::new();
        let mut book = RecipeBook::default();
        book.setGuiOpen(true);
        gui.init(320, 240, true, &book);
        assert!(gui.isOpen());
        assert!(gui.closeOnEscape(&mut book).is_some());
        assert!(!book.isGuiOpen());
    }
}
