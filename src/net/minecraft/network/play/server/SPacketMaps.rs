use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_bool, read_byte_array, read_i8, read_u8, read_var_i32, CodecError,
};
use crate::net::minecraft::world::storage::MapData::MapData;
use crate::net::minecraft::world::storage::MapDecoration::{MapDecoration, MapDecorationType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPacketMaps {
    mapId: i32,
    mapScale: i8,
    trackingPosition: bool,
    icons: Vec<MapDecoration>,
    minX: u8,
    minZ: u8,
    columns: u8,
    rows: u8,
    mapDataBytes: Vec<u8>,
}

impl SPacketMaps {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let mapId = read_var_i32(&mut input)?;
        let mapScale = read_i8(&mut input)?;
        let trackingPosition = read_bool(&mut input)?;
        let iconCount = read_var_i32(&mut input)?;
        if iconCount < 0 {
            return Err(CodecError::NegativeLength(iconCount));
        }
        if iconCount > 16_384 {
            return Err(CodecError::PacketTooLarge {
                actual: iconCount as usize,
                maximum: 16_384,
            });
        }
        let mut icons = Vec::with_capacity(iconCount as usize);
        for _ in 0..iconCount {
            let packed = read_u8(&mut input)?;
            let decorationType = MapDecorationType::fromId((packed >> 4) & 15);
            let rotation = (packed & 15) as i8;
            let x = read_i8(&mut input)?;
            let y = read_i8(&mut input)?;
            icons.push(MapDecoration::new(decorationType, x, y, rotation));
        }

        let columns = read_u8(&mut input)?;
        let mut rows = 0;
        let mut minX = 0;
        let mut minZ = 0;
        let mut mapDataBytes = Vec::new();
        if columns > 0 {
            rows = read_u8(&mut input)?;
            minX = read_u8(&mut input)?;
            minZ = read_u8(&mut input)?;
            mapDataBytes = read_byte_array(&mut input, MapData::PIXEL_COUNT)?;
            let expected = columns as usize * rows as usize;
            if mapDataBytes.len() != expected {
                return Err(CodecError::InvalidData(format!(
                    "map patch byte count {} does not match {}x{}={expected}",
                    mapDataBytes.len(),
                    columns,
                    rows,
                )));
            }
            if minX as usize + columns as usize > MapData::WIDTH
                || minZ as usize + rows as usize > MapData::HEIGHT
            {
                return Err(CodecError::InvalidData(format!(
                    "map patch ({minX},{minZ}) {columns}x{rows} exceeds 128x128",
                )));
            }
        }
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread map packet bytes",
                input.len(),
            )));
        }
        Ok(Self {
            mapId,
            mapScale,
            trackingPosition,
            icons,
            minX,
            minZ,
            columns,
            rows,
            mapDataBytes,
        })
    }

    pub const fn getMapId(&self) -> i32 {
        self.mapId
    }
    pub const fn getMapScale(&self) -> i8 {
        self.mapScale
    }
    pub const fn isTrackingPosition(&self) -> bool {
        self.trackingPosition
    }
    pub fn getIcons(&self) -> &[MapDecoration] {
        &self.icons
    }
    pub const fn getColumns(&self) -> u8 {
        self.columns
    }
    pub const fn getRows(&self) -> u8 {
        self.rows
    }
    pub const fn getMinX(&self) -> u8 {
        self.minX
    }
    pub const fn getMinZ(&self) -> u8 {
        self.minZ
    }

    /// MCP `SPacketMaps#setMapdataTo`.
    pub fn setMapdataTo(&self, mapData: &mut MapData) {
        mapData.scale = self.mapScale;
        mapData.trackingPosition = self.trackingPosition;
        mapData.mapDecorations.clear();
        mapData.mapDecorations.extend(self.icons.iter().copied());
        for column in 0..self.columns as usize {
            for row in 0..self.rows as usize {
                let destination =
                    self.minX as usize + column + (self.minZ as usize + row) * MapData::WIDTH;
                let source = column + row * self.columns as usize;
                mapData.colors[destination] = self.mapDataBytes[source];
            }
        }
        mapData.revision = mapData.revision.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::network::PacketBuffer::{
        write_bool, write_byte_array, write_var_i32,
    };

    #[test]
    fn packet_applies_column_major_patch_and_icon_nibbles() {
        let mut payload = Vec::new();
        write_var_i32(7, &mut payload);
        payload.push(2);
        write_bool(true, &mut payload);
        write_var_i32(1, &mut payload);
        payload.extend_from_slice(&[(8 << 4) | 3, 4, 0xFC]);
        payload.extend_from_slice(&[2, 2, 3, 4]);
        write_byte_array(&[10, 11, 12, 13], &mut payload).unwrap();
        let packet = SPacketMaps::readPacketData(&RawPacket::new(0x24, payload)).unwrap();
        let mut map = MapData::new(7);
        packet.setMapdataTo(&mut map);
        assert_eq!(map.scale, 2);
        assert!(map.trackingPosition);
        assert_eq!(
            map.mapDecorations[0].decorationType(),
            MapDecorationType::Mansion
        );
        assert_eq!(map.mapDecorations[0].getRotation(), 3);
        assert_eq!(map.mapDecorations[0].getY(), -4);
        assert_eq!(map.colors[3 + 4 * 128], 10);
        assert_eq!(map.colors[4 + 4 * 128], 11);
        assert_eq!(map.colors[3 + 5 * 128], 12);
        assert_eq!(map.colors[4 + 5 * 128], 13);
    }
}
