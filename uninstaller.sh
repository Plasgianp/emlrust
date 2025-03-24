#!/bin/bash

echo "====================================="
echo "    EmlRust Uninstaller Script      "
echo "====================================="

echo "This script will uninstall EmlRust from your system."
echo "Are you sure you want to proceed? (y/n)"
read -r confirm

if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
    echo "Uninstallation cancelled."
    exit 0
fi

remove_symlink() {
    if [ -L "$1/emlrust" ]; then
        echo "Removing symlink from $1..."
        rm -f "$1/emlrust"
        echo "✓ Symlink removed from $1"
        return 0
    fi
    return 1
}

echo "Searching for EmlRust symlinks..."

found=0

if remove_symlink "$HOME/.local/bin"; then
    found=1
fi

if remove_symlink "/usr/local/bin"; then
    found=1
fi

if [ $found -eq 0 ]; then
    echo "No symlinks found in standard locations."
    echo "If you created a symlink in a custom location, please remove it manually."
fi

if [ -d "./target" ] && [ -f "./Cargo.toml" ]; then
    echo "Would you like to remove the built binaries? (y/n)"
    read -r remove_binary

    if [[ "$remove_binary" =~ ^[Yy]$ ]]; then
        echo "Removing build artifacts..."
        rm -rf ./target
        echo "✓ Build artifacts removed"
    fi
else
    echo "Note: This doesn't appear to be the EmlRust project directory."
    echo "To completely remove EmlRust, you should also delete the project directory."
fi

if grep -q "\.local/bin" "$HOME/.bashrc" || grep -q "\.local/bin" "$HOME/.zshrc" || grep -q "\.local/bin" "$HOME/.profile"; then
    echo "The setup script may have modified your PATH in your shell configuration files."
    echo "Would you like to check and clean up these changes? (y/n)"
    read -r cleanup_path

    if [[ "$cleanup_path" =~ ^[Yy]$ ]]; then
        for file in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile"; do
            if [ -f "$file" ] && grep -q "\.local/bin" "$file"; then
                echo "Found PATH modification in $file"
                echo "Would you like to edit this file to remove the PATH modification? (y/n)"
                read -r edit_file

                if [[ "$edit_file" =~ ^[Yy]$ ]]; then
                    default_editor=${EDITOR:-nano}
                    echo "Opening $file with $default_editor..."
                    $default_editor "$file"
                    echo "Please remove the line that adds ~/.local/bin to your PATH."
                fi
            fi
        done
    fi
fi

echo ""
echo "====================================="
echo "   Uninstallation Complete!         "
echo "====================================="
echo ""
echo "EmlRust has been uninstalled from your system."
echo "If you want to completely remove all traces of EmlRust,"
echo "you should also delete the EmlRust project directory."
echo ""
