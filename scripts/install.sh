#!/usr/bin/env bash
# ==============================================================================
# InstallMon Installation & Uninstallation Script
#
# Inspects host operating system, CPU architecture, Linux distribution, and
# C standard library (libc). Downloads, verifies SHA-256 checksums, and installs
# the appropriate InstallMon binary. Also provides a clean uninstallation method.
#
# Documentation: https://github.com/ankur3-101106/installmon
# ==============================================================================

set -euo pipefail

REPO="ankur3-101106/installmon"
GITHUB_BASE="https://github.com/${REPO}"
GITHUB_API="https://api.github.com/repos/${REPO}"
DEFAULT_VERSION="v0.1.0"

# CLI option defaults
CUSTOM_VERSION=""
CUSTOM_TARGET=""
CUSTOM_INSTALL_DIR=""
FORCE_SYSTEM=false
FORCE_USER=false
PREFER_STATIC=true
WITH_SERVICE=false
ACTION="install"
PURGE_DATA=false
DRY_RUN=false
USE_COLOR=true

# Disable colors if stdout is not a TTY or NO_COLOR is set
if [[ ! -t 1 ]] || [[ -n "${NO_COLOR:-}" ]]; then
    USE_COLOR=false
fi

# Color formatting helpers
if [[ "${USE_COLOR}" == "true" ]]; then
    C_RESET="\033[0m"
    C_BOLD="\033[1m"
    C_GREEN="\033[32m"
    C_BLUE="\033[34m"
    C_CYAN="\033[36m"
    C_YELLOW="\033[33m"
    C_RED="\033[31m"
else
    C_RESET=""
    C_BOLD=""
    C_GREEN=""
    C_BLUE=""
    C_CYAN=""
    C_YELLOW=""
    C_RED=""
fi

log_info() {
    echo -e "${C_BLUE}${C_BOLD}[INFO]${C_RESET} $*"
}

log_success() {
    echo -e "${C_GREEN}${C_BOLD}[OK]${C_RESET} $*"
}

log_warn() {
    echo -e "${C_YELLOW}${C_BOLD}[WARN]${C_RESET} $*" >&2
}

log_error() {
    echo -e "${C_RED}${C_BOLD}[ERROR]${C_RESET} $*" >&2
    exit 1
}

print_header() {
    echo -e "${C_CYAN}${C_BOLD}"
    cat <<'EOF'
  ___           _        _ _ __  __             
 |_ _|_ __  ___| |_ __ _| | |  \/  | ___  _ __  
  | || '_ \/ __| __/ _` | | | |\/| |/ _ \| '_ \ 
  | || | | \__ \ || (_| | | | |  | | (_) | | | |
 |___|_| |_|___/\__\__,_|_|_|_|  |_|\___/|_| |_|
EOF
    echo -e "${C_RESET}"
    echo -e "${C_BOLD}Linux Software Installation Monitoring & Provenance Tracking${C_RESET}"
    echo -e "Repository: ${GITHUB_BASE}"
    echo "----------------------------------------------------------------"
}

show_help() {
    cat <<EOF
InstallMon Installation and Uninstallation Script

USAGE:
  install.sh [OPTIONS] [COMMAND]

COMMANDS:
  install          Install InstallMon binary (default)
  uninstall        Uninstall InstallMon binary and systemd service units

OPTIONS:
  -v, --version <VER>     Specify InstallMon version tag (e.g. 'v0.1.0')
  -t, --target <TRIPLE>   Specify target triple (e.g. 'x86_64-unknown-linux-musl')
  -d, --dir <PATH>        Specify destination directory (e.g. '/usr/local/bin' or '~/.local/bin')
      --system            Force system-wide installation (/usr/local/bin, requires root/sudo)
      --user              Force user-level installation (~/.local/bin)
      --static            Prefer statically linked musl binary (default: true)
      --gnu               Prefer dynamically linked glibc binary
      --service           Automatically install, enable, and start background monitoring service
  -u, --uninstall         Uninstall InstallMon binary and systemd unit (same as 'uninstall' command)
      --purge             In combination with uninstall, delete database and log files as well
      --dry-run           Perform detection and print actions without modifying the filesystem
      --no-color          Disable ANSI color formatting
  -h, --help              Show this help message

EXAMPLES:
  # Standard user installation:
  curl -fsSL https://raw.githubusercontent.com/${REPO}/main/scripts/install.sh | bash

  # System-wide installation with root:
  sudo bash install.sh --system

  # Install specific version and auto-enable background service:
  bash install.sh --version v0.1.0 --service

  # Clean uninstall preserving database:
  bash install.sh --uninstall

  # Complete purge including SQLite databases:
  bash install.sh --uninstall --purge
EOF
    exit 0
}

# ==============================================================================
# Parse Command-Line Arguments
# ==============================================================================
parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            install)
                ACTION="install"
                shift
                ;;
            uninstall)
                ACTION="uninstall"
                shift
                ;;
            -v|--version)
                [[ $# -lt 2 ]] && log_error "Option '$1' requires a version argument."
                CUSTOM_VERSION="$2"
                shift 2
                ;;
            -t|--target)
                [[ $# -lt 2 ]] && log_error "Option '$1' requires a target triple argument."
                CUSTOM_TARGET="$2"
                shift 2
                ;;
            -d|--dir)
                [[ $# -lt 2 ]] && log_error "Option '$1' requires a directory argument."
                CUSTOM_INSTALL_DIR="$2"
                shift 2
                ;;
            --system)
                FORCE_SYSTEM=true
                shift
                ;;
            --user)
                FORCE_USER=true
                shift
                ;;
            --static)
                PREFER_STATIC=true
                shift
                ;;
            --gnu)
                PREFER_STATIC=false
                shift
                ;;
            --service)
                WITH_SERVICE=true
                shift
                ;;
            -u|--uninstall)
                ACTION="uninstall"
                shift
                ;;
            --purge)
                PURGE_DATA=true
                shift
                ;;
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --no-color)
                USE_COLOR=false
                C_RESET=""
                C_BOLD=""
                C_GREEN=""
                C_BLUE=""
                C_CYAN=""
                C_YELLOW=""
                C_RED=""
                shift
                ;;
            -h|--help)
                show_help
                ;;
            *)
                log_error "Unknown option or argument: '$1'. Use --help for usage information."
                ;;
        esac
    done
}

# ==============================================================================
# Dependency Check
# ==============================================================================
check_dependencies() {
    local missing=()
    for cmd in curl tar sha256sum; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            missing+=("$cmd")
        fi
    done

    if [[ ${#missing[@]} -gt 0 ]]; then
        log_error "Missing required utilities: ${missing[*]}. Please install them before proceeding."
    fi
}

# ==============================================================================
# Platform & System Detection
# ==============================================================================
detect_platform() {
    log_info "Detecting host environment and hardware architecture..."

    # 1. OS check
    HOST_OS="$(uname -s)"
    if [[ "${HOST_OS}" != "Linux" ]]; then
        log_error "InstallMon requires Linux. Detected operating system: '${HOST_OS}'."
    fi

    # 2. CPU Architecture check
    RAW_ARCH="$(uname -m)"
    case "${RAW_ARCH}" in
        x86_64|amd64)
            TARGET_ARCH="x86_64"
            ;;
        aarch64|arm64)
            TARGET_ARCH="aarch64"
            ;;
        *)
            log_error "Unsupported CPU architecture: '${RAW_ARCH}'. InstallMon provides prebuilt binaries for 'x86_64' and 'aarch64' (ARM 64-bit)."
            ;;
    esac

    # 3. Linux Distribution detection
    DISTRO_ID="unknown"
    DISTRO_NAME="Linux"
    DISTRO_VERSION=""
    if [[ -f /etc/os-release ]]; then
        # shellcheck disable=SC1091
        DISTRO_ID="$(grep -E '^ID=' /etc/os-release | cut -d= -f2 | tr -d '"')"
        DISTRO_NAME="$(grep -E '^NAME=' /etc/os-release | cut -d= -f2 | tr -d '"')"
        DISTRO_VERSION="$(grep -E '^VERSION_ID=' /etc/os-release | cut -d= -f2 | tr -d '"' || true)"
    elif [[ -f /usr/lib/os-release ]]; then
        DISTRO_ID="$(grep -E '^ID=' /usr/lib/os-release | cut -d= -f2 | tr -d '"')"
        DISTRO_NAME="$(grep -E '^NAME=' /usr/lib/os-release | cut -d= -f2 | tr -d '"')"
        DISTRO_VERSION="$(grep -E '^VERSION_ID=' /usr/lib/os-release | cut -d= -f2 | tr -d '"' || true)"
    fi

    # 4. Standard C Library (libc) detection
    DETECTED_LIBC="glibc"
    if [[ "${DISTRO_ID}" == "alpine" ]] || [[ -f /etc/alpine-release ]]; then
        DETECTED_LIBC="musl"
    elif ldd --version 2>&1 | grep -qi "musl"; then
        DETECTED_LIBC="musl"
    elif ls /lib/ld-musl* >/dev/null 2>&1 || ls /usr/lib/ld-musl* >/dev/null 2>&1; then
        DETECTED_LIBC="musl"
    fi

    # 5. Virtualization / Environment check
    DETECTED_ENV="bare_metal"
    if [[ -n "${WSL_DISTRO_NAME:-}" ]] || uname -r | grep -qi "microsoft"; then
        DETECTED_ENV="wsl2"
    elif [[ -f /.dockerenv ]]; then
        DETECTED_ENV="docker"
    elif [[ -f /run/.containerenv ]]; then
        DETECTED_ENV="podman"
    fi

    echo "  * Operating System : ${HOST_OS}"
    echo "  * Architecture     : ${TARGET_ARCH} (${RAW_ARCH})"
    echo "  * Distribution     : ${DISTRO_NAME} [ID: ${DISTRO_ID}] ${DISTRO_VERSION}"
    echo "  * Standard C Lib   : ${DETECTED_LIBC}"
    echo "  * Environment      : ${DETECTED_ENV}"
}

# ==============================================================================
# Target Triple Resolution
# ==============================================================================
resolve_target() {
    if [[ -n "${CUSTOM_TARGET}" ]]; then
        RESOLVED_TARGET="${CUSTOM_TARGET}"
        log_info "Using user-specified target: ${RESOLVED_TARGET}"
        return
    fi

    # If host is musl-based, always use musl
    if [[ "${DETECTED_LIBC}" == "musl" ]]; then
        RESOLVED_TARGET="${TARGET_ARCH}-unknown-linux-musl"
        log_info "Musl libc detected. Selected target: ${RESOLVED_TARGET} (Statically linked)"
    elif [[ "${PREFER_STATIC}" == "true" ]]; then
        RESOLVED_TARGET="${TARGET_ARCH}-unknown-linux-musl"
        log_info "Selected target: ${RESOLVED_TARGET} (Universal static binary - recommended)"
    else
        RESOLVED_TARGET="${TARGET_ARCH}-unknown-linux-gnu"
        log_info "Selected target: ${RESOLVED_TARGET} (glibc dynamically linked)"
    fi
}

# ==============================================================================
# Version Resolution
# ==============================================================================
resolve_version() {
    if [[ -n "${CUSTOM_VERSION}" ]]; then
        RESOLVED_VERSION="${CUSTOM_VERSION}"
        [[ "${RESOLVED_VERSION}" != v* ]] && RESOLVED_VERSION="v${RESOLVED_VERSION}"
        log_info "Using requested version: ${RESOLVED_VERSION}"
        return
    fi

    log_info "Resolving latest InstallMon release version from GitHub..."
    local latest_tag=""
    if latest_tag="$(curl -sSfL -H "Accept: application/vnd.github.v3+json" "${GITHUB_API}/releases/latest" 2>/dev/null | grep -o '"tag_name": *"[^"]*"' | head -1 | cut -d'"' -f4)"; then
        if [[ -n "${latest_tag}" ]]; then
            RESOLVED_VERSION="${latest_tag}"
            log_info "Resolved latest release: ${RESOLVED_VERSION}"
            return
        fi
    fi

    log_warn "Could not query GitHub API (rate-limited or offline). Falling back to default release ${DEFAULT_VERSION}."
    RESOLVED_VERSION="${DEFAULT_VERSION}"
}

# ==============================================================================
# Installation Directory Determination
# ==============================================================================
resolve_install_dir() {
    if [[ -n "${CUSTOM_INSTALL_DIR}" ]]; then
        TARGET_BIN_DIR="${CUSTOM_INSTALL_DIR}"
        IS_SYSTEM_MODE=false
        [[ "${TARGET_BIN_DIR}" == "/usr"* ]] && IS_SYSTEM_MODE=true
    elif [[ "${FORCE_SYSTEM}" == "true" ]]; then
        TARGET_BIN_DIR="/usr/local/bin"
        IS_SYSTEM_MODE=true
    elif [[ "${FORCE_USER}" == "true" ]]; then
        TARGET_BIN_DIR="${HOME}/.local/bin"
        IS_SYSTEM_MODE=false
    elif [[ $EUID -eq 0 ]]; then
        TARGET_BIN_DIR="/usr/local/bin"
        IS_SYSTEM_MODE=true
    else
        TARGET_BIN_DIR="${HOME}/.local/bin"
        IS_SYSTEM_MODE=false
    fi

    log_info "Installation destination: ${TARGET_BIN_DIR}/installmon (Mode: $(if [[ "${IS_SYSTEM_MODE}" == "true" ]]; then echo "System"; else echo "User"; fi))"

    if [[ "${IS_SYSTEM_MODE}" == "true" ]] && [[ $EUID -ne 0 ]]; then
        log_error "System installation into '${TARGET_BIN_DIR}' requires root privileges. Re-run with sudo or pass --user."
    fi
}

# ==============================================================================
# Download, Checksum Verification & Binary Extraction
# ==============================================================================
install_binary() {
    local archive_name="installmon-${RESOLVED_VERSION}-${RESOLVED_TARGET}.tar.gz"
    local download_url="${GITHUB_BASE}/releases/download/${RESOLVED_VERSION}/${archive_name}"
    local checksums_url="${GITHUB_BASE}/releases/download/${RESOLVED_VERSION}/SHA256SUMS"

    echo ""
    log_info "Preparing to download and install InstallMon:"
    echo "  * Version    : ${RESOLVED_VERSION}"
    echo "  * Target     : ${RESOLVED_TARGET}"
    echo "  * Archive    : ${archive_name}"
    echo "  * Source URL : ${download_url}"
    echo "  * Destination: ${TARGET_BIN_DIR}/installmon"

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_warn "Dry run mode active. No files will be downloaded or modified."
        return 0
    fi

    TMP_DIR="$(mktemp -d -t installmon-install.XXXXXX)"
    trap 'rm -rf "${TMP_DIR:-}"' EXIT INT TERM

    log_info "Downloading archive from GitHub Releases..."
    if ! curl -fsSL --retry 3 --retry-delay 2 "${download_url}" -o "${TMP_DIR}/${archive_name}"; then
        log_error "Failed to download release archive from ${download_url}. Please verify release tag and target."
    fi

    log_info "Downloading SHA256SUMS checksum file..."
    if ! curl -fsSL --retry 3 --retry-delay 2 "${checksums_url}" -o "${TMP_DIR}/SHA256SUMS"; then
        log_error "Failed to download checksum manifest from ${checksums_url}."
    fi

    log_info "Verifying SHA-256 integrity..."
    local expected_hash
    expected_hash="$(grep -F "${archive_name}" "${TMP_DIR}/SHA256SUMS" | awk '{print $1}' || true)"

    if [[ -z "${expected_hash}" ]]; then
        log_error "Checksum for '${archive_name}' not found in SHA256SUMS."
    fi

    local actual_hash
    actual_hash="$(sha256sum "${TMP_DIR}/${archive_name}" | awk '{print $1}')"

    if [[ "${expected_hash}" != "${actual_hash}" ]]; then
        log_error "Checksum mismatch! Archive may be corrupted or tampered.\nExpected: ${expected_hash}\nActual  : ${actual_hash}"
    fi

    log_success "Checksum verified: ${actual_hash}"

    log_info "Extracting archive..."
    tar -xzf "${TMP_DIR}/${archive_name}" -C "${TMP_DIR}"

    if [[ ! -f "${TMP_DIR}/installmon" ]]; then
        log_error "Binary 'installmon' not found inside archive."
    fi

    log_info "Installing binary to ${TARGET_BIN_DIR}/installmon..."
    mkdir -p "${TARGET_BIN_DIR}"
    install -m 0755 "${TMP_DIR}/installmon" "${TARGET_BIN_DIR}/installmon"

    log_success "InstallMon installed successfully!"

    # Verification smoke test
    echo ""
    log_info "Verifying installed executable:"
    "${TARGET_BIN_DIR}/installmon" --version

    # PATH warning check
    if ! echo "${PATH}" | tr ':' '\n' | grep -qx "${TARGET_BIN_DIR}"; then
        echo ""
        log_warn "Notice: '${TARGET_BIN_DIR}' is currently not in your PATH environment variable."
        log_warn "Add the following line to your shell configuration (~/.bashrc, ~/.zshrc, or ~/.profile):"
        echo -e "    ${C_BOLD}export PATH=\"${TARGET_BIN_DIR}:\$PATH\"${C_RESET}"
    fi

    # Optional background service auto-setup
    if [[ "${WITH_SERVICE}" == "true" ]]; then
        echo ""
        log_info "Setting up continuous background monitoring service..."
        if [[ "${IS_SYSTEM_MODE}" == "true" ]]; then
            "${TARGET_BIN_DIR}/installmon" --system service setup || true
        else
            "${TARGET_BIN_DIR}/installmon" service setup || true
        fi
    fi

    echo ""
    log_success "Setup complete! Launch the interactive TUI with:"
    echo -e "    ${C_CYAN}${C_BOLD}${TARGET_BIN_DIR}/installmon${C_RESET}"
}

# ==============================================================================
# Uninstallation Method
# ==============================================================================
uninstall_installmon() {
    echo ""
    log_info "Initiating InstallMon uninstallation..."

    local found_binaries=()
    for bin_path in "${TARGET_BIN_DIR}/installmon" "/usr/local/bin/installmon" "/usr/bin/installmon" "${HOME}/.local/bin/installmon"; do
        if [[ -f "${bin_path}" ]]; then
            # Avoid duplicate entries
            local exists=false
            for b in "${found_binaries[@]}"; do
                [[ "$b" == "${bin_path}" ]] && exists=true
            done
            [[ "${exists}" == "false" ]] && found_binaries+=("${bin_path}")
        fi
    done

    # 1. Stop active services and clean systemd units
    log_info "Checking for running services and systemd units..."

    # User systemd service
    if command -v systemctl >/dev/null 2>&1; then
        if systemctl --user is-active installmon >/dev/null 2>&1; then
            log_info "Stopping systemd user service..."
            [[ "${DRY_RUN}" == "false" ]] && systemctl --user stop installmon || true
        fi
        if systemctl is-active installmon >/dev/null 2>&1 && [[ $EUID -eq 0 ]]; then
            log_info "Stopping system-wide systemd service..."
            [[ "${DRY_RUN}" == "false" ]] && systemctl stop installmon || true
        fi
    fi

    # Remove user unit file
    local user_unit="${HOME}/.config/systemd/user/installmon.service"
    if [[ -f "${user_unit}" ]]; then
        log_info "Removing user service unit: ${user_unit}"
        if [[ "${DRY_RUN}" == "false" ]]; then
            rm -f "${user_unit}"
            command -v systemctl >/dev/null 2>&1 && systemctl --user daemon-reload || true
        fi
    fi

    # Remove system unit file
    local sys_unit="/etc/systemd/system/installmon.service"
    if [[ -f "${sys_unit}" ]]; then
        if [[ $EUID -eq 0 ]]; then
            log_info "Removing system service unit: ${sys_unit}"
            if [[ "${DRY_RUN}" == "false" ]]; then
                rm -f "${sys_unit}"
                command -v systemctl >/dev/null 2>&1 && systemctl daemon-reload || true
            fi
        else
            log_warn "System unit exists at ${sys_unit} but uninstaller was run without root. Run with sudo to remove system unit."
        fi
    fi

    # 2. Terminate background processes if still alive
    if command -v killall >/dev/null 2>&1; then
        if pgrep -x installmon >/dev/null 2>&1; then
            log_info "Terminating residual installmon processes..."
            [[ "${DRY_RUN}" == "false" ]] && killall -SIGTERM installmon 2>/dev/null || true
        fi
    fi

    # 3. Remove runtime heartbeats & log files
    local user_data_dir="${XDG_DATA_HOME:-${HOME}/.local/share}/installmon"
    if [[ -f "${user_data_dir}/daemon.json" ]] || [[ -f "${user_data_dir}/daemon.log" ]]; then
        log_info "Removing user daemon heartbeat and logs..."
        if [[ "${DRY_RUN}" == "false" ]]; then
            rm -f "${user_data_dir}/daemon.json" "${user_data_dir}/daemon.log"
        fi
    fi

    if [[ $EUID -eq 0 ]] && [[ -d "/var/lib/installmon" ]]; then
        log_info "Removing system daemon heartbeat and logs..."
        if [[ "${DRY_RUN}" == "false" ]]; then
            rm -f "/var/lib/installmon/daemon.json" "/var/lib/installmon/daemon.log"
        fi
    fi

    # 4. Remove binaries
    if [[ ${#found_binaries[@]} -eq 0 ]]; then
        log_warn "No installed 'installmon' executable found in standard PATH directories."
    else
        for bin in "${found_binaries[@]}"; do
            if [[ -w "$(dirname "$bin")" ]] || [[ $EUID -eq 0 ]]; then
                log_info "Removing binary: ${bin}"
                [[ "${DRY_RUN}" == "false" ]] && rm -f "${bin}"
            else
                log_warn "Insufficient permissions to delete ${bin}. Please remove manually with: sudo rm -f ${bin}"
            fi
        done
    fi

    # 5. Database and data retention / purge
    if [[ "${PURGE_DATA}" == "true" ]]; then
        log_info "Purging database and configuration directories (--purge specified)..."
        if [[ "${DRY_RUN}" == "false" ]]; then
            rm -rf "${user_data_dir}"
            [[ -d "${HOME}/.config/installmon" ]] && rm -rf "${HOME}/.config/installmon"
            if [[ $EUID -eq 0 ]]; then
                rm -rf "/var/lib/installmon"
                rm -rf "/etc/installmon"
            fi
        fi
        log_success "All databases and tracking history have been purged."
    else
        echo ""
        log_info "Data preservation notice:"
        if [[ -f "${user_data_dir}/installmon.db" ]]; then
            echo "  * User database preserved at: ${user_data_dir}/installmon.db"
        fi
        if [[ -f "/var/lib/installmon/installmon.db" ]]; then
            echo "  * System database preserved at: /var/lib/installmon/installmon.db"
        fi
        echo "  To completely remove all tracking records, re-run with: ${C_BOLD}install.sh --uninstall --purge${C_RESET}"
    fi

    echo ""
    log_success "InstallMon uninstallation completed successfully."
}

# ==============================================================================
# Main Routine
# ==============================================================================
main() {
    parse_args "$@"
    print_header
    check_dependencies
    detect_platform
    resolve_install_dir

    if [[ "${ACTION}" == "uninstall" ]]; then
        uninstall_installmon
    else
        resolve_target
        resolve_version
        install_binary
    fi
}

main "$@"
