#!/bin/bash

# This file is for use if for some reason you can't set
# rustflags to use mold linker.

# Supported commands and their description
declare -a commands=(
    'cmd="run" desc="Run the project"'
    'cmd="build" desc="Build the project"'
    'cmd="test" desc="Run the project tests"'
    'cmd="check" desc="Check the project"'
    'cmd="bench" desc="Run benchmarks"'
    'cmd="free" desc="Run custom Rust commands"'
)

## Prints supported commands and their description
print_commands() {
    echo "Commands:"
    for i in "${!commands[@]}"; do
        eval "${commands[$i]}"
        printf "  %-7s   %s with mold linker\n" "$cmd" "$desc"
    done
}

# Check if valid argument were given (at least 1)
if [ $# -eq 0 ]; then 
    echo "Usage: ./mold.sh [cargo-command] [additional-args]"
    print_commands
    exit 1
fi

## Checks if mold is installed on the system
is_mold_installed() {
    if command -v mold &> /dev/null; then
        echo "mold $(mold --version) is installed!"
        return 1
    else
        echo "mold not installed or is not in PATH"
        return 0
    fi
}

## Installs mold to the system using it's respective package manager
install_mold() {
    local tries="${1:-0}"
    local max_attempts=3
    local delay=5  # seconds between attempts

    # Base case - exit if too many attempts
    if (( tries >= max_attempts )); then
        echo "Failed to install mold after $max_attempts attempt(s)"
        return 1
    fi

    # Show attempt number if retrying
    if (( tries > 0 )); then
        echo "Attempt $tries/$max_attempts to install mold via package manager..."
        sleep "$delay"
    fi

    # Check if we can get OS distro
    if [[ ! -f /etc/os-release ]]; then
        echo "Cannot detect OS distribution: /etc/os-release not found" >&2
        return 1
    fi

    local distro_id
    distro_id=$(. /etc/os-release && echo "$ID")
    echo "Installing mold on $distro_id..."

    case "$distro_id" in
        debian|ubuntu)
            if ! sudo apt-get update -qq && sudo apt-get install -y mold; then
                echo "apt-get installation failed" >&2
                install_mold $((tries + 1))
                return $?
            fi
            ;;
        fedora)
            if ! sudo dnf install -y mold; then
                echo "dnf installation failed" >&2
                install_mold $((tries + 1))
                return $?
            fi
            ;;
        arch|manjaro)
            if ! sudo pacman -S --noconfirm mold; then
                echo "pacman installation failed" >&2
                install_mold $((tries + 1))
                return $?
            fi
            ;;
        opensuse*|suse)
            if ! sudo zypper install -y mold; then
                echo "zypper installation failed" >&2
                install_mold $((tries + 1))
                return $?
            fi
            ;;
        *)
            echo "Unsupported distribution: $distro_id" >&2
            echo "Supported distributions: debian, ubuntu, fedora, arch, manjaro, opensuse" >&2
            return 1
            ;;
    esac

    if command -v mold >/dev/null 2>&1; then
        echo "mold successfully installed!"
        return 0
    else
        echo "Installation appeared to succeed but mold command not found" >&2
        install_mold $((tries + 1))
        return $?
    fi
}

## Checks if given command is supported
is_valid_command() {
    local given_cmd="$1"

    if [[ -z "$given_cmd" ]]; then
        return 1;
    fi

    for entry in "${commands[@]}"; do
        local cmd
        eval "$entry"
        if [[ "$given_cmd" == "$cmd" ]]; then
            return 0
        fi
    done

    return 1
}

# Get cargo command from CLA
CARGO_CMD=$1
shift

# Check if supported command was given
if ! is_valid_command "$CARGO_CMD"; then
    echo "Invalid command '$CARGO_CMD'"
    print_commands
    exit 1
fi

# Install mold linker if not already
if [[ ! is_mold_installed ]]; then
    echo "mold not installed or is not in PATH"
    install_mold
else
    echo "mold $(mold --version) is installed!"
fi

# Check if `free` command is being used
if [ "$CARGO_CMD" = "free" ]; then
    if [ $# -eq 0 ]; then
        echo "Error: 'free' requires additional cargo commands"
        echo "Usage: ./mold.sh free [cargo-commands]"
        exit 1
    fi
    echo "Running: mold -run cargo $@"
    mold -run cargo "$@"
else
    echo "Running: mold -run cargo $CARGO_CMD $@"
    mold -run cargo $CARGO_CMD "$@"
fi
