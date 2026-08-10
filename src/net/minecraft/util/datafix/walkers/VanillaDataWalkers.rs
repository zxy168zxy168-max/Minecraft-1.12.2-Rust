use crate::net::minecraft::nbt::NBTBase::{NBTBase, TAG_COMPOUND, TAG_LIST};
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::net::minecraft::util::datafix::FixTypes::FixTypes;
use crate::net::minecraft::util::datafix::IDataFixer::IDataFixer;
use crate::net::minecraft::util::datafix::IDataWalker::IDataWalker;

fn processItemStack(fixer: &dyn IDataFixer, mut compound: NBTTagCompound, versionIn: i32, key: &str) -> NBTTagCompound {
    if compound.hasKeyWithType(key, TAG_COMPOUND) {
        let fixed = fixer.processVersioned(FixTypes::ItemInstance, compound.getCompoundTag(key), versionIn);
        compound.setCompoundTag(key, fixed);
    }
    compound
}
fn processInventory(fixer: &dyn IDataFixer, mut compound: NBTTagCompound, versionIn: i32, key: &str) -> NBTTagCompound {
    if compound.hasKeyWithType(key, TAG_LIST) {
        let mut list=compound.getTagList(key,TAG_COMPOUND);
        for i in 0..list.tagCount() {
            let fixed=fixer.processVersioned(FixTypes::ItemInstance,list.getCompoundTagAt(i),versionIn);
            list.set(i,NBTBase::Compound(fixed));
        }
        compound.setTagList(key,list);
    }
    compound
}

/// Anonymous walker registered by `EntityPlayerMP#func_191522_a`.
pub struct PlayerRootVehicleDataWalker;
impl IDataWalker for PlayerRootVehicleDataWalker {
    fn process(&self, fixer:&dyn IDataFixer, mut compound:NBTTagCompound, versionIn:i32)->NBTTagCompound {
        if compound.hasKeyWithType("RootVehicle",TAG_COMPOUND) {
            let mut root=compound.getCompoundTag("RootVehicle");
            if root.hasKeyWithType("Entity",TAG_COMPOUND) {
                let fixed=fixer.processVersioned(FixTypes::Entity,root.getCompoundTag("Entity"),versionIn);
                root.setCompoundTag("Entity",fixed);
            }
            compound.setCompoundTag("RootVehicle",root);
        }
        compound
    }
}

/// Anonymous walker registered by `EntityPlayer#registerFixesPlayer`.
pub struct PlayerInventoryDataWalker;
impl IDataWalker for PlayerInventoryDataWalker {
    fn process(&self, fixer:&dyn IDataFixer, compound:NBTTagCompound, versionIn:i32)->NBTTagCompound {
        let mut compound=processInventory(fixer,compound,versionIn,"Inventory");
        compound=processInventory(fixer,compound,versionIn,"EnderItems");
        for key in ["ShoulderEntityLeft","ShoulderEntityRight"] {
            if compound.hasKeyWithType(key,TAG_COMPOUND) {
                let fixed=fixer.processVersioned(FixTypes::Entity,compound.getCompoundTag(key),versionIn);
                compound.setCompoundTag(key,fixed);
            }
        }
        compound
    }
}

/// Anonymous walker registered by `Template#func_191158_a`.
pub struct StructureTemplateDataWalker;
impl IDataWalker for StructureTemplateDataWalker {
    fn process(&self, fixer:&dyn IDataFixer, mut compound:NBTTagCompound, versionIn:i32)->NBTTagCompound {
        if compound.hasKeyWithType("entities",TAG_LIST) {
            let mut list=compound.getTagList("entities",TAG_COMPOUND);
            for i in 0..list.tagCount() {
                let mut entry=list.getCompoundTagAt(i);
                if entry.hasKeyWithType("nbt",TAG_COMPOUND) {
                    let fixed=fixer.processVersioned(FixTypes::Entity,entry.getCompoundTag("nbt"),versionIn);
                    entry.setCompoundTag("nbt",fixed);
                }
                list.set(i,NBTBase::Compound(entry));
            }
            compound.setTagList("entities",list);
        }
        if compound.hasKeyWithType("blocks",TAG_LIST) {
            let mut list=compound.getTagList("blocks",TAG_COMPOUND);
            for i in 0..list.tagCount() {
                let mut entry=list.getCompoundTagAt(i);
                if entry.hasKeyWithType("nbt",TAG_COMPOUND) {
                    let fixed=fixer.processVersioned(FixTypes::BlockEntity,entry.getCompoundTag("nbt"),versionIn);
                    entry.setCompoundTag("nbt",fixed);
                }
                list.set(i,NBTBase::Compound(entry));
            }
            compound.setTagList("blocks",list);
        }
        compound
    }
}

/// Second walker from `EntityVillager#registerFixesVillager`, covering trade recipes.
pub struct VillagerTradeDataWalker;
impl IDataWalker for VillagerTradeDataWalker {
    fn process(&self, fixer:&dyn IDataFixer, mut compound:NBTTagCompound, versionIn:i32)->NBTTagCompound {
        if ResourceLocation::parse(compound.getString("id")) != ResourceLocation::parse("villager") || !compound.hasKeyWithType("Offers",TAG_COMPOUND) { return compound; }
        let mut offers=compound.getCompoundTag("Offers");
        if offers.hasKeyWithType("Recipes",TAG_LIST) {
            let mut recipes=offers.getTagList("Recipes",TAG_COMPOUND);
            for i in 0..recipes.tagCount() {
                let mut recipe=recipes.getCompoundTagAt(i);
                recipe=processItemStack(fixer,recipe,versionIn,"buy");
                recipe=processItemStack(fixer,recipe,versionIn,"buyB");
                recipe=processItemStack(fixer,recipe,versionIn,"sell");
                recipes.set(i,NBTBase::Compound(recipe));
            }
            offers.setTagList("Recipes",recipes);
        }
        compound.setCompoundTag("Offers",offers);
        compound
    }
}

/// Anonymous walker from `TileEntityMobSpawner#registerFixesMobSpawner`.
pub struct MobSpawnerDataWalker;
impl IDataWalker for MobSpawnerDataWalker {
    fn process(&self, fixer:&dyn IDataFixer, mut compound:NBTTagCompound, versionIn:i32)->NBTTagCompound {
        if ResourceLocation::parse(compound.getString("id")) != ResourceLocation::parse("mob_spawner") { return compound; }
        if compound.hasKeyWithType("SpawnPotentials",TAG_LIST) {
            let mut list=compound.getTagList("SpawnPotentials",TAG_COMPOUND);
            for i in 0..list.tagCount() {
                let mut potential=list.getCompoundTagAt(i);
                let fixed=fixer.processVersioned(FixTypes::Entity,potential.getCompoundTag("Entity"),versionIn);
                potential.setCompoundTag("Entity",fixed);
                list.set(i,NBTBase::Compound(potential));
            }
            compound.setTagList("SpawnPotentials",list);
        }
        let fixed=fixer.processVersioned(FixTypes::Entity,compound.getCompoundTag("SpawnData"),versionIn);
        compound.setCompoundTag("SpawnData",fixed);
        compound
    }
}

/// Anonymous bridge from `EntityMinecartMobSpawner#registerFixesMinecartMobSpawner`.
pub struct MinecartSpawnerDataWalker;
impl IDataWalker for MinecartSpawnerDataWalker {
    fn process(&self, fixer:&dyn IDataFixer, mut compound:NBTTagCompound, versionIn:i32)->NBTTagCompound {
        let original=compound.getString("id");
        if ResourceLocation::parse(&original) != ResourceLocation::parse("spawner_minecart") { return compound; }
        compound.setString("id","minecraft:mob_spawner");
        compound=fixer.processVersioned(FixTypes::BlockEntity,compound,versionIn);
        compound.setString("id",original);
        compound
    }
}

/// Anonymous bridge from `EntityMinecartCommandBlock#registerFixesMinecartCommand`.
pub struct MinecartCommandBlockDataWalker;
impl IDataWalker for MinecartCommandBlockDataWalker {
    fn process(&self, fixer:&dyn IDataFixer, mut compound:NBTTagCompound, versionIn:i32)->NBTTagCompound {
        let original=compound.getString("id");
        // Preserve the literal MCP 1.12.2 comparison, even though it is
        // surprising: the source compares the entity compound's id against
        // TileEntityCommandBlock's registry key, not the minecart entity key.
        if ResourceLocation::parse(&original) != ResourceLocation::parse("command_block") { return compound; }
        compound.setString("id","Control");
        compound=fixer.processVersioned(FixTypes::BlockEntity,compound,versionIn);
        compound.setString("id",original);
        compound
    }
}
