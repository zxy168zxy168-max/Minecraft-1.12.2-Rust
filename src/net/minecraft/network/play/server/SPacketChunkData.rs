use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_bool, read_i32_be, read_nbt_compound, read_var_i32, write_bool, write_i32_be, write_nbt_compound, write_var_i32, CodecError};
use crate::net::minecraft::world::chunk::Chunk::Chunk;

#[derive(Debug, Clone, PartialEq)]
pub struct SPacketChunkData {
    chunkX:i32,
    chunkZ:i32,
    availableSections:i32,
    buffer:Vec<u8>,
    tileEntityTags:Vec<NBTTagCompound>,
    loadChunk:bool,
}
impl SPacketChunkData {
    /// MCP `SPacketChunkData(Chunk,int)` packet extraction.  The `hasSkyLight`
    /// argument is the Rust equivalent of `chunk.getWorld().provider.func_191066_m()`.
    pub fn new(chunk:&Chunk, changedSectionFilter:i32, hasSkyLight:bool) -> Self {
        let loadChunk=changedSectionFilter==65535;
        let mut buffer=Vec::new();
        let mut availableSections=0_i32;
        for (index,storage) in chunk.getBlockStorageArray().iter().enumerate() {
            let Some(storage)=storage.as_deref() else { continue; };
            if (!loadChunk || !storage.isEmpty()) && (changedSectionFilter & (1_i32<<index)) != 0 {
                availableSections |= 1_i32<<index;
                storage.getData().write(&mut buffer);
                buffer.extend_from_slice(storage.getBlocklightArray().getData());
                if hasSkyLight {
                    if let Some(skylight)=storage.getSkylightArray(){ buffer.extend_from_slice(skylight.getData()); }
                    else { buffer.extend(std::iter::repeat(0_u8).take(2048)); }
                }
            }
        }
        if loadChunk { buffer.extend_from_slice(chunk.getBiomeArray()); }
        let mut tileEntityTags=Vec::new();
        for (pos,tag) in chunk.getTileEntityMapData() {
            let section=pos.y>>4;
            if loadChunk || (section>=0 && section<16 && (changedSectionFilter & (1_i32<<section))!=0) {
                // Until concrete server TileEntity subclasses replace the NBT ownership
                // layer, the preserved compound is the only authoritative payload.
                tileEntityTags.push(tag.clone());
            }
        }
        Self{chunkX:chunk.xPosition,chunkZ:chunk.zPosition,availableSections,buffer,tileEntityTags,loadChunk}
    }
    pub fn writePacketData(&self)->Result<RawPacket,CodecError>{
        let mut payload=Vec::with_capacity(self.buffer.len()+32);
        write_i32_be(self.chunkX,&mut payload); write_i32_be(self.chunkZ,&mut payload); write_bool(self.loadChunk,&mut payload);
        write_var_i32(self.availableSections,&mut payload); write_var_i32(self.buffer.len() as i32,&mut payload); payload.extend_from_slice(&self.buffer);
        write_var_i32(self.tileEntityTags.len() as i32,&mut payload); for tag in &self.tileEntityTags { write_nbt_compound(Some(tag),&mut payload)?; }
        Ok(RawPacket::new(0x20,payload))
    }
    pub fn readPacketData(packet:&RawPacket)->Result<Self,CodecError>{
        let mut input=packet.payload.as_slice();
        let chunkX=read_i32_be(&mut input)?;
        let chunkZ=read_i32_be(&mut input)?;
        let loadChunk=read_bool(&mut input)?;
        let availableSections=read_var_i32(&mut input)?;
        let length=read_var_i32(&mut input)?;
        if length<0{return Err(CodecError::NegativeLength(length));}
        if length>2_097_152{return Err(CodecError::PacketTooLarge{actual:length as usize,maximum:2_097_152});}
        if input.len()<length as usize{return Err(CodecError::UnexpectedEof);}
        let (buffer,remainder)=input.split_at(length as usize); input=remainder;
        let count=read_var_i32(&mut input)?;
        if count<0{return Err(CodecError::NegativeLength(count));}
        let mut tileEntityTags=Vec::with_capacity(count as usize);
        for _ in 0..count { if let Some(tag)=read_nbt_compound(&mut input)?{tileEntityTags.push(tag);} }
        Ok(Self{chunkX,chunkZ,availableSections,buffer:buffer.to_vec(),tileEntityTags,loadChunk})
    }
    pub const fn getChunkX(&self)->i32{self.chunkX}
    pub const fn getChunkZ(&self)->i32{self.chunkZ}
    pub const fn getExtractedSize(&self)->i32{self.availableSections}
    pub const fn doChunkLoad(&self)->bool{self.loadChunk}
    pub fn getReadBuffer(&self)->&[u8]{&self.buffer}
    pub fn getTileEntityTags(&self)->&[NBTTagCompound]{&self.tileEntityTags}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::world::gen::ChunkGeneratorFlat::ChunkGeneratorFlat;
    use crate::net::minecraft::world::gen::IChunkGenerator::IChunkGenerator;

    #[test]
    fn default_flat_full_chunk_round_trips_packet_payload() {
        let mut generator=ChunkGeneratorFlat::new(12345,true,"");
        let chunk=generator.provideChunk(3,-5).unwrap();
        let packet=SPacketChunkData::new(&chunk,65535,true);
        assert_eq!(packet.getChunkX(),3);
        assert_eq!(packet.getChunkZ(),-5);
        assert!(packet.doChunkLoad());
        assert_eq!(packet.getExtractedSize(),1);
        assert!(!packet.getReadBuffer().is_empty());
        let raw=packet.writePacketData().unwrap();
        assert_eq!(raw.id,0x20);
        let decoded=SPacketChunkData::readPacketData(&raw).unwrap();
        assert_eq!(decoded,packet);
    }
}
