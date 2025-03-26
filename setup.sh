#!/bin/bash

echo "====================================="
echo "       EmlRust Setup Script         "
echo "====================================="

if ! command -v cargo &> /dev/null; then
    echo "Rust is not installed. Would you like to install it? (y/n)"
    read -r install_rust
    if [[ "$install_rust" =~ ^[Yy]$ ]]; then
        echo "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
        source "$HOME/.cargo/env"
        echo "Rust has been installed successfully."
    else
        echo "Rust is required to build emlrust. Please install Rust and try again."
        exit 1
    fi
else
    echo "✓ Rust is already installed."
fi

echo "Building emlrust..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "Build failed. Please check the error messages above."
    exit 1
fi

echo "✓ Build successful! Binary created at $(pwd)/target/release/emlrust"

echo ""
echo "Would you like to create a symlink to make emlrust accessible from anywhere? (y/n)"
read -r create_symlink

if [[ "$create_symlink" =~ ^[Yy]$ ]]; then
    echo "Where would you like to create the symlink?"
    echo "1) ~/.local/bin (recommended for local user)"
    echo "2) /usr/local/bin (system-wide, requires sudo)"
    echo "3) Custom location"
    read -r symlink_option
    
    case $symlink_option in
        1)
            symlink_path="$HOME/.local/bin"
            if [ ! -d "$symlink_path" ]; then
                echo "Directory $symlink_path doesn't exist. Creating it..."
                mkdir -p "$symlink_path"
            fi
            ln -sf "$(pwd)/target/release/emlrust" "$symlink_path/emlrust"
            echo "✓ Symlink created at $symlink_path/emlrust"
            
            if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
                echo "Note: $symlink_path is not in your PATH."
                echo "Would you like to add it to your PATH? (y/n)"
                read -r add_to_path
                if [[ "$add_to_path" =~ ^[Yy]$ ]]; then
                    shell_profile=""
                    if [ -f "$HOME/.bashrc" ]; then
                        shell_profile="$HOME/.bashrc"
                    elif [ -f "$HOME/.zshrc" ]; then
                        shell_profile="$HOME/.zshrc"
                    elif [ -f "$HOME/.profile" ]; then
                        shell_profile="$HOME/.profile"
                    fi
                    
                    if [ -n "$shell_profile" ]; then
                        echo "export PATH=\"\$HOME/.local/bin:\$PATH\"" >> "$shell_profile"
                        echo "✓ Added $symlink_path to PATH in $shell_profile"
                        echo "Please restart your terminal or run 'source $shell_profile' for the changes to take effect."
                    else
                        echo "Could not find a shell profile to update. Please add $symlink_path to your PATH manually."
                    fi
                fi
            fi
            ;;
        2)
            symlink_path="/usr/local/bin"
            echo "Creating symlink in $symlink_path (requires sudo)"
            sudo ln -sf "$(pwd)/target/release/emlrust" "$symlink_path/emlrust"
            
            if [ $? -eq 0 ]; then
                echo "✓ Symlink created at $symlink_path/emlrust"
            else
                echo "Failed to create symlink. You may need to manually copy the binary:"
                echo "sudo cp $(pwd)/target/release/emlrust $symlink_path/"
            fi
            ;;
        3)
            echo "Please enter the full path where you want to create the symlink:"
            read -r custom_path
            
            if [ -d "$custom_path" ]; then
                ln -sf "$(pwd)/target/release/emlrust" "$custom_path/emlrust"
                echo "✓ Symlink created at $custom_path/emlrust"
                
                if [[ ":$PATH:" != *":$custom_path:"* ]]; then
                    echo "Note: $custom_path is not in your PATH. You may need to add it."
                fi
            else
                echo "Directory $custom_path doesn't exist. Would you like to create it? (y/n)"
                read -r create_dir
                if [[ "$create_dir" =~ ^[Yy]$ ]]; then
                    mkdir -p "$custom_path"
                    if [ $? -eq 0 ]; then
                        ln -sf "$(pwd)/target/release/emlrust" "$custom_path/emlrust"
                        echo "✓ Symlink created at $custom_path/emlrust"
                    else
                        echo "Failed to create directory. You may need to manually copy the binary."
                    fi
                else
                    echo "Skipping symlink creation."
                fi
            fi
            ;;
        *)
            echo "Invalid option. Skipping symlink creation."
            ;;
    esac
else
    echo "Skipping symlink creation."
    echo "You can run the binary directly from $(pwd)/target/release/emlrust"
fi

echo ""
echo "====================================="
echo "       Setup Complete!              "
echo "====================================="
echo ""
echo "To get started, run:"
echo "  emlrust --help"
echo ""
echo "If you didn't create a symlink, run:"
echo "  $(pwd)/target/release/emlrust --help"