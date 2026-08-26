use std::net::IpAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerAddress {
    ipAddress: String,
    serverPort: u16,
}

impl ServerAddress {
    fn new(address: impl Into<String>, port: u16) -> Self {
        Self {
            ipAddress: address.into(),
            serverPort: port,
        }
    }
    pub fn getIP(&self) -> String {
        if self.ipAddress.parse::<IpAddr>().is_ok() {
            return self.ipAddress.clone();
        }
        idna::domain_to_ascii(&self.ipAddress).unwrap_or_default()
    }
    pub const fn getPort(&self) -> u16 {
        self.serverPort
    }

    pub fn fromString(addrString: &str) -> Self {
        let addrString = addrString.trim();
        let (host, portText) = if let Some(rest) = addrString.strip_prefix('[') {
            if let Some(close) = rest.find(']') {
                let host = &rest[..close];
                let suffix = rest[close + 1..].trim();
                (host, suffix.strip_prefix(':'))
            } else {
                (addrString, None)
            }
        } else {
            let colonCount = addrString.bytes().filter(|value| *value == b':').count();
            if colonCount == 1 {
                let (host, port) = addrString.split_once(':').unwrap_or((addrString, ""));
                (host, Some(port))
            } else {
                (addrString, None)
            }
        };
        let port = portText
            .and_then(|value| value.trim().parse::<u16>().ok())
            .unwrap_or(25565);
        // MCP performs an SRV lookup when the selected port is 25565. The
        // address parser remains deterministic here; ServerPinger owns the
        // resolver step so it can run off the GUI thread.
        Self::new(host, port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_vanilla_address_forms() {
        assert_eq!(ServerAddress::fromString("example.org").getPort(), 25565);
        assert_eq!(
            ServerAddress::fromString("example.org:25570").getPort(),
            25570
        );
        assert_eq!(ServerAddress::fromString("[::1]:25566").getIP(), "::1");
        assert_eq!(
            ServerAddress::fromString("2001:db8::1").getIP(),
            "2001:db8::1"
        );
    }
}
