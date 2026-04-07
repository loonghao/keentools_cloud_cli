#!/usr/bin/env bash
# keentools-cloud installer — downloads the latest (or a pinned) release binary.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/loonghao/keentools_cloud_cli/main/install.sh | bash
#   curl -fsSL https://raw.githubusercontent.com/loonghao/keentools_cloud_cli/main/install.sh | bash -s -- v0.2.0
#
# Environment overrides:
#   KEENTOOLS_INSTALL_DIR        – where to put the binary  (default: ~/.local/bin)
#   KEENTOOLS_INSTALL_VERSION    – version to install       (default: latest)
#   KEENTOOLS_INSTALL_REPOSITORY – GitHub owner/repo        (default: loonghao/keentools_cloud_cli)

set -euo pipefail

REPO="${KEENTOOLS_INSTALL_REPOSITORY:-loonghao/keentools_cloud_cli}"
INSTALL_DIR="${KEENTOOLS_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${1:-${KEENTOOLS_INSTALL_VERSION:-latest}}"

# ---------- helpers -----------------------------------------------------------

die() { echo "ERROR: $*" >&2; exit 1; }

need() {
  command -v "$1" > /dev/null 2>&1 || die "'$1' is required but not found"
}

# ---------- detect platform ---------------------------------------------------

need curl
need tar

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)  OS_TAG="unknown-linux-gnu" ;;
  Darwin) OS_TAG="apple-darwin" ;;
  *)      die "Unsupported OS: $OS" ;;
esac

case "$ARCH" in
  x86_64|amd64)  ARCH_TAG="x86_64" ;;
  aarch64|arm64) ARCH_TAG="aarch64" ;;
  *)             die "Unsupported architecture: $ARCH" ;;
esac

TARGET="${ARCH_TAG}-${OS_TAG}"

# ---------- resolve download URL ----------------------------------------------

if [ "$VERSION" = "latest" ]; then
  URL="https://github.com/${REPO}/releases/latest/download/keentools-cloud-${TARGET}.tar.gz"
else
  # Ensure v-prefix
  case "$VERSION" in
    v*) ;;
    *)  VERSION="v${VERSION}" ;;
  esac
  URL="https://github.com/${REPO}/releases/download/${VERSION}/keentools-cloud-${VERSION}-${TARGET}.tar.gz"
fi

echo "-> Downloading keentools-cloud (${VERSION}) for ${TARGET}..."
echo "   ${URL}"

# ---------- download & install ------------------------------------------------

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

curl -fSL --retry 3 "$URL" -o "$TMP/keentools-cloud.tar.gz"
tar -xzf "$TMP/keentools-cloud.tar.gz" -C "$TMP"

mkdir -p "$INSTALL_DIR"
install -m 755 "$TMP/keentools-cloud" "$INSTALL_DIR/keentools-cloud"

echo "✓ Installed keentools-cloud to ${INSTALL_DIR}/keentools-cloud"

# ---------- PATH hint ---------------------------------------------------------

case ":${PATH}:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    echo ""
    echo "⚠  ${INSTALL_DIR} is not in your PATH."
    echo "   Add it with:"
    echo ""
    echo "     export PATH=\"${INSTALL_DIR}:\$PATH\""
    echo ""
    ;;
esac

echo ""
echo "Get started:"
echo "  export KEENTOOLS_API_URL=https://your-api-endpoint.example.com"
echo "  export KEENTOOLS_API_TOKEN=your-token-here"
echo "  keentools-cloud --help"
