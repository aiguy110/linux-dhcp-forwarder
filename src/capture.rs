use anyhow::{Context, Result};
use log::{debug, info, warn};
use pcap::{Capture, Device};

const DHCP_SERVER_PORT: u16 = 67;
const DHCP_CLIENT_PORT: u16 = 68;

/// DHCP packet structure for identifying DHCP messages
#[derive(Debug)]
pub struct DhcpPacket {
    pub data: Vec<u8>,
    pub timestamp: std::time::SystemTime,
}

pub struct PacketCapture {
    interface: String,
}

impl PacketCapture {
    pub fn new(interface: String) -> Self {
        PacketCapture { interface }
    }

    /// Start capturing DHCP packets on the specified interface
    pub fn start_capture<F>(&self, mut callback: F) -> Result<()>
    where
        F: FnMut(DhcpPacket) -> Result<()>,
    {
        info!("Starting packet capture on interface: {}", self.interface);

        // Find the device
        let device = Device::list()?
            .into_iter()
            .find(|d| d.name == self.interface)
            .with_context(|| format!("Interface {} not found", self.interface))?;

        info!(
            "Found device: {} ({})",
            device.name,
            device
                .desc
                .as_ref()
                .unwrap_or(&"no description".to_string())
        );

        // Open the device for capturing
        let mut cap = Capture::from_device(device)?
            .promisc(true)
            .snaplen(1500)
            .timeout(1000)
            .open()?;

        // Set BPF filter to capture only DHCP traffic (UDP port 67 and 68)
        cap.filter("udp and (port 67 or port 68)", true)?;

        info!("Packet capture started. Listening for DHCP packets...");

        // Start capturing packets
        loop {
            match cap.next_packet() {
                Ok(packet) => {
                    debug!("Captured packet: {} bytes", packet.data.len());

                    if let Some(dhcp_packet) = self.parse_packet(packet.data) {
                        info!("DHCP packet detected: {} bytes", dhcp_packet.data.len());

                        if let Err(e) = callback(dhcp_packet) {
                            warn!("Error processing packet: {}", e);
                        }
                    }
                }
                Err(pcap::Error::TimeoutExpired) => {
                    // Timeout is normal, just continue
                    continue;
                }
                Err(e) => {
                    warn!("Error capturing packet: {}", e);
                }
            }
        }
    }

    /// Parse a raw packet and extract DHCP data if it's a DHCP packet
    fn parse_packet(&self, data: &[u8]) -> Option<DhcpPacket> {
        // Minimum Ethernet + IP + UDP headers = 14 + 20 + 8 = 42 bytes
        if data.len() < 42 {
            return None;
        }

        // Check if it's an IPv4 packet (EtherType 0x0800)
        let ethertype = u16::from_be_bytes([data[12], data[13]]);
        if ethertype != 0x0800 {
            return None;
        }

        // Get IP header length
        let ip_header_len = ((data[14] & 0x0F) * 4) as usize;

        // Check if it's UDP (protocol 17)
        let protocol = data[23];
        if protocol != 17 {
            return None;
        }

        // Calculate UDP header position
        let udp_start = 14 + ip_header_len;
        if data.len() < udp_start + 8 {
            return None;
        }

        // Extract source and destination ports
        let src_port = u16::from_be_bytes([data[udp_start], data[udp_start + 1]]);
        let dst_port = u16::from_be_bytes([data[udp_start + 2], data[udp_start + 3]]);

        // Check if it's DHCP traffic (ports 67 or 68)
        if (src_port == DHCP_CLIENT_PORT && dst_port == DHCP_SERVER_PORT)
            || (src_port == DHCP_SERVER_PORT && dst_port == DHCP_CLIENT_PORT)
        {
            debug!(
                "DHCP packet found: src_port={}, dst_port={}",
                src_port, dst_port
            );

            return Some(DhcpPacket {
                data: data.to_vec(),
                timestamp: std::time::SystemTime::now(),
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_capture_creation() {
        let capture = PacketCapture::new("eth0".to_string());
        assert_eq!(capture.interface, "eth0");
    }
}
