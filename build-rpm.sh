#!/bin/bash
set -e

echo "=== Building Linux DHCP Forwarder RPM ==="

# Check for required tools
if ! command -v rpmbuild &> /dev/null; then
    echo "Error: rpmbuild not found. Please install: sudo dnf install rpm-build"
    exit 1
fi

# Get version from Cargo.toml
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
echo "Building version: $VERSION"

# Create RPM build directory structure
echo "Creating RPM build directories..."
mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# Create source tarball
echo "Creating source tarball..."
tar czf ~/rpmbuild/SOURCES/linux-dhcp-forwarder-${VERSION}.tar.gz \
  --transform "s,^,linux-dhcp-forwarder-${VERSION}/," \
  --exclude='target' \
  --exclude='.git' \
  Cargo.toml Cargo.lock src/ config.example.json linux-dhcp-forwarder.service README.md

# Copy spec file
echo "Copying spec file..."
cp linux-dhcp-forwarder.spec ~/rpmbuild/SPECS/

# Build RPM
echo "Building RPM..."
rpmbuild -ba ~/rpmbuild/SPECS/linux-dhcp-forwarder.spec

echo ""
echo "=== Build Complete ==="
echo "RPM location: ~/rpmbuild/RPMS/x86_64/linux-dhcp-forwarder-${VERSION}-1.*.rpm"
echo ""
echo "To install:"
echo "  sudo rpm -ivh ~/rpmbuild/RPMS/x86_64/linux-dhcp-forwarder-${VERSION}-1.*.rpm"
