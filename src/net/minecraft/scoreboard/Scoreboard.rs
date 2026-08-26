use std::cmp::Ordering;
use std::collections::HashMap;

use crate::net::minecraft::scoreboard::IScoreCriteria::EnumRenderType;
use crate::net::minecraft::scoreboard::Score::Score;
use crate::net::minecraft::scoreboard::ScoreObjective::ScoreObjective;
use crate::net::minecraft::scoreboard::ScorePlayerTeam::ScorePlayerTeam;

#[derive(Debug, Clone)]
pub struct Scoreboard {
    scoreObjectives: HashMap<String, ScoreObjective>,
    objectiveDisplaySlots: [Option<String>; 19],
    entitiesScoreObjectives: HashMap<String, HashMap<String, i32>>,
    teams: HashMap<String, ScorePlayerTeam>,
    teamMemberships: HashMap<String, String>,
}

impl Default for Scoreboard {
    fn default() -> Self {
        Self {
            scoreObjectives: HashMap::new(),
            objectiveDisplaySlots: std::array::from_fn(|_| None),
            entitiesScoreObjectives: HashMap::new(),
            teams: HashMap::new(),
            teamMemberships: HashMap::new(),
        }
    }
}

impl Scoreboard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn addScoreObjective(
        &mut self,
        name: impl Into<String>,
        displayName: impl Into<String>,
        renderType: EnumRenderType,
    ) {
        let name = name.into();
        self.scoreObjectives.insert(
            name.clone(),
            ScoreObjective::new(name, displayName, renderType),
        );
    }

    pub fn updateScoreObjective(
        &mut self,
        name: &str,
        displayName: impl Into<String>,
        renderType: EnumRenderType,
    ) {
        if let Some(objective) = self.scoreObjectives.get_mut(name) {
            objective.setDisplayName(displayName);
            objective.setRenderType(renderType);
        }
    }

    pub fn removeObjective(&mut self, name: &str) {
        self.scoreObjectives.remove(name);
        for slot in &mut self.objectiveDisplaySlots {
            if slot.as_deref() == Some(name) {
                *slot = None;
            }
        }
        for scores in self.entitiesScoreObjectives.values_mut() {
            scores.remove(name);
        }
        self.entitiesScoreObjectives
            .retain(|_, scores| !scores.is_empty());
    }

    pub fn getObjective(&self, name: &str) -> Option<&ScoreObjective> {
        self.scoreObjectives.get(name)
    }

    pub fn setObjectiveInDisplaySlot(&mut self, slot: i32, objectiveName: impl Into<String>) {
        if !(0..self.objectiveDisplaySlots.len() as i32).contains(&slot) {
            return;
        }
        let target = &mut self.objectiveDisplaySlots[slot as usize];
        let name = objectiveName.into();
        *target = (!name.is_empty()).then_some(name);
    }

    pub fn getObjectiveInDisplaySlot(&self, slot: i32) -> Option<&ScoreObjective> {
        if !(0..self.objectiveDisplaySlots.len() as i32).contains(&slot) {
            return None;
        }
        self.objectiveDisplaySlots[slot as usize]
            .as_deref()
            .and_then(|name| self.scoreObjectives.get(name))
    }

    pub fn setScore(
        &mut self,
        playerName: impl Into<String>,
        objectiveName: impl Into<String>,
        value: i32,
    ) {
        let playerName = playerName.into();
        let objectiveName = objectiveName.into();
        self.entitiesScoreObjectives
            .entry(playerName)
            .or_default()
            .insert(objectiveName, value);
    }

    pub fn getScorePoints(&self, playerName: &str, objectiveName: &str) -> i32 {
        self.entitiesScoreObjectives
            .get(playerName)
            .and_then(|scores| scores.get(objectiveName))
            .copied()
            .unwrap_or(0)
    }

    pub fn removeScore(&mut self, playerName: &str, objectiveName: Option<&str>) {
        match objectiveName {
            Some(objectiveName) => {
                let removePlayer = self
                    .entitiesScoreObjectives
                    .get_mut(playerName)
                    .is_some_and(|scores| {
                        scores.remove(objectiveName);
                        scores.is_empty()
                    });
                if removePlayer {
                    self.entitiesScoreObjectives.remove(playerName);
                }
            }
            None => {
                self.entitiesScoreObjectives.remove(playerName);
            }
        }
    }

    pub fn getSortedScores(&self, objective: &ScoreObjective) -> Vec<Score> {
        let mut scores = self
            .entitiesScoreObjectives
            .iter()
            .filter_map(|(player, values)| {
                values
                    .get(objective.getName())
                    .map(|value| Score::new(objective.getName(), player, *value))
            })
            .collect::<Vec<_>>();
        scores.sort_by(
            |left, right| match left.getScorePoints().cmp(&right.getScorePoints()) {
                Ordering::Equal => right
                    .getPlayerName()
                    .to_ascii_lowercase()
                    .cmp(&left.getPlayerName().to_ascii_lowercase()),
                ordering => ordering,
            },
        );
        scores
    }

    pub fn createTeam(&mut self, name: impl Into<String>) -> &mut ScorePlayerTeam {
        let name = name.into();
        self.teams
            .entry(name.clone())
            .or_insert_with(|| ScorePlayerTeam::new(name))
    }

    pub fn removeTeam(&mut self, name: &str) {
        self.teams.remove(name);
        self.teamMemberships.retain(|_, team| team != name);
    }

    pub fn addPlayerToTeam(&mut self, playerName: impl Into<String>, teamName: &str) {
        let playerName = playerName.into();
        if let Some(previous) = self
            .teamMemberships
            .insert(playerName.clone(), teamName.to_owned())
        {
            if let Some(team) = self.teams.get_mut(&previous) {
                team.removePlayer(&playerName);
            }
        }
        self.createTeam(teamName).addPlayer(playerName);
    }

    pub fn removePlayerFromTeam(&mut self, playerName: &str, teamName: &str) {
        if self
            .teamMemberships
            .get(playerName)
            .is_some_and(|value| value == teamName)
        {
            self.teamMemberships.remove(playerName);
        }
        if let Some(team) = self.teams.get_mut(teamName) {
            team.removePlayer(playerName);
        }
    }

    pub fn getPlayersTeam(&self, playerName: &str) -> Option<&ScorePlayerTeam> {
        self.teamMemberships
            .get(playerName)
            .and_then(|name| self.teams.get(name))
    }

    pub fn getTeam(&self, name: &str) -> Option<&ScorePlayerTeam> {
        self.teams.get(name)
    }
    pub fn getTeamMut(&mut self, name: &str) -> Option<&mut ScorePlayerTeam> {
        self.teams.get_mut(name)
    }

    pub fn getSidebarObjective(&self, localPlayerName: &str) -> Option<&ScoreObjective> {
        let teamObjective = self
            .getPlayersTeam(localPlayerName)
            .map(ScorePlayerTeam::getChatFormatColorIndex)
            .filter(|color| *color >= 0)
            .and_then(|color| self.getObjectiveInDisplaySlot(3 + color));
        teamObjective.or_else(|| self.getObjectiveInDisplaySlot(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_order_matches_mcp_comparator() {
        let mut board = Scoreboard::new();
        board.addScoreObjective("obj", "Title", EnumRenderType::Integer);
        board.setScore("Bob", "obj", 5);
        board.setScore("Alice", "obj", 5);
        board.setScore("Low", "obj", 1);
        let scores = board.getSortedScores(board.getObjective("obj").unwrap());
        assert_eq!(
            scores.iter().map(Score::getPlayerName).collect::<Vec<_>>(),
            ["Low", "Bob", "Alice"]
        );
    }
}
