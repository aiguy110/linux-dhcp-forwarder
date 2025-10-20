Name:           linux-dhcp-forwarder
Version:        0.1.0
Release:        1%{?dist}
Summary:        A DHCP packet forwarder for informational monitoring and device fingerprinting

License:        MIT OR Apache-2.0
URL:            https://github.com/yourusername/linux-dhcp-forwarder
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  libpcap-devel
BuildRequires:  gcc

Requires:       libpcap

%description
A one-way DHCP packet forwarder designed to run on DHCP servers. It uses
libpcap to capture copies of DHCP requests and forwards them to remote
servers for informational purposes such as NAC device fingerprinting.
This is NOT a DHCP relay - it does not send offers or ACKs back to clients.
The service reads its configuration from a JSON file and can be deployed
as a system service.

%prep
%setup -q

%build
# Binary pre-built in CI pipeline

%install
rm -rf $RPM_BUILD_ROOT
mkdir -p $RPM_BUILD_ROOT%{_bindir}
mkdir -p $RPM_BUILD_ROOT%{_sysconfdir}/dhcp-forwarder
mkdir -p $RPM_BUILD_ROOT%{_unitdir}

# Install binary
install -m 0755 target/release/linux-dhcp-forwarder $RPM_BUILD_ROOT%{_bindir}/linux-dhcp-forwarder

# Install example config
install -m 0644 config.example.json $RPM_BUILD_ROOT%{_sysconfdir}/dhcp-forwarder/config.json

# Install systemd service file
install -m 0644 linux-dhcp-forwarder.service $RPM_BUILD_ROOT%{_unitdir}/linux-dhcp-forwarder.service

%files
%{_bindir}/linux-dhcp-forwarder
%config(noreplace) %{_sysconfdir}/dhcp-forwarder/config.json
%{_unitdir}/linux-dhcp-forwarder.service

%post
%systemd_post linux-dhcp-forwarder.service

%preun
%systemd_preun linux-dhcp-forwarder.service

%postun
%systemd_postun_with_restart linux-dhcp-forwarder.service

%changelog
* Fri Oct 17 2025 Your Name <your.email@example.com> - 0.1.0-1
- Initial RPM release
- One-way DHCP packet forwarding for informational purposes
- DHCP packet capture using libpcap
- Configurable remote forwarding to NAC or monitoring servers
- Systemd service support
