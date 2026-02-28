//! Skill Validator - 技能验证器
//!
//! 验证生成技能的质量和安全

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub status: ValidationStatus,
    pub message: String,
    pub warnings: Vec<String>,
    pub details: Vec<ValidationDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationStatus {
    Approved,
    Rejected,
    NeedsReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationDetail {
    pub rule: String,
    pub passed: bool,
    pub message: String,
}

pub struct SkillValidator {
    config: ValidatorConfig,
    dangerous_patterns: Vec<Regex>,
}

#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    pub max_function_lines: usize,
    pub max_loop_count: usize,
    pub allow_network: bool,
    pub allow_filesystem: bool,
    pub allow_shell: bool,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            max_function_lines: 100,
            max_loop_count: 10,
            allow_network: false,
            allow_filesystem: false,
            allow_shell: false,
        }
    }
}

impl Default for SkillValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillValidator {
    pub fn new() -> Self {
        let dangerous_patterns = vec![
            Regex::new(r"std::fs::remove").unwrap(),
            Regex::new(r"std::fs::remove_dir_all").unwrap(),
            Regex::new(r"std::process::Command::new\s*\(").unwrap(),
            Regex::new(r"\.exec\s*\(").unwrap(),
            Regex::new(r"\.system\s*\(").unwrap(),
            Regex::new(r"eval\s*\(").unwrap(),
            Regex::new(r"unsafe\s*\{").unwrap(),
        ];

        Self {
            config: ValidatorConfig::default(),
            dangerous_patterns,
        }
    }

    pub fn with_config(config: ValidatorConfig) -> Self {
        Self {
            config,
            dangerous_patterns: vec![
                Regex::new(r"std::fs::remove").unwrap(),
                Regex::new(r"std::fs::remove_dir_all").unwrap(),
                Regex::new(r"std::process::Command::new\s*\(").unwrap(),
                Regex::new(r"\.exec\s*\(").unwrap(),
                Regex::new(r"\.system\s*\(").unwrap(),
                Regex::new(r"eval\s*\(").unwrap(),
                Regex::new(r"unsafe\s*\{").unwrap(),
            ],
        }
    }

    pub fn validate(&self, code: &str) -> ValidationResult {
        let mut warnings = Vec::new();
        let mut details = Vec::new();

        let detail = self.check_dangerous_operations(code);
        let passed = detail.passed.clone();
        let msg = detail.message.clone();
        details.push(detail);
        if !passed {
            warnings.push(msg);
        }

        let detail = self.check_complexity(code);
        details.push(detail);

        let detail = self.check_loop_count(code);
        details.push(detail);

        let detail = self.check_error_handling(code);
        details.push(detail);

        let detail = self.check_async_usage(code);
        details.push(detail);

        let rejected = details.iter().any(|d| !d.passed && d.rule == "dangerous_operations");
        let needs_review = details.iter().any(|d| !d.passed && d.rule != "dangerous_operations");

        let status = if rejected {
            ValidationStatus::Rejected
        } else if needs_review {
            ValidationStatus::NeedsReview
        } else {
            ValidationStatus::Approved
        };

        let message = match status {
            ValidationStatus::Approved => "Skill validation passed".to_string(),
            ValidationStatus::Rejected => "Skill validation failed".to_string(),
            ValidationStatus::NeedsReview => "Skill needs manual review".to_string(),
        };

        ValidationResult {
            status,
            message,
            warnings,
            details,
        }
    }

    fn check_dangerous_operations(&self, code: &str) -> ValidationDetail {
        let mut found_dangerous = Vec::new();

        for pattern in &self.dangerous_patterns {
            if pattern.is_match(code) {
                found_dangerous.push(pattern.to_string());
            }
        }

        if found_dangerous.is_empty() {
            ValidationDetail {
                rule: "dangerous_operations".to_string(),
                passed: true,
                message: "No dangerous operations detected".to_string(),
            }
        } else {
            ValidationDetail {
                rule: "dangerous_operations".to_string(),
                passed: false,
                message: format!("Dangerous patterns detected: {}", found_dangerous.join(", ")),
            }
        }
    }

    fn check_complexity(&self, code: &str) -> ValidationDetail {
        let lines: Vec<&str> = code.lines().collect();
        let line_count = lines.len();

        if line_count > self.config.max_function_lines {
            ValidationDetail {
                rule: "complexity".to_string(),
                passed: false,
                message: format!(
                    "Function too complex: {} lines, max allowed {}",
                    line_count,
                    self.config.max_function_lines
                ),
            }
        } else {
            ValidationDetail {
                rule: "complexity".to_string(),
                passed: true,
                message: "Code complexity is acceptable".to_string(),
            }
        }
    }

    fn check_loop_count(&self, code: &str) -> ValidationDetail {
        let for_loops = code.matches("for ").count();
        let while_loops = code.matches("while ").count();
        let loop_count = for_loops + while_loops;

        if loop_count > self.config.max_loop_count {
            ValidationDetail {
                rule: "loop_count".to_string(),
                passed: false,
                message: format!(
                    "Too many loops: {}, max allowed {}",
                    loop_count, self.config.max_loop_count
                ),
            }
        } else {
            ValidationDetail {
                rule: "loop_count".to_string(),
                passed: true,
                message: "Loop count is acceptable".to_string(),
            }
        }
    }

    fn check_error_handling(&self, code: &str) -> ValidationDetail {
        let has_result = code.contains("Result<");
        let has_option = code.contains("Option<");
        let has_question_mark = code.contains('?');
        let has_if_let = code.contains("if let");
        let has_match = code.contains("match ");

        let has_error_handling = has_result || has_option || has_question_mark || has_if_let || has_match;

        if has_error_handling {
            ValidationDetail {
                rule: "error_handling".to_string(),
                passed: true,
                message: "Proper error handling detected".to_string(),
            }
        } else {
            ValidationDetail {
                rule: "error_handling".to_string(),
                passed: false,
                message: "No error handling detected".to_string(),
            }
        }
    }

    fn check_async_usage(&self, code: &str) -> ValidationDetail {
        let has_async_fn = code.contains("async fn");
        let has_await = code.contains(".await");

        if has_async_fn || has_await {
            ValidationDetail {
                rule: "async_usage".to_string(),
                passed: true,
                message: "Proper async/await usage".to_string(),
            }
        } else {
            ValidationDetail {
                rule: "async_usage".to_string(),
                passed: false,
                message: "No async/await usage detected".to_string(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_safe_code() {
        let validator = SkillValidator::new();

        let code = r#"
async fn fetch_data(url: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await
        .map_err(|e| e.to_string())?;
    Ok(response.text().await.map_err(|e| e.to_string())?)
}
"#;

        let result = validator.validate(code);

        assert_eq!(result.status, ValidationStatus::Approved);
    }

    #[test]
    fn test_validate_dangerous_code() {
        let validator = SkillValidator::new();

        let code = r#"
fn delete_file(path: String) {
    std::fs::remove_file(path).unwrap();
}
"#;

        let result = validator.validate(code);

        assert_eq!(result.status, ValidationStatus::Rejected);
    }

    #[test]
    fn test_validate_shell_code() {
        let validator = SkillValidator::new();

        let code = r#"
fn run_command(cmd: String) {
    std::process::Command::new("sh").arg("-c").arg(cmd).spawn().unwrap();
}
"#;

        let result = validator.validate(code);

        assert_eq!(result.status, ValidationStatus::Rejected);
    }

    #[test]
    fn test_validate_complex_code() {
        let validator = SkillValidator::new();

        let code = std::iter::repeat("let x = 1;\n")
            .take(150)
            .collect::<String>();

        let result = validator.validate(&code);

        let complexity = result.details.iter().find(|d| d.rule == "complexity");
        assert!(complexity.is_some());
        assert!(!complexity.unwrap().passed);
    }

    #[test]
    fn test_validate_many_loops() {
        let validator = SkillValidator::new();

        let code = r#"
fn process() {
    let mut count = 0;
    for i in 0..11 { count += i; }
    for i in 0..11 { count += i; }
    for i in 0..11 { count += i; }
    for i in 0..11 { count += i; }
    for i in 0..11 { count += i; }
    for i in 0..11 { count += i; }
    for i in 0..11 { count += i; }
    for i in 0..11 { count += i; }
    for i in 0..11 { count += i; }
    for i in 0..11 { count += i; }
    for i in 0..11 { count += i; }
}
"#;

        let result = validator.validate(code);

        let loop_detail = result.details.iter().find(|d| d.rule == "loop_count");
        assert!(loop_detail.is_some());
        assert!(!loop_detail.unwrap().passed);
    }

    #[test]
    fn test_validate_no_error_handling() {
        let validator = SkillValidator::new();

        let code = r#"
fn fetch(url: String) -> String {
    reqwest::blocking::get(url).unwrap().text().unwrap()
}
"#;

        let result = validator.validate(code);

        assert!(!result.details.iter().find(|d| d.rule == "error_handling").unwrap().passed);
    }

    #[test]
    fn test_validate_with_config() {
        let config = ValidatorConfig {
            max_function_lines: 50,
            max_loop_count: 5,
            allow_network: true,
            allow_filesystem: true,
            allow_shell: true,
        };

        let validator = SkillValidator::with_config(config);

        assert_eq!(validator.config.max_function_lines, 50);
        assert_eq!(validator.config.max_loop_count, 5);
    }

    #[test]
    fn test_validate_sync_code() {
        let validator = SkillValidator::new();

        let code = r#"
fn fetch_sync(url: String) -> Result<String, String> {
    Ok("result".to_string())
}
"#;

        let result = validator.validate(code);

        let async_detail = result.details.iter().find(|d| d.rule == "async_usage");
        assert!(async_detail.is_some());
        assert!(!async_detail.unwrap().passed);
    }

    #[test]
    fn test_validate_empty_code() {
        let validator = SkillValidator::new();

        let result = validator.validate("");

        assert_eq!(result.status, ValidationStatus::NeedsReview);
    }

    #[test]
    fn test_multiple_warnings() {
        let validator = SkillValidator::new();

        let code = r#"
fn fetch(url: String) -> String {
    std::process::Command::new("ls").spawn().unwrap();
    for i in 0..1000 { println!("{}", i); }
    reqwest::blocking::get(url).unwrap()
}
"#;

        let result = validator.validate(code);

        assert!(result.status == ValidationStatus::Rejected || result.status == ValidationStatus::NeedsReview);
    }
}
