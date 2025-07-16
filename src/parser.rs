use serde::Deserialize;
use std::{collections::HashMap, fs};

#[derive(Debug, Deserialize)]
struct OpenAPI {
    paths: HashMap<String, HashMap<String, Operation>>,
}

#[derive(Debug, Deserialize)]
struct Operation {
    summary: Option<String>,
}

pub fn parse_openapi(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let file_content = fs::read_to_string(file_path)?;
    let openapi: OpenAPI = serde_json::from_str(&file_content)?;

    let mut output = String::new();

    for (path, methods) in openapi.paths {
        for (method, op) in methods {
            let summary = op.summary.unwrap_or_else(|| "No summary".to_string());
            output.push_str(&format!(
                "{} ({}): {}\n ",
                path, method.to_uppercase(), summary
            ));
        }
    }

    Ok(output)
}
