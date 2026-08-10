use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::net::minecraft::world::biome::Biome::Biome;

#[derive(Debug, Clone)]
pub struct BiomeCache { inner: Arc<Mutex<State>> }
#[derive(Debug)] struct State { lastCleanupTime:i64, cacheMap:HashMap<i64,Block> }
#[derive(Debug, Clone)] struct Block { biomes:Vec<Biome>, xPosition:i32, zPosition:i32, lastAccessTime:i64 }

impl BiomeCache {
 pub fn new()->Self{Self{inner:Arc::new(Mutex::new(State{lastCleanupTime:0,cacheMap:HashMap::with_capacity(4096)}))}}
 fn key(x:i32,z:i32)->i64{((x as u32 as u64)|((z as u32 as u64)<<32))as i64}
 fn now()->i64{SystemTime::now().duration_since(UNIX_EPOCH).map(|d|d.as_millis() as i64).unwrap_or(0)}
 pub fn getBiome<F>(&self,x:i32,z:i32,defaultValue:Biome,mut populate:F)->Biome where F:FnMut(i32,i32)->Vec<Biome>{let cx=x>>4;let cz=z>>4;let key=Self::key(cx,cz);let now=Self::now();let mut state=self.inner.lock().unwrap();if !state.cacheMap.contains_key(&key){let biomes=populate(cx<<4,cz<<4);state.cacheMap.insert(key,Block{biomes,xPosition:cx,zPosition:cz,lastAccessTime:now});}let block=state.cacheMap.get_mut(&key).unwrap();block.lastAccessTime=now;block.biomes.get(((x&15)|((z&15)<<4))as usize).copied().unwrap_or(defaultValue)}
 pub fn getCachedBiomes<F>(&self,x:i32,z:i32,mut populate:F)->Vec<Biome> where F:FnMut(i32,i32)->Vec<Biome>{let cx=x>>4;let cz=z>>4;let key=Self::key(cx,cz);let now=Self::now();let mut state=self.inner.lock().unwrap();if !state.cacheMap.contains_key(&key){let biomes=populate(cx<<4,cz<<4);state.cacheMap.insert(key,Block{biomes,xPosition:cx,zPosition:cz,lastAccessTime:now});}let block=state.cacheMap.get_mut(&key).unwrap();block.lastAccessTime=now;block.biomes.clone()}
 pub fn cleanupCache(&self){let now=Self::now();let mut state=self.inner.lock().unwrap();let delta=now-state.lastCleanupTime;if delta>7500||delta<0{state.lastCleanupTime=now;state.cacheMap.retain(|_,b|{let age=now-b.lastAccessTime;!(age>30000||age<0)});}}
 pub fn len(&self)->usize{self.inner.lock().unwrap().cacheMap.len()}
}
impl Default for BiomeCache{fn default()->Self{Self::new()}}
