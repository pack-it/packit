#!/bin/sh
set -eu

# Gets user input and uses /dev/tty when stdin is not available.
get_answer() {
    if [ -t 0 ]; then
        read answer
    else
        read answer < /dev/tty
    fi

    echo "$answer"
}

# Converts a string to lowercase.
to_lower() {
    printf "%s" "$1" | tr "[:upper:]" "[:lower:]"
}

# A function to handle (yes/no) questions.
# Returns 1 if the default option was chosen, 0 otherwise.
ask() {
    # The preferred answer to the question (the default), needs to be a string of value 'Y' or 'N'
    PREFERRED=$1

    # The question to ask
    QUESTION=$2

    if [ "$PREFERRED" = "Y" ]; then
        printf "$QUESTION? (Y/n) "
        answer=$(to_lower "$(get_answer)")
        ! { [ "$answer" = "n" ] || [ "$answer" = "no" ]; }
    else
        printf "$QUESTION? (y/N) "
        answer=$(to_lower "$(get_answer)")
        ! { [ "$answer" = "y" ] || [ "$answer" = "yes" ]; }
    fi
}

# Removes all created files in case of an error.
cleanup() {
    # Don't cleanup if `SHOULD_CLEANUP` is not true
    if [ ${SHOULD_CLEANUP-} -eq 0 ]; then
        exit 0
    fi

    echo "Removing installed Packit files"

    # Remove the prefix directory if `PREFIX_DIR` is set and it exists
    if [ -n "${PREFIX_DIR-}" ] && [ -d "$PREFIX_DIR" ]; then
        sudo rm -r "$PREFIX_DIR"
    fi
    
    # Remove the config directory if `CONFIG_DIR` is set and it exists
    if [ -n "${CONFIG_DIR-}" ] && [ -d "$CONFIG_DIR" ]; then
        sudo rm -r "$CONFIG_DIR"
    fi
}

main() {
# Execute `handle_exit` in case of ctrl+C interupt, termination signal or exit
# And SHOULD_CLEANUP by default
trap cleanup INT TERM EXIT
SHOULD_CLEANUP=1

VERSION="0.0.2"
REVISION="0"
CURRENT_OS="$(uname -s)"

echo "Installing Packit $VERSION ($REVISION)"
echo "Current OS: $CURRENT_OS"

# Get the OS name
if [ $CURRENT_OS = "Darwin" ]; then
    CURRENT_OS_NAME="apple-darwin"
elif [ $CURRENT_OS = "Linux" ]; then
    if echo "$(ldd --version)" | grep -q "musl"; then
        CURRENT_OS_NAME="unknown-linux-musl"
    elif echo "$(ldd --version)" | grep -q "GLIBC"; then
        CURRENT_OS_NAME="unknown-linux-gnu"
    else
        echo "Current platform unsupported, stopping install"
        exit 1
    fi
else
    echo "Current platform unsupported, stopping install"
    exit 1
fi

# Get the current architecture
if [ "$(uname -m)" = "aarch64" ] || [ "$(uname -m)" = "arm64" ]; then
    CURRENT_ARCH="aarch64"
elif [ "$(uname -m)" = "x86_64" ]; then
    CURRENT_ARCH="x86_64"
else
    echo "Current architecture unsupported, stopping install"
    exit 1
fi

# Create target
TARGET="$CURRENT_ARCH-$CURRENT_OS_NAME"

echo "Current target: $TARGET"

SOURCE_URL="https://github.com/pack-it/packit/releases/download/$VERSION/packit@$VERSION.tar.gz"
PREBUILD_URL="https://github.com/pack-it/packit/releases/download/$VERSION/packit@$VERSION-$REVISION-$TARGET.tar.gz"

# Determine the prefix and config directory
PREFIX_DIR="/opt/packit"
if [ $CURRENT_OS = "Darwin" ]; then
    CONFIG_DIR="/Library/Application Support/packit"
else
    CONFIG_DIR="/etc/packit"
fi

echo "Prefix directory: $PREFIX_DIR"
echo "Config directory: $CONFIG_DIR"

# Ask the user for admin rights
if ! ask "Y" "The Packit install script requires root to modify '$PREFIX_DIR' and '$CONFIG_DIR', do you wish to continue"; then
    echo "Canceling installation of Packit"
    exit 1
fi

# Execute sudo without doing anything
sudo true

# Exit early with code 0 if there already is a version of Packit installed
# Note that we can't rely on the `packit init` command, because we don't know if it fails because of an already existing config file
if [ -f "$CONFIG_DIR/Config.toml" ]; then
    echo "Packit already seems to be installed, config file found in '$CONFIG_DIR'"
    SHOULD_CLEANUP=0
    exit 0
fi
if [ -f "$PREFIX_DIR/Register.toml" ]; then
    echo "Packit already seems to be installed, register file found in '$PREFIX_DIR'"
    SHOULD_CLEANUP=0
    exit 0
fi

USERNAME=$(whoami)

# Go into the prefix directory
sudo mkdir -p "$PREFIX_DIR/packages/packit/"
sudo chmod -R 755 "$PREFIX_DIR"
sudo chown -R $USERNAME "$PREFIX_DIR"
cd "$PREFIX_DIR/packages/packit/"

# Install Packit to the prefix directory
echo "Downloading Packit prebuild from '$PREBUILD_URL'"
if curl --proto "=https" -sSfL $PREBUILD_URL --output packit@$VERSION-$REVISION-$TARGET.tar.gz; then
    tar -xf packit@$VERSION-$REVISION-$TARGET.tar.gz
    rm packit@$VERSION-$REVISION-$TARGET.tar.gz
    mv packit@$VERSION-$REVISION-$TARGET $VERSION
    
    echo "Downloading Packit prebuild successful"
else
    # Check internet connection with reliable site
    if ! curl -sSf http://www.google.com > /dev/null 2>&1; then
        echo "Retrieving Packit prebuilds failed, because there is no working internet connection"
        echo "Canceling installation of Packit"
        exit 1
    fi

    if ! ask "Y" "Retrieving prebuilds failed. Do you wish to build Packit from source"; then
        echo "Canceling installation of Packit"
        exit 1
    fi

    RUSTUP_INSTALLED=0

    # Make sure cargo exists before building Packit
    if ! command -v cargo >/dev/null 2>&1; then
        if ask "N" "Cargo is not installed, do you wish to install it to build Packit"; then
            echo "Canceling installation of Packit"
            exit 1
        fi

        echo "Installing cargo from 'https://sh.rustup.rs'"
        curl --proto '=https' --tlsv1.2 -sSfL https://sh.rustup.rs | sh

        # Make sure that the rustup install was successful
        if ! command -v cargo >/dev/null 2>&1; then
            echo "Installing rustup failed, canceling Packit installation"
            exit 1
        fi

        echo "Installing cargo successful"
        RUSTUP_INSTALLED=1
    fi

    echo "Downloading Packit source files from '$SOURCE_URL'"
    curl --proto "=https" -sSfL $SOURCE_URL --output packit@$VERSION.tar.gz
    echo "Downloading Packit source files successful"

    echo "Unpacking Packit source files"
    tar -xf packit@$VERSION.tar.gz
    echo "Unpacking Packit source files successful"

    rm packit@$VERSION.tar.gz
    cd packit@$VERSION

    echo "Building Packit from source"
    cargo build-install --destination ../$VERSION
    cd ..
    rm -r ./packit@$VERSION

    if [ $RUSTUP_INSTALLED -eq 1 ]; then
        if ask "Y" "You installed rustup to install Packit. This installation is not registered in Packit. Do you wish to uninstall it"; then
            echo "Uninstalling rustup"
            rustup self uninstall
            echo "Uninstalling rustup successful"
        fi
    fi

    echo "Building Packit from source successful"
fi

sudo mkdir -p "$CONFIG_DIR"
sudo chmod -R 755 "$CONFIG_DIR"
sudo chown -R $USERNAME "$CONFIG_DIR"

echo "Initializing Packit"
"$PREFIX_DIR/packages/packit/$VERSION/bin/packit" init
echo "Initializing Packit successful"

# Make sure that pit works
echo "Testing Packit install"
if ! command -v $PREFIX_DIR/bin/pit -h >/dev/null 2>&1; then
    echo "Unsuccessfull install of Packit, the 'pit' command cannot be found"
    exit 1
fi

# Make sure that packit works
if ! command -v $PREFIX_DIR/bin/packit -h >/dev/null 2>&1; then
    echo "Unsuccessfull install of Packit, the 'packit' command cannot be found"
    exit 1
fi

echo "Successfully installed Packit!"

# Exit early if Packit is already in the PATH
if echo ":$PATH:" | grep -q ":$PREFIX_DIR/bin:"; then
    echo "Packit already found in PATH, no further actions should be required"
    SHOULD_CLEANUP=0
    exit 0
fi

SHELL_CONFIG_PATH=""

case "$SHELL" in
    *zsh)
        SHELL_CONFIG_PATH="$HOME/.zshrc"
        ;;
    *bash)
        SHELL_CONFIG_PATH="$HOME/.bashrc"
        ;;
    *fish)
        # Fish is not POSIX, so it needs custom handling
        if ask "Y" "Do you wish to automatically add Packit to your PATH"; then
            fish -c "fish_add_path $PREFIX_DIR/bin"
            echo "Restart your shell to refresh your path and use Packit"
            SHOULD_CLEANUP=0
            exit 0
        fi
        ;;
    *)
        ;;
esac

if [ -e "$SHELL_CONFIG_PATH" ]; then
    if ask "Y" "Do you wish to automatically add Packit to your PATH by adding it to '$SHELL_CONFIG_PATH'"; then
        echo "export PATH=\"$PREFIX_DIR/bin:\$PATH\"" >> "$SHELL_CONFIG_PATH"
        echo "Restart your shell to refresh your path and use Packit"
        SHOULD_CLEANUP=0
        exit 0
    fi
fi

# If the shell is not recognized or user did not want to add Packit automatically tell the user how to add Packit to their shell config
echo "Add '$PREFIX_DIR/bin' to your PATH by adding the command below to your shell config:"
echo "export PATH=\"$PREFIX_DIR/bin:\$PATH\""
SHOULD_CLEANUP=0
}

main "$@"
