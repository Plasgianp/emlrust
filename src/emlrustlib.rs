use regex::Regex;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use mail_parser::Message;
use dirs::home_dir;

#[allow(dead_code)]
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
    let mut file = fs::File::open(eml_file)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let message = Message::parse(&buffer).ok_or("Failed to parse email")?;

    let mut html_bodies = message.html_bodies();
    if let Some(html_part) = html_bodies.next() {
        let content = html_part.contents();
        return Ok(String::from_utf8_lossy(content).to_string());
    }

    let mut text_bodies = message.text_bodies();
    if let Some(text_part) = text_bodies.next() {
        let content = text_part.contents();
        return Ok(format!("<html><body>{}</body></html>", String::from_utf8_lossy(content)));
    }

    Ok("<html><body>No content found</body></html>".to_string())
}

pub fn add_href_to_anchor_tags(html_content: &str, new_href: &str) -> Result<String, Box<dyn std::error::Error>> {
    if !html_content.contains("<html") && !html_content.contains("<body") && !html_content.contains("<a ") {
        return Ok(html_content.to_string());
    }

    let re = Regex::new(r#"<a\s+([^>]*)href\s*=\s*["']([^"']*)["']([^>]*)>"#).unwrap();
    let modified_html = re.replace_all(html_content, |caps: &regex::Captures| {
        format!("<a {}href=\"{}\"{}>{}", 
            &caps[1], 
            new_href,
            &caps[3],
            ""
        )
    }).to_string();
    
    let re_no_href = Regex::new(r#"<a\s+([^>]*)>"#).unwrap();
    let modified_html = re_no_href.replace_all(&modified_html, |caps: &regex::Captures| {
        if !caps[1].contains("href=") {
            format!("<a {}href=\"{}\">", &caps[1], new_href)
        } else {
            format!("<a {}>", &caps[1])
        }
    }).to_string();
    
    Ok(modified_html)
}

pub fn add_href_to_file(file_path: &Path, new_href: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let modified_html = add_href_to_anchor_tags(&content, new_href)?;
    let modified_html = if modified_html.contains("</html>") {
        modified_html.replace("</html>", "{{.Tracker}}\n</html>")
    } else {
        format!("{}\n{{{{.Tracker}}}}", modified_html)
    };

    fs::write(file_path, modified_html)?;
    
    Ok(())
}

pub fn anonymizer(html_content: &str) -> String {
    let email_pattern = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b").unwrap();
    let modified_html = email_pattern.replace_all(html_content, "{{.Email}}").to_string();
    modified_html
}

pub fn remove_scripts(html_content: &str) -> Result<String, Box<dyn std::error::Error>> {
    if !html_content.contains("<script") {
        return Ok(html_content.to_string());
    }

    let re = Regex::new(r"(?s)<script[^>]*>.*?</script>").unwrap();
    let modified_html = re.replace_all(html_content, "").to_string();
    
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
            let _ = remove_scripts_from_file(path);
        }
    }
    
    Ok(())
}

pub fn gophishing_everything(directory_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut _found_files = 0;
    let mut _processed_files = 0;
    let mut _error_files = 0;
    
    let _total_files = WalkDir::new(directory_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            !e.path().is_dir() && 
            e.path().extension().map_or(false, |ext| ext.eq_ignore_ascii_case("eml"))
        })
        .count();
    
    for entry in WalkDir::new(directory_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        
        if !path.is_dir() {
            if let Some(ext) = path.extension() {
                if ext.eq_ignore_ascii_case("eml") {
                    _found_files += 1;
                    
                    let process_result = (|| -> Result<(), Box<dyn std::error::Error>> {
                        let html_content = eml_to_html(path)?;
                        let modified_html = anonymizer(&html_content);
                        let modified_html_with_href = add_href_to_anchor_tags(&modified_html, "{{.URL}}")?;
                        
                        let modified_html_with_href = if modified_html_with_href.contains("</html>") {
                            modified_html_with_href.replace("</html>", "{{.Tracker}}\n</html>")
                        } else {
                            format!("{}\n{{{{.Tracker}}}}", modified_html_with_href)
                        };
                        
                        let modified_html_no_script = remove_scripts(&modified_html_with_href)?;
                        let output_file = path.with_extension("html");
                        fs::write(&output_file, modified_html_no_script)?;
                        
                        Ok(())
                    })();
                    
                    match process_result {
                        Ok(_) => {
                            _processed_files += 1;
                        },
                        Err(_) => {
                            _error_files += 1;
                            let output_file = path.with_extension("html");
                            let _ = fs::write(&output_file, "<html><body>Error processing email</body></html>");
                        }
                    }
                }
            }
        }
    }

    Ok(())
}