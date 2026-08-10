use std::io::{Read, Write};
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use thiserror::Error;
use crate::net::minecraft::network::Packet::RawPacket;

#[derive(Debug, Error)]
pub enum CodecError {
    #[error("VarInt is too large")]
    VarIntTooLarge,
    #[error("VarLong is too large")]
    VarLongTooLarge,
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("negative packet length: {0}")]
    NegativeLength(i32),
    #[error("packet length exceeds configured maximum: {actual} > {maximum}")]
    PacketTooLarge { actual: usize, maximum: usize },
    #[error("compressed packet below negotiated threshold: {actual} < {threshold}")]
    CompressedBelowThreshold { actual: usize, threshold: usize },
    #[error("uncompressed packet reached negotiated compression threshold: {actual} >= {threshold}")]
    UncompressedAboveThreshold { actual: usize, threshold: usize },
    #[error("decompressed packet length mismatch: declared {declared}, actual {actual}")]
    DecompressedLengthMismatch { declared: usize, actual: usize },
    #[error("string length exceeds configured maximum: {actual} > {maximum}")]
    StringTooLong { actual: usize, maximum: usize },
    #[error("invalid UTF-8 string: {0}")]
    InvalidUtf8(#[from] std::str::Utf8Error),
    #[error("invalid packet data: {0}")]
    InvalidData(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn write_var_i32(mut value: i32, output: &mut Vec<u8>) {
    loop {
        if (value & !0x7F) == 0 {
            output.push(value as u8);
            return;
        }
        output.push(((value & 0x7F) | 0x80) as u8);
        value = ((value as u32) >> 7) as i32;
    }
}

pub fn read_var_i32(input: &mut &[u8]) -> Result<i32, CodecError> {
    let mut value = 0_i32;
    let mut position = 0_u32;
    loop {
        let byte = take_byte(input)?;
        value |= ((byte & 0x7F) as i32) << position;
        if byte & 0x80 == 0 { return Ok(value); }
        position += 7;
        if position >= 35 { return Err(CodecError::VarIntTooLarge); }
    }
}

pub fn write_var_i64(mut value: i64, output: &mut Vec<u8>) {
    loop {
        if (value & !0x7F) == 0 {
            output.push(value as u8);
            return;
        }
        output.push(((value & 0x7F) | 0x80) as u8);
        value = ((value as u64) >> 7) as i64;
    }
}

pub fn read_var_i64(input: &mut &[u8]) -> Result<i64, CodecError> {
    let mut value = 0_i64;
    let mut position = 0_u32;
    loop {
        let byte = take_byte(input)?;
        value |= ((byte & 0x7F) as i64) << position;
        if byte & 0x80 == 0 { return Ok(value); }
        position += 7;
        if position >= 70 { return Err(CodecError::VarLongTooLarge); }
    }
}


pub fn write_string(value: &str, maximum_characters: usize, output: &mut Vec<u8>) -> Result<(), CodecError> {
    let characters = value.encode_utf16().count();
    if characters > maximum_characters { return Err(CodecError::StringTooLong { actual: characters, maximum: maximum_characters }); }
    let bytes = value.as_bytes();
    if bytes.len() > maximum_characters.saturating_mul(4) { return Err(CodecError::StringTooLong { actual: bytes.len(), maximum: maximum_characters.saturating_mul(4) }); }
    write_var_i32(bytes.len() as i32, output);
    output.extend_from_slice(bytes);
    Ok(())
}

pub fn read_string(input: &mut &[u8], maximum_characters: usize) -> Result<String, CodecError> {
    let byte_length = read_var_i32(input)?;
    if byte_length < 0 { return Err(CodecError::NegativeLength(byte_length)); }
    let byte_length = byte_length as usize;
    let maximum_bytes = maximum_characters.saturating_mul(4);
    if byte_length > maximum_bytes { return Err(CodecError::StringTooLong { actual: byte_length, maximum: maximum_bytes }); }
    if input.len() < byte_length { return Err(CodecError::UnexpectedEof); }
    let (bytes, remainder) = input.split_at(byte_length);
    *input = remainder;
    let value = std::str::from_utf8(bytes)?;
    let characters = value.encode_utf16().count();
    if characters > maximum_characters { return Err(CodecError::StringTooLong { actual: characters, maximum: maximum_characters }); }
    Ok(value.to_owned())
}



pub fn write_nbt_compound(value: Option<&crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound>, output: &mut Vec<u8>) -> Result<(), CodecError> {
    match value {
        None => output.push(0),
        Some(compound) => crate::net::minecraft::nbt::CompressedStreamTools::writeRoot(compound, output)?,
    }
    Ok(())
}

pub fn read_nbt_compound(input: &mut &[u8]) -> Result<Option<crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound>, CodecError> {
    if input.first().copied().ok_or(CodecError::UnexpectedEof)? == 0 {
        *input = &input[1..];
        return Ok(None);
    }
    crate::net::minecraft::nbt::CompressedStreamTools::readRoot(input)
        .map(Some)
        .map_err(CodecError::Io)
}

pub fn write_byte_array(value: &[u8], output: &mut Vec<u8>) -> Result<(), CodecError> {
    write_var_i32(i32::try_from(value.len()).map_err(|_| CodecError::PacketTooLarge { actual: value.len(), maximum: i32::MAX as usize })?, output);
    output.extend_from_slice(value);
    Ok(())
}

pub fn read_byte_array(input: &mut &[u8], maximum: usize) -> Result<Vec<u8>, CodecError> {
    let length = read_var_i32(input)?;
    if length < 0 { return Err(CodecError::NegativeLength(length)); }
    let length = length as usize;
    if length > maximum { return Err(CodecError::PacketTooLarge { actual: length, maximum }); }
    if input.len() < length { return Err(CodecError::UnexpectedEof); }
    let (bytes, remainder) = input.split_at(length);
    *input = remainder;
    Ok(bytes.to_vec())
}

pub fn write_f64_be(value: f64, output: &mut Vec<u8>) { output.extend_from_slice(&value.to_bits().to_be_bytes()); }
pub fn read_f64_be(input: &mut &[u8]) -> Result<f64, CodecError> { if input.len()<8{return Err(CodecError::UnexpectedEof);}let(bytes,remainder)=input.split_at(8);*input=remainder;Ok(f64::from_bits(u64::from_be_bytes(bytes.try_into().expect("exactly eight bytes")))) }
pub fn write_f32_be(value: f32, output: &mut Vec<u8>) { output.extend_from_slice(&value.to_bits().to_be_bytes()); }
pub fn read_f32_be(input: &mut &[u8]) -> Result<f32, CodecError> { if input.len()<4{return Err(CodecError::UnexpectedEof);}let(bytes,remainder)=input.split_at(4);*input=remainder;Ok(f32::from_bits(u32::from_be_bytes(bytes.try_into().expect("exactly four bytes")))) }

pub fn write_i32_be(value: i32, output: &mut Vec<u8>) { output.extend_from_slice(&value.to_be_bytes()); }

pub fn read_i32_be(input: &mut &[u8]) -> Result<i32, CodecError> {
    if input.len() < 4 { return Err(CodecError::UnexpectedEof); }
    let (bytes, remainder) = input.split_at(4);
    *input = remainder;
    Ok(i32::from_be_bytes(bytes.try_into().expect("exactly four bytes")))
}

pub fn read_u8(input: &mut &[u8]) -> Result<u8, CodecError> { take_byte(input) }
pub fn read_bool(input: &mut &[u8]) -> Result<bool, CodecError> { Ok(take_byte(input)? != 0) }
pub fn write_bool(value: bool, output: &mut Vec<u8>) { output.push(u8::from(value)); }

pub fn write_i64_be(value: i64, output: &mut Vec<u8>) { output.extend_from_slice(&value.to_be_bytes()); }

pub fn read_i64_be(input: &mut &[u8]) -> Result<i64, CodecError> {
    if input.len() < 8 { return Err(CodecError::UnexpectedEof); }
    let (bytes, remainder) = input.split_at(8);
    *input = remainder;
    Ok(i64::from_be_bytes(bytes.try_into().expect("exactly eight bytes")))
}


pub fn write_i16_be(value: i16, output: &mut Vec<u8>) { output.extend_from_slice(&value.to_be_bytes()); }

pub fn read_i16_be(input: &mut &[u8]) -> Result<i16, CodecError> {
    if input.len() < 2 { return Err(CodecError::UnexpectedEof); }
    let (bytes, remainder) = input.split_at(2);
    *input = remainder;
    Ok(i16::from_be_bytes(bytes.try_into().expect("exactly two bytes")))
}

pub fn read_i8(input: &mut &[u8]) -> Result<i8, CodecError> { Ok(take_byte(input)? as i8) }

pub fn write_uuid(value: uuid::Uuid, output: &mut Vec<u8>) {
    output.extend_from_slice(value.as_bytes());
}

pub fn read_uuid(input: &mut &[u8]) -> Result<uuid::Uuid, CodecError> {
    if input.len() < 16 { return Err(CodecError::UnexpectedEof); }
    let (bytes, remainder) = input.split_at(16);
    *input = remainder;
    uuid::Uuid::from_slice(bytes).map_err(|error| CodecError::InvalidData(error.to_string()))
}

pub fn read_text_component(input: &mut &[u8]) -> Result<crate::net::minecraft::util::text::ITextComponent::ITextComponent, CodecError> {
    let json = read_string(input, 32767)?;
    crate::net::minecraft::util::text::ITextComponent::ITextComponent::fromJsonLenient(&json)
        .map_err(|error| CodecError::InvalidData(error.to_string()))
}

pub fn write_u16_be(value: u16, output: &mut Vec<u8>) { output.extend_from_slice(&value.to_be_bytes()); }

pub fn read_u16_be(input: &mut &[u8]) -> Result<u16, CodecError> {
    if input.len() < 2 { return Err(CodecError::UnexpectedEof); }
    let (bytes, remainder) = input.split_at(2);
    *input = remainder;
    Ok(u16::from_be_bytes(bytes.try_into().expect("exactly two bytes")))
}

pub fn var_i32_size(value: i32) -> usize {
    for bytes in 1..5 {
        if (value & (-1_i32 << (bytes * 7))) == 0 { return bytes; }
    }
    5
}

fn take_byte(input: &mut &[u8]) -> Result<u8, CodecError> {
    let (&first, rest) = input.split_first().ok_or(CodecError::UnexpectedEof)?;
    *input = rest;
    Ok(first)
}

#[derive(Debug, Clone)]
pub struct PacketCodec {
    compression_threshold: Option<usize>,
    maximum_packet_size: usize,
}

impl Default for PacketCodec {
    fn default() -> Self {
        Self { compression_threshold: None, maximum_packet_size: 2 * 1024 * 1024 }
    }
}

impl PacketCodec {
    pub fn new(compression_threshold: Option<usize>, maximum_packet_size: usize) -> Self {
        Self { compression_threshold, maximum_packet_size }
    }

    pub fn set_compression_threshold(&mut self, threshold: Option<usize>) {
        self.compression_threshold = threshold;
    }

    pub fn encode(&self, packet: &RawPacket) -> Result<Vec<u8>, CodecError> {
        let mut body = Vec::with_capacity(var_i32_size(packet.id) + packet.payload.len());
        write_var_i32(packet.id, &mut body);
        body.extend_from_slice(&packet.payload);
        self.ensure_size(body.len())?;

        let framed_body = match self.compression_threshold {
            None => body,
            Some(threshold) if body.len() < threshold => {
                let mut result = Vec::with_capacity(body.len() + 1);
                write_var_i32(0, &mut result);
                result.extend_from_slice(&body);
                result
            }
            Some(_) => {
                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(&body)?;
                let compressed = encoder.finish()?;
                let mut result = Vec::with_capacity(compressed.len() + 5);
                write_var_i32(body.len() as i32, &mut result);
                result.extend_from_slice(&compressed);
                result
            }
        };

        let mut frame = Vec::with_capacity(var_i32_size(framed_body.len() as i32) + framed_body.len());
        write_var_i32(framed_body.len() as i32, &mut frame);
        frame.extend_from_slice(&framed_body);
        Ok(frame)
    }

    pub fn decode(&self, input: &mut &[u8]) -> Result<RawPacket, CodecError> {
        let packet_length = read_var_i32(input)?;
        if packet_length < 0 { return Err(CodecError::NegativeLength(packet_length)); }
        let packet_length = packet_length as usize;
        self.ensure_size(packet_length)?;
        if input.len() < packet_length { return Err(CodecError::UnexpectedEof); }
        let (packet_data, remainder) = input.split_at(packet_length);
        *input = remainder;

        let body = match self.compression_threshold {
            None => packet_data.to_vec(),
            Some(threshold) => {
                let mut compressed_view = packet_data;
                let declared_length = read_var_i32(&mut compressed_view)?;
                if declared_length < 0 { return Err(CodecError::NegativeLength(declared_length)); }
                if declared_length == 0 {
                    if compressed_view.len() >= threshold {
                        return Err(CodecError::UncompressedAboveThreshold { actual: compressed_view.len(), threshold });
                    }
                    compressed_view.to_vec()
                } else {
                    let declared_length = declared_length as usize;
                    self.ensure_size(declared_length)?;
                    if declared_length < threshold {
                        return Err(CodecError::CompressedBelowThreshold { actual: declared_length, threshold });
                    }
                    let mut decoder = ZlibDecoder::new(compressed_view);
                    let mut decompressed = Vec::with_capacity(declared_length);
                    decoder.read_to_end(&mut decompressed)?;
                    if decompressed.len() != declared_length {
                        return Err(CodecError::DecompressedLengthMismatch {
                            declared: declared_length,
                            actual: decompressed.len(),
                        });
                    }
                    decompressed
                }
            }
        };

        let mut body_view = body.as_slice();
        let id = read_var_i32(&mut body_view)?;
        Ok(RawPacket::new(id, body_view))
    }

    fn ensure_size(&self, length: usize) -> Result<(), CodecError> {
        if length > self.maximum_packet_size {
            Err(CodecError::PacketTooLarge { actual: length, maximum: self.maximum_packet_size })
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wiki_varint_vectors() {
        for (value, bytes) in [
            (0, vec![0x00]),
            (1, vec![0x01]),
            (127, vec![0x7f]),
            (128, vec![0x80, 0x01]),
            (255, vec![0xff, 0x01]),
            (2_147_483_647, vec![0xff, 0xff, 0xff, 0xff, 0x07]),
            (-1, vec![0xff, 0xff, 0xff, 0xff, 0x0f]),
            (-2_147_483_648, vec![0x80, 0x80, 0x80, 0x80, 0x08]),
        ] {
            let mut encoded = Vec::new();
            write_var_i32(value, &mut encoded);
            assert_eq!(encoded, bytes);
            let mut view = encoded.as_slice();
            assert_eq!(read_var_i32(&mut view).unwrap(), value);
            assert!(view.is_empty());
        }
    }

    #[test]
    fn packet_roundtrip_without_compression() {
        let codec = PacketCodec::default();
        let original = RawPacket::new(0x2f, [1, 2, 3, 4]);
        let encoded = codec.encode(&original).unwrap();
        let mut view = encoded.as_slice();
        assert_eq!(codec.decode(&mut view).unwrap(), original);
        assert!(view.is_empty());
    }

    #[test]
    fn packet_roundtrip_with_compression() {
        let codec = PacketCodec::new(Some(16), 2 * 1024 * 1024);
        let original = RawPacket::new(0x20, vec![0x55; 512]);
        let encoded = codec.encode(&original).unwrap();
        let mut view = encoded.as_slice();
        assert_eq!(codec.decode(&mut view).unwrap(), original);
    }

    #[test]
    fn minecraft_string_roundtrip_preserves_unicode() {
        let mut encoded = Vec::new();
        write_string("服务器😀", 32767, &mut encoded).unwrap();
        let mut view = encoded.as_slice();
        assert_eq!(read_string(&mut view, 32767).unwrap(), "服务器😀");
        assert!(view.is_empty());
    }

    #[test]
    fn string_limit_uses_java_utf16_code_units() {
        let mut output = Vec::new();
        assert!(write_string("😀", 1, &mut output).is_err());
        output.clear();
        write_string("😀", 2, &mut output).unwrap();
        let mut view = output.as_slice();
        assert_eq!(read_string(&mut view, 2).unwrap(), "😀");
    }

}
