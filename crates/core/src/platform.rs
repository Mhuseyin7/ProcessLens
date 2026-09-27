use crate::{NetworkConnection, PortBinding, Protocol};
use std::{io, process::Command};

pub trait PlatformPortProvider: Send + Sync {
    fn list_ports(&self) -> io::Result<Vec<PortBinding>>;
}
pub trait PlatformNetworkProvider: Send + Sync {
    fn list_connections(&self) -> io::Result<Vec<NetworkConnection>>;
}

pub struct SystemPortProvider;

impl PlatformPortProvider for SystemPortProvider {
    fn list_ports(&self) -> io::Result<Vec<PortBinding>> {
        #[cfg(windows)]
        {
            windows_netstat_ports()
        }
        #[cfg(not(windows))]
        {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Port inspection adapter is not implemented for this platform yet",
            ))
        }
    }
}
impl PlatformNetworkProvider for SystemPortProvider {
    fn list_connections(&self) -> io::Result<Vec<NetworkConnection>> {
        #[cfg(windows)]
        {
            windows_netstat_connections()
        }
        #[cfg(not(windows))]
        {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Network inspection adapter is not implemented for this platform yet",
            ))
        }
    }
}

#[cfg(windows)]
fn netstat_lines() -> io::Result<Vec<String>> {
    let output = Command::new("netstat").args(["-ano"]).output()?;
    if !output.status.success() {
        return Err(io::Error::other("netstat failed"));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_owned)
        .collect())
}
#[cfg(windows)]
fn windows_netstat_ports() -> io::Result<Vec<PortBinding>> {
    Ok(netstat_lines()?
        .iter()
        .filter_map(|line| parse_windows_port(line))
        .collect())
}
#[cfg(windows)]
fn windows_netstat_connections() -> io::Result<Vec<NetworkConnection>> {
    Ok(netstat_lines()?
        .iter()
        .filter_map(|line| parse_windows_connection(line))
        .collect())
}
#[cfg(windows)]
fn parse_windows_port(line: &str) -> Option<PortBinding> {
    let bits: Vec<_> = line.split_whitespace().collect();
    let protocol = match bits.first()?.to_ascii_uppercase().as_str() {
        "TCP" => Protocol::Tcp,
        "UDP" => Protocol::Udp,
        _ => return None,
    };
    let required = if protocol == Protocol::Tcp { 5 } else { 4 };
    if bits.len() < required {
        return None;
    }
    let endpoint = bits[1];
    let separator = endpoint.rfind(':')?;
    let local_port = endpoint[separator + 1..].parse().ok()?;
    let (state, pid) = if protocol == Protocol::Tcp {
        (bits[3].to_owned(), bits[4].parse().ok())
    } else {
        ("BOUND".to_owned(), bits[3].parse().ok())
    };
    Some(PortBinding {
        protocol,
        local_address: endpoint[..separator].trim_matches(['[', ']']).to_owned(),
        local_port,
        state,
        pid,
    })
}
#[cfg(windows)]
fn parse_windows_connection(line: &str) -> Option<NetworkConnection> {
    let bits: Vec<_> = line.split_whitespace().collect();
    if bits.len() < 5 || !bits.first()?.eq_ignore_ascii_case("TCP") {
        return None;
    }
    Some(NetworkConnection {
        protocol: Protocol::Tcp,
        local_endpoint: bits[1].to_owned(),
        remote_endpoint: bits[2].to_owned(),
        state: Some(bits[3].to_owned()),
        pid: bits[4].parse().ok(),
        address_family: if bits[1].starts_with('[') {
            "IPv6".into()
        } else {
            "IPv4".into()
        },
    })
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    #[test]
    fn parses_tcp_and_udp() {
        let tcp = parse_windows_port("  TCP    127.0.0.1:3000  0.0.0.0:0 LISTENING  123").unwrap();
        assert_eq!(tcp.local_port, 3000);
        assert_eq!(tcp.pid, Some(123));
        let udp = parse_windows_port("  UDP    0.0.0.0:5353  *:*  321").unwrap();
        assert_eq!(udp.protocol, Protocol::Udp);
    }
}
