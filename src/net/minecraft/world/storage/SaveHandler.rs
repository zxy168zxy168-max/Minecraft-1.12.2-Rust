use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::net::minecraft::nbt::CompressedStreamTools;
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::world::storage::WorldInfo::WorldInfo;
use crate::net::minecraft::entity::player::EntityPlayerMP::EntityPlayerMP;
use crate::net::minecraft::util::datafix::DataFixer::DataFixer;
use crate::net::minecraft::util::datafix::DataFixesManager::DataFixesManager;
use crate::net::minecraft::util::datafix::FixTypes::FixTypes;

/// Initial MCP 1.12.2 `SaveHandler` storage responsibilities needed by
/// single-player world creation and discovery.
#[derive(Debug, Clone)]
pub struct SaveHandler {
    worldDirectory: PathBuf,
    playersDirectory: PathBuf,
    mapDataDir: PathBuf,
    initializationTime: i64,
    dataFixer: DataFixer,
}

impl SaveHandler {
    pub fn new(savesDirectory: impl AsRef<Path>, saveName: &str, storePlayerdata: bool) -> io::Result<Self> {
        let worldDirectory = savesDirectory.as_ref().join(saveName);
        fs::create_dir_all(&worldDirectory)?;
        let playersDirectory = worldDirectory.join("playerdata");
        let mapDataDir = worldDirectory.join("data");
        fs::create_dir_all(&mapDataDir)?;
        if storePlayerdata { fs::create_dir_all(&playersDirectory)?; }
        let initializationTime = current_time_millis();
        let handler = Self { worldDirectory, playersDirectory, mapDataDir, initializationTime, dataFixer: DataFixesManager::createFixer() };
        handler.setSessionLock()?;
        Ok(handler)
    }

    /// MCP `SaveHandler#setSessionLock`: an 8-byte big-endian timestamp.
    fn setSessionLock(&self) -> io::Result<()> {
        let mut file = File::create(self.worldDirectory.join("session.lock"))?;
        file.write_i64::<BigEndian>(self.initializationTime)?;
        file.flush()
    }

    /// MCP `SaveHandler#checkSessionLock`.
    pub fn checkSessionLock(&self) -> io::Result<()> {
        let mut file = File::open(self.worldDirectory.join("session.lock"))?;
        let stored = file.read_i64::<BigEndian>()?;
        if stored != self.initializationTime {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "The save is being accessed from another location, aborting",
            ));
        }
        Ok(())
    }

    pub fn loadWorldInfo(&self) -> io::Result<Option<WorldInfo>> {
        for name in ["level.dat", "level.dat_old"] {
            let path = self.worldDirectory.join(name);
            if !path.is_file() { continue; }
            let root = CompressedStreamTools::readCompressed(File::open(path)?)?;
            let fixed = self.dataFixer.process(FixTypes::Level, root.getCompoundTag("Data"));
            return Ok(Some(WorldInfo::fromNBT(&fixed)));
        }
        Ok(None)
    }

    pub fn saveWorldInfo(&self, info: &WorldInfo) -> io::Result<()> { self.saveWorldInfoWithPlayer(info, None) }

    /// MCP `SaveHandler#saveWorldInfoWithPlayer` level.dat_new -> old -> dat
    /// rotation, including IntegratedPlayerList host-player NBT when supplied.
    pub fn saveWorldInfoWithPlayer(&self, info: &WorldInfo, player: Option<&NBTTagCompound>) -> io::Result<()> {
        let mut root = NBTTagCompound::new();
        root.setCompoundTag("Data", info.cloneNBTCompoundWithPlayer(player));
        let newFile = self.worldDirectory.join("level.dat_new");
        let oldFile = self.worldDirectory.join("level.dat_old");
        let levelFile = self.worldDirectory.join("level.dat");
        CompressedStreamTools::writeCompressed(&root, File::create(&newFile)?)?;
        if oldFile.exists() { fs::remove_file(&oldFile)?; }
        if levelFile.exists() { fs::rename(&levelFile, &oldFile)?; }
        if levelFile.exists() { fs::remove_file(&levelFile)?; }
        fs::rename(&newFile, &levelFile)?;
        if newFile.exists() { fs::remove_file(newFile)?; }
        Ok(())
    }

    /// MCP `SaveHandler#writePlayerData`: compressed temporary file followed
    /// by replacement of `<uuid>.dat`.
    pub fn writePlayerData(&self, player: &EntityPlayerMP) -> io::Result<()> {
        fs::create_dir_all(&self.playersDirectory)?;
        let uuid=player.getGameProfile().getId().ok_or_else(||io::Error::new(io::ErrorKind::InvalidInput,"player profile has no UUID"))?;
        let temp=self.playersDirectory.join(format!("{uuid}.dat.tmp"));
        let target=self.playersDirectory.join(format!("{uuid}.dat"));
        CompressedStreamTools::writeCompressed(&player.writeToNBT(),File::create(&temp)?)?;
        if target.exists(){fs::remove_file(&target)?;} fs::rename(&temp,&target)?; Ok(())
    }

    /// MCP `SaveHandler#readPlayerData`: PLAYER DataFix runs before the
    /// EntityPlayerMP-owned subset is interpreted.
    pub fn readPlayerData(&self, player: &mut EntityPlayerMP) -> io::Result<Option<NBTTagCompound>> {
        let uuid=player.getGameProfile().getId().ok_or_else(||io::Error::new(io::ErrorKind::InvalidInput,"player profile has no UUID"))?;
        let path=self.playersDirectory.join(format!("{uuid}.dat")); if !path.is_file(){return Ok(None);}
        let raw=CompressedStreamTools::readCompressed(File::open(path)?)?;
        let fixed=self.dataFixer.process(FixTypes::Player,raw); player.readFromNBT(fixed.clone()); Ok(Some(fixed))
    }

    pub fn getWorldDirectory(&self) -> &Path { &self.worldDirectory }
    pub fn getPlayersDirectory(&self) -> &Path { &self.playersDirectory }
    pub fn getMapDataDir(&self) -> &Path { &self.mapDataDir }
}

fn current_time_millis() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as i64
}

#[cfg(test)]
mod player_data_tests {
    use super::*;
    use uuid::Uuid;
    use crate::com::mojang::authlib::GameProfile::GameProfile;
    use crate::net::minecraft::item::ItemStack::ItemStack;
    use crate::net::minecraft::world::GameType::GameType;
    use crate::net::minecraft::world::WorldSettings::WorldSettings;
    use crate::net::minecraft::world::WorldType::WorldType;
    use crate::net::minecraft::world::WorldServer::WorldServer;
    use crate::net::minecraft::world::chunk::storage::AnvilSaveHandler::AnvilSaveHandler;

    #[test]
    fn playerdata_round_trip_preserves_position_inventory_and_unknown_tags() {
        let root=std::env::temp_dir().join(format!("mc1122-playerdata-roundtrip-{}",std::process::id()));
        let _=std::fs::remove_dir_all(&root);
        let settings=WorldSettings::new(7,GameType::Creative,true,false,WorldType::Flat);
        let handler=AnvilSaveHandler::new(&root,"World",true).unwrap();
        let info=WorldInfo::new(&settings,"World");
        let mut world=WorldServer::new(handler,info,0).init().unwrap();
        world.initialize(&settings).unwrap();
        let id=Uuid::parse_str("12345678-1234-5678-9234-567812345678").unwrap();
        let profile=GameProfile::new(Some(id),"Player");
        let mut original=crate::net::minecraft::entity::player::EntityPlayerMP::EntityPlayerMP::new(&mut world,profile.clone(),GameType::Creative).unwrap();
        original.setPlayerLocation(12.5,70.0,-8.25,33.0,-12.0);
        original.inventory.currentItem=4;
        original.inventory.mainInventory[4]=ItemStack{itemId:57,count:23,itemDamage:0,tagCompound:None};
        let mut opaque=NBTTagCompound::new(); opaque.setString("FutureOwner","preserve-me"); original.readFromNBT({let mut tag=original.writeToNBT();tag.setCompoundTag("OpaqueFuture",opaque);tag});
        world.saveHandler().base().writePlayerData(&original).unwrap();

        let mut loaded=crate::net::minecraft::entity::player::EntityPlayerMP::EntityPlayerMP::new(&mut world,profile,GameType::Creative).unwrap();
        let fixed=world.saveHandler().base().readPlayerData(&mut loaded).unwrap().unwrap();
        assert_eq!((loaded.entity.posX,loaded.entity.posY,loaded.entity.posZ),(12.5,70.0,-8.25));
        assert_eq!((loaded.entity.rotationYaw,loaded.entity.rotationPitch),(33.0,-12.0));
        assert_eq!(loaded.inventory.currentItem,4);
        assert_eq!(loaded.inventory.mainInventory[4].itemId,57);
        assert_eq!(loaded.inventory.mainInventory[4].count,23);
        assert_eq!(fixed.getCompoundTag("OpaqueFuture").getString("FutureOwner"),"preserve-me");
        assert_eq!(loaded.writeToNBT().getCompoundTag("OpaqueFuture").getString("FutureOwner"),"preserve-me");
        drop(world); let _=std::fs::remove_dir_all(root);
    }
}
