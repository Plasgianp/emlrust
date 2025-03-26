mod emlrustlib;

use clap::{Parser, ArgAction, Command};
use dotenv::dotenv;
use emlrustlib::*;
use std::env;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'r', long = "emls_to_htmls", action = ArgAction::SetTrue)]
    emls_to_htmls: bool,

    #[arg(short = 'u', long = "modify_href", action = ArgAction::SetTrue)]
    modify_href: bool,

    #[arg(short = 's', long = "script_removal", action = ArgAction::SetTrue)]
    script_removal: bool,

    #[arg(long = "modify_email", action = ArgAction::SetTrue)]
    modify_email: bool,

    #[arg(short = 'd', long = "directory")]
    directory: Option<String>,

    #[arg(short = 'e', long = "eml_file")]
    eml_file: Option<String>,

    #[arg(short = 'f', long = "html_file")]
    html_file: Option<String>,

    #[arg(short = 'a', long = "go")]
    go_cmd: Option<String>,

    #[arg(long = "all", alias = "gall", action = ArgAction::SetTrue)]
    goes: bool,

    #[arg(short = 'c', long = "curl", action = ArgAction::SetTrue)]
    curl: bool,

    #[arg(short = 'g', long = "get_campaign_summary", action = ArgAction::SetTrue)]
    get_campaign_summary: bool,

    #[arg(short = 'h', long = "get_campaigns_summaries", action = ArgAction::SetTrue)]
    get_campaigns_summaries: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let url = env::var("url").unwrap_or_default();
    let api_key = env::var("api_key").unwrap_or_default();

    let args = Args::parse();
    
    // Check if any action flag is set
    let has_action = args.emls_to_htmls || args.modify_href || args.script_removal || 
                    args.modify_email || args.eml_file.is_some() || args.html_file.is_some() ||
                    args.go_cmd.is_some() || args.goes || args.curl || 
                    args.get_campaign_summary || args.get_campaigns_summaries;
    
    // If no flags are set, print help
    if !has_action {
        let mut cmd = Command::new("emlrust")
            .about("Process email (.eml) files for phishing simulations")
            .version(env!("CARGO_PKG_VERSION"));
        
        // Add all your arguments here (similar to how they're defined in Args)
        cmd = cmd.arg(
            clap::Arg::new("emls_to_htmls")
                .short('r')
                .long("emls_to_htmls")
                .help("Convert .eml files to HTML")
                .action(ArgAction::SetTrue),
        );
        
        cmd = cmd.arg(
            clap::Arg::new("modify_href")
                .short('u')
                .long("modify_href")
                .help("Add {{.URL}} href in HTML files")
                .action(ArgAction::SetTrue),
        );
        
        cmd = cmd.arg(
            clap::Arg::new("script_removal")
                .short('s')
                .long("script_removal")
                .help("Remove all script content in HTML files")
                .action(ArgAction::SetTrue),
        );
        
        cmd = cmd.arg(
            clap::Arg::new("modify_email")
                .long("modify_email")
                .help("Modify email addresses in HTML files")
                .action(ArgAction::SetTrue),
        );
        
        cmd = cmd.arg(
            clap::Arg::new("directory")
                .short('d')
                .long("directory")
                .help("Directory containing .eml or HTML files")
                .value_name("DIR"),
        );
        
        cmd = cmd.arg(
            clap::Arg::new("eml_file")
                .short('e')
                .long("eml_file")
                .help("Convert a single .eml file to HTML")
                .value_name("FILE"),
        );
        
        cmd = cmd.arg(
            clap::Arg::new("html_file")
                .short('f')
                .long("html_file")
                .help("Add {{.URL}} href in a single HTML file")
                .value_name("FILE"),
        );
        
        cmd = cmd.arg(
            clap::Arg::new("go")
                .short('a')
                .long("go")
                .help("Combine --eml_file, --html_file, and --modify_email")
                .value_name("FILE"),
        );
        
        cmd = cmd.arg(
            clap::Arg::new("all")
                .long("all")
                .alias("gall")
                .help("Does --go recursively")
                .action(ArgAction::SetTrue),
        );
        
        cmd = cmd.arg(
            clap::Arg::new("curl")
                .short('c')
                .long("curl")
                .help("General Curl Request to your Gophish")
                .action(ArgAction::SetTrue),
        );
        
        cmd = cmd.arg(
            clap::Arg::new("get_campaign_summary")
                .short('g')
                .long("get_campaign_summary")
                .help("Get Summary of a campaign")
                .action(ArgAction::SetTrue),
        );
        
        cmd = cmd.arg(
            clap::Arg::new("get_campaigns_summaries")
                .short('h')
                .long("get_campaigns_summaries")
                .help("Get Summary of all campaigns")
                .action(ArgAction::SetTrue),
        );
        
        cmd.print_help()?;
        println!();
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
                            let _ = fs::write(&output_file, html_content);
                        }
                        Err(_) => {}
                    }
                }
            }
        } else {
            println!("Please specify a directory with -d or --directory flag.");
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
                    let _ = add_href_to_file(path, new_href);
                }
            }
        } else {
            println!("Please specify a directory with -d or --directory flag.");
        }
    }

    if let Some(eml_file_path) = &args.eml_file {
        let path = Path::new(eml_file_path);
        if !path.exists() || path.is_dir() || !path.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("eml")) {
            println!("Invalid .eml file specified.");
            return Ok(());
        }

        match eml_to_html(path) {
            Ok(html_content) => {
                let output_file = path.with_extension("html");
                let _ = fs::write(&output_file, html_content);
            }
            Err(_) => {}
        }
    }

    if let Some(html_file_path) = &args.html_file {
        let path = Path::new(html_file_path);
        if !path.exists() || path.is_dir() || !path.extension().map_or(false, |ext| {
            ext.eq_ignore_ascii_case("html") || ext.eq_ignore_ascii_case("htm")
        }) {
            println!("Invalid HTML file specified.");
            return Ok(());
        }

        let _ = add_href_to_file(path, "{{.URL}}");
    }

    if let Some(go_cmd_path) = &args.go_cmd {
        let path = Path::new(go_cmd_path);
        if !path.exists() || path.is_dir() || !path.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("eml")) {
            println!("Invalid .eml file specified.");
            return Ok(());
        }

        match eml_to_html(path) {
            Ok(html_content) => {
                let modified_html = if args.modify_email {
                    anonymizer(&html_content)
                } else {
                    html_content
                };
                
                match add_href_to_anchor_tags(&modified_html, "{{.URL}}") {
                    Ok(modified_html_with_href) => {
                        let output_file = path.with_extension("html");
                        let _ = fs::write(&output_file, modified_html_with_href);
                    }
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }
        return Ok(());
    }

    if args.goes {
        if let Some(directory_path) = &args.directory {
            let _ = gophishing_everything(directory_path);
        } else {
            println!("Please specify a directory with -d or --directory flag.");
        }
        return Ok(());
    }

    if args.script_removal {
        if let Some(directory_path) = &args.directory {
            let _ = remove_scripts_from_directory(directory_path);
        } else if let Some(html_file_path) = &args.html_file {
            let path = Path::new(html_file_path);
            if !path.exists() || path.is_dir() || !path.extension().map_or(false, |ext| {
                ext.eq_ignore_ascii_case("html") || ext.eq_ignore_ascii_case("htm")
            }) {
                println!("Invalid HTML file specified.");
                return Ok(());
            }

            let _ = remove_scripts_from_file(path);
        } else {
            println!("Please specify a directory with -d or --directory flag or an HTML file with --html_file flag.");
        }
        return Ok(());
    }

    if args.curl || args.get_campaign_summary || args.get_campaigns_summaries {
        println!("API operations: Not implemented yet");
        println!("URL: {}, API Key: {}", url, api_key);
        
        if args.curl {
            println!("General Curl request to be implemented");
        }
        
        if args.get_campaign_summary {
            println!("Get campaign summary to be implemented");
        }
        
        if args.get_campaigns_summaries {
            println!("Get all campaigns summaries to be implemented");
        }
    }

    Ok(())
}