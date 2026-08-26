use crate::net::minecraft::client::gui::recipebook::GuiRecipeBook::GuiRect;
use crate::net::minecraft::client::gui::recipebook::RecipeList::RecipeList;
use crate::net::minecraft::item::crafting::CraftingManager::CraftingManager;
use crate::net::minecraft::item::crafting::RecipeRegistryData::RecipeKind;
use crate::net::minecraft::stats::RecipeBook::RecipeBook;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecipeOverlayButtonState {
    pub rect: GuiRect,
    pub recipeId: i32,
    pub craftable: bool,
    pub ingredientWidth: usize,
    pub ingredientHeight: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecipeOverlayRenderState {
    pub visible: bool,
    pub left: i32,
    pub top: i32,
    pub columns: usize,
    pub rows: usize,
    pub animationTicks: f32,
    pub buttons: Vec<RecipeOverlayButtonState>,
}

/// State and geometry port of MCP 1.12.2 `GuiRecipeOverlay`.
///
/// Rendering remains in the Vulkan GUI pass, but opening, viewport clamping,
/// craftable-first ordering, button geometry, animation and click routing are
/// retained here under the original class responsibility.
#[derive(Debug, Clone, Default)]
pub struct GuiRecipeOverlay {
    visible: bool,
    left: i32,
    top: i32,
    listIndex: Option<usize>,
    buttons: Vec<RecipeOverlayButtonState>,
    animationTicks: f32,
}

impl GuiRecipeOverlay {
    pub const fn isVisible(&self) -> bool {
        self.visible
    }
    pub const fn listIndex(&self) -> Option<usize> {
        self.listIndex
    }

    #[allow(clippy::too_many_arguments)]
    pub fn open(
        &mut self,
        listIndex: usize,
        list: &RecipeList,
        buttonX: i32,
        buttonY: i32,
        panelCenterX: i32,
        panelCenterY: i32,
        buttonWidth: f32,
        book: &RecipeBook,
    ) {
        let craftable = list.recipesByCraftability(true);
        let nonCraftable = if book.isFilteringCraftable() {
            Vec::new()
        } else {
            list.recipesByCraftability(false)
        };
        let craftableCount = craftable.len();
        let total = craftableCount + nonCraftable.len();
        if total == 0 {
            self.close();
            return;
        }

        let columns = if total <= 16 { 4 } else { 5 };
        let rows = ((total as f32) / (columns as f32)).ceil() as usize;
        self.left = buttonX;
        self.top = buttonY;

        let right = (self.left + total.min(columns) as i32 * 25) as f32;
        let rightBoundary = (panelCenterX + 50) as f32;
        if right > rightBoundary {
            self.left = (self.left as f32
                - buttonWidth * (((right - rightBoundary) / buttonWidth) as i32) as f32)
                as i32;
        }

        let bottom = (self.top + rows as i32 * 25) as f32;
        let bottomBoundary = (panelCenterY + 50) as f32;
        if bottom > bottomBoundary {
            self.top = (self.top as f32
                - buttonWidth * ((bottom - bottomBoundary) / buttonWidth).ceil())
                as i32;
        }

        let top = self.top as f32;
        let topBoundary = (panelCenterY - 100) as f32;
        if top < topBoundary {
            self.top =
                (self.top as f32 - buttonWidth * ((top - topBoundary) / buttonWidth).ceil()) as i32;
        }

        self.visible = true;
        self.listIndex = Some(listIndex);
        self.buttons.clear();
        for (index, recipeId) in craftable.into_iter().chain(nonCraftable).enumerate() {
            let isCraftable = index < craftableCount;
            let Some(recipe) = CraftingManager::getRecipe(recipeId) else {
                continue;
            };
            let (ingredientWidth, ingredientHeight) = if recipe.getKind() == RecipeKind::Shaped {
                (
                    recipe.definition().width as usize,
                    recipe.definition().height as usize,
                )
            } else {
                (3, 3)
            };
            self.buttons.push(RecipeOverlayButtonState {
                rect: GuiRect {
                    x: self.left + 4 + 25 * (index as i32 % columns as i32),
                    y: self.top + 5 + 25 * (index as i32 / columns as i32),
                    width: 24,
                    height: 24,
                },
                recipeId,
                craftable: isCraftable,
                ingredientWidth,
                ingredientHeight,
            });
        }
    }

    /// MCP accepts only the primary mouse button while the overlay is open.
    pub fn click(&mut self, mouseX: i32, mouseY: i32, mouseButton: i32) -> Option<i32> {
        if !self.visible || mouseButton != 0 {
            return None;
        }
        self.buttons
            .iter()
            .find(|button| button.rect.contains(mouseX, mouseY))
            .map(|button| button.recipeId)
    }

    pub fn tick(&mut self, partialTicks: f32) {
        if self.visible {
            self.animationTicks += partialTicks;
        }
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.listIndex = None;
    }

    pub fn renderState(&self) -> RecipeOverlayRenderState {
        // MCP `GuiRecipeOverlay#func_191842_a` uses a 4/5-column placement
        // grid, but draws only `min(recipe_count, grid_columns)` background
        // columns.  Keeping four columns for a one-recipe overlay makes the
        // popup 75 pixels too wide and changes its edge clamping.
        let placementColumns = if self.buttons.len() <= 16 { 4 } else { 5 };
        let columns = self.buttons.len().min(placementColumns);
        let rows = if self.buttons.is_empty() {
            0
        } else {
            ((self.buttons.len() as f32) / (placementColumns as f32)).ceil() as usize
        };
        RecipeOverlayRenderState {
            visible: self.visible,
            left: self.left,
            top: self.top,
            columns,
            rows,
            animationTicks: self.animationTicks,
            buttons: self.buttons.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_background_uses_only_the_visible_columns() {
        let mut overlay = GuiRecipeOverlay::default();
        overlay.visible = true;
        overlay.buttons = vec![RecipeOverlayButtonState {
            rect: GuiRect {
                x: 0,
                y: 0,
                width: 24,
                height: 24,
            },
            recipeId: 11,
            craftable: true,
            ingredientWidth: 1,
            ingredientHeight: 1,
        }];
        let state = overlay.renderState();
        assert_eq!(state.columns, 1);
        assert_eq!(state.rows, 1);
    }

    #[test]
    fn overlay_switches_to_five_column_placement_after_sixteen_recipes() {
        let button = RecipeOverlayButtonState {
            rect: GuiRect {
                x: 0,
                y: 0,
                width: 24,
                height: 24,
            },
            recipeId: 11,
            craftable: true,
            ingredientWidth: 1,
            ingredientHeight: 1,
        };
        let mut overlay = GuiRecipeOverlay::default();
        overlay.visible = true;
        overlay.buttons = vec![button; 17];
        let state = overlay.renderState();
        assert_eq!(state.columns, 5);
        assert_eq!(state.rows, 4);
    }
}
