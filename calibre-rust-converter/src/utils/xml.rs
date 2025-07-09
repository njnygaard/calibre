use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::{BufRead, Read};

/// Extract text content from XML
pub fn extract_text<R: Read + BufRead>(reader: &mut Reader<R>) -> Result<String> {
    let mut content = String::new();
    let mut buf = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(_) => {
                // Handle start tags if needed
            }
            Event::End(_) => {
                // Handle end tags if needed
            }
            Event::Text(e) => {
                content.push_str(std::str::from_utf8(&e.into_inner())?);
            }
            Event::CData(e) => {
                content.push_str(std::str::from_utf8(&e.into_inner())?);
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    
    Ok(content)
}

/// Extract attribute value from XML
pub fn extract_attribute(xml: &str, tag: &str, attribute: &str) -> Option<String> {
    let pattern = format!(r#"<{}[^>]*{}="([^"]*)"[^>]*>"#, tag, attribute);
    
    if let Some(captures) = regex::Regex::new(&pattern).ok()?.captures(xml) {
        captures.get(1).map(|m| m.as_str().to_string())
    } else {
        None
    }
}

/// Extract content between XML tags
pub fn extract_tag_content(xml: &str, tag: &str) -> Option<String> {
    let pattern = format!(r#"<{}[^>]*>(.*?)</{}>"#, tag, tag);
    
    if let Some(captures) = regex::Regex::new(&pattern).ok()?.captures(xml) {
        captures.get(1).map(|m| m.as_str().to_string())
    } else {
        None
    }
}

/// Parse XML and extract specific elements
pub fn parse_xml_elements<R: Read + BufRead>(reader: &mut Reader<R>, target_tag: &str) -> Result<Vec<String>> {
    let mut elements = Vec::new();
    let mut buf = Vec::new();
    let mut current_element = String::new();
    let mut in_target = false;
    
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) => {
                let name = e.name();
                let tag_name = std::str::from_utf8(name.as_ref())?;
                if tag_name == target_tag {
                    in_target = true;
                    current_element.clear();
                }
            }
            Event::End(ref e) => {
                let name = e.name();
                let tag_name = std::str::from_utf8(name.as_ref())?;
                if tag_name == target_tag {
                    in_target = false;
                    if !current_element.trim().is_empty() {
                        elements.push(current_element.clone());
                    }
                }
            }
            Event::Text(e) => {
                if in_target {
                    current_element.push_str(std::str::from_utf8(&e.into_inner())?);
                }
            }
            Event::CData(e) => {
                if in_target {
                    current_element.push_str(std::str::from_utf8(&e.into_inner())?);
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    
    Ok(elements)
}

/// Validate XML structure
pub fn validate_xml_structure<R: Read + BufRead>(reader: &mut Reader<R>) -> Result<bool> {
    let mut buf = Vec::new();
    let mut stack = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) => {
                let name = e.name();
                let tag_name = std::str::from_utf8(name.as_ref())?.to_string();
                stack.push(tag_name);
            }
            Event::End(ref e) => {
                let name = e.name();
                let tag_name = std::str::from_utf8(name.as_ref())?;
                if let Some(expected) = stack.pop() {
                    if expected != tag_name {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    
    Ok(stack.is_empty())
} 