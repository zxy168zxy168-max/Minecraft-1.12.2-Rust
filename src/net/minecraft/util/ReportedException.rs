use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::net::minecraft::crash::CrashReport::CrashReport;

/// Rust equivalent of MCP `net.minecraft.util.ReportedException`.
#[derive(Debug)]
pub struct ReportedException {
    crashReport: CrashReport,
}

impl ReportedException {
    pub fn new(report: CrashReport) -> Self {
        Self {
            crashReport: report,
        }
    }

    pub fn getCrashReport(&self) -> &CrashReport {
        &self.crashReport
    }

    pub fn getCrashReportMut(&mut self) -> &mut CrashReport {
        &mut self.crashReport
    }
}

impl Display for ReportedException {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.crashReport.getDescription())
    }
}

impl Error for ReportedException {}
