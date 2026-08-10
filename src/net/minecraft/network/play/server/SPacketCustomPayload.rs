use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_string, write_string, CodecError};
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct SPacketCustomPayload{channel:String,data:Vec<u8>}
impl SPacketCustomPayload{
 pub fn new(channel:impl Into<String>,data:impl Into<Vec<u8>>)->Self{Self{channel:channel.into(),data:data.into()}}
 pub fn writePacketData(&self)->Result<RawPacket,CodecError>{if self.data.len()>1_048_576{return Err(CodecError::PacketTooLarge{actual:self.data.len(),maximum:1_048_576});}let mut payload=Vec::new();write_string(&self.channel,20,&mut payload)?;payload.extend_from_slice(&self.data);Ok(RawPacket::new(0x18,payload))}
 pub fn readPacketData(packet:&RawPacket)->Result<Self,CodecError>{let mut input=packet.payload.as_slice();let channel=read_string(&mut input,20)?;if input.len()>1_048_576{return Err(CodecError::PacketTooLarge{actual:input.len(),maximum:1_048_576});}Ok(Self{channel,data:input.to_vec()})}
 pub fn getChannelName(&self)->&str{&self.channel} pub fn getBufferData(&self)->&[u8]{&self.data}
}
