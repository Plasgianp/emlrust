# EmlRust

EmlRust is a command-line tool written in Rust for processing email (.eml) files, specifically designed for preparing emails for use in phishing simulations. It can convert .eml files to HTML, modify anchor tags to use template URLs, anonymize email addresses, and remove potentially dangerous JavaScript.

## Features

- Convert .eml files to HTML format
- Replace all hyperlinks (anchor tags) with template URLs for phishing simulation tools
- Anonymize email addresses in the content
- Remove JavaScript from HTML files
- Process individual files or entire directories
- Add tracking codes to emails

## Installation

### Quick Setup (Linux/macOS)

The easiest way to get started is using the provided setup script:

```bash
git clone https://github.com/yourusername/emlrust.git
cd emlrust
chmod +x setup.sh
./setup.sh
```

The setup script will:
1. Check if Rust is installed and install it if needed
2. Build the project
3. Optionally create a symlink to make `emlrust` accessible from any directory

### Manual Installation

If you prefer to install manually:

```bash
# Install Rust if you don't have it
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build the project
git clone https://github.com/yourusername/emlrust.git
cd emlrust
cargo build --release

# The binary will be at ./target/release/emlrust
```

## Usage

### Basic Commands

```bash
# Convert a single .eml file to HTML
emlrust -e /path/to/file.eml

# Convert all .eml files in a directory to HTML
emlrust -r -d /path/to/directory

# Convert .eml files and add template URL to all links
emlrust -a /path/to/file.eml

# Process all .eml files in a directory (convert, add template URL, anonymize emails, remove scripts)
emlrust --all -d /path/to/directory

# Add template URL to links in existing HTML files
emlrust -u -d /path/to/html/files

# Remove JavaScript from HTML files
emlrust -s -d /path/to/html/files
```

### Command Line Options

| Option | Long Form | Description |
|--------|-----------|-------------|
| `-r` | `--emls_to_htmls` | Convert .eml files to HTML |
| `-u` | `--modify_href` | Add {{.URL}} href in HTML files |
| `-s` | `--script_removal` | Remove all script content in HTML files |
| | `--modify_email` | Modify email addresses in HTML files |
| `-d` | `--directory` | Directory containing .eml or HTML files |
| `-e` | `--eml_file` | Convert a single .eml file to HTML |
| `-f` | `--html_file` | Add {{.URL}} href in a single HTML file |
| `-a` | `--go` | Combine --eml_file, --html_file, and --modify_email |
| | `--all` or `--gall` | Process all .eml files in a directory completely |

## Examples

### Converting a Batch of .eml Files

```bash
emlrust --all -d /path/to/email/directory
```

This will:
1. Find all .eml files in the directory
2. Convert them to HTML
3. Anonymize email addresses
4. Replace all links with {{.URL}} template
5. Add tracker code
6. Remove JavaScript
7. Create .html files with the same base filename

### Processing a Single Email

```bash
emlrust -a /path/to/email.eml --modify_email
```

This converts a single email to HTML, anonymizes email addresses, and adds template URLs to links.

## Building from Source

To build the latest version from source:

```bash
git clone https://github.com/yourusername/emlrust.git
cd emlrust
cargo build --release
```

The compiled binary will be in `./target/release/emlrust`.

## License

[Your chosen license]

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.