# Emlrust

Emlrust is a command-line tool written in Rust for processing email (.eml) files, specifically designed for preparing emails for use in phishing simulations. It can convert .eml files to HTML, modify anchor tags to use template URLs, anonymize email addresses, remove potentially dangerous JavaScript, and analyze IP addresses from campaign results using AbuseIPDB.

## Features

### Email Processing
- Convert .eml files to HTML format
- Replace all hyperlinks (anchor tags) with template URLs for phishing simulation tools
- Anonymize email addresses in the content
- Remove JavaScript from HTML files
- Process individual files or entire directories
- Add tracking codes to emails

### IP Reputation Analysis
- Extract IP addresses from Gophish campaign CSV exports
- Check IP reputation using AbuseIPDB API
- Generate detailed CSV reports with risk categorization
- Enhanced original CSV with reputation data
- Filter invalid IP addresses automatically
- Rate limiting to respect API limits

## Installation

### Quick Setup (Linux/macOS)

The easiest way to get started is using the provided setup script:

```bash
git clone https://github.com/Plasgianp/emlrust.git
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
git clone https://github.com/Plasgianp/emlrust.git
cd emlrust
cargo build --release

# The binary will be at ./target/release/emlrust
```

## Configuration

### Environment Variables

For IP analysis functionality, you'll need to set up API keys:

```bash
# Create a .env file in the project directory
echo "ABUSEIPDB_API_KEY=your_abuseipdb_api_key_here" > .env

# Or set as environment variable
export ABUSEIPDB_API_KEY="your_abuseipdb_api_key_here"
```

Get your AbuseIPDB API key from [https://www.abuseipdb.com/api](https://www.abuseipdb.com/api)

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
emlrust --script_removal -d /path/to/html/files

# Analyze IP addresses from campaign results
emlrust --analyze_ips campaign_results.csv
```

### Command Line Options

#### Email Processing
| Option | Long Form | Description |
|--------|-----------|-------------|
| `-r` | `--emls_to_htmls` | Convert .eml files to HTML |
| `-u` | `--modify_href` | Add {{.URL}} href in HTML files |
| `-e` | `--eml_file` | Convert a single .eml file to HTML |
| `-f` | `--html_file` | Add {{.URL}} href in a single HTML file |
| `-a` | `--go` | Combine --eml_file, --html_file, and --modify_email |
| `-d` | `--directory` | Directory containing .eml or HTML files |
| | `--script_removal` | Remove all script content in HTML files |
| | `--modify_email` | Modify email addresses in HTML files |
| | `--all` or `--gall` | Process all .eml files in a directory completely |

#### IP Analysis
| Option | Description |
|--------|-------------|
| `--analyze_ips <CSV_FILE>` | Analyze IP reputation from CSV file |
| `--output_report <FILE>` | Output CSV filename for IP reputation report (default: ip_reputation_report.csv) |
| `--enhanced_csv <FILE>` | Enhanced original CSV filename (default: enhanced_original_file.csv) |
| `--delay <SECONDS>` | Delay between API calls in seconds (default: 1.0) |
| `--api_key <KEY>` | AbuseIPDB API key (can also use ABUSEIPDB_API_KEY env var) |

#### Help & Examples
| Option | Description |
|--------|-------------|
| `--help` | Show help information |
| `--examples` | Show all example categories |
| `--examples <CATEGORY>` | Show examples for specific category (eml, html, ip, api, env) |

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

### IP Reputation Analysis

```bash
# Basic IP analysis from campaign results
export ABUSEIPDB_API_KEY="your_api_key_here"
emlrust --analyze_ips campaign_results.csv

# Advanced analysis with custom settings
emlrust --analyze_ips data.csv \
  --output_report detailed_ip_report.csv \
  --enhanced_csv enhanced_campaign_data.csv \
  --delay 2.0
```

### Getting Help and Examples

```bash
# Show general help
emlrust --help

# Show all example categories
emlrust --examples

# Show IP analysis examples
emlrust --examples ip

# Show email processing examples
emlrust --examples eml
```

## CSV File Requirements

For IP analysis, your CSV file must contain:

- **`message` column**: Should contain "Clicked Link" or "Submitted Data"
- **`details` column**: JSON data with browser information in the format:
  ```json
  {"browser":{"address":"192.168.1.1"}}
  ```

The tool will automatically filter and process only relevant rows and skip invalid IP addresses.

## Output Files

### IP Reputation Report
- Contains detailed reputation data for each unique IP address
- Includes abuse confidence percentage, country information, ISP details
- Risk categorization: High (>75%), Medium (25-75%), Low (≤25%)

### Enhanced Original CSV
- Your original CSV with additional IP reputation columns
- Maintains all original data while adding security insights
- Easy to import into analysis tools

## Troubleshooting

### Common Issues

**"API key not provided"**
```bash
# Set environment variable
export ABUSEIPDB_API_KEY="your_key"
# Or use command line argument
emlrust --analyze_ips data.csv --api_key YOUR_KEY
```

**"Invalid IP address: unknown"**
- This is normal - the tool automatically filters out invalid IPs

**Rate limiting errors**
- Increase delay between API calls: `--delay 2.0`
- Check your AbuseIPDB plan limits

### Getting Examples

Use the built-in examples system for specific help:
```bash
emlrust --examples ip      # IP analysis examples
emlrust --examples eml     # Email processing examples
emlrust --examples env     # Environment setup
```

## Building from Source

To build the latest version from source:

```bash
git clone https://github.com/Plasgianp/emlrust.git
cd emlrust
cargo build --release
```

The compiled binary will be in `./target/release/emlrust`.

## Dependencies

The tool uses the following main dependencies:
- `mail-parser` for .eml file parsing
- `scraper` for HTML manipulation
- `reqwest` for API calls
- `serde` for JSON handling
- `csv` for CSV processing
- `clap` for command-line interface


## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

---

**Security Note**: This tool is designed for authorized security testing and educational purposes. Always ensure you have permission before conducting any security assessments.
