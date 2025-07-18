use serde::Deserialize;
use std::{collections::HashMap, fs};
use ureq;
#[derive(Debug, Deserialize)]
struct OpenAPI {
    paths: HashMap<String, HashMap<String, Operation>>,
}

#[derive(Debug, Deserialize)]
struct Operation {
    summary: Option<String>,
}

pub fn parse_openapi(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let content: String = ureq::get(url)
        .header("Example-Header", "header value")
        .call()?
        .body_mut()
        .read_to_string()?;
    // let file_content = fs::read_to_string(file_path)?;
    let openapi: OpenAPI = serde_json::from_str(&content)?;

    let mut output = String::new();

    for (path, methods) in openapi.paths {
        for (method, op) in methods {
            let summary = op.summary.unwrap_or_else(|| "No summary".to_string());
            output.push_str(&format!(
                "{} {}: {}\n",
                method.to_uppercase(), path, summary
            ));
        }
    }

    Ok(output)
}

pub fn parse_codebase(root_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    // 1. determine languages
    // 2. rg codebase for "type" or "class" depending on langs
    // 3. for each match, jump to file and parse that class, load into memory
    // Extra: create class dependency tree

    Ok("Test".to_string())
}