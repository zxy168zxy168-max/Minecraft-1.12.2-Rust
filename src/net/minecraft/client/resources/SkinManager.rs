use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::thread;

use crate::com::mojang::authlib::minecraft::MinecraftProfileTexture::{
    MinecraftProfileTexture, TextureType,
};
use crate::com::mojang::authlib::minecraft::MinecraftSessionService::MinecraftSessionService;
use crate::com::mojang::authlib::GameProfile::GameProfile;
use crate::net::minecraft::client::network::NetworkPlayerInfo::{
    NetworkPlayerInfo, PlayerTextureState,
};
use crate::net::minecraft::client::renderer::ImageBufferDownload::ImageBufferDownload;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::vulkan::NativeImage::NativeImage;

#[derive(Debug)]
enum SkinDownloadTask {
    Download {
        cacheDir: PathBuf,
        textureType: TextureType,
        location: ResourceLocation,
        profileTexture: MinecraftProfileTexture,
    },
    ResolveProfile {
        key: String,
        profile: GameProfile,
        requireSecure: bool,
    },
    Stop,
}

#[derive(Debug)]
struct SkinDownloadResult {
    textureType: TextureType,
    location: ResourceLocation,
    profileTexture: MinecraftProfileTexture,
    image: Result<NativeImage, String>,
}

#[derive(Debug)]
struct ProfileCompletionResult {
    key: String,
    profile: Result<GameProfile, String>,
}

#[derive(Debug)]
enum SkinWorkerResult {
    Texture(SkinDownloadResult),
    Profile(ProfileCompletionResult),
}

#[derive(Debug)]
struct SkinCallback {
    state: Arc<RwLock<PlayerTextureState>>,
    textureType: TextureType,
}

#[derive(Debug)]
pub struct DownloadedPlayerTexture {
    pub textureType: TextureType,
    pub location: ResourceLocation,
    pub image: NativeImage,
}

#[derive(Debug)]
pub struct CompletedPlayerProfile {
    pub key: String,
    pub profile: GameProfile,
}

#[derive(Debug, Default)]
pub struct SkinManagerUpdates {
    pub textures: Vec<DownloadedPlayerTexture>,
    pub profiles: Vec<CompletedPlayerProfile>,
}

/// MCP 1.12.2 `SkinManager` responsibility adapted to the Vulkan texture
/// backend: signed profile-property extraction remains in the session service,
/// while at most two workers perform cache I/O and HTTP image downloads.
pub struct SkinManager {
    skinCacheDir: PathBuf,
    sessionService: MinecraftSessionService,
    taskSender: mpsc::Sender<SkinDownloadTask>,
    resultReceiver: mpsc::Receiver<SkinWorkerResult>,
    workers: Vec<thread::JoinHandle<()>>,
    pending: HashMap<ResourceLocation, Vec<SkinCallback>>,
    pendingProfiles: HashSet<String>,
    failedProfiles: HashSet<String>,
    loaded: HashMap<ResourceLocation, MinecraftProfileTexture>,
}

impl SkinManager {
    pub fn new(skinCacheDir: PathBuf, sessionService: MinecraftSessionService) -> Self {
        let (taskSender, taskReceiver) = mpsc::channel::<SkinDownloadTask>();
        let (resultSender, resultReceiver) = mpsc::channel::<SkinWorkerResult>();
        let sharedReceiver = Arc::new(Mutex::new(taskReceiver));
        let mut workers = Vec::with_capacity(2);
        for index in 0..2 {
            let receiver = Arc::clone(&sharedReceiver);
            let sender = resultSender.clone();
            let workerSessionService = sessionService.clone();
            workers.push(
                thread::Builder::new()
                    .name(format!("Texture Downloader #{index}"))
                    .spawn(move || loop {
                        let task = receiver
                            .lock()
                            .unwrap_or_else(|poison| poison.into_inner())
                            .recv();
                        match task {
                            Ok(SkinDownloadTask::Download {
                                cacheDir,
                                textureType,
                                location,
                                profileTexture,
                            }) => {
                                let image =
                                    download_texture(&cacheDir, &profileTexture, textureType)
                                        .map_err(|error| error.to_string());
                                if sender
                                    .send(SkinWorkerResult::Texture(SkinDownloadResult {
                                        textureType,
                                        location,
                                        profileTexture,
                                        image,
                                    }))
                                    .is_err()
                                {
                                    break;
                                }
                            }
                            Ok(SkinDownloadTask::ResolveProfile {
                                key,
                                profile,
                                requireSecure,
                            }) => {
                                let profile = workerSessionService
                                    .completeProfile(&profile, requireSecure)
                                    .map_err(|error| error.to_string());
                                if sender
                                    .send(SkinWorkerResult::Profile(ProfileCompletionResult {
                                        key,
                                        profile,
                                    }))
                                    .is_err()
                                {
                                    break;
                                }
                            }
                            Ok(SkinDownloadTask::Stop) | Err(_) => break,
                        }
                    })
                    .expect("failed spawning SkinManager texture worker"),
            );
        }
        Self {
            skinCacheDir,
            sessionService,
            taskSender,
            resultReceiver,
            workers,
            pending: HashMap::new(),
            pendingProfiles: HashSet::new(),
            failedProfiles: HashSet::new(),
            loaded: HashMap::new(),
        }
    }

    /// `NetworkPlayerInfo#loadPlayerTextures` +
    /// `SkinManager#loadProfileTextures`. The one-shot flag is stored in the
    /// player-info object, so every cloned tab-list snapshot observes the same
    /// callback state.
    pub fn requestProfileTextures(&mut self, info: &NetworkPlayerInfo, requireSecure: bool) {
        if !info.beginPlayerTexturesLoad() {
            return;
        }
        let textures = match self
            .sessionService
            .getTextures(info.getGameProfile(), requireSecure)
        {
            Ok(textures) => textures,
            Err(error) => {
                log::warn!(
                    "failed loading secure textures for {}: {error}",
                    info.getGameProfile().getName(),
                );
                return;
            }
        };
        // MCP 1.12.2 `SkinManager#loadProfileTextures` forwards every
        // profile texture returned by the session service.
        // `NetworkPlayerInfo#loadPlayerTextures` handles SKIN, CAPE and ELYTRA.
        for textureType in [TextureType::Skin, TextureType::Cape, TextureType::Elytra] {
            if let Some(profileTexture) = textures.get(&textureType) {
                self.requestTexture(info.textureState(), textureType, profileTexture.clone());
            }
        }
    }

    /// Queue MCP `TileEntitySkull#updateGameprofile` work for legacy
    /// name-only owner tags. The shared two-thread executor matches
    /// SkinManager's bounded downloader pool and keeps network I/O off the
    /// renderer thread.
    pub fn requestProfileCompletion(
        &mut self,
        key: String,
        profile: GameProfile,
        requireSecure: bool,
    ) {
        if profile.getName().trim().is_empty()
            || (profile.isComplete()
                && profile
                    .getProperties()
                    .iter()
                    .any(|property| property.getName() == "textures"))
            || self.failedProfiles.contains(&key)
            || !self.pendingProfiles.insert(key.clone())
        {
            return;
        }
        if let Err(error) = self.taskSender.send(SkinDownloadTask::ResolveProfile {
            key: key.clone(),
            profile,
            requireSecure,
        }) {
            self.pendingProfiles.remove(&key);
            log::warn!("failed queueing skull profile completion {key}: {error}");
        }
    }

    pub fn invalidateProfileCompletion(&mut self, key: &str) {
        self.pendingProfiles.remove(key);
        self.failedProfiles.remove(key);
    }

    fn requestTexture(
        &mut self,
        state: Arc<RwLock<PlayerTextureState>>,
        textureType: TextureType,
        profileTexture: MinecraftProfileTexture,
    ) {
        let location =
            ResourceLocation::new("minecraft", format!("skins/{}", profileTexture.getHash()));
        if let Some(loadedTexture) = self.loaded.get(&location) {
            NetworkPlayerInfo::applyPlayerTexture(&state, textureType, location, loadedTexture);
            return;
        }
        if let Some(callbacks) = self.pending.get_mut(&location) {
            callbacks.push(SkinCallback { state, textureType });
            return;
        }
        self.pending
            .insert(location.clone(), vec![SkinCallback { state, textureType }]);
        if let Err(error) = self.taskSender.send(SkinDownloadTask::Download {
            cacheDir: self.skinCacheDir.clone(),
            textureType,
            location: location.clone(),
            profileTexture,
        }) {
            self.pending.remove(&location);
            log::warn!("failed queueing player texture {location}: {error}");
        }
    }

    /// Main-thread equivalent of `Minecraft#addScheduledTask` callbacks.
    pub fn drainCompleted(&mut self) -> SkinManagerUpdates {
        let mut completed = SkinManagerUpdates::default();
        while let Ok(result) = self.resultReceiver.try_recv() {
            match result {
                SkinWorkerResult::Texture(result) => {
                    let callbacks = self.pending.remove(&result.location).unwrap_or_default();
                    match result.image {
                        Ok(image) => {
                            for callback in callbacks {
                                NetworkPlayerInfo::applyPlayerTexture(
                                    &callback.state,
                                    callback.textureType,
                                    result.location.clone(),
                                    &result.profileTexture,
                                );
                            }
                            self.loaded
                                .insert(result.location.clone(), result.profileTexture);
                            completed.textures.push(DownloadedPlayerTexture {
                                textureType: result.textureType,
                                location: result.location,
                                image,
                            });
                        }
                        Err(error) => {
                            log::warn!(
                                "failed downloading player texture {}: {error}",
                                result.location
                            );
                        }
                    }
                }
                SkinWorkerResult::Profile(result) => {
                    self.pendingProfiles.remove(&result.key);
                    match result.profile {
                        Ok(profile) => {
                            self.failedProfiles.remove(&result.key);
                            completed.profiles.push(CompletedPlayerProfile {
                                key: result.key,
                                profile,
                            });
                        }
                        Err(error) => {
                            self.failedProfiles.insert(result.key.clone());
                            log::warn!(
                                "failed completing skull player profile {}: {error}",
                                result.key,
                            );
                        }
                    }
                }
            }
        }
        completed
    }
}

impl Drop for SkinManager {
    fn drop(&mut self) {
        for _ in &self.workers {
            let _ = self.taskSender.send(SkinDownloadTask::Stop);
        }
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

fn download_texture(
    cacheDir: &Path,
    profileTexture: &MinecraftProfileTexture,
    textureType: TextureType,
) -> Result<NativeImage, String> {
    let hash = profileTexture.getHash();
    let directory = cacheDir.join(if hash.len() > 2 { &hash[..2] } else { "xx" });
    let file = directory.join(&hash);
    let cached = fs::read(&file).ok();
    let mut bytes = cached.unwrap_or_default();
    let mut image = NativeImage::decode_png(&bytes).ok();
    if image.is_none() {
        let response = ureq::get(profileTexture.getUrl())
            .call()
            .map_err(|error| error.to_string())?;
        bytes.clear();
        response
            .into_reader()
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        let decoded = NativeImage::decode_png(&bytes).map_err(|error| error.to_string())?;
        fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
        let temporary = file.with_extension("tmp");
        fs::write(&temporary, &bytes).map_err(|error| error.to_string())?;
        fs::rename(&temporary, &file)
            .or_else(|_| {
                fs::remove_file(&file).ok();
                fs::rename(&temporary, &file)
            })
            .map_err(|error| error.to_string())?;
        image = Some(decoded);
    }
    let image = image.ok_or_else(|| "decoded cache or downloaded PNG was absent".to_owned())?;
    if textureType == TextureType::Skin {
        ImageBufferDownload::parseUserSkin(image).map_err(|error| error.to_string())
    } else {
        Ok(image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn skin_manager_shared_callback_types_remain_thread_safe() {
        assert_send_sync::<PlayerTextureState>();
        assert_send_sync::<NetworkPlayerInfo>();
    }
}
