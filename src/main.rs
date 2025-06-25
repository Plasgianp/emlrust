mod emlrustlib;

use clap::{Parser, ArgAction};
use dotenv::dotenv;
use emlrustlib::*;
use std::env;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use tokio;

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "EML and IP Analysis Tool for Gophish",
    long_about = r#"
EML and IP Analysis Tool for Gophish

EXAMPLES:
  Basic EML Processing:
    emlrust -r -d /path/to/eml/directory              # Convert all .eml files to HTML
    emlrust -e single_email.eml                       # Convert single .eml file to HTML
    emlrust -u -d /path/to/html/directory             # Add {{.URL}} href to all HTML files
    emlrust -f single.html                            # Add {{.URL}} href to single HTML file

  Combined Operations:
    emlrust -a single_email.eml --modify_email        # Convert EML, add href, and anonymize emails
    emlrust --all -d /path/to/directory               # Process all .eml files recursively

  Script Removal:
    emlrust --script_removal -d /path/to/directory    # Remove scripts from all HTML files
    emlrust --script_removal -f single.html          # Remove scripts from single HTML file

  IP Analysis:
    emlrust --csvents campaign_results.csv        # Analyze IPs from CSV file
    emlrust --csvents data.csv --output_report my_report.csv --enhanced_csv enhanced.csv --delay 2.0
    
  API Operations (Coming Soon):
    emlrust -c                                        # General curl request to Gophish
    emlrust --get_campaign_summary                    # Get campaign summary
    emlrust --get_campaigns_summaries                 # Get all campaigns summaries

ENVIRONMENT VARIABLES:
  ABUSEIPDB_API_KEY or API_KEY    - API key for AbuseIPDB
  url                             - Gophish server URL
  api_key                        - Gophish API key
"#
)]
struct Args {
    /// Convert .eml files to HTML
    /// Example: emlrust -r -d /path/to/eml/directory
    #[arg(short = 'r', long = "emls_to_htmls", action = ArgAction::SetTrue)]
    emls_to_htmls: bool,

    /// Add {{.URL}} href in HTML files
    /// Example: emlrust -u -d /path/to/html/directory
    #[arg(short = 'u', long = "modify_href", action = ArgAction::SetTrue)]
    modify_href: bool,

    /// Remove all script content in HTML files
    /// Example: emlrust --script_removal -d /path/to/directory
    #[arg(long = "script_removal", action = ArgAction::SetTrue)]
    script_removal: bool,

    /// Modify email addresses in HTML files (anonymize with {{.Email}})
    /// Example: emlrust -a email.eml --modify_email
    #[arg(long = "modify_email", action = ArgAction::SetTrue)]
    modify_email: bool,

    /// Directory containing .eml or HTML files
    /// Example: emlrust -r -d /home/user/emails/
    #[arg(short = 'd', long = "directory")]
    directory: Option<String>,

    /// Convert a single .eml file to HTML
    /// Example: emlrust -e phishing_email.eml
    #[arg(short = 'e', long = "eml_file")]
    eml_file: Option<String>,

    /// Add {{.URL}} href in a single HTML file
    /// Example: emlrust -f template.html
    #[arg(short = 'f', long = "html_file")]
    html_file: Option<String>,

    /// Combine --eml_file, --html_file, and --modify_email
    /// Example: emlrust -a phishing_template.eml --modify_email
    #[arg(short = 'a', long = "go")]
    go_cmd: Option<String>,

    /// Process all .eml files recursively (like --go but for entire directory)
    /// Example: emlrust --all -d /path/to/email/templates/
    #[arg(long = "all", alias = "gall", action = ArgAction::SetTrue)]
    goes: bool,

    /// General Curl Request to your Gophish (requires env vars: url, api_key)
    /// Example: emlrust -c
    #[arg(short = 'c', long = "curl", action = ArgAction::SetTrue)]
    curl: bool,

    /// Get Summary of a campaign (requires env vars: url, api_key)
    /// Example: emlrust --get_campaign_summary
    #[arg(long = "get_campaign_summary", action = ArgAction::SetTrue)]
    get_campaign_summary: bool,

    /// Get Summary of all campaigns (requires env vars: url, api_key)
    /// Example: emlrust --get_campaigns_summaries
    #[arg(long = "get_campaigns_summaries", action = ArgAction::SetTrue)]
    get_campaigns_summaries: bool,

    // IP Analysis flags
    /// Analyze IP reputation from CSV file (requires ABUSEIPDB_API_KEY)
    /// Example: emlrust --csvents campaign_results.csv
    /// Example: emlrust --csvents data.csv --output_report ip_report.csv --delay 2.0
    #[arg(long = "csvents")]
    csvents: Option<String>,

    /// Output CSV filename for IP reputation report
    /// Example: --output_report custom_ip_report.csv
    #[arg(long = "output_report", default_value = "ip_reputation_report.csv")]
    output_report: String,

    /// Enhanced original CSV filename with IP data added
    /// Example: --enhanced_csv enhanced_campaign_data.csv
    #[arg(long = "enhanced_csv", default_value = "enhanced_original_file.csv")]
    enhanced_csv: String,

    /// Delay between API calls in seconds (respect rate limits)
    /// Example: --delay 2.0
    #[arg(long = "delay", default_value = "1.0")]
    delay: f64,

    /// AbuseIPDB API key (can also be set via ABUSEIPDB_API_KEY env var)
    /// Example: --api_key YOUR_ABUSEIPDB_API_KEY_HERE
    #[arg(long = "api_key")]
    api_key: Option<String>,

    /// Show examples for specific commands
    /// Example: emlrust --examples ip_analysis
    #[arg(long = "examples")]
    examples: Option<String>,
}

fn show_examples(category: &str) {
    match category {
        "eml" | "eml_processing" => {
            println!(r#"
EML PROCESSING EXAMPLES:

1. Convert single .eml file to HTML:
   emlrust -e phishing_email.eml

2. Convert all .eml files in directory to HTML:
   emlrust -r -d /path/to/eml/directory

3. Process single .eml with full pipeline (convert + add URL + anonymize):
   emlrust -a phishing_template.eml --modify_email

4. Process all .eml files recursively with full pipeline:
   emlrust --all -d /path/to/email/templates/
"#);
        }
        "html" | "html_processing" => {
            println!(r#"
HTML PROCESSING EXAMPLES:

1. Add {{.URL}} href to single HTML file:
   emlrust -f template.html

2. Add {{.URL}} href to all HTML files in directory:
   emlrust -u -d /path/to/html/directory

3. Remove scripts from single HTML file:
   emlrust --script_removal -f template.html

4. Remove scripts from all HTML files in directory:
   emlrust --script_removal -d /path/to/html/directory
"#);
        }
        "ip" | "ip_analysis" => {
            println!(r#"
IP ANALYSIS EXAMPLES:

1. Basic IP analysis (requires ABUSEIPDB_API_KEY environment variable):
   export ABUSEIPDB_API_KEY="your_api_key_here"
   emlrust --csvents campaign_results.csv

2. IP analysis with custom output files:
   emlrust --csvents data.csv --output_report my_ip_report.csv --enhanced_csv my_enhanced_data.csv

3. IP analysis with custom delay (slower, more respectful to API):
   emlrust --csvents data.csv --delay 2.0

4. IP analysis with API key as argument:
   emlrust --csvents data.csv --api_key YOUR_ABUSEIPDB_API_KEY_HERE

Note: CSV file must have 'message' and 'details' columns.
Only processes rows where message = 'Clicked Link' or 'Submitted Data'.
"#);
        }
        "api" | "gophish" => {
            println!(r#"
GOPHISH API EXAMPLES (Coming Soon):

Set environment variables first:
   export url="https://your-gophish-server.com"
   export api_key="your_gophish_api_key"

1. General curl request:
   emlrust -c

2. Get campaign summary:
   emlrust --get_campaign_summary

3. Get all campaigns summaries:
   emlrust --get_campaigns_summaries
"#);
        }
        "env" | "environment" => {
            println!(r#"
ENVIRONMENT VARIABLES:

Required for IP Analysis:
   export ABUSEIPDB_API_KEY="your_abuseipdb_api_key"
   # OR
   export API_KEY="your_abuseipdb_api_key"

Required for Gophish API operations:
   export url="https://your-gophish-server.com"
   export api_key="your_gophish_api_key"

Using .env file:
   Create a .env file in your project directory:
   ABUSEIPDB_API_KEY=your_abuseipdb_api_key
   url=https://your-gophish-server.com
   api_key=your_gophish_api_key
"#);
        }
        _ => {
            println!(r#"
AVAILABLE EXAMPLE CATEGORIES:

  emlrust --examples eml              # EML processing examples
  emlrust --examples html             # HTML processing examples  
  emlrust --examples ip               # IP analysis examples
  emlrust --examples api              # Gophish API examples
  emlrust --examples env              # Environment variables

QUICK START EXAMPLES:

  # Convert EML to HTML and add tracking
  emlrust -a phishing_template.eml --modify_email

  # Analyze IPs from campaign results
  export ABUSEIPDB_API_KEY="your_key"
  emlrust --csvents campaign_results.csv

  # Process entire directory
  emlrust --all -d /path/to/templates/
"#);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let url = env::var("url").unwrap_or_default();
    let gophish_api_key = env::var("api_key").unwrap_or_default();
    
    let args = Args::parse();

    if let Some(category) = &args.examples {
        show_examples(category);
        return Ok(());
    }

    if let Some(csv_file_path) = &args.csvents {
        let abuseipdb_api_key = args.api_key
            .or_else(|| env::var("ABUSEIPDB_API_KEY").ok())
            .or_else(|| env::var("API_KEY").ok());
        
        if let Some(api_key) = abuseipdb_api_key {
            match run_ip_analysis(
                csv_file_path,
                &api_key,
                &args.output_report,
                &args.enhanced_csv,
                args.delay,
            ).await {
                Ok(_) => println!("IP analysis completed successfully!"),
                Err(e) => eprintln!("Error during IP analysis: {}", e),
            }
        } else {
            eprintln!("Error: AbuseIPDB API key not provided.");
            eprintln!("Set it via --api_key flag or ABUSEIPDB_API_KEY/API_KEY environment variable.");
            eprintln!("\nFor examples: emlrust --examples ip");
        }
        return Ok(());
    }

    if args.emls_to_htmls {
        if let Some(directory_path) = &args.directory {
            for entry in WalkDir::new(directory_path).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_dir() && path.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("eml")) {
                    match eml_to_html(path) {
                        Ok(html_content) => {
                            let output_file = path.with_extension("html");
                            match fs::write(&output_file, html_content) {
                                Ok(_) => println!("Created {}", output_file.display()),
                                Err(err) => println!("Error writing to {}: {}", output_file.display(), err),
                            }
                        }
                        Err(err) => println!("Error processing {}: {}", path.display(), err),
                    }
                }
            }
        } else {
            println!("Please specify a directory with -d or --directory flag.");
            println!("Example: emlrust -r -d /path/to/eml/directory");
            println!("For more examples: emlrust --examples eml");
        }
    }

    if args.modify_href {
        if let Some(directory_path) = &args.directory {
            let new_href = "{{.URL}}";
            for entry in WalkDir::new(directory_path).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_dir() && path.extension().map_or(false, |ext| {
                    ext.eq_ignore_ascii_case("html") || ext.eq_ignore_ascii_case("htm")
                }) {
                    match add_href_to_file(path, new_href) {
                        Ok(_) => println!("Modified {}", path.display()),
                        Err(err) => println!("Error modifying {}: {}", path.display(), err),
                    }
                }
            }
        } else {
            println!("Please specify a directory with -d or --directory flag.");
            println!("Example: emlrust -u -d /path/to/html/directory");
            println!("For more examples: emlrust --examples html");
        }
    }

    if let Some(eml_file_path) = &args.eml_file {
        let path = Path::new(eml_file_path);
        if !path.exists() || path.is_dir() || !path.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("eml")) {
            println!("Invalid .eml file specified.");
            println!("Example: emlrust -e phishing_email.eml");
            return Ok(());
        }

        match eml_to_html(path) {
            Ok(html_content) => {
                let output_file = path.with_extension("html");
                match fs::write(&output_file, html_content) {
                    Ok(_) => println!("Created {}", output_file.display()),
                    Err(err) => println!("Error writing to {}: {}", output_file.display(), err),
                }
            }
            Err(err) => println!("Error processing {}: {}", path.display(), err),
        }
    }

    if let Some(html_file_path) = &args.html_file {
        let path = Path::new(html_file_path);
        if !path.exists() || path.is_dir() || !path.extension().map_or(false, |ext| {
            ext.eq_ignore_ascii_case("html") || ext.eq_ignore_ascii_case("htm")
        }) {
            println!("Invalid HTML file specified.");
            println!("Example: emlrust -f template.html");
            return Ok(());
        }

        match add_href_to_file(path, "{{.URL}}") {
            Ok(_) => println!("Modified {}", path.display()),
            Err(err) => println!("Error modifying {}: {}", path.display(), err),
        }
    }

    if let Some(go_cmd_path) = &args.go_cmd {
        let path = Path::new(go_cmd_path);
        if !path.exists() || path.is_dir() || !path.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("eml")) {
            println!("Invalid .eml file specified.");
            println!("Example: emlrust -a phishing_template.eml --modify_email");
            return Ok(());
        }

        match eml_to_html(path) {
            Ok(html_content) => {
                let modified_html = if args.modify_email {
                    anonymizer(&html_content, None, None)
                } else {
                    html_content
                };
                
                match add_href_to_anchor_tags(&modified_html, "{{.URL}}") {
                    Ok(modified_html_with_href) => {
                        let output_file = path.with_extension("html");
                        match fs::write(&output_file, modified_html_with_href) {
                            Ok(_) => println!("Created {}", output_file.display()),
                            Err(err) => println!("Error writing to {}: {}", output_file.display(), err),
                        }
                    }
                    Err(err) => println!("Error adding href: {}", err),
                }
            }
            Err(err) => println!("Error processing {}: {}", path.display(), err),
        }
        return Ok(());
    }

    if args.goes {
        if let Some(directory_path) = &args.directory {
            match gophishing_everything(directory_path) {
                Ok(_) => println!("Successfully processed all .eml files in {}", directory_path),
                Err(err) => println!("Error in GophishingEverything: {}", err),
            }
        } else {
            println!("Please specify a directory with -d or --directory flag.");
            println!("Example: emlrust --all -d /path/to/email/templates/");
            println!("For more examples: emlrust --examples eml");
        }
        return Ok(());
    }

    if args.script_removal {
        if let Some(directory_path) = &args.directory {
            match remove_scripts_from_directory(directory_path) {
                Ok(_) => println!("Successfully removed scripts from all HTML files in {}", directory_path),
                Err(err) => println!("Error removing scripts from directory: {}", err),
            }
        } else if let Some(html_file_path) = &args.html_file {
            let path = Path::new(html_file_path);
            if !path.exists() || path.is_dir() || !path.extension().map_or(false, |ext| {
                ext.eq_ignore_ascii_case("html") || ext.eq_ignore_ascii_case("htm")
            }) {
                println!("Invalid HTML file specified.");
                println!("Example: emlrust --script_removal -f template.html");
                return Ok(());
            }

            match remove_scripts_from_file(path) {
                Ok(_) => println!("Successfully removed scripts from {}", path.display()),
                Err(err) => println!("Error removing scripts from {}: {}", path.display(), err),
            }
        } else {
            println!("Please specify a directory with -d or --directory flag or an HTML file with --html_file flag.");
            println!("Examples:");
            println!("  emlrust --script_removal -d /path/to/html/directory");
            println!("  emlrust --script_removal -f template.html");
            println!("For more examples: emlrust --examples html");
        }
        return Ok(());
    }

    if args.curl || args.get_campaign_summary || args.get_campaigns_summaries {
        println!("API operations: Not implemented yet");
        println!("URL: {}, API Key: {}", url, gophish_api_key);
        
        if args.curl {
            println!("General Curl request to be implemented");
        }
        
        if args.get_campaign_summary {
            println!("Get campaign summary to be implemented");
        }
        
        if args.get_campaigns_summaries {
            println!("Get all campaigns summaries to be implemented");
        }

        println!("For API examples: emlrust --examples api");
    }

    if !args.emls_to_htmls && !args.modify_href && args.eml_file.is_none() && args.html_file.is_none() 
        && args.go_cmd.is_none() && !args.goes && !args.script_removal && !args.curl 
        && !args.get_campaign_summary && !args.get_campaigns_summaries && args.csvents.is_none() {
        println!("No action specified. Use --help for usage information or --examples for examples.");
        println!("\nQuick examples:");
        println!("  emlrust --examples           # Show all example categories");
        println!("  emlrust -a template.eml      # Process single EML file");
        println!("  emlrust --csvents data.csv  # Analyze IP addresses");
    }

    Ok(())
}
