use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError},
    Arc,
};
use std::time::Duration;

use thiserror::Error;

use crate::net::minecraft::network::EnumConnectionState::ConnectionState;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{read_var_i32, PacketCodec, CodecError};
use crate::net::minecraft::util::CryptManager::{NetCipher, SecretKey};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const IO_TIMEOUT: Duration = Duration::from_millis(250);

static NEXT_NETWORK_MANAGER_ID: AtomicU64 = AtomicU64::new(1);

fn nextNetworkManagerId() -> u64 {
    NEXT_NETWORK_MANAGER_ID.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug, Error)]
pub enum NetworkManagerError {
    #[error("unknown host")]
    UnknownHost,
    #[error("network operation timed out")]
    Timeout,
    #[error("connection closed by remote host")]
    Closed,
    #[error("local endpoint is no longer accepting connections")]
    LocalEndpointClosed,
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error(transparent)]
    Codec(#[from] CodecError),
}

/// Rust equivalent of Netty's `LocalAddress` used by MCP
/// `NetworkSystem#addLocalEndpoint` / `NetworkManager#provideLocalClient`.
///
/// Local single-player traffic is packet-object traffic, not loopback TCP:
/// MCP's local pipeline contains only `packet_handler` and intentionally
/// bypasses the byte encoder/decoder, encryption and compression handlers.
#[derive(Clone)]
pub struct LocalEndpointAddress {
    id: u64,
    connector: Sender<LocalConnectionRequest>,
}

impl std::fmt::Debug for LocalEndpointAddress {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_tuple("LocalAddress").field(&self.id).finish()
    }
}

impl std::fmt::Display for LocalEndpointAddress {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "local:{}", self.id)
    }
}

impl LocalEndpointAddress {
    pub const fn id(&self) -> u64 { self.id }
}

pub(crate) struct LocalConnectionRequest {
    endpointId: u64,
    inbound: Receiver<RawPacket>,
    outbound: Sender<RawPacket>,
    open: Arc<AtomicBool>,
}

impl LocalConnectionRequest {
    pub(crate) fn close(&self) { self.open.store(false, Ordering::Release); }
}

pub(crate) fn createLocalEndpointChannel() -> (LocalEndpointAddress, Receiver<LocalConnectionRequest>) {
    static NEXT_LOCAL_ENDPOINT_ID: AtomicU64 = AtomicU64::new(1);
    let id = NEXT_LOCAL_ENDPOINT_ID.fetch_add(1, Ordering::Relaxed);
    let (connector, receiver) = mpsc::channel();
    (LocalEndpointAddress { id, connector }, receiver)
}

#[derive(Debug)]
enum NetworkTransport {
    Tcp(TcpStream),
    Local {
        endpointId: u64,
        inbound: Receiver<RawPacket>,
        outbound: Sender<RawPacket>,
        open: Arc<AtomicBool>,
        readTimeout: Duration,
    },
}

#[derive(Debug)]
pub struct NetworkManager {
    managerId: u64,
    transport: NetworkTransport,
    socketAddress: SocketAddr,
    connectionState: ConnectionState,
    codec: PacketCodec,
    encryptor: Option<NetCipher>,
    decryptor: Option<NetCipher>,
    channelOpen: bool,
}

impl NetworkManager {
    pub fn createNetworkManagerAndConnect(host: &str, port: u16) -> Result<Self, NetworkManagerError> {
        let socketAddress = (host, port).to_socket_addrs()
            .map_err(|_| NetworkManagerError::UnknownHost)?
            .next().ok_or(NetworkManagerError::UnknownHost)?;
        let stream = TcpStream::connect_timeout(&socketAddress, CONNECT_TIMEOUT)?;
        stream.set_nodelay(true).ok();
        stream.set_read_timeout(Some(IO_TIMEOUT)).ok();
        stream.set_write_timeout(Some(CONNECT_TIMEOUT)).ok();
        Ok(Self {
            managerId: nextNetworkManagerId(),
            transport: NetworkTransport::Tcp(stream),
            socketAddress,
            connectionState: ConnectionState::Handshaking,
            codec: PacketCodec::default(),
            encryptor: None,
            decryptor: None,
            channelOpen: true,
        })
    }

    /// MCP `NetworkManager#provideLocalClient`.
    ///
    /// The pair of mpsc queues is the Rust equivalent of Netty LocalChannel:
    /// `RawPacket` objects cross the local endpoint without wire encoding.
    pub fn provideLocalClient(address: &LocalEndpointAddress) -> Result<Self, NetworkManagerError> {
        let (clientToServer, serverInbound) = mpsc::channel::<RawPacket>();
        let (serverToClient, clientInbound) = mpsc::channel::<RawPacket>();
        let open = Arc::new(AtomicBool::new(true));
        address.connector.send(LocalConnectionRequest {
            endpointId: address.id,
            inbound: serverInbound,
            outbound: serverToClient,
            open: Arc::clone(&open),
        }).map_err(|_| NetworkManagerError::LocalEndpointClosed)?;
        Ok(Self {
            managerId: nextNetworkManagerId(),
            transport: NetworkTransport::Local {
                endpointId: address.id,
                inbound: clientInbound,
                outbound: clientToServer,
                open,
                readTimeout: IO_TIMEOUT,
            },
            socketAddress: SocketAddr::from(([127, 0, 0, 1], 0)),
            connectionState: ConnectionState::Handshaking,
            codec: PacketCodec::default(),
            encryptor: None,
            decryptor: None,
            channelOpen: true,
        })
    }

    pub(crate) fn fromLocalServer(request: LocalConnectionRequest) -> Self {
        Self {
            managerId: nextNetworkManagerId(),
            transport: NetworkTransport::Local {
                endpointId: request.endpointId,
                inbound: request.inbound,
                outbound: request.outbound,
                open: request.open,
                readTimeout: Duration::ZERO,
            },
            socketAddress: SocketAddr::from(([127, 0, 0, 1], 0)),
            connectionState: ConnectionState::Handshaking,
            codec: PacketCodec::default(),
            encryptor: None,
            decryptor: None,
            channelOpen: true,
        }
    }

    pub fn sendPacket(&mut self, packet: &RawPacket) -> Result<(), NetworkManagerError> {
        match &mut self.transport {
            NetworkTransport::Tcp(stream) => {
                let mut encoded = self.codec.encode(packet)?;
                if let Some(cipher) = self.encryptor.as_mut() { cipher.apply(&mut encoded); }
                stream.write_all(&encoded)?;
                stream.flush()?;
                Ok(())
            }
            NetworkTransport::Local { outbound, open, .. } => {
                if !open.load(Ordering::Acquire) {
                    self.channelOpen = false;
                    return Err(NetworkManagerError::Closed);
                }
                if outbound.send(packet.clone()).is_err() {
                    open.store(false, Ordering::Release);
                    self.channelOpen = false;
                    return Err(NetworkManagerError::Closed);
                }
                Ok(())
            }
        }
    }

    pub fn readPacket(&mut self) -> Result<RawPacket, NetworkManagerError> {
        if !self.isLocalChannel() {
            return self.readTcpPacket();
        }

        let NetworkTransport::Local { inbound, open, readTimeout, .. } = &mut self.transport else {
            unreachable!("local transport was checked above");
        };
        if !open.load(Ordering::Acquire) {
            self.channelOpen = false;
            return Err(NetworkManagerError::Closed);
        }
        let received = if readTimeout.is_zero() {
            match inbound.try_recv() {
                Ok(packet) => Ok(packet),
                Err(TryRecvError::Empty) => Err(NetworkManagerError::Timeout),
                Err(TryRecvError::Disconnected) => Err(NetworkManagerError::Closed),
            }
        } else {
            match inbound.recv_timeout(*readTimeout) {
                Ok(packet) => Ok(packet),
                Err(RecvTimeoutError::Timeout) => Err(NetworkManagerError::Timeout),
                Err(RecvTimeoutError::Disconnected) => Err(NetworkManagerError::Closed),
            }
        };
        if matches!(&received, Err(NetworkManagerError::Closed)) {
            open.store(false, Ordering::Release);
            self.channelOpen = false;
        }
        received
    }

    fn readTcpPacket(&mut self) -> Result<RawPacket, NetworkManagerError> {
        let mut lengthBytes = Vec::with_capacity(5);
        let packetLength = loop {
            if lengthBytes.len() >= 5 { return Err(NetworkManagerError::Codec(CodecError::VarIntTooLarge)); }
            let byte = self.readNetworkByte(lengthBytes.is_empty())?;
            lengthBytes.push(byte);
            if byte & 0x80 == 0 {
                let mut view = lengthBytes.as_slice();
                break read_var_i32(&mut view)?;
            }
        };
        if packetLength < 0 { return Err(NetworkManagerError::Codec(CodecError::NegativeLength(packetLength))); }
        if packetLength as usize > 2 * 1024 * 1024 {
            return Err(NetworkManagerError::Codec(CodecError::PacketTooLarge { actual: packetLength as usize, maximum: 2 * 1024 * 1024 }));
        }
        let mut body = vec![0_u8; packetLength as usize];
        self.readNetworkExact(&mut body, false)?;
        let mut frame = lengthBytes;
        frame.extend_from_slice(&body);
        let mut view = frame.as_slice();
        Ok(self.codec.decode(&mut view)?)
    }

    pub fn enableEncryption(&mut self, secretKey: &SecretKey) {
        // MCP never installs encryption handlers on a LocalChannel.
        if self.isLocalChannel() { return; }
        self.encryptor = Some(NetCipher::new(secretKey, true));
        self.decryptor = Some(NetCipher::new(secretKey, false));
    }

    pub fn setCompressionThreshold(&mut self, threshold: i32) {
        // NetHandlerLoginServer suppresses SPacketEnableCompression for local
        // connections. Keep this guard as a second source-shaped invariant.
        if self.isLocalChannel() { return; }
        self.codec.set_compression_threshold(if threshold >= 0 { Some(threshold as usize) } else { None });
    }

    pub fn setReadTimeout(&mut self, timeout: Duration) -> Result<(), NetworkManagerError> {
        match &mut self.transport {
            NetworkTransport::Tcp(stream) => stream.set_read_timeout(Some(timeout))?,
            NetworkTransport::Local { readTimeout, .. } => *readTimeout = timeout,
        }
        Ok(())
    }

    /// Rust-side identity for associating Netty-channel-equivalent handler state.
    /// This never appears on the Minecraft protocol or in persisted world data.
    pub const fn id(&self) -> u64 { self.managerId }

    pub fn setConnectionState(&mut self, state: ConnectionState) { self.connectionState = state; }
    pub const fn getConnectionState(&self) -> ConnectionState { self.connectionState }
    pub fn isChannelOpen(&self) -> bool {
        self.channelOpen && match &self.transport {
            NetworkTransport::Tcp(_) => true,
            NetworkTransport::Local { open, .. } => open.load(Ordering::Acquire),
        }
    }
    /// MCP `NetworkManager#isEncrypted`, used by GuiPlayerTabOverlay to
    /// decide whether the authenticated player-head branch is available.
    pub const fn isEncrypted(&self) -> bool { self.encryptor.is_some() }
    /// MCP `NetworkManager#isLocalChannel`.
    pub const fn isLocalChannel(&self) -> bool { matches!(&self.transport, NetworkTransport::Local { .. }) }
    pub const fn getRemoteAddress(&self) -> SocketAddr { self.socketAddress }
    pub fn getLocalEndpointId(&self) -> Option<u64> {
        match &self.transport {
            NetworkTransport::Local { endpointId, .. } => Some(*endpointId),
            NetworkTransport::Tcp(_) => None,
        }
    }

    pub fn closeChannel(&mut self) {
        self.channelOpen = false;
        match &mut self.transport {
            NetworkTransport::Tcp(stream) => { let _ = stream.shutdown(std::net::Shutdown::Both); }
            NetworkTransport::Local { open, .. } => open.store(false, Ordering::Release),
        }
    }

    fn readNetworkByte(&mut self, allowIdleTimeout: bool) -> Result<u8, NetworkManagerError> {
        let mut byte = [0_u8; 1];
        self.readNetworkExact(&mut byte, allowIdleTimeout)?;
        Ok(byte[0])
    }

    /// Reads exactly one already-started network segment without discarding
    /// protocol bytes when the socket timeout expires. An idle timeout is only
    /// surfaced before the first byte of a new packet; once a VarInt prefix or
    /// body has started, the same call retains its local progress until the
    /// frame is complete.
    fn readNetworkExact(&mut self, output: &mut [u8], allowIdleTimeout: bool) -> Result<(), NetworkManagerError> {
        let NetworkTransport::Tcp(stream) = &mut self.transport else {
            return Err(NetworkManagerError::Io(io::Error::new(io::ErrorKind::InvalidInput, "byte reads are not used by LocalChannel")));
        };
        let mut offset = 0_usize;
        while offset < output.len() {
            match stream.read(&mut output[offset..]) {
                Ok(0) => { self.channelOpen = false; return Err(NetworkManagerError::Closed); }
                Ok(read) => {
                    if let Some(cipher) = self.decryptor.as_mut() { cipher.apply(&mut output[offset..offset + read]); }
                    offset += read;
                }
                Err(error) if matches!(error.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut) => {
                    if offset == 0 && allowIdleTimeout {
                        return Err(NetworkManagerError::Timeout);
                    }
                }
                Err(error) if matches!(error.kind(), io::ErrorKind::UnexpectedEof | io::ErrorKind::ConnectionReset | io::ErrorKind::ConnectionAborted | io::ErrorKind::BrokenPipe) => {
                    self.channelOpen = false; return Err(NetworkManagerError::Closed);
                }
                Err(error) => return Err(NetworkManagerError::Io(error)),
            }
        }
        Ok(())
    }
}

impl Drop for NetworkManager { fn drop(&mut self) { self.closeChannel(); } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_channel_transfers_raw_packets_without_wire_codec() {
        let (address, requests) = createLocalEndpointChannel();
        let mut client = NetworkManager::provideLocalClient(&address).unwrap();
        let request = requests.recv().unwrap();
        let mut server = NetworkManager::fromLocalServer(request);
        assert!(client.isLocalChannel());
        assert!(server.isLocalChannel());
        assert_eq!(client.getLocalEndpointId(), Some(address.id()));
        assert_eq!(server.getLocalEndpointId(), Some(address.id()));
        assert_ne!(client.id(), server.id());

        let outbound = RawPacket::new(0x2A, vec![0x80, 0x00, 0xFF]);
        client.sendPacket(&outbound).unwrap();
        assert_eq!(server.readPacket().unwrap(), outbound);

        let reply = RawPacket::new(0x01, vec![1, 2, 3, 4]);
        server.sendPacket(&reply).unwrap();
        assert_eq!(client.readPacket().unwrap(), reply);
    }

    #[test]
    fn local_channel_ignores_compression_and_encryption_handlers() {
        let (address, requests) = createLocalEndpointChannel();
        let mut client = NetworkManager::provideLocalClient(&address).unwrap();
        let request = requests.recv().unwrap();
        let mut server = NetworkManager::fromLocalServer(request);
        client.setCompressionThreshold(1);
        server.setCompressionThreshold(1);
        assert!(!client.isEncrypted());
        assert!(!server.isEncrypted());
        let packet = RawPacket::new(0x00, vec![0; 128]);
        client.sendPacket(&packet).unwrap();
        assert_eq!(server.readPacket().unwrap(), packet);
    }

    #[test]
    fn closing_one_local_side_closes_the_shared_channel() {
        let (address, requests) = createLocalEndpointChannel();
        let mut client = NetworkManager::provideLocalClient(&address).unwrap();
        let request = requests.recv().unwrap();
        let server = NetworkManager::fromLocalServer(request);
        assert!(client.isChannelOpen());
        assert!(server.isChannelOpen());
        client.closeChannel();
        assert!(!client.isChannelOpen());
        assert!(!server.isChannelOpen());
    }
}
