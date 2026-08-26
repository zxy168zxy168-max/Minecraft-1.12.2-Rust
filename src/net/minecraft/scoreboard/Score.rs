#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Score {
    objectiveName: String,
    scorePlayerName: String,
    scorePoints: i32,
}

impl Score {
    pub fn new(
        objectiveName: impl Into<String>,
        scorePlayerName: impl Into<String>,
        scorePoints: i32,
    ) -> Self {
        Self {
            objectiveName: objectiveName.into(),
            scorePlayerName: scorePlayerName.into(),
            scorePoints,
        }
    }
    pub fn getObjectiveName(&self) -> &str {
        &self.objectiveName
    }
    pub fn getPlayerName(&self) -> &str {
        &self.scorePlayerName
    }
    pub const fn getScorePoints(&self) -> i32 {
        self.scorePoints
    }
    pub fn setScorePoints(&mut self, value: i32) {
        self.scorePoints = value;
    }
}
