use std::sync::mpsc::{Receiver, TryRecvError};

use crate::net::minecraft::network::NetworkManager::{
    createLocalEndpointChannel, LocalConnectionRequest, LocalEndpointAddress, NetworkManager,
};

/// Source-shaped subset of MCP 1.12.2 `NetworkSystem`.
///
/// This tranche implements the local-memory endpoint used by integrated
/// single-player. TCP/LAN listening remains in the existing multiplayer
/// client/server work and is deliberately not substituted here.
#[derive(Debug)]
pub struct NetworkSystem {
    isAlive: bool,
    endpoints: Vec<Receiver<LocalConnectionRequest>>,
    networkManagers: Vec<NetworkManager>,
}

impl Default for NetworkSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkSystem {
    pub fn new() -> Self {
        Self {
            isAlive: true,
            endpoints: Vec::new(),
            networkManagers: Vec::new(),
        }
    }

    /// MCP `NetworkSystem#addLocalEndpoint`.
    ///
    /// The returned address can be handed to
    /// `NetworkManager::provideLocalClient`; the Rust local-event-loop pump accepts
    /// the local connection into this system's manager list.
    pub fn addLocalEndpoint(&mut self) -> LocalEndpointAddress {
        let (address, receiver) = createLocalEndpointChannel();
        self.endpoints.push(receiver);
        address
    }

    /// Rust event-loop equivalent of the `LocalServerChannel` child-handler
    /// callback in MCP `addLocalEndpoint`: accept newly connected local
    /// channels into the manager list. Packet-handler dispatch and dead-channel
    /// pruning belong to the later full `networkTick` port and are deliberately
    /// not folded into this accept pump.
    pub fn pollLocalEndpoints(&mut self) {
        if !self.isAlive {
            return;
        }
        for endpoint in &self.endpoints {
            loop {
                match endpoint.try_recv() {
                    Ok(request) => self.networkManagers.push(NetworkManager::fromLocalServer(request)),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => break,
                }
            }
        }
    }

    /// MCP `NetworkSystem#terminateEndpoints`: stop accepting new channels.
    /// Accepted child NetworkManagers are not force-closed here; vanilla also
    /// closes only the listening ChannelFuture endpoints in this method.
    pub fn terminateEndpoints(&mut self) {
        self.isAlive = false;
        for endpoint in &self.endpoints {
            while let Ok(request) = endpoint.try_recv() { request.close(); }
        }
        self.endpoints.clear();
    }

    pub const fn isAlive(&self) -> bool { self.isAlive }
    pub fn networkManagers(&self) -> &[NetworkManager] { &self.networkManagers }
    pub fn networkManagersMut(&mut self) -> &mut [NetworkManager] { &mut self.networkManagers }
    pub fn connectionCount(&self) -> usize { self.networkManagers.len() }
}

impl Drop for NetworkSystem {
    fn drop(&mut self) {
        self.terminateEndpoints();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::network::Packet::RawPacket;

    #[test]
    fn add_local_endpoint_accepts_local_network_manager_on_tick() {
        let mut system = NetworkSystem::new();
        let address = system.addLocalEndpoint();
        let mut client = NetworkManager::provideLocalClient(&address).unwrap();
        assert_eq!(system.connectionCount(), 0);
        system.pollLocalEndpoints();
        assert_eq!(system.connectionCount(), 1);
        assert!(system.networkManagers()[0].isLocalChannel());

        client.sendPacket(&RawPacket::new(0, vec![1, 2, 3])).unwrap();
        let packet = system.networkManagersMut()[0].readPacket().unwrap();
        assert_eq!(packet, RawPacket::new(0, vec![1, 2, 3]));
    }

    #[test]
    fn terminate_endpoints_stops_listening_without_stealing_child_lifetime() {
        let mut system = NetworkSystem::new();
        let address = system.addLocalEndpoint();
        let client = NetworkManager::provideLocalClient(&address).unwrap();
        system.pollLocalEndpoints();
        assert!(client.isChannelOpen());
        system.terminateEndpoints();
        assert!(!system.isAlive());
        assert!(client.isChannelOpen());
        assert_eq!(system.connectionCount(), 1);
        assert!(NetworkManager::provideLocalClient(&address).is_err());
    }

    #[test]
    fn terminate_endpoints_closes_unaccepted_local_connect_requests() {
        let mut system = NetworkSystem::new();
        let address = system.addLocalEndpoint();
        let client = NetworkManager::provideLocalClient(&address).unwrap();
        assert!(client.isChannelOpen());
        system.terminateEndpoints();
        assert!(!client.isChannelOpen());
    }
}
