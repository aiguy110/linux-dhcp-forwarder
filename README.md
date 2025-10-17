# Linux DHCP Forwarder

A one-way DHCP packet forwarder written in Rust, designed to run on DHCP servers to send copies of DHCP requests to remote servers for informational purposes.

## Important: This is NOT a DHCP Relay

This service is a **one-way packet forwarder**, not a traditional DHCP relay:
- It captures copies of DHCP requests and forwards them to remote servers
- It does **NOT** send DHCP offers or ACKs back to clients
- It is designed to run alongside your existing DHCP server
- Typical use case: Forwarding DHCP requests to NAC (Network Access Control) servers for device fingerprinting and monitoring

## Use Cases

- **NAC Integration**: Forward DHCP requests to NAC servers for device fingerprinting
- **Network Monitoring**: Send copies of DHCP traffic to monitoring/analytics systems
- **Security Analysis**: Forward requests to security appliances for threat detection
- **Audit Logging**: Maintain centralized logs of DHCP activity across multiple servers

## Features

- Captures DHCP packets using libpcap
- One-way forwarding of DHCP request copies to remote servers
- JSON-based configuration
- Command-line argument support
- Systemd service integration
- RPM packaging support
- Comprehensive logging
- Minimal performance impact on DHCP server

## Requirements

- Rust 1.70 or later
- libpcap development libraries
- Linux system with network interfaces

## Installation

### From Source

```bash
cargo build --release
sudo cp target/release/linux-dhcp-forwarder /usr/local/bin/
sudo mkdir -p /etc/dhcp-forwarder
sudo cp config.example.json /etc/dhcp-forwarder/config.json
```

### From RPM

```bash
# Build the RPM using the provided script
./build-rpm.sh

# Install the RPM
sudo rpm -ivh ~/rpmbuild/RPMS/x86_64/linux-dhcp-forwarder-*.rpm
```

## Configuration

Create a configuration file at `/etc/dhcp-forwarder/config.json`:

```json
{
  "remote_ip": "192.168.1.100",
  "remote_port": 67,
  "interface": "eth0"
}
```

### Configuration Options

- `remote_ip`: The IP address of the remote server to forward packet copies to (e.g., NAC server)
- `remote_port`: The UDP port on the remote server (default: 67)
- `interface`: The network interface to listen on (e.g., "eth0", "ens33") - typically the interface where your DHCP server is listening

## Usage

### Command Line

```bash
# Run with default configuration
sudo linux-dhcp-forwarder

# Specify custom config file
sudo linux-dhcp-forwarder --config /path/to/config.json

# Override config with command-line arguments
sudo linux-dhcp-forwarder --interface eth1 --remote-ip 10.0.0.1

# Enable verbose logging
sudo linux-dhcp-forwarder --verbose

# View help
linux-dhcp-forwarder --help
```

### Systemd Service

```bash
# Start the service
sudo systemctl start linux-dhcp-forwarder

# Enable on boot
sudo systemctl enable linux-dhcp-forwarder

# Check status
sudo systemctl status linux-dhcp-forwarder

# View logs
sudo journalctl -u linux-dhcp-forwarder -f
```

## Building RPM

To build an RPM package:

1. Install required build dependencies:
   ```bash
   sudo dnf install rpm-build rust cargo libpcap-devel gcc
   ```

2. Use the build script (recommended):
   ```bash
   ./build-rpm.sh
   ```

   Or manually:
   ```bash
   mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
   tar czf ~/rpmbuild/SOURCES/linux-dhcp-forwarder-0.1.0.tar.gz \
     --transform 's,^,linux-dhcp-forwarder-0.1.0/,' \
     Cargo.toml Cargo.lock src/ config.example.json linux-dhcp-forwarder.service
   rpmbuild -ba linux-dhcp-forwarder.spec
   ```

3. Install the RPM:
   ```bash
   sudo rpm -ivh ~/rpmbuild/RPMS/x86_64/linux-dhcp-forwarder-*.rpm
   ```

## Security Considerations

- The service requires `CAP_NET_RAW` and `CAP_NET_ADMIN` capabilities to capture packets
- Run with appropriate permissions (typically requires root or specific capabilities)
- The systemd service is configured with security hardening options
- This is a **read-only** service that only captures and forwards copies - it does not modify network traffic
- Ensure your firewall allows UDP traffic to the remote destination
- The forwarder only sends data one-way to the configured remote server
- No replies are received or processed from remote servers

## Troubleshooting

### Permission Denied

If you get permission errors, ensure you're running with sufficient privileges:

```bash
sudo linux-dhcp-forwarder
```

### Interface Not Found

List available interfaces:

```bash
ip link show
```

Update your config file with the correct interface name.

### No Packets Captured

- Verify DHCP traffic is actually present on the interface
- Check that the interface is up: `ip link show <interface>`
- Use tcpdump to verify DHCP packets: `sudo tcpdump -i <interface> port 67 or port 68`
- Ensure your DHCP server is running and receiving requests
- Verify the forwarder is listening on the same interface as your DHCP server

### Packets Not Reaching Remote Server

- Check network connectivity to the remote server: `ping <remote_ip>`
- Verify firewall rules allow outbound UDP traffic
- Check the remote server is listening on the configured port
- Use tcpdump on the remote server to verify packets are arriving: `sudo tcpdump -i <interface> port 67`
- Remember: This is one-way forwarding only - the forwarder does not expect or process any responses

## Development

### Running Tests

```bash
cargo test
```

### Building

```bash
cargo build --release
```

### Debugging

Enable verbose logging:

```bash
sudo RUST_LOG=debug linux-dhcp-forwarder --verbose
```

## How It Works

1. **Packet Capture**: The forwarder uses libpcap to capture all traffic on the specified network interface
2. **DHCP Filtering**: It applies a BPF (Berkeley Packet Filter) to only capture UDP traffic on ports 67 and 68
3. **Packet Parsing**: Each captured packet is parsed to extract the DHCP payload from the Ethernet/IP/UDP headers
4. **One-Way Forwarding**: The DHCP payload is sent via UDP socket to the configured remote server
5. **No Response Handling**: The forwarder does not listen for or process any responses - it's purely informational

This design ensures minimal impact on DHCP server performance while providing real-time packet copies to monitoring systems.

## License

Licensed under either of:

- MIT license
- Apache License, Version 2.0

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
