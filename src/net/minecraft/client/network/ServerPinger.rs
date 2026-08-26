use std::collections::HashSet;
use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::net::minecraft::client::multiplayer::ServerAddress::ServerAddress;
use crate::net::minecraft::network::handshake::client::C00Handshake::C00Handshake;
use crate::net::minecraft::network::status::client::CPacketPing::CPacketPing;
use crate::net::minecraft::network::status::client::CPacketServerQuery::CPacketServerQuery;
use crate::net::minecraft::network::status::server::SPacketPong::SPacketPong;
use crate::net::minecraft::network::status::server::SPacketServerInfo::SPacketServerInfo;
use crate::net::minecraft::network::EnumConnectionState::ConnectionState;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_var_i32, PacketCodec};

const STATUS_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPingSuccess {
    pub serverMOTD: String,
    pub populationInfo: String,
    pub pingToServer: i64,
    pub version: i32,
    pub gameVersion: String,
    pub playerList: Option<String>,
    pub serverIcon: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerPingFailure {
    CannotResolve,
    CannotConnect(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPingResult {
    pub requestId: u64,
    pub serverIndex: usize,
    pub serverIP: String,
    pub result: Result<ServerPingSuccess, ServerPingFailure>,
}

pub struct ServerPinger {
    sender: Sender<ServerPingResult>,
    receiver: Receiver<ServerPingResult>,
    pending: HashSet<u64>,
    nextRequestId: u64,
}

impl Default for ServerPinger {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver,
            pending: HashSet::new(),
            nextRequestId: 1,
        }
    }
}

impl ServerPinger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ping(&mut self, serverIndex: usize, serverIP: String) -> u64 {
        let requestId = self.nextRequestId;
        self.nextRequestId = self.nextRequestId.wrapping_add(1).max(1);
        self.pending.insert(requestId);
        let sender = self.sender.clone();
        let threadServerIP = serverIP.clone();
        let spawnResult = std::thread::Builder::new()
            .name(format!("Server Pinger #{requestId}"))
            .spawn(move || {
                let result = pingServer(&threadServerIP);
                let _ = sender.send(ServerPingResult {
                    requestId,
                    serverIndex,
                    serverIP: threadServerIP,
                    result,
                });
            });
        if let Err(error) = spawnResult {
            self.pending.remove(&requestId);
            let _ = self.sender.send(ServerPingResult {
                requestId,
                serverIndex,
                serverIP,
                result: Err(ServerPingFailure::CannotConnect(error.to_string())),
            });
        }
        requestId
    }

    pub fn pingPendingNetworks(&mut self) -> Vec<ServerPingResult> {
        let mut results = Vec::new();
        while let Ok(result) = self.receiver.try_recv() {
            self.pending.remove(&result.requestId);
            results.push(result);
        }
        results
    }

    pub fn clearPendingNetworks(&mut self) {
        self.pending.clear();
    }
    pub fn hasPendingNetworks(&self) -> bool {
        !self.pending.is_empty()
    }
}

pub fn pingServer(serverIP: &str) -> Result<ServerPingSuccess, ServerPingFailure> {
    match pingServerModern(serverIP) {
        Ok(status) => Ok(status),
        Err(ServerPingFailure::CannotResolve) => Err(ServerPingFailure::CannotResolve),
        Err(modernFailure) => tryCompatibilityPing(serverIP).or(Err(modernFailure)),
    }
}

fn pingServerModern(serverIP: &str) -> Result<ServerPingSuccess, ServerPingFailure> {
    let address = ServerAddress::fromString(serverIP);
    let host = address.getIP();
    if host.is_empty() {
        return Err(ServerPingFailure::CannotResolve);
    }
    let socketAddress = (host.as_str(), address.getPort())
        .to_socket_addrs()
        .map_err(|_| ServerPingFailure::CannotResolve)?
        .next()
        .ok_or(ServerPingFailure::CannotResolve)?;
    let mut stream = TcpStream::connect_timeout(&socketAddress, STATUS_TIMEOUT)
        .map_err(|error| ServerPingFailure::CannotConnect(error.to_string()))?;
    configureStatusStream(&stream);

    let codec = PacketCodec::default();
    let handshake = C00Handshake::new(host, address.getPort(), ConnectionState::Status)
        .writePacketData()
        .map_err(|error| ServerPingFailure::CannotConnect(error.to_string()))?;
    writePacket(&mut stream, &codec, &handshake)?;
    writePacket(&mut stream, &codec, &CPacketServerQuery.writePacketData())?;

    let responsePacket = readPacket(&mut stream, &codec)?;
    if responsePacket.id != 0 {
        return Err(ServerPingFailure::CannotConnect(format!(
            "unexpected status packet id {}",
            responsePacket.id
        )));
    }
    let response = SPacketServerInfo::readPacketData(&responsePacket)
        .map_err(|error| ServerPingFailure::CannotConnect(error.to_string()))?;
    let response = response.getResponse();

    let pingSentAt = currentTimeMillis();
    let pingStarted = Instant::now();
    writePacket(
        &mut stream,
        &codec,
        &CPacketPing::new(pingSentAt).writePacketData(),
    )?;
    let pongPacket = readPacket(&mut stream, &codec)?;
    if pongPacket.id != 1 {
        return Err(ServerPingFailure::CannotConnect(format!(
            "unexpected pong packet id {}",
            pongPacket.id
        )));
    }
    let pong = SPacketPong::readPacketData(&pongPacket)
        .map_err(|error| ServerPingFailure::CannotConnect(error.to_string()))?;
    if pong.getClientTime() != pingSentAt {
        return Err(ServerPingFailure::CannotConnect(
            "pong timestamp did not match request".to_owned(),
        ));
    }
    let pingToServer = i64::try_from(pingStarted.elapsed().as_millis()).unwrap_or(i64::MAX);

    let (version, gameVersion) = response
        .getVersion()
        .map(|value| (value.getProtocol(), value.getName().to_owned()))
        .unwrap_or((0, "Old".to_owned()));
    let (populationInfo, playerList) = if let Some(players) = response.getPlayers() {
        let populationInfo = format!(
            "§7{}§8/§7{}",
            players.getOnlinePlayerCount(),
            players.getMaxPlayers()
        );
        let mut names: Vec<String> = players
            .getPlayers()
            .iter()
            .map(|profile| profile.name.clone())
            .collect();
        let unlisted = players
            .getOnlinePlayerCount()
            .saturating_sub(names.len() as i32);
        if unlisted > 0 {
            names.push(format!("... and {unlisted} more ..."));
        }
        (
            populationInfo,
            if names.is_empty() {
                None
            } else {
                Some(names.join("\n"))
            },
        )
    } else {
        ("§8???".to_owned(), None)
    };
    let serverIcon = response
        .getFavicon()
        .and_then(|value| value.strip_prefix("data:image/png;base64,"))
        .map(str::to_owned);

    Ok(ServerPingSuccess {
        serverMOTD: response
            .getServerDescription()
            .unwrap_or_default()
            .to_owned(),
        populationInfo,
        pingToServer,
        version,
        gameVersion,
        playerList,
        serverIcon,
    })
}

/// 1.6-era fallback used by MCP ServerPinger.tryCompatibilityPing.
fn tryCompatibilityPing(serverIP: &str) -> Result<ServerPingSuccess, ServerPingFailure> {
    let address = ServerAddress::fromString(serverIP);
    let host = address.getIP();
    if host.is_empty() {
        return Err(ServerPingFailure::CannotResolve);
    }
    let socketAddress = (host.as_str(), address.getPort())
        .to_socket_addrs()
        .map_err(|_| ServerPingFailure::CannotResolve)?
        .next()
        .ok_or(ServerPingFailure::CannotResolve)?;
    let mut stream = TcpStream::connect_timeout(&socketAddress, STATUS_TIMEOUT)
        .map_err(|error| ServerPingFailure::CannotConnect(error.to_string()))?;
    configureStatusStream(&stream);

    stream.write_all(&[0xFE, 0x01, 0xFA]).map_err(mapIo)?;
    writeUtf16Be(&mut stream, "MC|PingHost").map_err(mapIo)?;
    let hostUnits: Vec<u16> = host.encode_utf16().collect();
    let payloadLength = 7_usize.saturating_add(hostUnits.len().saturating_mul(2));
    stream
        .write_u16::<BigEndian>(u16::try_from(payloadLength).map_err(|_| {
            ServerPingFailure::CannotConnect("legacy ping host is too long".to_owned())
        })?)
        .map_err(mapIo)?;
    stream.write_u8(127).map_err(mapIo)?;
    stream
        .write_u16::<BigEndian>(u16::try_from(hostUnits.len()).map_err(|_| {
            ServerPingFailure::CannotConnect("legacy ping host is too long".to_owned())
        })?)
        .map_err(mapIo)?;
    for unit in hostUnits {
        stream.write_u16::<BigEndian>(unit).map_err(mapIo)?;
    }
    stream
        .write_i32::<BigEndian>(i32::from(address.getPort()))
        .map_err(mapIo)?;
    stream.flush().map_err(mapIo)?;

    if stream.read_u8().map_err(mapIo)? != 0xFF {
        return Err(ServerPingFailure::CannotConnect(
            "legacy ping returned an unexpected packet".to_owned(),
        ));
    }
    let charCount = stream.read_u16::<BigEndian>().map_err(mapIo)? as usize;
    let mut units = Vec::with_capacity(charCount);
    for _ in 0..charCount {
        units.push(stream.read_u16::<BigEndian>().map_err(mapIo)?);
    }
    let response = String::from_utf16(&units).map_err(|_| {
        ServerPingFailure::CannotConnect("legacy ping returned invalid UTF-16".to_owned())
    })?;
    let fields: Vec<&str> = response.splitn(6, '\0').collect();
    if fields.len() != 6 || fields[0] != "§1" {
        return Err(ServerPingFailure::CannotConnect(
            "legacy ping returned an invalid response".to_owned(),
        ));
    }
    let online = fields[4].parse::<i32>().unwrap_or(-1);
    let maximum = fields[5].parse::<i32>().unwrap_or(-1);
    Ok(ServerPingSuccess {
        serverMOTD: fields[3].to_owned(),
        populationInfo: format!("§7{online}§8/§7{maximum}"),
        pingToServer: -1,
        version: -1,
        gameVersion: fields[2].to_owned(),
        playerList: None,
        serverIcon: None,
    })
}

fn configureStatusStream(stream: &TcpStream) {
    stream.set_read_timeout(Some(STATUS_TIMEOUT)).ok();
    stream.set_write_timeout(Some(STATUS_TIMEOUT)).ok();
    stream.set_nodelay(true).ok();
}

fn writeUtf16Be(output: &mut impl Write, value: &str) -> io::Result<()> {
    let units: Vec<u16> = value.encode_utf16().collect();
    output.write_u16::<BigEndian>(
        u16::try_from(units.len()).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidInput, "UTF-16 string is too long")
        })?,
    )?;
    for unit in units {
        output.write_u16::<BigEndian>(unit)?;
    }
    Ok(())
}

fn writePacket(
    stream: &mut TcpStream,
    codec: &PacketCodec,
    packet: &RawPacket,
) -> Result<(), ServerPingFailure> {
    let encoded = codec
        .encode(packet)
        .map_err(|error| ServerPingFailure::CannotConnect(error.to_string()))?;
    stream
        .write_all(&encoded)
        .map_err(|error| ServerPingFailure::CannotConnect(error.to_string()))
}

fn readPacket(stream: &mut TcpStream, codec: &PacketCodec) -> Result<RawPacket, ServerPingFailure> {
    let (packetLength, lengthBytes) = readVarIntFromStream(stream).map_err(mapIo)?;
    if packetLength < 0 || packetLength as usize > 2 * 1024 * 1024 {
        return Err(ServerPingFailure::CannotConnect(format!(
            "invalid status packet length {packetLength}"
        )));
    }
    let mut body = vec![0_u8; packetLength as usize];
    stream.read_exact(&mut body).map_err(mapIo)?;
    let mut frame = lengthBytes;
    frame.extend_from_slice(&body);
    let mut input = frame.as_slice();
    codec
        .decode(&mut input)
        .map_err(|error| ServerPingFailure::CannotConnect(error.to_string()))
}

fn readVarIntFromStream(stream: &mut TcpStream) -> io::Result<(i32, Vec<u8>)> {
    let mut bytes = Vec::with_capacity(5);
    for _ in 0..5 {
        let mut byte = [0_u8; 1];
        stream.read_exact(&mut byte)?;
        bytes.push(byte[0]);
        if byte[0] & 0x80 == 0 {
            let value = {
                let mut view = bytes.as_slice();
                read_var_i32(&mut view)
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?
            };
            return Ok((value, bytes));
        }
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "VarInt is too large",
    ))
}

fn mapIo(error: io::Error) -> ServerPingFailure {
    if matches!(
        error.kind(),
        io::ErrorKind::NotFound | io::ErrorKind::AddrNotAvailable
    ) {
        ServerPingFailure::CannotResolve
    } else {
        ServerPingFailure::CannotConnect(error.to_string())
    }
}

fn currentTimeMillis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn status_failure_categories_are_stable() {
        assert!(matches!(
            pingServer("[invalid"),
            Err(ServerPingFailure::CannotResolve) | Err(ServerPingFailure::CannotConnect(_))
        ));
    }

    #[test]
    fn legacy_channel_name_uses_java_utf16be_shape() {
        let mut bytes = Vec::new();
        writeUtf16Be(&mut bytes, "MC|PingHost").unwrap();
        assert_eq!(&bytes[..2], &[0, 11]);
        assert_eq!(bytes.len(), 24);
    }
}
