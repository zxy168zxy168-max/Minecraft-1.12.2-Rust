use std::collections::HashMap;
use std::sync::Arc;

use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::FixTypes::FixTypes;
use crate::net::minecraft::util::datafix::IDataFixer::IDataFixer;
use crate::net::minecraft::util::datafix::IDataWalker::IDataWalker;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;

/// MCP 1.12.2 `DataFixer` generic engine.
///
/// This is the real ordered fixer/walker mechanism. Individual vanilla
/// migrations are registered by their owning classes as those classes are
/// ported; an unregistered fix type is intentionally unchanged rather than
/// guessed. 1.12.2 data (`DataVersion >= 1343`) bypasses the engine exactly as
/// the source does.
#[derive(Clone)]
pub struct DataFixer {
    walkerMap: HashMap<FixTypes, Vec<Arc<dyn IDataWalker>>>,
    fixMap: HashMap<FixTypes, Vec<Arc<dyn IFixableData>>>,
    version: i32,
}

impl std::fmt::Debug for DataFixer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataFixer")
            .field("version", &self.version)
            .field("walkerTypes", &self.walkerMap.len())
            .field("fixTypes", &self.fixMap.len())
            .finish()
    }
}

impl DataFixer {
    pub fn new(versionIn: i32) -> Self {
        Self { walkerMap: HashMap::new(), fixMap: HashMap::new(), version: versionIn }
    }

    /// MCP overload `process(IFixType, NBTTagCompound)`.
    pub fn process(&self, fixType: FixTypes, compound: NBTTagCompound) -> NBTTagCompound {
        let version = if compound.hasKeyWithType("DataVersion", 99) { compound.getInteger("DataVersion") } else { -1 };
        if version >= 1343 { compound } else { self.processVersioned(fixType, compound, version) }
    }

    pub fn registerWalker(&mut self, fixType: FixTypes, walker: Arc<dyn IDataWalker>) {
        self.walkerMap.entry(fixType).or_default().push(walker);
    }

    pub fn registerFix(&mut self, fixType: FixTypes, fixable: Arc<dyn IFixableData>) {
        let fixVersion = fixable.getFixVersion();
        if fixVersion > self.version {
            log::warn!("Ignored fix registered for version: {} as the DataVersion of the game is: {}", fixVersion, self.version);
            return;
        }
        let list = self.fixMap.entry(fixType).or_default();
        let index = list.iter().position(|existing| existing.getFixVersion() > fixVersion).unwrap_or(list.len());
        list.insert(index, fixable);
    }

    pub const fn version(&self) -> i32 { self.version }
}

impl IDataFixer for DataFixer {
    fn processVersioned(&self, fixType: FixTypes, mut compound: NBTTagCompound, versionIn: i32) -> NBTTagCompound {
        if versionIn < self.version {
            if let Some(fixes) = self.fixMap.get(&fixType) {
                for fix in fixes {
                    if fix.getFixVersion() > versionIn {
                        compound = fix.fixTagCompound(compound);
                    }
                }
            }
            if let Some(walkers) = self.walkerMap.get(&fixType) {
                for walker in walkers {
                    compound = walker.process(self, compound, versionIn);
                }
            }
        }
        compound
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fix(i32, &'static str);
    impl IFixableData for Fix {
        fn getFixVersion(&self) -> i32 { self.0 }
        fn fixTagCompound(&self, mut compound: NBTTagCompound) -> NBTTagCompound {
            let current = compound.getString("order");
            compound.setString("order", format!("{}{}", current, self.1));
            compound
        }
    }
    #[test]
    fn fixes_are_version_sorted_and_only_newer_fixes_run() {
        let mut fixer = DataFixer::new(1343);
        fixer.registerFix(FixTypes::Chunk, Arc::new(Fix(100, "a")));
        fixer.registerFix(FixTypes::Chunk, Arc::new(Fix(50, "b")));
        fixer.registerFix(FixTypes::Chunk, Arc::new(Fix(150, "c")));
        let out = fixer.processVersioned(FixTypes::Chunk, NBTTagCompound::new(), 75);
        assert_eq!(out.getString("order"), "ac");
    }
}
