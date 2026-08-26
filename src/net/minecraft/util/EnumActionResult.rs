/// Direct Rust equivalent of Minecraft 1.12.2 `EnumActionResult`.
///
/// The three values are intentionally not collapsed into a boolean: `PASS`
/// allows the next hand/item branch to run, while `FAIL` prevents the current
/// block-use path from being treated as successful.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnumActionResult {
    Success,
    Pass,
    Fail,
}

impl EnumActionResult {
    pub const fn isSuccess(self) -> bool {
        matches!(self, Self::Success)
    }
    pub const fn isPass(self) -> bool {
        matches!(self, Self::Pass)
    }
    pub const fn isFail(self) -> bool {
        matches!(self, Self::Fail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_three_mcp_results_remain_distinct() {
        assert!(EnumActionResult::Success.isSuccess());
        assert!(EnumActionResult::Pass.isPass());
        assert!(EnumActionResult::Fail.isFail());
    }
}
