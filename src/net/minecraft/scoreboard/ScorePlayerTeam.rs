use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScorePlayerTeam {
    registeredName: String,
    teamName: String,
    colorPrefix: String,
    colorSuffix: String,
    friendlyFlags: i32,
    nameTagVisibility: String,
    collisionRule: String,
    chatColor: i32,
    membership: HashSet<String>,
}

impl ScorePlayerTeam {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            registeredName: name.clone(),
            teamName: name,
            colorPrefix: String::new(),
            colorSuffix: String::new(),
            friendlyFlags: 0,
            nameTagVisibility: "always".to_owned(),
            collisionRule: "always".to_owned(),
            chatColor: -1,
            membership: HashSet::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        displayName: impl Into<String>,
        prefix: impl Into<String>,
        suffix: impl Into<String>,
        friendlyFlags: i32,
        nameTagVisibility: impl Into<String>,
        collisionRule: impl Into<String>,
        color: i32,
    ) {
        self.teamName = displayName.into();
        self.colorPrefix = prefix.into();
        self.colorSuffix = suffix.into();
        self.friendlyFlags = friendlyFlags;
        self.nameTagVisibility = nameTagVisibility.into();
        self.collisionRule = collisionRule.into();
        self.chatColor = color;
    }

    pub fn addPlayer(&mut self, player: impl Into<String>) {
        self.membership.insert(player.into());
    }
    pub fn removePlayer(&mut self, player: &str) {
        self.membership.remove(player);
    }
    pub fn getRegisteredName(&self) -> &str {
        &self.registeredName
    }
    pub fn getTeamName(&self) -> &str {
        &self.teamName
    }
    pub fn getColorPrefix(&self) -> &str {
        &self.colorPrefix
    }
    pub fn getColorSuffix(&self) -> &str {
        &self.colorSuffix
    }
    pub const fn getFriendlyFlags(&self) -> i32 {
        self.friendlyFlags
    }
    /// MCP `Team#getSeeFriendlyInvisiblesEnabled`: bit 1 of the friendly
    /// flags sent by `SPacketTeams`.
    pub const fn getSeeFriendlyInvisiblesEnabled(&self) -> bool {
        self.friendlyFlags & 2 != 0
    }
    /// Raw MCP `Team.EnumVisible.internalName` retained from the packet.
    pub fn getNameTagVisibility(&self) -> &str {
        &self.nameTagVisibility
    }
    pub const fn getChatFormatColorIndex(&self) -> i32 {
        self.chatColor
    }
    pub fn getMembershipCollection(&self) -> &HashSet<String> {
        &self.membership
    }

    pub fn isSameTeam(&self, other: &ScorePlayerTeam) -> bool {
        self.registeredName == other.registeredName
    }

    pub fn formatPlayerName(team: Option<&ScorePlayerTeam>, playerName: &str) -> String {
        team.map_or_else(
            || playerName.to_owned(),
            |team| format!("{}{}{}", team.colorPrefix, playerName, team.colorSuffix),
        )
    }
}
