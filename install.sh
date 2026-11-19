#!/usr/bin/env bash
# ==============================================================================
# InstallMon - Unified Lifecycle Assistant (Install / Update / Uninstall)
#
# Single-command installation, updating, and uninstallation for InstallMon on Linux.
# Detects CPU architecture, distribution, libc, and environment automatically.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/ankur3-101106/installmon/main/install.sh | bash
#
# Repository: https://github.com/ankur3-101106/installmon
# ==============================================================================

set -euo pipefail

REPO="ankur3-101106/installmon"
GITHUB_BASE="https://github.com/${REPO}"
GITHUB_API="https://api.github.com/repos/${REPO}"
DEFAULT_VERSION="v0.1.0"

# Terminal formatting & colors
if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
    C_RESET="\033[0m"
    C_BOLD="\033[1m"
    C_GREEN="\033[32m"
    C_BLUE="\033[34m"
    C_CYAN="\033[36m"
    C_YELLOW="\033[33m"
    C_RED="\033[31m"
    C_DIM="\033[2m"
else
    C_RESET=""
    C_BOLD=""
    C_GREEN=""
    C_BLUE=""
    C_CYAN=""
    C_YELLOW=""
    C_RED=""
    C_DIM=""
fi

print_banner() {
    clear 2>/dev/null || true
    echo -e "${C_CYAN}${C_BOLD}"
    cat <<'EOF'
  ___           _        _ _ __  __             
 |_ _|_ __  ___| |_ __ _| | |  \/  | ___  _ __  
  | || '_ \/ __| __/ _` | | | |\/| |/ _ \| '_ \ 
  | || | | \__ \ || (_| | | | |  | | (_) | | | |
 |___|_| |_|___/\__\__,_|_|_|_|  |_|\___/|_| |_|
EOF
    echo -e "${C_RESET}"
    echo -e " ${C_BOLD}Linux Software Installation Monitoring & Provenance Tracking${C_RESET}"
    echo -e " ${C_DIM}Unified Lifecycle Assistant (Install • Update • Uninstall)${C_RESET}"
    echo "=================================================================="
}

log_info() {
    echo -e " ${C_BLUE}${C_BOLD}[+]${C_RESET} $*"
}

log_success() {
    echo -e " ${C_GREEN}${C_BOLD}[✓]${C_RESET} $*"
}

log_warn() {
    echo -e " ${C_YELLOW}${C_BOLD}[!]${C_RESET} $*"
}

log_error() {
    echo -e " ${C_RED}${C_BOLD}[✗]${C_RESET} $*" >&2
}

# Prompt user for input, supporting interactive terminals, piped input, and curl | bash
prompt_user() {
    local prompt_text="$1"
    local default_value="$2"
    local input=""

    echo -ne " ${C_YELLOW}${C_BOLD}[?]${C_RESET} ${prompt_text} " >&2

    # Non-interactive fallback
    if [[ "${NON_INTERACTIVE:-false}" == "true" ]] || [[ -n "${CI:-}" ]]; then
        echo "${default_value}" >&2
        echo "${default_value}"
        return
    fi

    if [[ -t 0 ]]; then
        read -r input || true
    elif read -t 0 2>/dev/null; then
        read -r input || true
    elif [[ -r /dev/tty ]]; then
        read -r input </dev/tty || true
    else
        input="${default_value}"
        echo "${default_value}" >&2
    fi

    if [[ -z "${input}" ]]; then
        echo "${default_value}"
    else
        echo "${input}"
    fi
}

# ==============================================================================
# System & Hardware Detection
# ==============================================================================
detect_system() {
    log_info "Detecting system specifications..."

    # 1. Operating System
    OS_NAME="$(uname -s)"
    if [[ "${OS_NAME}" != "Linux" ]]; then
        log_error "InstallMon currently supports Linux only. Detected OS: ${OS_NAME}."
        exit 1
    fi

    # 2. CPU Architecture
    RAW_ARCH="$(uname -m)"
    case "${RAW_ARCH}" in
        x86_64|amd64)
            TARGET_ARCH="x86_64"
            ;;
        aarch64|arm64)
            TARGET_ARCH="aarch64"
            ;;
        *)
            log_error "Unsupported CPU architecture: '${RAW_ARCH}'. InstallMon supports x86_64 and aarch64 (ARM 64-bit)."
            exit 1
            ;;
    esac

    # 3. Distribution Identification
    DISTRO_NAME="Linux"
    DISTRO_ID="unknown"
    if [[ -f /etc/os-release ]]; then
        # shellcheck disable=SC1091
        DISTRO_NAME="$(grep -E '^NAME=' /etc/os-release | cut -d= -f2 | tr -d '"')"
        DISTRO_ID="$(grep -E '^ID=' /etc/os-release | cut -d= -f2 | tr -d '"')"
    fi

    # 4. Standard C Library (libc)
    DETECTED_LIBC="glibc"
    if [[ "${DISTRO_ID}" == "alpine" ]] || ldd --version 2>&1 | grep -qi "musl" || ls /lib/ld-musl* >/dev/null 2>&1; then
        DETECTED_LIBC="musl"
    fi

    # 5. Environment
    HOST_ENV="Bare Metal"
    if [[ -n "${WSL_DISTRO_NAME:-}" ]] || uname -r | grep -qi "microsoft"; then
        HOST_ENV="WSL2 (Windows Subsystem for Linux)"
    elif [[ -f /.dockerenv ]]; then
        HOST_ENV="Docker Container"
    elif [[ -f /run/.containerenv ]]; then
        HOST_ENV="Podman Container"
    fi

    # Release target triple: Statically linked musl is universal across all Linux distributions
    TARGET_TRIPLE="${TARGET_ARCH}-unknown-linux-musl"

    echo "     * OS & Kernel    : Linux ($(uname -r))"
    echo "     * Architecture   : ${TARGET_ARCH} (${RAW_ARCH})"
    echo "     * Distribution   : ${DISTRO_NAME}"
    echo "     * C Library      : ${DETECTED_LIBC}"
    echo "     * Environment    : ${HOST_ENV}"
    echo "     * Recommended    : ${TARGET_TRIPLE} (Static - zero dependencies)"
    echo ""
}

# Check if installmon is currently installed
check_installation_status() {
    EXISTING_BIN=""
    EXISTING_VER=""
    for p in "${HOME}/.local/bin/installmon" "/usr/local/bin/installmon" "/usr/bin/installmon"; do
        if [[ -x "$p" ]]; then
            EXISTING_BIN="$p"
            EXISTING_VER="$("${EXISTING_BIN}" --version 2>/dev/null || echo "installed")"
            break
        fi
    done

    # Resolve latest available release tag
    LATEST_VERSION=""
    if command -v curl >/dev/null 2>&1; then
        LATEST_VERSION="$(curl -sSfL -H "Accept: application/vnd.github.v3+json" "${GITHUB_API}/releases/latest" 2>/dev/null | grep -o '"tag_name": *"[^"]*"' | head -1 | cut -d'"' -f4 || true)"
    fi
    if [[ -z "${LATEST_VERSION}" ]]; then
        LATEST_VERSION="${DEFAULT_VERSION}"
    fi

    if [[ -n "${EXISTING_BIN}" ]]; then
        log_info "Status: Installed at ${C_BOLD}${EXISTING_BIN}${C_RESET} (${EXISTING_VER})"
        echo "     * Latest Release : ${LATEST_VERSION}"
    else
        log_info "Status: Not currently installed on this system."
        echo "     * Available Release: ${LATEST_VERSION}"
    fi
    echo ""
}

# ==============================================================================
# Installation Routine
# ==============================================================================
install_installmon() {
    echo -e "${C_BOLD}--- Installation Setup ---${C_RESET}"
    echo " Choose installation location:"
    echo "   1) User Mode (~/.local/bin)     - Recommended (no root / sudo required)"
    echo "   2) System Mode (/usr/local/bin) - System-wide (requires root / sudo)"
    local scope_choice
    scope_choice="$(prompt_user "Enter choice [1-2]" "1")"

    local install_dir="${HOME}/.local/bin"
    local is_system=false

    if [[ "${scope_choice}" == "2" ]]; then
        install_dir="/usr/local/bin"
        is_system=true
    fi

    echo ""
    log_info "Destination: ${install_dir}/installmon"

    # Background service prompt
    echo ""
    local enable_service_prompt
    enable_service_prompt="$(prompt_user "Configure continuous background monitoring service? (y/n)" "y")"
    local with_service=false
    if [[ "${enable_service_prompt}" =~ ^[Yy] ]]; then
        with_service=true
    fi

    # Confirmation prompt
    echo ""
    local confirm_install
    confirm_install="$(prompt_user "Install InstallMon ${LATEST_VERSION} now? (Y/n)" "y")"
    if [[ ! "${confirm_install}" =~ ^[Yy] ]]; then
        log_warn "Installation cancelled."
        exit 0
    fi

    echo ""
    log_info "Downloading release package (${TARGET_TRIPLE})..."

    local archive_name="installmon-${LATEST_VERSION}-${TARGET_TRIPLE}.tar.gz"
    local download_url="${GITHUB_BASE}/releases/download/${LATEST_VERSION}/${archive_name}"
    local checksums_url="${GITHUB_BASE}/releases/download/${LATEST_VERSION}/SHA256SUMS"

    TMP_DIR="$(mktemp -d -t installmon-install.XXXXXX)"
    trap 'rm -rf "${TMP_DIR:-}"' EXIT INT TERM

    if ! curl -fsSL --retry 3 "${download_url}" -o "${TMP_DIR}/${archive_name}"; then
        log_error "Failed to download ${download_url}."
        exit 1
    fi

    log_info "Verifying cryptographic SHA-256 checksum..."
    if curl -fsSL --retry 3 "${checksums_url}" -o "${TMP_DIR}/SHA256SUMS"; then
        local expected_hash
        expected_hash="$(grep -F "${archive_name}" "${TMP_DIR}/SHA256SUMS" | awk '{print $1}' || true)"
        if [[ -n "${expected_hash}" ]]; then
            local actual_hash
            actual_hash="$(sha256sum "${TMP_DIR}/${archive_name}" | awk '{print $1}')"
            if [[ "${expected_hash}" != "${actual_hash}" ]]; then
                log_error "Checksum mismatch!\nExpected: ${expected_hash}\nActual:   ${actual_hash}"
                exit 1
            fi
            log_success "Checksum verified: ${actual_hash:0:16}..."
        fi
    fi

    log_info "Extracting binary..."
    tar -xzf "${TMP_DIR}/${archive_name}" -C "${TMP_DIR}"

    log_info "Placing binary in ${install_dir}/installmon..."
    if [[ "${is_system}" == "true" ]] && [[ $EUID -ne 0 ]]; then
        sudo mkdir -p "${install_dir}"
        sudo install -m 0755 "${TMP_DIR}/installmon" "${install_dir}/installmon"
    else
        mkdir -p "${install_dir}"
        install -m 0755 "${TMP_DIR}/installmon" "${install_dir}/installmon"
    fi

    log_success "InstallMon installed successfully!"
    "${install_dir}/installmon" --version

    # Check PATH
    if ! echo "${PATH}" | tr ':' '\n' | grep -qx "${install_dir}"; then
        echo ""
        log_warn "'${install_dir}' is not in your current PATH."
        echo "     Add this to your ~/.bashrc or ~/.zshrc:"
        echo -e "     ${C_CYAN}export PATH=\"${install_dir}:\$PATH\"${C_RESET}"
    fi

    # Background service setup
    if [[ "${with_service}" == "true" ]]; then
        echo ""
        log_info "Initializing background monitoring service..."
        if [[ "${is_system}" == "true" ]] && [[ $EUID -ne 0 ]]; then
            sudo "${install_dir}/installmon" --system service setup || true
        elif [[ "${is_system}" == "true" ]]; then
            "${install_dir}/installmon" --system service setup || true
        else
            "${install_dir}/installmon" service setup || true
        fi
    fi

    # Launch TUI prompt
    echo ""
    local launch_tui
    launch_tui="$(prompt_user "Launch interactive InstallMon TUI dashboard now? (y/n)" "n")"
    if [[ "${launch_tui}" =~ ^[Yy] ]]; then
        exec "${install_dir}/installmon"
    else
        echo ""
        echo "=================================================================="
        log_success "Setup complete! Run InstallMon anytime with:"
        echo -e "     ${C_CYAN}${C_BOLD}${install_dir}/installmon${C_RESET}"
        echo "=================================================================="
    fi
}

# ==============================================================================
# Uninstallation Routine
# ==============================================================================
uninstall_installmon() {
    echo -e "${C_BOLD}--- Uninstallation ---${C_RESET}"
    echo ""

    local confirm
    confirm="$(prompt_user "Are you sure you want to uninstall InstallMon? (y/n)" "n")"
    if [[ ! "${confirm}" =~ ^[Yy] ]]; then
        log_info "Uninstallation cancelled."
        exit 0
    fi

    echo ""
    log_info "Stopping active services and systemd units..."

    # Stop active services
    if command -v systemctl >/dev/null 2>&1; then
        systemctl --user stop installmon >/dev/null 2>&1 || true
        if [[ $EUID -eq 0 ]]; then
            systemctl stop installmon >/dev/null 2>&1 || true
        fi
    fi

    # Remove unit files
    local user_unit="${HOME}/.config/systemd/user/installmon.service"
    if [[ -f "${user_unit}" ]]; then
        rm -f "${user_unit}"
        command -v systemctl >/dev/null 2>&1 && systemctl --user daemon-reload || true
        log_success "Removed user service unit"
    fi

    local sys_unit="/etc/systemd/system/installmon.service"
    if [[ -f "${sys_unit}" ]]; then
        if [[ $EUID -eq 0 ]]; then
            rm -f "${sys_unit}"
            command -v systemctl >/dev/null 2>&1 && systemctl daemon-reload || true
            log_success "Removed system service unit"
        else
            log_warn "System unit at ${sys_unit} requires root. Run with sudo to remove."
        fi
    fi

    # Terminate lingering background processes
    if pgrep -x installmon >/dev/null 2>&1; then
        killall -SIGTERM installmon 2>/dev/null || true
    fi

    # Delete executable binaries
    log_info "Removing executable binaries..."
    for bin_path in "${HOME}/.local/bin/installmon" "/usr/local/bin/installmon" "/usr/bin/installmon"; do
        if [[ -f "${bin_path}" ]]; then
            if [[ -w "$(dirname "${bin_path}")" ]] || [[ $EUID -eq 0 ]]; then
                rm -f "${bin_path}"
                log_success "Removed ${bin_path}"
            else
                log_warn "Permission required to remove ${bin_path}. Run: sudo rm -f ${bin_path}"
            fi
        fi
    done

    # Remove heartbeat and temporary logs
    local user_data="${XDG_DATA_HOME:-${HOME}/.local/share}/installmon"
    rm -f "${user_data}/daemon.json" "${user_data}/daemon.log" 2>/dev/null || true

    # Database retention vs purge prompt
    echo ""
    local delete_db
    delete_db="$(prompt_user "Also delete database and tracking history? (y/n)" "n")"
    if [[ "${delete_db}" =~ ^[Yy] ]]; then
        rm -rf "${user_data}"
        rm -rf "${HOME}/.config/installmon"
        if [[ $EUID -eq 0 ]]; then
            rm -rf "/var/lib/installmon" "/etc/installmon"
        fi
        log_success "Database history purged."
    else
        log_info "Database preserved at: ${user_data}/installmon.db"
    fi

    echo ""
    log_success "InstallMon uninstallation complete."
}

# ==============================================================================
# Main Interactive Flow
# ==============================================================================
main() {
    print_banner
    detect_system
    check_installation_status

    # Direct CLI shortcuts if passed
    if [[ "${1:-}" == "uninstall" ]] || [[ "${1:-}" == "--uninstall" ]] || [[ "${1:-}" == "-u" ]]; then
        uninstall_installmon
        exit 0
    fi

    if [[ "${1:-}" == "install" ]] || [[ "${1:-}" == "--install" ]] || [[ "${1:-}" == "-i" ]]; then
        install_installmon
        exit 0
    fi

    # Interactive Menu
    if [[ -n "${EXISTING_BIN}" ]]; then
        echo -e "${C_BOLD}Select an action:${C_RESET}"
        echo "  1) Reinstall / Update InstallMon (Recommended)"
        echo "  2) Uninstall InstallMon & Clean Services"
        echo "  3) Exit"
    else
        echo -e "${C_BOLD}Select an action:${C_RESET}"
        echo "  1) Install InstallMon (Recommended)"
        echo "  2) Uninstall / Clean Residual Data"
        echo "  3) Exit"
    fi
    echo ""

    local action_choice
    action_choice="$(prompt_user "Enter choice [1-3]" "1")"

    case "${action_choice}" in
        1)
            install_installmon
            ;;
        2)
            uninstall_installmon
            ;;
        3)
            echo "Exiting."
            exit 0
            ;;
        *)
            log_warn "Invalid selection. Proceeding with installation..."
            install_installmon
            ;;
    esac
}

main "$@"
