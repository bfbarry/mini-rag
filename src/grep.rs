use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use regex::Regex;

#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub line_content: String,
    pub match_start: usize,
    pub match_end: usize,
}

#[derive(Debug, Clone)]
pub struct ClassMatch {
    pub file_path: PathBuf,
    pub class_name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub full_definition: String,
    pub language: String,
}

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub case_insensitive: bool,
    pub whole_word: bool,
    pub max_depth: Option<usize>,
    pub file_extensions: Option<Vec<String>>,
    pub ignore_hidden: bool,
    pub ignore_gitignore: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            case_insensitive: false,
            whole_word: false,
            max_depth: None,
            file_extensions: None,
            ignore_hidden: true,
            ignore_gitignore: true,
        }
    }
}

pub fn search_directory_recursively(
    root_dir: &Path,
    search_term: &str,
) -> io::Result<Vec<SearchMatch>> {
    search_directory_with_options(root_dir, search_term, &SearchOptions::default())
}

pub fn search_directory_with_options(
    root_dir: &Path,
    search_term: &str,
    options: &SearchOptions,
) -> io::Result<Vec<SearchMatch>> {
    let mut results = Vec::new();
    
    // Build regex pattern
    let pattern = if options.whole_word {
        format!(r"\b{}\b", regex::escape(search_term))
    } else {
        regex::escape(search_term)
    };
    
    let regex = if options.case_insensitive {
        Regex::new(&format!("(?i){}", pattern))
    } else {
        Regex::new(&pattern)
    }.map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    
    search_directory_recursive(root_dir, &regex, options, &mut results, 0)?;
    
    Ok(results)
}

fn search_directory_recursive(
    dir: &Path,
    regex: &Regex,
    options: &SearchOptions,
    results: &mut Vec<SearchMatch>,
    current_depth: usize,
) -> io::Result<()> {
    // Check depth limit
    if let Some(max_depth) = options.max_depth {
        if current_depth >= max_depth {
            return Ok(());
        }
    }
    
    let entries = fs::read_dir(dir)?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        // Skip hidden files/directories if requested
        if options.ignore_hidden {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') {
                    continue;
                }
            }
        }
        
        if path.is_dir() {
            search_directory_recursive(&path, regex, options, results, current_depth + 1)?;
        } else if path.is_file() {
            // Check file extension filter
            if let Some(extensions) = &options.file_extensions {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if !extensions.iter().any(|allowed| allowed == ext) {
                        continue;
                    }
                }
            }
            
            search_file(&path, regex, results)?;
        }
    }
    
    Ok(())
}

fn search_file(file_path: &Path, regex: &Regex, results: &mut Vec<SearchMatch>) -> io::Result<()> {
    let file = fs::File::open(file_path)?;
    let reader = BufReader::new(file);
    
    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;
        
        // Find all matches in this line
        for mat in regex.find_iter(&line) {
            results.push(SearchMatch {
                file_path: file_path.to_path_buf(),
                line_number: line_number + 1, // 1-based line numbers
                line_content: line.clone(),
                match_start: mat.start(),
                match_end: mat.end(),
            });
        }
    }
    
    Ok(())
}

// Enhanced function to find full class definitions
pub fn find_class_definitions(
    root_dir: &Path,
    class_name: Option<&str>,
) -> io::Result<Vec<ClassMatch>> {
    let mut results = Vec::new();
    
    let mut options = SearchOptions::default();
    options.file_extensions = Some(vec![
        "rs".to_string(),
        "java".to_string(),
        "py".to_string(),
        "cpp".to_string(),
        "h".to_string(),
        "cs".to_string(),
        "ts".to_string(),
        "js".to_string(),
    ]);
    
    find_classes_recursive(root_dir, class_name, &options, &mut results, 0)?;
    
    Ok(results)
}

fn find_classes_recursive(
    dir: &Path,
    class_name: Option<&str>,
    options: &SearchOptions,
    results: &mut Vec<ClassMatch>,
    current_depth: usize,
) -> io::Result<()> {
    if let Some(max_depth) = options.max_depth {
        if current_depth >= max_depth {
            return Ok(());
        }
    }
    
    let entries = fs::read_dir(dir)?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if options.ignore_hidden {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') {
                    continue;
                }
            }
        }
        
        if path.is_dir() {
            find_classes_recursive(&path, class_name, options, results, current_depth + 1)?;
        } else if path.is_file() {
            if let Some(extensions) = &options.file_extensions {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if !extensions.iter().any(|allowed| allowed == ext) {
                        continue;
                    }
                }
            }
            
            extract_class_definitions(&path, class_name, results)?;
        }
    }
    
    Ok(())
}

fn extract_class_definitions(
    file_path: &Path,
    target_class: Option<&str>,
    results: &mut Vec<ClassMatch>,
) -> io::Result<()> {
    let content = fs::read_to_string(file_path)?;
    let lines: Vec<&str> = content.lines().collect();
    
    let language = get_language_from_extension(file_path);
    
    match language.as_str() {
        "python" => extract_python_classes(&lines, file_path, target_class, results),
        "rust" => extract_rust_structs(&lines, file_path, target_class, results),
        "java" | "cs" => extract_java_cs_classes(&lines, file_path, target_class, results),
        "cpp" | "c" => extract_cpp_classes(&lines, file_path, target_class, results),
        "typescript" | "javascript" => extract_ts_js_classes(&lines, file_path, target_class, results),
        _ => Ok(()),
    }
}

fn get_language_from_extension(file_path: &Path) -> String {
    match file_path.extension().and_then(|e| e.to_str()) {
        Some("py") => "python".to_string(),
        Some("rs") => "rust".to_string(),
        Some("java") => "java".to_string(),
        Some("cs") => "cs".to_string(),
        Some("cpp") | Some("cc") | Some("cxx") => "cpp".to_string(),
        Some("h") | Some("hpp") => "c".to_string(),
        Some("ts") => "typescript".to_string(),
        Some("js") => "javascript".to_string(),
        _ => "unknown".to_string(),
    }
}

fn extract_python_classes(
    lines: &[&str],
    file_path: &Path,
    target_class: Option<&str>,
    results: &mut Vec<ClassMatch>,
) -> io::Result<()> {
    let class_regex = Regex::new(r"^class\s+(\w+).*:").unwrap();
    let mut i = 0;
    
    while i < lines.len() {
        if let Some(caps) = class_regex.captures(lines[i]) {
            let class_name = caps.get(1).unwrap().as_str();
            
            // Skip if we're looking for a specific class and this isn't it
            if let Some(target) = target_class {
                if class_name != target {
                    i += 1;
                    continue;
                }
            }
            
            let start_line = i + 1;
            let mut end_line = i + 1;
            let mut class_def = lines[i].to_string();
            
            // Find the end of the class by looking for the next non-indented line
            let base_indent = get_base_indent(lines[i]);
            i += 1;
            
            while i < lines.len() {
                let line = lines[i];
                if line.trim().is_empty() {
                    class_def.push('\n');
                    class_def.push_str(line);
                    i += 1;
                    continue;
                }
                
                let current_indent = get_base_indent(line);
                if current_indent <= base_indent && !line.trim().is_empty() {
                    break;
                }
                
                class_def.push('\n');
                class_def.push_str(line);
                end_line = i + 1;
                i += 1;
            }
            
            results.push(ClassMatch {
                file_path: file_path.to_path_buf(),
                class_name: class_name.to_string(),
                start_line,
                end_line,
                full_definition: class_def,
                language: "python".to_string(),
            });
        } else {
            i += 1;
        }
    }
    
    Ok(())
}

fn extract_rust_structs(
    lines: &[&str],
    file_path: &Path,
    target_class: Option<&str>,
    results: &mut Vec<ClassMatch>,
) -> io::Result<()> {
    let struct_regex = Regex::new(r"^\s*(?:pub\s+)?struct\s+(\w+)").unwrap();
    let mut i = 0;
    
    while i < lines.len() {
        if let Some(caps) = struct_regex.captures(lines[i]) {
            let struct_name = caps.get(1).unwrap().as_str();
            
            if let Some(target) = target_class {
                if struct_name != target {
                    i += 1;
                    continue;
                }
            }
            
            let start_line = i + 1;
            let mut class_def = lines[i].to_string();
            
            // Handle different struct formats
            if lines[i].contains('{') {
                // Multi-line struct
                let mut brace_count = lines[i].matches('{').count() as i32 - lines[i].matches('}').count() as i32;
                i += 1;
                
                while i < lines.len() && brace_count > 0 {
                    class_def.push('\n');
                    class_def.push_str(lines[i]);
                    brace_count += lines[i].matches('{').count() as i32 - lines[i].matches('}').count() as i32;
                    i += 1;
                }
            } else {
                // Look for the opening brace on the next line
                i += 1;
                while i < lines.len() && !lines[i].contains('{') {
                    class_def.push('\n');
                    class_def.push_str(lines[i]);
                    i += 1;
                }
                
                if i < lines.len() {
                    class_def.push('\n');
                    class_def.push_str(lines[i]);
                    let mut brace_count = lines[i].matches('{').count() as i32 - lines[i].matches('}').count() as i32;
                    i += 1;
                    
                    while i < lines.len() && brace_count > 0 {
                        class_def.push('\n');
                        class_def.push_str(lines[i]);
                        brace_count += lines[i].matches('{').count() as i32 - lines[i].matches('}').count() as i32;
                        i += 1;
                    }
                }
            }
            
            results.push(ClassMatch {
                file_path: file_path.to_path_buf(),
                class_name: struct_name.to_string(),
                start_line,
                end_line: i,
                full_definition: class_def,
                language: "rust".to_string(),
            });
        } else {
            i += 1;
        }
    }
    
    Ok(())
}

fn extract_java_cs_classes(
    lines: &[&str],
    file_path: &Path,
    target_class: Option<&str>,
    results: &mut Vec<ClassMatch>,
) -> io::Result<()> {
    let class_regex = Regex::new(r"^\s*(?:public\s+|private\s+|protected\s+)?class\s+(\w+)").unwrap();
    let mut i = 0;
    
    while i < lines.len() {
        if let Some(caps) = class_regex.captures(lines[i]) {
            let class_name = caps.get(1).unwrap().as_str();
            
            if let Some(target) = target_class {
                if class_name != target {
                    i += 1;
                    continue;
                }
            }
            
            let start_line = i + 1;
            let mut class_def = lines[i].to_string();
            
            // Find opening brace
            while i < lines.len() && !lines[i].contains('{') {
                i += 1;
                if i < lines.len() {
                    class_def.push('\n');
                    class_def.push_str(lines[i]);
                }
            }
            
            if i < lines.len() {
                let mut brace_count = lines[i].matches('{').count() as i32 - lines[i].matches('}').count() as i32;
                i += 1;
                
                while i < lines.len() && brace_count > 0 {
                    class_def.push('\n');
                    class_def.push_str(lines[i]);
                    brace_count += lines[i].matches('{').count() as i32 - lines[i].matches('}').count() as i32;
                    i += 1;
                }
            }
            
            results.push(ClassMatch {
                file_path: file_path.to_path_buf(),
                class_name: class_name.to_string(),
                start_line,
                end_line: i,
                full_definition: class_def,
                language: get_language_from_extension(file_path),
            });
        } else {
            i += 1;
        }
    }
    
    Ok(())
}

fn extract_cpp_classes(
    lines: &[&str],
    file_path: &Path,
    target_class: Option<&str>,
    results: &mut Vec<ClassMatch>,
) -> io::Result<()> {
    let class_regex = Regex::new(r"^\s*class\s+(\w+)").unwrap();
    let mut i = 0;
    
    while i < lines.len() {
        if let Some(caps) = class_regex.captures(lines[i]) {
            let class_name = caps.get(1).unwrap().as_str();
            
            if let Some(target) = target_class {
                if class_name != target {
                    i += 1;
                    continue;
                }
            }
            
            let start_line = i + 1;
            let mut class_def = lines[i].to_string();
            
            // Find opening brace
            while i < lines.len() && !lines[i].contains('{') {
                i += 1;
                if i < lines.len() {
                    class_def.push('\n');
                    class_def.push_str(lines[i]);
                }
            }
            
            if i < lines.len() {
                let mut brace_count = lines[i].matches('{').count() as i32 - lines[i].matches('}').count() as i32;
                i += 1;
                
                while i < lines.len() && brace_count > 0 {
                    class_def.push('\n');
                    class_def.push_str(lines[i]);
                    brace_count += lines[i].matches('{').count() as i32 - lines[i].matches('}').count() as i32;
                    i += 1;
                }
            }
            
            results.push(ClassMatch {
                file_path: file_path.to_path_buf(),
                class_name: class_name.to_string(),
                start_line,
                end_line: i,
                full_definition: class_def,
                language: "cpp".to_string(),
            });
        } else {
            i += 1;
        }
    }
    
    Ok(())
}

fn extract_ts_js_classes(
    lines: &[&str],
    file_path: &Path,
    target_class: Option<&str>,
    results: &mut Vec<ClassMatch>,
) -> io::Result<()> {
    let class_regex = Regex::new(r"^\s*(?:export\s+)?class\s+(\w+)").unwrap();
    let mut i = 0;
    
    while i < lines.len() {
        if let Some(caps) = class_regex.captures(lines[i]) {
            let class_name = caps.get(1).unwrap().as_str();
            
            if let Some(target) = target_class {
                if class_name != target {
                    i += 1;
                    continue;
                }
            }
            
            let start_line = i + 1;
            let mut class_def = lines[i].to_string();
            
            // Find opening brace
            while i < lines.len() && !lines[i].contains('{') {
                i += 1;
                if i < lines.len() {
                    class_def.push('\n');
                    class_def.push_str(lines[i]);
                }
            }
            
            if i < lines.len() {
                let mut brace_count = lines[i].matches('{').count() as i32 - lines[i].matches('}').count() as i32;
                i += 1;
                
                while i < lines.len() && brace_count > 0 {
                    class_def.push('\n');
                    class_def.push_str(lines[i]);
                    brace_count += lines[i].matches('{').count() as i32 - lines[i].matches('}').count() as i32;
                    i += 1;
                }
            }
            
            results.push(ClassMatch {
                file_path: file_path.to_path_buf(),
                class_name: class_name.to_string(),
                start_line,
                end_line: i,
                full_definition: class_def,
                language: get_language_from_extension(file_path),
            });
        } else {
            i += 1;
        }
    }
    
    Ok(())
}

fn get_base_indent(line: &str) -> usize {
    line.chars().take_while(|c| c.is_whitespace()).count()
}

// Helper function to print class definitions
pub fn print_class_definitions(classes: &[ClassMatch]) {
    for class in classes {
        println!("{}:{}-{}: class {}", 
            class.file_path.display(),
            class.start_line,
            class.end_line,
            class.class_name
        );
        println!("{}", class.full_definition);
        println!("---");
    }
}

// Helper function to print results in a ripgrep-like format
pub fn print_search_results(results: &[SearchMatch]) {
    for result in results {
        println!(
            "{}:{}:{}",
            result.file_path.display(),
            result.line_number,
            result.line_content.trim()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_basic_search() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello world\nThis is a test\nHello again").unwrap();

        let results = search_directory_recursively(temp_dir.path(), "Hello").unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].line_number, 1);
        assert_eq!(results[1].line_number, 3);
    }

    #[test]
    fn test_case_insensitive_search() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello world\nhello again").unwrap();

        let mut options = SearchOptions::default();
        options.case_insensitive = true;

        let results = search_directory_with_options(temp_dir.path(), "HELLO", &options).unwrap();
        assert_eq!(results.len(), 2);
    }
}

// Example usage
fn example() -> io::Result<()> {
    let root_dir = Path::new("./src");
    
    // Basic text search
    let search_term = "struct";
    let results = search_directory_recursively(root_dir, search_term)?;
    println!("Found {} text matches:", results.len());
    print_search_results(&results);
    
    // Find all class definitions
    let all_classes = find_class_definitions(root_dir, None)?;
    println!("\nFound {} class definitions:", all_classes.len());
    print_class_definitions(&all_classes);
    
    // Find specific class definition
    let specific_class = find_class_definitions(root_dir, Some("Thing"))?;
    println!("\nFound {} matches for class 'Thing':", specific_class.len());
    print_class_definitions(&specific_class);
    
    Ok(())
}