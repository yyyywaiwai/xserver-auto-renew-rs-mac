#!/usr/bin/env bash
set -euo pipefail

# simple colors for a slightly fancier output
GREEN="\033[0;32m"
BOLD="\033[1m"
RESET="\033[0m"

repo="h-sumiya/xserver-auto-renew-rs"
version="${VERSION:-latest}"

os=$(uname)
arch=$(uname -m)

case "$os" in
  Linux)
    case "$arch" in
      x86_64) target="x86_64-unknown-linux-gnu" ;;
      aarch64|arm64) target="aarch64-unknown-linux-gnu" ;;
      *) echo "Unsupported Linux architecture: $arch" >&2; exit 1 ;;
    esac
    
    if [ "$version" = "latest" ]; then
      version=$(curl -sSfL "https://api.github.com/repos/$repo/releases/latest" | grep -Po '"tag_name":\s*"\K[^"]+')
    fi

    echo -e "${BOLD}Installing xrenew ${version} for ${target}...${RESET}"
    tmpdir=$(mktemp -d)
    url="https://github.com/$repo/releases/download/$version/xrenew-${target}.tar.gz"
    echo -e "${BOLD}Downloading${RESET} $url"
    curl -sSfL "$url" | tar -xz -C "$tmpdir"
    install -Dm755 "$tmpdir/xrenew" "$HOME/.local/bin/xrenew" || sudo install -Dm755 "$tmpdir/xrenew" /usr/local/bin/xrenew
    rm -rf "$tmpdir"
    ;;
    
  Darwin)
    case "$arch" in
      x86_64|arm64) ;;
      *) echo "Unsupported macOS architecture: $arch" >&2; exit 1 ;;
    esac
    
    echo -e "${BOLD}Installing xrenew for macOS ${arch}...${RESET}"
    
    # Check if we're in a git repository
    if [ -d ".git" ]; then
      echo -e "${BOLD}Building from source...${RESET}"
      if ! command -v cargo >/dev/null 2>&1; then
        echo "Rust/Cargo not found. Please install Rust first:" >&2
        echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" >&2
        exit 1
      fi
      cargo build --release
      mkdir -p "$HOME/.local/bin"
      cp "target/release/xrenew" "$HOME/.local/bin/xrenew"
    else
      echo "For macOS installation, please clone the repository and run install.sh from the project directory" >&2
      echo "git clone https://github.com/$repo.git && cd xserver-auto-renew-rs && ./install.sh" >&2
      exit 1
    fi
    ;;
    
  *)
    echo "Unsupported operating system: $os" >&2
    echo "This installer supports Linux and macOS only" >&2
    exit 1
    ;;
esac

hash -r 2>/dev/null || true

if [ -x "$HOME/.local/bin/xrenew" ]; then
  export PATH="$HOME/.local/bin:$PATH"
  "$HOME/.local/bin/xrenew" refresh || true
elif [ -x "/usr/local/bin/xrenew" ]; then
  "/usr/local/bin/xrenew" refresh || true
else
  echo "Warning: xrenew not found in expected locations" >&2
fi

printf "${GREEN}${BOLD}xrenew installed!${RESET}\n"
echo -e "まず ${BOLD}xrenew login${RESET}\n次に ${BOLD}xrenew enable${RESET} を実行してください。"
