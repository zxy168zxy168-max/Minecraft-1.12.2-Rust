use std::backtrace::Backtrace;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::CrashReportCategory::CrashReportCategory;

const WITTY_COMMENTS: &[&str] = &[
    "Who set us up the TNT?",
    "Everything's going to plan. No, really, that was supposed to happen.",
    "Uh... Did I do that?",
    "Oops.",
    "Why did you do that?",
    "I feel sad now :(",
    "My bad.",
    "I'm sorry, Dave.",
    "I let you down. Sorry :(",
    "On the bright side, I bought you a teddy bear!",
    "Daisy, daisy...",
    "Oh - I know what I did wrong!",
    "Hey, that tickles! Hehehe!",
    "I blame Dinnerbone.",
    "You should try our sister game, Minceraft!",
    "Don't be sad. I'll do better next time, I promise!",
    "Don't be sad, have a hug! <3",
    "I just don't know what went wrong :(",
    "Shall we play a game?",
    "Quite honestly, I wouldn't worry myself about that.",
    "I bet Cylons wouldn't have this problem.",
    "Sorry :(",
    "Surprise! Haha. Well, this is awkward.",
    "Would you like a cupcake?",
    "Hi. I'm Minecraft, and I'm a crashaholic.",
    "Ooh. Shiny.",
    "This doesn't make any sense!",
    "Why is it breaking :(",
    "Don't do that.",
    "Ouch. That hurt :(",
    "You're mean.",
    "This is a token for 1 free hug. Redeem at your nearest Mojangsta: [~~HUG~~]",
    "There are four lights!",
    "But it works on my machine.",
];

/// Rust equivalent of MCP `net.minecraft.crash.CrashReport`.
#[derive(Debug)]
pub struct CrashReport {
    description: String,
    cause: String,
    backtrace: Backtrace,
    systemDetailsCategory: CrashReportCategory,
    crashReportSections: Vec<CrashReportCategory>,
    crashReportFile: Option<PathBuf>,
    createdAt: SystemTime,
}

impl CrashReport {
    pub fn new(descriptionIn: impl Into<String>, causeThrowable: impl Into<String>) -> Self {
        let mut report = Self {
            description: descriptionIn.into(),
            cause: causeThrowable.into(),
            backtrace: Backtrace::force_capture(),
            systemDetailsCategory: CrashReportCategory::new("System Details"),
            crashReportSections: Vec::new(),
            crashReportFile: None,
            createdAt: SystemTime::now(),
        };
        report.populateEnvironment();
        report
    }

    fn populateEnvironment(&mut self) {
        self.systemDetailsCategory
            .addCrashSection("Minecraft Version", "1.12.2");
        self.systemDetailsCategory.addCrashSection(
            "Operating System",
            format!("{} ({})", std::env::consts::OS, std::env::consts::ARCH),
        );
        self.systemDetailsCategory
            .addCrashSection("Rust Client Version", env!("CARGO_PKG_VERSION"));
        self.systemDetailsCategory
            .addCrashSection("Rust Package", env!("CARGO_PKG_NAME"));
        self.systemDetailsCategory
            .addCrashSection("Thread", std::thread::current().name().unwrap_or("main"));
    }

    pub fn getDescription(&self) -> &str {
        &self.description
    }

    pub fn getCrashCause(&self) -> &str {
        &self.cause
    }

    pub fn getCategory(&mut self) -> &mut CrashReportCategory {
        &mut self.systemDetailsCategory
    }

    pub fn makeCategory(&mut self, name: impl Into<String>) -> &mut CrashReportCategory {
        self.crashReportSections
            .push(CrashReportCategory::new(name));
        self.crashReportSections
            .last_mut()
            .expect("category was just appended")
    }

    pub fn getFile(&self) -> Option<&Path> {
        self.crashReportFile.as_deref()
    }

    pub fn getCauseStackTraceOrString(&self) -> String {
        format!("{}\n{:?}", self.cause, self.backtrace)
    }

    pub fn getCompleteReport(&self) -> String {
        let mut builder = String::new();
        builder.push_str("---- Minecraft Crash Report ----\n");
        builder.push_str("// ");
        builder.push_str(Self::getWittyComment(self.createdAt));
        builder.push_str("\n\nTime: ");
        builder.push_str(&formatSystemTime(self.createdAt));
        builder.push_str("\nDescription: ");
        builder.push_str(&self.description);
        builder.push_str("\n\n");
        builder.push_str(&self.getCauseStackTraceOrString());
        builder.push_str("\n\nA detailed walkthrough of the error, its code path and all known details is as follows:\n");
        builder.push_str(&"-".repeat(87));
        builder.push_str("\n\n");
        for category in &self.crashReportSections {
            category.appendToStringBuilder(&mut builder);
            builder.push_str("\n\n");
        }
        self.systemDetailsCategory
            .appendToStringBuilder(&mut builder);
        builder.push('\n');
        builder
    }

    pub fn saveToFile(&mut self, toFile: impl AsRef<Path>) -> bool {
        if self.crashReportFile.is_some() {
            return false;
        }
        let toFile = toFile.as_ref();
        if let Some(parent) = toFile.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                log::error!(
                    "Could not create crash report directory {}: {error}",
                    parent.display()
                );
                return false;
            }
        }
        match fs::write(toFile, self.getCompleteReport()) {
            Ok(()) => {
                self.crashReportFile = Some(toFile.to_path_buf());
                true
            }
            Err(error) => {
                log::error!(
                    "Could not save crash report to {}: {error}",
                    toFile.display()
                );
                false
            }
        }
    }

    pub fn defaultClientReportPath(&self, gameDir: impl AsRef<Path>) -> PathBuf {
        gameDir.as_ref().join("crash-reports").join(format!(
            "crash-{}-client.txt",
            formatFileTime(self.createdAt)
        ))
    }

    fn getWittyComment(time: SystemTime) -> &'static str {
        let nanos = time
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as usize;
        WITTY_COMMENTS
            .get(nanos % WITTY_COMMENTS.len())
            .copied()
            .unwrap_or("Witty comment unavailable :(")
    }
}

fn formatSystemTime(time: SystemTime) -> String {
    let seconds = time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let (year, month, day, hour, minute, second) = splitUtc(seconds);
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02} UTC")
}

fn formatFileTime(time: SystemTime) -> String {
    let seconds = time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let (year, month, day, hour, minute, second) = splitUtc(seconds);
    format!("{year:04}-{month:02}-{day:02}_{hour:02}.{minute:02}.{second:02}")
}

fn splitUtc(seconds: u64) -> (i32, u32, u32, u32, u32, u32) {
    let days = (seconds / 86_400) as i64;
    let daySeconds = seconds % 86_400;
    let hour = (daySeconds / 3_600) as u32;
    let minute = ((daySeconds % 3_600) / 60) as u32;
    let second = (daySeconds % 60) as u32;
    let (year, month, day) = civilFromDays(days);
    (year, month, day, hour, minute, second)
}

// Howard Hinnant's civil-from-days algorithm, with day zero at Unix epoch.
fn civilFromDays(unixDays: i64) -> (i32, u32, u32) {
    let z = unixDays + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let dayOfEra = z - era * 146_097;
    let yearOfEra = (dayOfEra - dayOfEra / 1_460 + dayOfEra / 36_524 - dayOfEra / 146_096) / 365;
    let mut year = yearOfEra + era * 400;
    let dayOfYear = dayOfEra - (365 * yearOfEra + yearOfEra / 4 - yearOfEra / 100);
    let monthPrime = (5 * dayOfYear + 2) / 153;
    let day = dayOfYear - (153 * monthPrime + 2) / 5 + 1;
    let month = monthPrime + if monthPrime < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };
    (year as i32, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::{civilFromDays, CrashReport};

    #[test]
    fn unix_epoch_date_is_correct() {
        assert_eq!(civilFromDays(0), (1970, 1, 1));
        assert_eq!(civilFromDays(20_454), (2026, 1, 1));
    }

    #[test]
    fn complete_report_has_mcp_sections() {
        let report = CrashReport::new("Manually triggered debug crash", "debug crash");
        let text = report.getCompleteReport();
        assert!(text.starts_with("---- Minecraft Crash Report ----"));
        assert!(text.contains("Description: Manually triggered debug crash"));
        assert!(text.contains("-- System Details --"));
    }
}
