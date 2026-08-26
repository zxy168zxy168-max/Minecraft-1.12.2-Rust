use std::path::PathBuf;

use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiButton::{GuiButton, GuiSoundCommand};
use crate::net::minecraft::client::gui::GuiScreen::GuiScreen;
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::client::resources::ResourcePackRepository::{
    ResourcePackEntry, ResourcePackRepository,
};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::vulkan::GuiDrawList::GuiDrawList;

const LIST_WIDTH: i32 = 200;
const LIST_TOP: i32 = 32;
const LIST_BOTTOM_MARGIN: i32 = 51;
const ROW_HEIGHT: i32 = 36;
const HEADER_HEIGHT: i32 = 13;
const ICON_SIZE: i32 = 32;
const RESOURCE_PACK_CONTROLS: &str = "textures/gui/resource_packs.png";
const UNKNOWN_PACK_ICON: &str = "textures/misc/unknown_pack.png";
const DEFAULT_PACK_ICON: &str = "dynamic/default_pack_icon.png";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuiScreenResourcePacksAction {
    Toggle(String),
    MoveSelected { index: usize, delta: i32 },
    OpenFolder,
    Done,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuiScreenResourcePacksInteraction {
    pub action: GuiScreenResourcePacksAction,
    pub sound: GuiSoundCommand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResourcePackPanel {
    Available,
    Selected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DrawnRowKind {
    Available(String),
    /// Index in the low-to-high priority GameSettings list.
    Selected {
        internalIndex: usize,
        name: String,
    },
    Default,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DrawnRow {
    panel: ResourcePackPanel,
    x: i32,
    y: i32,
    kind: DrawnRowKind,
}

#[derive(Debug, Clone)]
struct IncompatibleConfirmation {
    name: String,
    title: String,
    message: String,
    yes: GuiButton,
    no: GuiButton,
}

/// MCP 1.12.2 `GuiScreenResourcePacks`, `GuiResourcePackList` and
/// `ResourcePackListEntry`. The two 200-pixel `GuiListExtended` lists retain
/// vanilla's 36-pixel slots, 32-pixel icons and texture-overlay controls.
#[derive(Debug, Clone)]
pub struct GuiScreenResourcePacks {
    pub GuiScreen: GuiScreen,
    title: String,
    availableTitle: String,
    selectedTitle: String,
    folderInfo: String,
    defaultName: String,
    defaultDescription: String,
    incompatible: String,
    incompatibleOld: String,
    incompatibleNew: String,
    confirmTitle: String,
    confirmOld: String,
    confirmNew: String,
    yesLabel: String,
    noLabel: String,
    all: Vec<ResourcePackEntry>,
    available: Vec<String>,
    selected: Vec<String>,
    availableScroll: f32,
    selectedScroll: f32,
    hoveredPanel: ResourcePackPanel,
    draggingPanel: Option<ResourcePackPanel>,
    dragStartY: i32,
    dragStartScroll: f32,
    dragScrollMultiplier: f32,
    drawnRows: Vec<DrawnRow>,
    confirmation: Option<IncompatibleConfirmation>,
    changed: bool,
    open: GuiButton,
    done: GuiButton,
    folder: PathBuf,
}

impl GuiScreenResourcePacks {
    pub fn new(repository: ResourcePackRepository, selected: Vec<String>, folder: PathBuf) -> Self {
        let all = repository.getRepositoryEntriesAll().to_vec();
        let selected = selected
            .into_iter()
            .filter(|name| all.iter().any(|entry| entry.resourcePackName == *name))
            .collect::<Vec<_>>();
        let available = all
            .iter()
            .map(|entry| entry.resourcePackName.clone())
            .filter(|name| !selected.contains(name))
            .collect();
        Self {
            GuiScreen: GuiScreen::default(),
            title: "Select Resource Packs".to_owned(),
            availableTitle: "Available Resource Packs".to_owned(),
            selectedTitle: "Selected Resource Packs".to_owned(),
            folderInfo: "Place resource pack files here".to_owned(),
            defaultName: "Default".to_owned(),
            defaultDescription: "The default look and feel of Minecraft".to_owned(),
            incompatible: "Incompatible".to_owned(),
            incompatibleOld: "Made for an older version of Minecraft".to_owned(),
            incompatibleNew: "Made for a newer version of Minecraft".to_owned(),
            confirmTitle: "Incompatible resource pack".to_owned(),
            confirmOld: "This resource pack was made for an older version of Minecraft and may no longer work correctly.".to_owned(),
            confirmNew: "This resource pack was made for a newer version of Minecraft and may not work correctly.".to_owned(),
            yesLabel: "Yes".to_owned(),
            noLabel: "No".to_owned(),
            all,
            available,
            selected,
            availableScroll: 0.0,
            selectedScroll: 0.0,
            hoveredPanel: ResourcePackPanel::Available,
            draggingPanel: None,
            dragStartY: 0,
            dragStartScroll: 0.0,
            dragScrollMultiplier: 1.0,
            drawnRows: Vec::new(),
            confirmation: None,
            changed: false,
            open: GuiButton::newWithSize(2, 0, 0, 150, 20, "Open Resource Pack Folder"),
            done: GuiButton::newWithSize(1, 0, 0, 150, 20, "Done"),
            folder,
        }
    }

    pub fn initGui(&mut self, width: i32, height: i32, locale: &Locale) {
        self.GuiScreen.width = width;
        self.GuiScreen.height = height;
        self.title = translated_or(locale, "resourcePack.title", "Select Resource Packs");
        self.availableTitle = translated_or(
            locale,
            "resourcePack.available.title",
            "Available Resource Packs",
        );
        self.selectedTitle = translated_or(
            locale,
            "resourcePack.selected.title",
            "Selected Resource Packs",
        );
        self.folderInfo = translated_or(
            locale,
            "resourcePack.folderInfo",
            "Place resource pack files here",
        );
        self.defaultName = translated_or(locale, "resourcePack.defaultName", "Default");
        self.defaultDescription = translated_or(
            locale,
            "resourcePack.defaultDescription",
            "The default look and feel of Minecraft",
        );
        self.incompatible = translated_or(locale, "resourcePack.incompatible", "Incompatible");
        self.incompatibleOld = translated_or(
            locale,
            "resourcePack.incompatible.old",
            "Made for an older version of Minecraft",
        );
        self.incompatibleNew = translated_or(
            locale,
            "resourcePack.incompatible.new",
            "Made for a newer version of Minecraft",
        );
        self.confirmTitle = translated_or(
            locale,
            "resourcePack.incompatible.confirm.title",
            "Incompatible resource pack",
        );
        self.confirmOld = translated_or(locale, "resourcePack.incompatible.confirm.old", "This resource pack was made for an older version of Minecraft and may no longer work correctly.");
        self.confirmNew = translated_or(locale, "resourcePack.incompatible.confirm.new", "This resource pack was made for a newer version of Minecraft and may not work correctly.");
        self.yesLabel = translated_or(locale, "gui.yes", "Yes");
        self.noLabel = translated_or(locale, "gui.no", "No");
        self.open.x = width / 2 - 154;
        self.open.y = height - 48;
        self.open.displayString = translated_or(
            locale,
            "resourcePack.openFolder",
            "Open Resource Pack Folder",
        );
        self.done.x = width / 2 + 4;
        self.done.y = height - 48;
        self.done.displayString = translated_or(locale, "gui.done", "Done");
        self.clampScroll();
        self.layoutRows();
        if let Some(confirmation) = self.confirmation.as_mut() {
            layout_confirmation(confirmation, width, height);
        }
    }

    pub fn selected(&self) -> Vec<String> {
        self.selected.clone()
    }
    pub fn hasChanges(&self) -> bool {
        self.changed
    }
    pub fn folder(&self) -> &PathBuf {
        &self.folder
    }
    pub const fn isDraggingScrollbar(&self) -> bool {
        self.draggingPanel.is_some()
    }

    pub fn drawScreen(
        &mut self,
        draw: &mut GuiDrawList,
        font: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
    ) {
        self.draw(draw, font, mouseX, mouseY, partialTicks, false);
    }

    pub fn drawScreenInWorld(
        &mut self,
        draw: &mut GuiDrawList,
        font: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
    ) {
        self.draw(draw, font, mouseX, mouseY, partialTicks, true);
    }

    fn draw(
        &mut self,
        draw: &mut GuiDrawList,
        font: &mut FontRenderer,
        mouseX: i32,
        mouseY: i32,
        partialTicks: f32,
        _world: bool,
    ) {
        let left = self.leftPanelX();
        let right = self.rightPanelX();
        self.hoveredPanel = if mouseX >= right {
            ResourcePackPanel::Selected
        } else {
            ResourcePackPanel::Available
        };
        // GuiScreenResourcePacks#drawScreen calls drawBackground(0), not
        // drawDefaultBackground(); the dirt texture therefore replaces the world.
        self.GuiScreen.drawDefaultBackground(draw);

        let bottom = self.listBottom();
        draw_list_background(
            draw,
            left,
            LIST_TOP,
            left + LIST_WIDTH,
            bottom,
            self.availableScroll,
        );
        draw_list_background(
            draw,
            right,
            LIST_TOP,
            right + LIST_WIDTH,
            bottom,
            self.selectedScroll,
        );
        self.layoutRows();
        for row in self.drawnRows.clone() {
            self.drawEntry(draw, font, &row, mouseX, mouseY);
        }
        self.drawScrollBar(draw, ResourcePackPanel::Available);
        self.drawScrollBar(draw, ResourcePackPanel::Selected);

        self.GuiScreen.Gui.drawCenteredString(
            font,
            draw,
            &self.title,
            self.GuiScreen.width / 2,
            16,
            0x00FF_FFFF,
        );
        self.GuiScreen.Gui.drawCenteredString(
            font,
            draw,
            &format!("§n§l{}", self.availableTitle),
            left + LIST_WIDTH / 2,
            LIST_TOP + 3,
            0x00FF_FFFF,
        );
        self.GuiScreen.Gui.drawCenteredString(
            font,
            draw,
            &format!("§n§l{}", self.selectedTitle),
            right + LIST_WIDTH / 2,
            LIST_TOP + 3,
            0x00FF_FFFF,
        );
        self.open
            .drawButton(draw, font, mouseX, mouseY, partialTicks);
        self.done
            .drawButton(draw, font, mouseX, mouseY, partialTicks);
        self.GuiScreen.Gui.drawCenteredString(
            font,
            draw,
            &self.folderInfo,
            self.GuiScreen.width / 2 - 77,
            self.GuiScreen.height - 26,
            0x0080_8080,
        );

        if let Some(confirmation) = self.confirmation.as_mut() {
            draw.draw_rect(
                0,
                0,
                self.GuiScreen.width,
                self.GuiScreen.height,
                0x9000_0000_u32 as i32,
            );
            self.GuiScreen.Gui.drawCenteredString(
                font,
                draw,
                &confirmation.title,
                self.GuiScreen.width / 2,
                self.GuiScreen.height / 2 - 50,
                0x00FF_FFFF,
            );
            for (line, text) in font
                .list_formatted_string_to_width(&confirmation.message, self.GuiScreen.width - 50)
                .into_iter()
                .take(3)
                .enumerate()
            {
                self.GuiScreen.Gui.drawCenteredString(
                    font,
                    draw,
                    &text,
                    self.GuiScreen.width / 2,
                    self.GuiScreen.height / 2 - 25 + line as i32 * 10,
                    0x00FF_FFFF,
                );
            }
            confirmation
                .yes
                .drawButton(draw, font, mouseX, mouseY, partialTicks);
            confirmation
                .no
                .drawButton(draw, font, mouseX, mouseY, partialTicks);
        }
    }

    fn drawEntry(
        &self,
        draw: &mut GuiDrawList,
        font: &mut FontRenderer,
        row: &DrawnRow,
        mouseX: i32,
        mouseY: i32,
    ) {
        if row.y + ICON_SIZE < LIST_TOP || row.y > self.listBottom() {
            return;
        }
        let hovered = mouseX >= row.x
            && mouseX < row.x + LIST_WIDTH - 6
            && mouseY >= row.y
            && mouseY < row.y + ICON_SIZE;
        let (entry, selectedInternal) = match &row.kind {
            DrawnRowKind::Available(name) => (
                self.all
                    .iter()
                    .find(|entry| &entry.resourcePackName == name),
                None,
            ),
            DrawnRowKind::Selected {
                internalIndex,
                name,
            } => (
                self.all
                    .iter()
                    .find(|entry| &entry.resourcePackName == name),
                Some(*internalIndex),
            ),
            DrawnRowKind::Default => (None, None),
        };
        let packFormat = entry.map_or(3, |entry| entry.packFormat);
        if packFormat != 3 {
            draw.draw_rect(
                row.x - 1,
                row.y - 1,
                row.x + LIST_WIDTH - 9,
                row.y + ROW_HEIGHT - 3,
                -8_978_432,
            );
        }
        let icon = match &row.kind {
            DrawnRowKind::Default => ResourceLocation::parse(DEFAULT_PACK_ICON),
            _ => entry.filter(|entry| entry.iconBytes.is_some()).map_or_else(
                || ResourceLocation::parse(UNKNOWN_PACK_ICON),
                |entry| entry.iconLocation.clone(),
            ),
        };
        draw.draw_modal_rect_with_custom_sized_texture(
            icon,
            row.x as f32,
            row.y as f32,
            0.0,
            0.0,
            32.0,
            32.0,
            32.0,
            32.0,
        );

        let mut name = match &row.kind {
            DrawnRowKind::Default => self.defaultName.clone(),
            DrawnRowKind::Available(name) => name.clone(),
            DrawnRowKind::Selected { name, .. } => name.clone(),
        };
        let mut description = match &row.kind {
            DrawnRowKind::Default => self.defaultDescription.clone(),
            _ => entry.map_or_else(String::new, |entry| entry.description.clone()),
        };

        if hovered && !matches!(row.kind, DrawnRowKind::Default) {
            draw.draw_rect(
                row.x,
                row.y,
                row.x + ICON_SIZE,
                row.y + ICON_SIZE,
                0xA08B_8B8B_u32 as i32,
            );
            if packFormat < 3 {
                name = self.incompatible.clone();
                description = self.incompatibleOld.clone();
            } else if packFormat > 3 {
                name = self.incompatible.clone();
                description = self.incompatibleNew.clone();
            }
            let relX = mouseX - row.x;
            let relY = mouseY - row.y;
            let controls = ResourceLocation::parse(RESOURCE_PACK_CONTROLS);
            match row.panel {
                ResourcePackPanel::Available => {
                    draw.draw_modal_rect_with_custom_sized_texture(
                        controls,
                        row.x as f32,
                        row.y as f32,
                        0.0,
                        if relX < 32 { 32.0 } else { 0.0 },
                        32.0,
                        32.0,
                        256.0,
                        256.0,
                    );
                }
                ResourcePackPanel::Selected => {
                    let canUp =
                        selectedInternal.is_some_and(|index| index + 1 < self.selected.len());
                    let canDown = selectedInternal.is_some_and(|index| index > 0);
                    draw.draw_modal_rect_with_custom_sized_texture(
                        controls.clone(),
                        row.x as f32,
                        row.y as f32,
                        32.0,
                        if relX < 16 { 32.0 } else { 0.0 },
                        32.0,
                        32.0,
                        256.0,
                        256.0,
                    );
                    if canUp {
                        draw.draw_modal_rect_with_custom_sized_texture(
                            controls.clone(),
                            row.x as f32,
                            row.y as f32,
                            96.0,
                            if relX > 16 && relX < 32 && relY < 16 {
                                32.0
                            } else {
                                0.0
                            },
                            32.0,
                            32.0,
                            256.0,
                            256.0,
                        );
                    }
                    if canDown {
                        draw.draw_modal_rect_with_custom_sized_texture(
                            controls,
                            row.x as f32,
                            row.y as f32,
                            64.0,
                            if relX > 16 && relX < 32 && relY > 16 {
                                32.0
                            } else {
                                0.0
                            },
                            32.0,
                            32.0,
                            256.0,
                            256.0,
                        );
                    }
                }
            }
        }

        if font.get_string_width(&name) > 157 {
            name = format!(
                "{}...",
                font.trim_string_to_width(&name, 157 - font.get_string_width("..."), false)
            );
        }
        font.draw_string_with_shadow(
            draw,
            &name,
            (row.x + 34) as f32,
            (row.y + 1) as f32,
            0x00FF_FFFF,
        );
        for (line, text) in font
            .list_formatted_string_to_width(&description, 157)
            .into_iter()
            .take(2)
            .enumerate()
        {
            font.draw_string_with_shadow(
                draw,
                &text,
                (row.x + 34) as f32,
                (row.y + 12 + line as i32 * 10) as f32,
                0x0080_8080,
            );
        }
    }

    pub fn mouseClicked(
        &mut self,
        mouseX: i32,
        mouseY: i32,
        mouseButton: i32,
    ) -> Option<GuiScreenResourcePacksInteraction> {
        if mouseButton != 0 {
            return None;
        }
        if let Some(confirmation) = self.confirmation.as_ref() {
            let name = confirmation.name.clone();
            let yes = confirmation.yes.mousePressed(mouseX, mouseY);
            let no = confirmation.no.mousePressed(mouseX, mouseY);
            let sound = if yes {
                confirmation.yes.playPressSound()
            } else if no {
                confirmation.no.playPressSound()
            } else {
                return None;
            };
            if yes && !self.selected.contains(&name) {
                self.available.retain(|candidate| candidate != &name);
                // Visual top/highest priority is end of the internal list.
                self.selected.push(name.clone());
                self.changed = true;
            }
            self.confirmation = None;
            self.clampScroll();
            self.layoutRows();
            return Some(GuiScreenResourcePacksInteraction {
                action: GuiScreenResourcePacksAction::Toggle(name),
                sound,
            });
        }
        if self.open.mousePressed(mouseX, mouseY) {
            return Some(GuiScreenResourcePacksInteraction {
                action: GuiScreenResourcePacksAction::OpenFolder,
                sound: self.open.playPressSound(),
            });
        }
        if self.done.mousePressed(mouseX, mouseY) {
            return Some(GuiScreenResourcePacksInteraction {
                action: GuiScreenResourcePacksAction::Done,
                sound: self.done.playPressSound(),
            });
        }
        if self.beginScrollbarDrag(mouseX, mouseY) {
            return None;
        }

        let row = self
            .drawnRows
            .iter()
            .find(|row| {
                mouseX >= row.x
                    && mouseX <= row.x + ICON_SIZE
                    && mouseY >= row.y
                    && mouseY <= row.y + ICON_SIZE
            })?
            .clone();
        let sound = self.done.playPressSound();
        match row.kind {
            DrawnRowKind::Available(name) => {
                let entry = self
                    .all
                    .iter()
                    .find(|entry| entry.resourcePackName == name)?;
                if entry.isCompatibleWith1122() {
                    self.available.retain(|candidate| candidate != &name);
                    self.selected.push(name.clone());
                    self.changed = true;
                } else {
                    let newer = entry.packFormat > 3;
                    self.confirmation = Some(IncompatibleConfirmation {
                        name: name.clone(),
                        title: self.confirmTitle.clone(),
                        message: if newer {
                            self.confirmNew.clone()
                        } else {
                            self.confirmOld.clone()
                        },
                        yes: GuiButton::newWithSize(0, 0, 0, 150, 20, self.yesLabel.clone()),
                        no: GuiButton::newWithSize(1, 0, 0, 150, 20, self.noLabel.clone()),
                    });
                    if let Some(confirmation) = self.confirmation.as_mut() {
                        layout_confirmation(
                            confirmation,
                            self.GuiScreen.width,
                            self.GuiScreen.height,
                        );
                    }
                }
                self.clampScroll();
                self.layoutRows();
                Some(GuiScreenResourcePacksInteraction {
                    action: GuiScreenResourcePacksAction::Toggle(name),
                    sound,
                })
            }
            DrawnRowKind::Selected {
                internalIndex,
                name,
            } => {
                let relX = mouseX - row.x;
                let relY = mouseY - row.y;
                if relX < 16 {
                    self.selected.remove(internalIndex);
                    self.available.insert(0, name.clone());
                    self.changed = true;
                    self.clampScroll();
                    self.layoutRows();
                    Some(GuiScreenResourcePacksInteraction {
                        action: GuiScreenResourcePacksAction::Toggle(name),
                        sound,
                    })
                } else if relY < 16 && internalIndex + 1 < self.selected.len() {
                    self.selected.swap(internalIndex, internalIndex + 1);
                    self.changed = true;
                    self.layoutRows();
                    Some(GuiScreenResourcePacksInteraction {
                        action: GuiScreenResourcePacksAction::MoveSelected {
                            index: internalIndex,
                            delta: 1,
                        },
                        sound,
                    })
                } else if relY > 16 && internalIndex > 0 {
                    self.selected.swap(internalIndex, internalIndex - 1);
                    self.changed = true;
                    self.layoutRows();
                    Some(GuiScreenResourcePacksInteraction {
                        action: GuiScreenResourcePacksAction::MoveSelected {
                            index: internalIndex,
                            delta: -1,
                        },
                        sound,
                    })
                } else {
                    None
                }
            }
            DrawnRowKind::Default => None,
        }
    }

    /// MCP `GuiSlot#handleMouseInput`: one wheel notch is half a 36px slot.
    pub fn scroll(&mut self, lines: f32) -> bool {
        if lines == 0.0 || self.confirmation.is_some() {
            return false;
        }
        let amount = if lines > 0.0 { -18.0 } else { 18.0 };
        match self.hoveredPanel {
            ResourcePackPanel::Available => self.availableScroll += amount,
            ResourcePackPanel::Selected => self.selectedScroll += amount,
        }
        self.clampScroll();
        self.layoutRows();
        true
    }

    pub fn mouseDragged(&mut self, mouseY: i32) -> bool {
        let Some(panel) = self.draggingPanel else {
            return false;
        };
        let value =
            self.dragStartScroll + (mouseY - self.dragStartY) as f32 * self.dragScrollMultiplier;
        match panel {
            ResourcePackPanel::Available => self.availableScroll = value,
            ResourcePackPanel::Selected => self.selectedScroll = value,
        }
        self.clampScroll();
        self.layoutRows();
        true
    }

    pub fn mouseReleased(&mut self) {
        self.draggingPanel = None;
    }
    pub fn cancelConfirmation(&mut self) -> bool {
        self.confirmation.take().is_some()
    }

    fn leftPanelX(&self) -> i32 {
        self.GuiScreen.width / 2 - 4 - LIST_WIDTH
    }
    fn rightPanelX(&self) -> i32 {
        self.GuiScreen.width / 2 + 4
    }
    fn listBottom(&self) -> i32 {
        self.GuiScreen.height - LIST_BOTTOM_MARGIN
    }
    fn availableEntries(&self) -> Vec<&ResourcePackEntry> {
        self.available
            .iter()
            .filter_map(|name| {
                self.all
                    .iter()
                    .find(|entry| &entry.resourcePackName == name)
            })
            .collect()
    }
    fn entryCount(&self, panel: ResourcePackPanel) -> usize {
        match panel {
            ResourcePackPanel::Available => self.available.len(),
            ResourcePackPanel::Selected => self.selected.len() + 1,
        }
    }
    fn contentHeight(&self, panel: ResourcePackPanel) -> i32 {
        self.entryCount(panel) as i32 * ROW_HEIGHT + HEADER_HEIGHT
    }
    fn maxScroll(&self, panel: ResourcePackPanel) -> f32 {
        let viewport = self.listBottom() - LIST_TOP;
        (self.contentHeight(panel) - (viewport - 4)).max(0) as f32
    }
    fn clampScroll(&mut self) {
        self.availableScroll = self
            .availableScroll
            .clamp(0.0, self.maxScroll(ResourcePackPanel::Available));
        self.selectedScroll = self
            .selectedScroll
            .clamp(0.0, self.maxScroll(ResourcePackPanel::Selected));
    }
    fn layoutRows(&mut self) {
        self.drawnRows.clear();
        let availableFirstY = LIST_TOP + 4 - self.availableScroll as i32 + HEADER_HEIGHT;
        let available = self
            .availableEntries()
            .into_iter()
            .map(|entry| entry.resourcePackName.clone())
            .collect::<Vec<_>>();
        for (slot, name) in available.into_iter().enumerate() {
            self.drawnRows.push(DrawnRow {
                panel: ResourcePackPanel::Available,
                x: self.leftPanelX() + 2,
                y: availableFirstY + slot as i32 * ROW_HEIGHT,
                kind: DrawnRowKind::Available(name),
            });
        }
        let selectedFirstY = LIST_TOP + 4 - self.selectedScroll as i32 + HEADER_HEIGHT;
        let mut selectedVisual = self
            .selected
            .iter()
            .enumerate()
            .rev()
            .map(|(index, name)| DrawnRowKind::Selected {
                internalIndex: index,
                name: name.clone(),
            })
            .collect::<Vec<_>>();
        selectedVisual.push(DrawnRowKind::Default);
        for (slot, kind) in selectedVisual.into_iter().enumerate() {
            self.drawnRows.push(DrawnRow {
                panel: ResourcePackPanel::Selected,
                x: self.rightPanelX() + 2,
                y: selectedFirstY + slot as i32 * ROW_HEIGHT,
                kind,
            });
        }
    }
    fn scrollbarGeometry(&self, panel: ResourcePackPanel) -> Option<(i32, i32, i32, i32, f32)> {
        let maxScroll = self.maxScroll(panel);
        if maxScroll <= 0.0 {
            return None;
        }
        let top = LIST_TOP;
        let bottom = self.listBottom();
        let viewport = bottom - top;
        let content = self.contentHeight(panel).max(1);
        let thumb = (viewport * viewport / content).clamp(32, viewport - 8);
        let scroll = match panel {
            ResourcePackPanel::Available => self.availableScroll,
            ResourcePackPanel::Selected => self.selectedScroll,
        };
        let thumbY = (scroll as i32 * (viewport - thumb) / maxScroll as i32).max(0) + top;
        let x = match panel {
            ResourcePackPanel::Available => self.leftPanelX() + LIST_WIDTH - 6,
            ResourcePackPanel::Selected => self.rightPanelX() + LIST_WIDTH - 6,
        };
        Some((x, thumbY, thumb, viewport - thumb, maxScroll))
    }
    fn beginScrollbarDrag(&mut self, mouseX: i32, mouseY: i32) -> bool {
        for panel in [ResourcePackPanel::Available, ResourcePackPanel::Selected] {
            let Some((x, thumbY, thumb, travel, maxScroll)) = self.scrollbarGeometry(panel) else {
                continue;
            };
            if mouseX >= x && mouseX <= x + 6 && mouseY >= LIST_TOP && mouseY <= self.listBottom() {
                let current = match panel {
                    ResourcePackPanel::Available => self.availableScroll,
                    ResourcePackPanel::Selected => self.selectedScroll,
                };
                let adjusted = if mouseY < thumbY {
                    (current - (self.listBottom() - LIST_TOP) as f32).max(0.0)
                } else if mouseY > thumbY + thumb {
                    (current + (self.listBottom() - LIST_TOP) as f32).min(maxScroll)
                } else {
                    current
                };
                match panel {
                    ResourcePackPanel::Available => self.availableScroll = adjusted,
                    ResourcePackPanel::Selected => self.selectedScroll = adjusted,
                }
                self.draggingPanel = Some(panel);
                self.dragStartY = mouseY;
                self.dragStartScroll = adjusted;
                self.dragScrollMultiplier = if travel <= 0 {
                    1.0
                } else {
                    maxScroll / travel as f32
                };
                self.layoutRows();
                return true;
            }
        }
        false
    }
    fn drawScrollBar(&self, draw: &mut GuiDrawList, panel: ResourcePackPanel) {
        let Some((x, thumbY, thumb, _, _)) = self.scrollbarGeometry(panel) else {
            return;
        };
        draw.draw_rect(
            x,
            LIST_TOP,
            x + 6,
            self.listBottom(),
            0xFF00_0000_u32 as i32,
        );
        draw.draw_rect(x, thumbY, x + 6, thumbY + thumb, 0xFF80_8080_u32 as i32);
        draw.draw_rect(x, thumbY, x + 5, thumbY + thumb - 1, 0xFFC0_C0C0_u32 as i32);
    }
}

fn draw_list_background(
    draw: &mut GuiDrawList,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    amountScrolled: f32,
) {
    let texture = crate::net::minecraft::client::gui::Gui::OPTIONS_BACKGROUND.clone();
    let color = 0xFF20_2020;
    let mut y = top;
    while y < bottom {
        let sourceY = (y + amountScrolled as i32).rem_euclid(32);
        let height = (32 - sourceY).min(bottom - y);
        let mut x = left;
        while x < right {
            let sourceX = x.rem_euclid(32);
            let width = (32 - sourceX).min(right - x);
            let u0 = sourceX as f32 / 32.0;
            let v0 = sourceY as f32 / 32.0;
            let u1 = (sourceX + width) as f32 / 32.0;
            let v1 = (sourceY + height) as f32 / 32.0;
            draw.push_textured_quad(
                texture.clone(),
                [
                    (x as f32, (y + height) as f32, u0, v1, color),
                    ((x + width) as f32, (y + height) as f32, u1, v1, color),
                    ((x + width) as f32, y as f32, u1, v0, color),
                    (x as f32, y as f32, u0, v0, color),
                ],
            );
            x += width;
        }
        y += height;
    }
    draw.draw_gradient_rect(
        left,
        top,
        right,
        top + 4,
        0xFF00_0000_u32 as i32,
        0x0100_0000,
    );
    draw.draw_gradient_rect(
        left,
        bottom - 4,
        right,
        bottom,
        0x0100_0000,
        0xFF00_0000_u32 as i32,
    );
}

fn layout_confirmation(confirmation: &mut IncompatibleConfirmation, width: i32, height: i32) {
    confirmation.yes.x = width / 2 - 155;
    confirmation.yes.y = height / 2 + 20;
    confirmation.no.x = width / 2 + 5;
    confirmation.no.y = height / 2 + 20;
}

fn translated_or(locale: &Locale, key: &str, fallback: &str) -> String {
    let translated = locale.translate_key(key);
    if translated == key {
        fallback.to_owned()
    } else {
        translated.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::client::resources::ResourcePackRepository::ResourcePackKind;

    fn entry(name: &str, format: i32) -> ResourcePackEntry {
        ResourcePackEntry {
            resourcePackName: name.to_owned(),
            resourcePackFile: PathBuf::from(name),
            kind: ResourcePackKind::Folder,
            packFormat: format,
            description: format!("{name} description"),
            iconBytes: None,
            iconLocation: ResourceLocation::new(
                "minecraft",
                format!("resourcepackicons/{name}.png"),
            ),
        }
    }

    #[test]
    fn selected_order_is_low_to_high_and_visual_up_increases_priority() {
        let repository =
            ResourcePackRepository::fromEntriesForTest(vec![entry("low", 3), entry("high", 3)]);
        let mut screen = GuiScreenResourcePacks::new(
            repository,
            vec!["low".to_owned(), "high".to_owned()],
            PathBuf::new(),
        );
        screen.initGui(854, 480, &Locale::default());
        let low = screen
            .drawnRows
            .iter()
            .find(|row| matches!(&row.kind, DrawnRowKind::Selected { name, .. } if name == "low"))
            .unwrap()
            .clone();
        screen.mouseClicked(low.x + 24, low.y + 4, 0).unwrap();
        assert_eq!(screen.selected(), vec!["high", "low"]);
    }

    #[test]
    fn default_pack_is_immutable() {
        let repository = ResourcePackRepository::fromEntriesForTest(vec![entry("pack", 3)]);
        let mut screen =
            GuiScreenResourcePacks::new(repository, vec!["pack".to_owned()], PathBuf::new());
        screen.initGui(854, 480, &Locale::default());
        let default = screen
            .drawnRows
            .iter()
            .find(|row| row.kind == DrawnRowKind::Default)
            .unwrap();
        assert!(screen
            .mouseClicked(default.x + 2, default.y + 2, 0)
            .is_none());
    }

    #[test]
    fn wheel_scrolls_half_one_vanilla_slot() {
        let repository = ResourcePackRepository::fromEntriesForTest(
            (0..20)
                .map(|index| entry(&format!("pack-{index}"), 3))
                .collect(),
        );
        let mut screen = GuiScreenResourcePacks::new(repository, Vec::new(), PathBuf::new());
        screen.initGui(854, 240, &Locale::default());
        screen.hoveredPanel = ResourcePackPanel::Available;
        assert!(screen.scroll(-1.0));
        assert_eq!(screen.availableScroll, 18.0);
        assert!(!screen.hasChanges());
    }
}
