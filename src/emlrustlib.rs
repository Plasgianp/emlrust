use regex::Regex;
use scraper::{Html, Selector};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use csv::{Reader, Writer};
use std::net::IpAddr;

use reqwest;
use tokio;


pub fn read_values_from_file(file_path: &str) -> Result<Vec<String>, io::Error> {
    let expanded_path = if file_path.starts_with("~") {
        let home = home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Home directory not found"))?;
        home.join(&file_path[1..])
    } else {
        PathBuf::from(file_path)
    };

    let content = fs::read_to_string(expanded_path)?;
    
    Ok(content.lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect())
}

pub fn eml_to_html(eml_file: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(eml_file)?;

    let message = mail_parser::MessageParser::default()
        .parse(content.as_bytes())
        .ok_or("Failed to parse email message")?;

    if let Some(html_body) = message.body_html(0) {
        return Ok(html_body.to_string());
    }

    if let Some(text_body) = message.body_text(0) {
        return Ok(format!("<html><body><pre>{}</pre></body></html>", text_body));
    }

    Ok("<html><body>No content found</body></html>".to_string())
}

pub fn add_href_to_anchor_tags(html_content: &str, new_href: &str) -> Result<String, Box<dyn std::error::Error>> {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("a").unwrap();
    
    let mut modified_html = html_content.to_string();
    
    for element in document.select(&selector) {
        let html_before = element.html();
        let node = element.value();
        
        let mut attrs = node.attrs().collect::<Vec<_>>();
        let mut has_href = false;
        
        for (idx, (name, _)) in attrs.iter().enumerate() {
            if *name == "href" {
                has_href = true;
                attrs[idx] = ("href", new_href);
                break;
            }
        }
        
        if !has_href {
            attrs.push(("href", new_href));
        }
        
        let mut new_html = String::from("<a");
        for (name, value) in attrs {
            new_html.push_str(&format!(" {}=\"{}\"", name, value));
        }
        new_html.push('>');
        
        let inner_html = element.inner_html();
        new_html.push_str(&inner_html);
        new_html.push_str("</a>");
        
        modified_html = modified_html.replace(&html_before, &new_html);
    }
    
    Ok(modified_html)
}

pub fn add_href_to_file(file_path: &Path, new_href: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;

    let modified_html = add_href_to_anchor_tags(&content, new_href)?;

    let modified_html = modified_html.replace("</html>", "{{.Tracker}}\n</html>");

    fs::write(file_path, modified_html)?;
    
    Ok(())
}

pub fn anonymizer(html_content: &str, _nomi: Option<&[String]>, _cognomi: Option<&[String]>) -> String {
    let email_pattern = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b").unwrap();
    let modified_html = email_pattern.replace_all(html_content, "{{.Email}}").to_string();

    modified_html
}

pub fn is_valid_ip(ip_str: &str) -> bool {
    ip_str.parse::<IpAddr>().is_ok()
}

pub fn gophishing_everything(directory_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    for entry in WalkDir::new(directory_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        
        if !path.is_dir() && path.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("eml")) {
            let html_content = eml_to_html(path)?;

            let modified_html = anonymizer(&html_content, None, None);
            let modified_html_with_href = add_href_to_anchor_tags(&modified_html, "{{.URL}}")?;

            let modified_html_with_href = modified_html_with_href.replace("</html>", "{{.Tracker}}\n</html>");

            let modified_html_no_script = remove_scripts(&modified_html_with_href)?;

            let output_file = path.with_extension("html");
            fs::write(&output_file, modified_html_no_script)?;
            println!("Created {}", output_file.display());
        }
    }

    Ok(())
}

pub fn remove_scripts(html_content: &str) -> Result<String, Box<dyn std::error::Error>> {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("script").unwrap();
    
    let mut modified_html = html_content.to_string();
    
    for element in document.select(&selector) {
        let html = element.html();
        modified_html = modified_html.replace(&html, "");
    }
    
    Ok(modified_html)
}

pub fn remove_scripts_from_file(html_file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(html_file)?;

    let modified_html = remove_scripts(&content)?;

    fs::write(html_file, modified_html)?;
    
    Ok(())
}

pub fn remove_scripts_from_directory(directory_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    for entry in WalkDir::new(directory_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        
        if !path.is_dir() && path.extension().map_or(false, |ext| {
            ext.eq_ignore_ascii_case("html") || ext.eq_ignore_ascii_case("htm")
        }) {
            remove_scripts_from_file(path)?;
            println!("Removed scripts from {}", path.display());
        }
    }
    
    Ok(())
}


#[derive(Debug, Serialize, Deserialize)]
pub struct BrowserInfo {
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Details {
    pub browser: BrowserInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CsvRow {
    pub message: String,
    pub details: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpReputationData {
    #[serde(rename = "isPublic")]
    pub is_public: Option<bool>,  
    #[serde(rename = "ipVersion")]
    pub ip_version: Option<u8>,   
    #[serde(rename = "isWhitelisted")]
    pub is_whitelisted: Option<bool>,  
    #[serde(rename = "abuseConfidencePercentage")]
    pub abuse_confidence: Option<u8>,  
    #[serde(rename = "countryCode")]
    pub country_code: Option<String>,  
    #[serde(rename = "countryName")]
    pub country_name: Option<String>,  
    #[serde(rename = "usageType")]
    pub usage_type: Option<String>,    
    pub isp: Option<String>,           
    pub domain: Option<String>,        
    #[serde(rename = "totalReports")]
    pub total_reports: Option<u32>,    
    #[serde(rename = "numDistinctUsers")]
    pub num_distinct_users: Option<u32>, 
    #[serde(rename = "lastReportedAt")]
    pub last_reported_at: Option<String>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct AbuseIpDbResponse {
    pub data: IpReputationData,
}

#[derive(Debug, Serialize)]
pub struct IpReputationReport {
    pub ip_address: String,
    pub is_public: Option<bool>,
    pub ip_version: Option<u8>,
    pub is_whitelisted: Option<bool>,
    pub abuse_confidence: Option<u8>,
    pub country_code: Option<String>,
    pub country_name: Option<String>,
    pub usage_type: Option<String>,
    pub isp: Option<String>,
    pub domain: Option<String>,
    pub total_reports: Option<u32>,
    pub num_distinct_users: Option<u32>,
    pub last_reported_at: Option<String>,
    pub checked_at: String,
}

pub fn extract_unique_ips_from_csv(csv_file_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut unique_ips = HashSet::new();
    let file = fs::File::open(csv_file_path)?;
    let mut rdr = Reader::from_reader(file);
    
    let target_messages = vec!["Clicked Link", "Submitted Data"];
    let mut total_rows = 0;
    let mut filtered_rows = 0;
    let mut invalid_ips = Vec::new();
    
    let headers = rdr.headers()?.clone();
    let message_col_idx = headers.iter().position(|h| h == "message")
        .ok_or("'message' column not found in CSV file")?;
    let details_col_idx = headers.iter().position(|h| h == "details")
        .ok_or("'details' column not found in CSV file")?;
    
    for result in rdr.records() {
        total_rows += 1;
        let record = result?;
        
        let message = record.get(message_col_idx).unwrap_or("");
        let details = record.get(details_col_idx).unwrap_or("");
        
        if target_messages.contains(&message) {
            filtered_rows += 1;
            
            if !details.is_empty() {
                match serde_json::from_str::<Details>(details) {
                    Ok(parsed_details) => {
                        let ip = parsed_details.browser.address;
                        
                        if is_valid_ip(&ip) {
                            unique_ips.insert(ip);
                        } else {
                            invalid_ips.push(ip);
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Invalid JSON in row {}, skipping... Error: {}", total_rows, e);
                        continue;
                    }
                }
            }
        }
    }
    
    println!("Found {} rows with target messages out of {} total rows", filtered_rows, total_rows);
    
    if !invalid_ips.is_empty() {
        println!("Skipped {} invalid IP addresses:", invalid_ips.len());
        for invalid_ip in invalid_ips {
            println!("  - {} (not a valid IP address)", invalid_ip);
        }
    }
    
    Ok(unique_ips.into_iter().collect())
}

pub async fn check_ip_reputation(
    ip_address: &str, 
    api_key: &str, 
    max_age_days: u32
) -> Result<IpReputationData, Box<dyn std::error::Error>> {
    if !is_valid_ip(ip_address) {
        return Err(format!("Invalid IP address: {}", ip_address).into());
    }

    let client = reqwest::Client::new();
    let url = "https://api.abuseipdb.com/api/v2/check";
    
    let response = client
        .get(url)
        .header("Key", api_key)
        .header("Accept", "application/json")
        .query(&[
            ("ipAddress", ip_address),
            ("maxAgeInDays", &max_age_days.to_string()),
            ("verbose", ""),
        ])
        .timeout(Duration::from_secs(10))
        .send()
        .await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("API request failed with status: {} - {}", status, error_text).into());
    }
    
    let response_text = response.text().await?;
    
    match serde_json::from_str::<AbuseIpDbResponse>(&response_text) {
        Ok(abuse_response) => Ok(abuse_response.data),
        Err(e) => {
            eprintln!("JSON parsing error for IP {}: {}", ip_address, e);
            eprintln!("Response body: {}", response_text);
            Err(format!("Failed to parse API response for IP {}: {}", ip_address, e).into())
        }
    }
}

pub async fn analyze_ips_with_abuseipdb(
    ip_list: Vec<String>,
    api_key: &str,
    output_csv: &str,
    delay_seconds: f64,
) -> Result<Vec<IpReputationReport>, Box<dyn std::error::Error>> {
    println!("Analyzing {} unique IP addresses...", ip_list.len());
    
    let mut results = Vec::new();
    let delay_duration = Duration::from_secs_f64(delay_seconds);
    
    for (i, ip) in ip_list.iter().enumerate() {
        println!("Checking IP {}/{}: {}", i + 1, ip_list.len(), ip);
        
        let reputation_data = check_ip_reputation(ip, api_key, 90).await;
        let checked_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        let result = match reputation_data {
            Ok(data) => IpReputationReport {
                ip_address: ip.clone(),
                is_public: data.is_public,
                ip_version: data.ip_version,
                is_whitelisted: data.is_whitelisted,
                abuse_confidence: data.abuse_confidence,
                country_code: data.country_code,
                country_name: data.country_name,
                usage_type: data.usage_type,
                isp: data.isp,
                domain: data.domain,
                total_reports: data.total_reports,
                num_distinct_users: data.num_distinct_users,
                last_reported_at: data.last_reported_at,
                checked_at,
            },
            Err(e) => {
                eprintln!("Error checking IP {}: {}", ip, e);
                IpReputationReport {
                    ip_address: ip.clone(),
                    is_public: None,
                    ip_version: None,
                    is_whitelisted: None,
                    abuse_confidence: None,
                    country_code: None,
                    country_name: None,
                    usage_type: None,
                    isp: None,
                    domain: None,
                    total_reports: None,
                    num_distinct_users: None,
                    last_reported_at: None,
                    checked_at,
                }
            }
        };
        
        results.push(result);
        
        if i < ip_list.len() - 1 {
            tokio::time::sleep(delay_duration).await;
        }
    }
    
    let file = fs::File::create(output_csv)?;
    let mut wtr = Writer::from_writer(file);
    
    for result in &results {
        wtr.serialize(result)?;
    }
    wtr.flush()?;
    
    println!("\nIP reputation report saved to: {}", output_csv);
    
    let valid_results: Vec<_> = results.iter()
        .filter(|r| r.abuse_confidence.is_some())
        .collect();
    
    if !valid_results.is_empty() {
        let _high_risk: Vec<_> = valid_results.iter()
            .filter(|r| r.abuse_confidence.unwrap_or(0) > 75)
            .collect();
        let _medium_risk: Vec<_> = valid_results.iter()
            .filter(|r| {
                let conf = r.abuse_confidence.unwrap_or(0);
                conf > 25 && conf <= 75
            })
            .collect();
        let _low_risk: Vec<_> = valid_results.iter()
            .filter(|r| r.abuse_confidence.unwrap_or(0) <= 25)
            .collect();
        
        if !_high_risk.is_empty() {
            println!("\nHigh risk IPs:");
            for result in _high_risk {
                println!(
                    "  - {} ({}% confidence, {})",
                    result.ip_address,
                    result.abuse_confidence.unwrap_or(0),
                    result.country_name.as_ref().unwrap_or(&"Unknown".to_string())
                );
            }
        }
    } else {
        println!("\nNote: All IP lookups failed. Please check your API key and network connection.");
    }
    
    Ok(results)
}

pub fn create_enhanced_csv(
    original_csv_path: &str,
    reputation_data: &HashMap<String, &IpReputationReport>,
    output_csv: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let input_file = fs::File::open(original_csv_path)?;
    let mut rdr = Reader::from_reader(input_file);
    
    let output_file = fs::File::create(output_csv)?;
    let mut wtr = Writer::from_writer(output_file);
    
    let headers = rdr.headers()?.clone();
    let mut new_headers = headers.iter().collect::<Vec<_>>();
    new_headers.extend_from_slice(&[
        "ip_address",
        "abuse_confidence", 
        "country_name",
        "usage_type",
        "isp",
        "domain",
        "total_reports",
        "num_distinct_users"
    ]);
    wtr.write_record(&new_headers)?;
    
    let details_col_idx = headers.iter().position(|h| h == "details")
        .ok_or("'details' column not found in CSV file")?;
    
    let mut enhanced_rows = 0;
    
    for result in rdr.records() {
        let record = result?;
        let mut new_record = record.iter().collect::<Vec<_>>();
        
        let mut ip_address = String::new();
        let mut abuse_confidence = String::new();
        let mut country_name = String::new();
        let mut usage_type = String::new();
        let mut isp = String::new();
        let mut domain = String::new();
        let mut total_reports = String::new();
        let mut num_distinct_users = String::new();
        
        if let Some(details) = record.get(details_col_idx) {
            if !details.is_empty() {
                if let Ok(parsed_details) = serde_json::from_str::<Details>(details) {
                    ip_address = parsed_details.browser.address.clone();
                    
                    if let Some(rep_data) = reputation_data.get(&ip_address) {
                        enhanced_rows += 1;
                        abuse_confidence = rep_data.abuse_confidence
                            .map(|c| c.to_string())
                            .unwrap_or_default();
                        country_name = rep_data.country_name.clone().unwrap_or_default();
                        usage_type = rep_data.usage_type.clone().unwrap_or_default();
                        isp = rep_data.isp.clone().unwrap_or_default();
                        domain = rep_data.domain.clone().unwrap_or_default();
                        total_reports = rep_data.total_reports
                            .map(|r| r.to_string())
                            .unwrap_or_default();
                        num_distinct_users = rep_data.num_distinct_users
                            .map(|u| u.to_string())
                            .unwrap_or_default();
                    }
                }
            }
        }
        
        new_record.extend_from_slice(&[
            &ip_address,
            &abuse_confidence,
            &country_name,
            &usage_type,
            &isp,
            &domain,
            &total_reports,
            &num_distinct_users,
        ]);
        
        wtr.write_record(&new_record)?;
    }
    
    wtr.flush()?;
    println!("Enhanced CSV saved to: {}", output_csv);
    println!("Enhanced {} rows with reputation data", enhanced_rows);
    
    Ok(())
}

pub async fn run_ip_analysis(
    csv_file_path: &str,
    api_key: &str,
    output_report: &str,
    enhanced_csv: &str,
    delay: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Analyzing CSV file: {}", csv_file_path);
    
    println!("Extracting IP addresses from CSV...");
    let ip_addresses = extract_unique_ips_from_csv(csv_file_path)?;
    println!("Found {} unique IP addresses:", ip_addresses.len());
    for ip in &ip_addresses {
        println!("  - {}", ip);
    }
    
    if ip_addresses.is_empty() {
        println!("No IP addresses found to analyze.");
        println!("Make sure your CSV has 'details' and 'message' columns with the expected data.");
        return Ok(());
    }
    
    println!("\nChecking IP reputation using AbuseIPDB API...");
    let reputation_results = analyze_ips_with_abuseipdb(
        ip_addresses,
        api_key,
        output_report,
        delay,
    ).await?;
    
    println!("\nCreating enhanced copy of original CSV...");
    println!("Processing {} IP records...", reputation_results.len());
    
    let reputation_map: HashMap<String, &IpReputationReport> = reputation_results
        .iter()
        .map(|r| (r.ip_address.clone(), r))
        .collect();
    
    create_enhanced_csv(csv_file_path, &reputation_map, enhanced_csv)?;
    
    println!("\n=== PROCESS COMPLETE ===");
    println!("1. IP reputation report: {}", output_report);
    println!("2. Enhanced original CSV: {}", enhanced_csv);
    
    Ok(())
}
