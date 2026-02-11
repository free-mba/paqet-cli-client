use anyhow::{Context, Result};
use default_net;
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Config {
    role: String,
    log: LogConfig,
    socks5: Vec<Socks5Config>,
    network: NetworkConfig,
    server: ServerConfig,
    transport: TransportConfig,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct LogConfig {
    level: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Socks5Config {
    listen: String,
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct NetworkConfig {
    interface: String,
    ipv4: Ipv4Config,
    ipv6: Ipv6Config,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Ipv4Config {
    addr: String,
    router_mac: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Ipv6Config {
    addr: String,
    router_mac: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ServerConfig {
    addr: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TransportConfig {
    protocol: String,
    conn: u32,
    kcp: KcpConfig,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct KcpConfig {
    mode: String,
    key: String,
}

fn get_gateway_mac(ip: &str, _is_ipv6: bool) -> Result<String> {
    // Cross-platform MAC discovery logic
    #[cfg(target_os = "macos")]
    {
        // macOS: arp -a or ndp -a
        let cmd = if _is_ipv6 { "ndp" } else { "arp" };
        let output = Command::new(cmd).arg("-a").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains(ip) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                for part in parts {
                    if part.contains(':') && part.len() >= 11 {
                        return Ok(part.to_string());
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: ip neighbor or /proc/net/arp
        let output = Command::new("ip").args(&["neighbor", "show", ip]).output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = stdout.lines().next() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(pos) = parts.iter().position(|&p| p == "lladdr") {
                if let Some(mac) = parts.get(pos + 1) {
                    return Ok(mac.to_string());
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: arp -a or Get-NetNeighbor
        let output = Command::new("arp").args(&["-a", ip]).output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains(ip) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                for part in parts {
                    if part.contains('-') && part.len() >= 17 {
                        return Ok(part.replace("-", ":").to_lowercase());
                    }
                }
            }
        }
    }

    Ok("aa:bb:cc:dd:ee:ff".to_string())
}

fn main() -> Result<()> {
    println!("🔍 Discovering network settings...");

    let interfaces = default_net::get_interfaces();
    let default_iface = default_net::get_default_interface().map_err(|e| anyhow::anyhow!(e)).context("Could not find default interface")?;
    
    let interface = interfaces.iter()
        .find(|iface| (iface.name.starts_with("en") || iface.name.starts_with("eth") || iface.name.starts_with("wlan")) && !iface.ipv4.is_empty())
        .unwrap_or(&default_iface);

    let gateway = default_net::get_default_gateway().map_err(|e| anyhow::anyhow!(e)).context("Could not find default gateway")?;

    let iface_name = interface.name.clone();
    let ipv4_addr = interface.ipv4.first()
        .map(|net| format!("{}:0", net.addr))
        .unwrap_or_else(|| "127.0.0.1:0".to_string());
    
    let ipv6_addr = interface.ipv6.iter()
        .find(|net| net.addr.is_unicast_link_local())
        .map(|net| format!("[{}]:0", net.addr))
        .unwrap_or_else(|| "[::1]:0".to_string());

    let gw_ip = gateway.ip_addr.to_string();
    let gw_mac = gateway.mac_addr.to_string();

    println!("✅ Found Interface: {}", iface_name);
    println!("✅ Found IPv4: {}", ipv4_addr);
    println!("✅ Found IPv6: {}", ipv6_addr);

    let gw_mac_v4 = if !gw_ip.is_empty() {
        get_gateway_mac(&gw_ip, false).unwrap_or_else(|_| "aa:bb:cc:dd:ee:ff".to_string())
    } else {
        "aa:bb:cc:dd:ee:ff".to_string()
    };

    let gw_mac_v6 = if gw_mac != "00:00:00:00:00:00" && !gw_mac.is_empty() {
        gw_mac
    } else {
        gw_mac_v4.clone()
    };

    let config = Config {
        role: "client".to_string(),
        log: LogConfig { level: "info".to_string() },
        socks5: vec![Socks5Config {
            listen: "127.0.0.1:1080".to_string(),
            username: "".to_string(),
            password: "".to_string(),
        }],
        network: NetworkConfig {
            interface: iface_name,
            ipv4: Ipv4Config {
                addr: ipv4_addr,
                router_mac: gw_mac_v4,
            },
            ipv6: Ipv6Config {
                addr: ipv6_addr,
                router_mac: gw_mac_v6,
            },
        },
        server: ServerConfig {
            addr: "45.141.148.77:8443".to_string(),
        },
        transport: TransportConfig {
            protocol: "kcp".to_string(),
            conn: 1,
            kcp: KcpConfig {
                mode: "fast".to_string(),
                key: "RkCrATRTO0uAQQwCYBs26JDBaE1fzq2d".to_string(),
            },
        },
    };

    let yaml = serde_yaml::to_string(&config)?;
    let output_path = "auto_client.yaml";
    fs::write(output_path, yaml)?;

    println!("\n🚀 Generated configuration saved to: {}", output_path);
    
    #[cfg(target_os = "windows")]
    println!("You can now run paqet with:\n.\\paqet.exe run -c {}", output_path);
    
    #[cfg(not(target_os = "windows"))]
    println!("You can now run paqet with:\nsudo ./paqet run -c {}", output_path);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_serialization() {
        let config = Config {
            role: "client".to_string(),
            log: LogConfig { level: "info".to_string() },
            socks5: vec![Socks5Config {
                listen: "127.0.0.1:1080".to_string(),
                username: "".to_string(),
                password: "".to_string(),
            }],
            network: NetworkConfig {
                interface: "en1".to_string(),
                ipv4: Ipv4Config {
                    addr: "192.168.1.91:0".to_string(),
                    router_mac: "d4:01:c3:a6:36:71".to_string(),
                },
                ipv6: Ipv6Config {
                    addr: "[fe80::1]:0".to_string(),
                    router_mac: "30:a2:20:fe:46:18".to_string(),
                },
            },
            server: ServerConfig {
                addr: "1.2.3.4:8443".to_string(),
            },
            transport: TransportConfig {
                protocol: "kcp".to_string(),
                conn: 1,
                kcp: KcpConfig {
                    mode: "fast".to_string(),
                    key: "secret".to_string(),
                },
            },
        };

        let yaml = serde_yaml::to_string(&config).unwrap();
        let decoded: Config = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(config, decoded);
    }
}
