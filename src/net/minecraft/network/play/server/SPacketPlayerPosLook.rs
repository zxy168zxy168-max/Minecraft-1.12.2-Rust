use bitflags::bitflags;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_f32_be,read_f64_be,read_u8,read_var_i32,write_f32_be,write_f64_be,write_var_i32,CodecError};
bitflags!{#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub struct EnumFlags:u8{const X=1;const Y=2;const Z=4;const Y_ROT=8;const X_ROT=16;}}
#[derive(Debug,Clone,Copy,PartialEq)]pub struct SPacketPlayerPosLook{x:f64,y:f64,z:f64,yaw:f32,pitch:f32,flags:EnumFlags,teleportId:i32}
impl SPacketPlayerPosLook{
 pub fn new(x:f64,y:f64,z:f64,yaw:f32,pitch:f32,flags:EnumFlags,teleportId:i32)->Self{Self{x,y,z,yaw,pitch,flags,teleportId}}
 pub fn writePacketData(&self)->RawPacket{let mut payload=Vec::new();write_f64_be(self.x,&mut payload);write_f64_be(self.y,&mut payload);write_f64_be(self.z,&mut payload);write_f32_be(self.yaw,&mut payload);write_f32_be(self.pitch,&mut payload);payload.push(self.flags.bits());write_var_i32(self.teleportId,&mut payload);RawPacket::new(0x2F,payload)}
 pub fn readPacketData(packet:&RawPacket)->Result<Self,CodecError>{let mut input=packet.payload.as_slice();Ok(Self{x:read_f64_be(&mut input)?,y:read_f64_be(&mut input)?,z:read_f64_be(&mut input)?,yaw:read_f32_be(&mut input)?,pitch:read_f32_be(&mut input)?,flags:EnumFlags::from_bits_truncate(read_u8(&mut input)?),teleportId:read_var_i32(&mut input)?})}pub const fn getX(&self)->f64{self.x}pub const fn getY(&self)->f64{self.y}pub const fn getZ(&self)->f64{self.z}pub const fn getYaw(&self)->f32{self.yaw}pub const fn getPitch(&self)->f32{self.pitch}pub const fn getTeleportId(&self)->i32{self.teleportId}pub const fn getFlags(&self)->EnumFlags{self.flags}}
