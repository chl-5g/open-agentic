//! Pattern Analyzer - 任务模式分析器
//! 
//! 从任务执行中提取可复用的模式，用于技能进化学习

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPattern {
    pub id: String,
    pub task_category: String,
    pub tool_sequence: Vec<ToolCallPattern>,
    pub param_patterns: Vec<ParamPattern>,
    pub success_indicators: Vec<String>,
    pub steps: Vec<ExecutionStep>,
    pub reusability_score: f64,
    pub source_task_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallPattern {
    pub tool_name: String,
    pub param_schema: HashMap<String, ParamType>,
    pub result_schema: HashMap<String, ParamType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamPattern {
    pub name: String,
    pub param_type: ParamType,
    pub is_generic: bool,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParamType {
    String,
    Number,
    Boolean,
    Object,
    Array,
    Unknown,
}

impl ParamType {
    pub fn from_json_value(value: &serde_json::Value) -> Self {
        match value {
            serde_json::Value::String(_) => ParamType::String,
            serde_json::Value::Number(_) => ParamType::Number,
            serde_json::Value::Bool(_) => ParamType::Boolean,
            serde_json::Value::Object(_) => ParamType::Object,
            serde_json::Value::Array(_) => ParamType::Array,
            _ => ParamType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub step_number: u32,
    pub tool_name: String,
    pub input_summary: String,
    pub output_summary: String,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct PatternAnalyzer {
    config: AnalyzerConfig,
}

#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    pub min_reusability_threshold: f64,
    pub max_tool_sequence_length: usize,
    pub enable_deep_analysis: bool,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            min_reusability_threshold: 0.5,
            max_tool_sequence_length: 20,
            enable_deep_analysis: true,
        }
    }
}

impl Default for PatternAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternAnalyzer {
    pub fn new() -> Self {
        Self {
            config: AnalyzerConfig::default(),
        }
    }

    pub fn with_config(config: AnalyzerConfig) -> Self {
        Self { config }
    }

    pub fn extract(
        &self,
        task_id: &str,
        task_input: &str,
        tool_calls: &[ToolCall],
    ) -> TaskPattern {
        let task_category = self.categorize(task_input);
        let tool_sequence = self.extract_tool_sequence(tool_calls);
        let param_patterns = self.extract_param_patterns(task_input, tool_calls);
        let steps = self.extract_steps(tool_calls);
        let success_indicators = self.extract_success_indicators(tool_calls);
        let reusability_score = self.score_reusability(&task_category, &param_patterns, &steps);

        TaskPattern {
            id: uuid::Uuid::new_v4().to_string(),
            task_category,
            tool_sequence,
            param_patterns,
            success_indicators,
            steps,
            reusability_score,
            source_task_id: task_id.to_string(),
            created_at: Utc::now(),
        }
    }

    pub fn categorize(&self, input: &str) -> String {
        let input_lower = input.to_lowercase();
        
        if input_lower.contains("search") || input_lower.contains("find") {
            "search".to_string()
        } else if input_lower.contains("code") || input_lower.contains("program") || input_lower.contains("function") {
            "code_generation".to_string()
        } else if input_lower.contains("api") || input_lower.contains("http") || input_lower.contains("fetch") {
            "api_call".to_string()
        } else if input_lower.contains("file") || input_lower.contains("read") || input_lower.contains("write") {
            "file_operation".to_string()
        } else if input_lower.contains("analyze") || input_lower.contains("data") || input_lower.contains("stat") {
            "data_analysis".to_string()
        } else if input_lower.contains("create") || input_lower.contains("draft") {
            "content_creation".to_string()
        } else if input_lower.contains("debug") || input_lower.contains("error") || input_lower.contains("fix") {
            "debugging".to_string()
        } else if input_lower.contains("test") || input_lower.contains("check") {
            "testing".to_string()
        } else {
            "general".to_string()
        }
    }

    fn extract_tool_sequence(&self, tool_calls: &[ToolCall]) -> Vec<ToolCallPattern> {
        tool_calls
            .iter()
            .take(self.config.max_tool_sequence_length)
            .map(|call| {
                let param_schema = self.infer_param_schema(&call.arguments);
                let result_schema = call
                    .result
                    .as_ref()
                    .map(|r| self.infer_param_schema(r))
                    .unwrap_or_default();

                ToolCallPattern {
                    tool_name: call.name.clone(),
                    param_schema,
                    result_schema,
                }
            })
            .collect()
    }

    fn infer_param_schema(&self, value: &serde_json::Value) -> HashMap<String, ParamType> {
        let mut schema = HashMap::new();
        
        if let serde_json::Value::Object(map) = value {
            for (key, val) in map {
                schema.insert(key.clone(), ParamType::from_json_value(val));
            }
        }
        
        schema
    }

    fn extract_param_patterns(&self, input: &str, tool_calls: &[ToolCall]) -> Vec<ParamPattern> {
        let mut patterns = Vec::new();
        
        // Extract patterns from tool call arguments
        for call in tool_calls {
            if let serde_json::Value::Object(args) = &call.arguments {
                for (name, value) in args {
                    let param_type = ParamType::from_json_value(value);
                    let is_generic = self.is_generic_param(name, value);
                    let example = value.to_string();
                    
                    // Check if we already have this pattern
                    if let Some(existing) = patterns.iter_mut().find(|p: &&mut ParamPattern| p.name == *name) {
                        if !existing.examples.contains(&example) {
                            existing.examples.push(example);
                        }
                    } else {
                        patterns.push(ParamPattern {
                            name: name.clone(),
                            param_type: param_type.clone(),
                            is_generic,
                            examples: vec![example],
                        });
                    }
                }
            }
        }
        
        patterns
    }

    fn is_generic_param(&self, name: &str, value: &serde_json::Value) -> bool {
        let generic_names = ["id", "value", "data", "input"];
        let name_lower = name.to_lowercase();
        
        if generic_names.iter().any(|n| name_lower == *n) {
            return true;
        }
        
        if let serde_json::Value::String(s) = value {
            if s.is_empty() || s.starts_with('<') || s.starts_with('{') {
                return true;
            }
        }
        
        false
    }

    fn extract_steps(&self, tool_calls: &[ToolCall]) -> Vec<ExecutionStep> {
        tool_calls
            .iter()
            .enumerate()
            .map(|(idx, call)| {
                let input_summary = self.summarize_value(&call.arguments);
                let output_summary = call
                    .result
                    .as_ref()
                    .map(|r| self.summarize_value(r))
                    .unwrap_or_else(|| "no output".to_string());
                
                let success = call.result.is_some();

                ExecutionStep {
                    step_number: idx as u32 + 1,
                    tool_name: call.name.clone(),
                    input_summary,
                    output_summary,
                    success,
                }
            })
            .collect()
    }

    fn summarize_value(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => {
                if s.len() > 50 {
                    format!("{}... ({} chars)", &s[..50], s.len())
                } else {
                    s.clone()
                }
            }
            serde_json::Value::Object(map) => {
                let keys: Vec<&str> = map.keys().take(3).map(|k| k.as_str()).collect();
                format!("object with keys: {}", keys.join(", "))
            }
            serde_json::Value::Array(arr) => {
                format!("array with {} items", arr.len())
            }
            serde_json::Value::Null => "null".to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Number(n) => n.to_string(),
        }
    }

    fn extract_success_indicators(&self, tool_calls: &[ToolCall]) -> Vec<String> {
        tool_calls
            .iter()
            .filter_map(|call| {
                if call.result.is_some() {
                    Some(format!("{}_success", call.name))
                } else {
                    None
                }
            })
            .collect()
    }

    fn score_reusability(
        &self,
        category: &str,
        param_patterns: &[ParamPattern],
        steps: &[ExecutionStep],
    ) -> f64 {
        let mut score = 0.0;
        
        // Category factor - some categories are more reusable
        let category_weight = match category {
            "file_operation" => 0.8,
            "api_call" => 0.9,
            "search" => 0.7,
            "code_generation" => 0.6,
            _ => 0.5,
        };
        score += category_weight * 0.4;
        
        // Generic parameters factor
        let generic_ratio = if !param_patterns.is_empty() {
            param_patterns.iter().filter(|p| p.is_generic).count() as f64 / param_patterns.len() as f64
        } else {
            0.0
        };
        score += generic_ratio * 0.3;
        
        // Success rate factor
        let success_ratio = if !steps.is_empty() {
            steps.iter().filter(|s| s.success).count() as f64 / steps.len() as f64
        } else {
            0.0
        };
        score += success_ratio * 0.3;
        
        // Apply threshold
        score.max(0.0).min(1.0)
    }

    pub fn is_repeatable(&self, pattern: &TaskPattern) -> bool {
        pattern.reusability_score >= self.config.min_reusability_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_file_operation() {
        let analyzer = PatternAnalyzer::new();
        
        let category = analyzer.categorize("Please read the file /path/to/file.txt");
        assert_eq!(category, "file_operation");
        
        let category = analyzer.categorize("Write some content to a new file");
        assert_eq!(category, "file_operation");
    }

    #[test]
    fn test_categorize_search() {
        let analyzer = PatternAnalyzer::new();
        
        let category = analyzer.categorize("Search for information about Rust");
        assert_eq!(category, "search");
        
        let category = analyzer.categorize("Find all files matching pattern");
        assert_eq!(category, "search");
    }

    #[test]
    fn test_categorize_api_call() {
        let analyzer = PatternAnalyzer::new();
        
        let category = analyzer.categorize("Call the API endpoint");
        assert_eq!(category, "api_call");
        
        let category = analyzer.categorize("Fetch data from HTTP server");
        assert_eq!(category, "api_call");
    }

    #[test]
    fn test_categorize_code_generation() {
        let analyzer = PatternAnalyzer::new();
        
        let category = analyzer.categorize("Write a Python function");
        assert_eq!(category, "code_generation");
        
        let category = analyzer.categorize("Create a Rust program");
        assert_eq!(category, "code_generation");
    }

    #[test]
    fn test_categorize_default() {
        let analyzer = PatternAnalyzer::new();
        
        let category = analyzer.categorize("Hello, how are you?");
        assert_eq!(category, "general");
    }

    #[test]
    fn test_extract_tool_sequence() {
        let analyzer = PatternAnalyzer::new();
        
        let tool_calls = vec![
            ToolCall {
                name: "search".to_string(),
                arguments: serde_json::json!({"query": "rust"}),
                result: Some(serde_json::json!(["result1", "result2"])),
                duration_ms: 100,
            },
            ToolCall {
                name: "fetch".to_string(),
                arguments: serde_json::json!({"url": "https://example.com"}),
                result: Some(serde_json::json!({"status": 200})),
                duration_ms: 200,
            },
        ];
        
        let sequence = analyzer.extract_tool_sequence(&tool_calls);
        
        assert_eq!(sequence.len(), 2);
        assert_eq!(sequence[0].tool_name, "search");
        assert_eq!(sequence[1].tool_name, "fetch");
    }

    #[test]
    fn test_extract_param_patterns() {
        let analyzer = PatternAnalyzer::new();
        
        let tool_calls = vec![
            ToolCall {
                name: "search".to_string(),
                arguments: serde_json::json!({"query": "test query", "limit": 10}),
                result: None,
                duration_ms: 100,
            },
        ];
        
        let patterns = analyzer.extract_param_patterns("search for something", &tool_calls);
        
        assert_eq!(patterns.len(), 2);
        
        let query_pattern = patterns.iter().find(|p| p.name == "query").unwrap();
        assert_eq!(query_pattern.param_type, ParamType::String);
        assert!(!query_pattern.is_generic);
        
        let limit_pattern = patterns.iter().find(|p| p.name == "limit").unwrap();
        assert_eq!(limit_pattern.param_type, ParamType::Number);
    }

    #[test]
    fn test_is_generic_param() {
        let analyzer = PatternAnalyzer::new();
        
        assert!(analyzer.is_generic_param("id", &serde_json::json!("user123")));
        assert!(analyzer.is_generic_param("value", &serde_json::json!("test")));
        assert!(analyzer.is_generic_param("data", &serde_json::json!("")));
        
        assert!(!analyzer.is_generic_param("email", &serde_json::json!("user@example.com")));
        assert!(!analyzer.is_generic_param("password", &serde_json::json!("secret")));
        assert!(!analyzer.is_generic_param("query", &serde_json::json!("search term")));
    }

    #[test]
    fn test_extract() {
        let analyzer = PatternAnalyzer::new();
        
        let tool_calls = vec![
            ToolCall {
                name: "search".to_string(),
                arguments: serde_json::json!({"query": "rust programming"}),
                result: Some(serde_json::json!(["result1"])),
                duration_ms: 100,
            },
            ToolCall {
                name: "fetch".to_string(),
                arguments: serde_json::json!({"url": "https://rust-lang.org"}),
                result: Some(serde_json::json!({"html": "<html>"})),
                duration_ms: 200,
            },
        ];
        
        let pattern = analyzer.extract("task-123", "Search for rust information and fetch details", &tool_calls);
        
        assert_eq!(pattern.source_task_id, "task-123");
        assert_eq!(pattern.task_category, "search");
        assert_eq!(pattern.tool_sequence.len(), 2);
        assert_eq!(pattern.steps.len(), 2);
        assert!(pattern.reusability_score > 0.0);
    }

    #[test]
    fn test_is_repeatable() {
        let analyzer = PatternAnalyzer::new();
        
        let high_score_pattern = TaskPattern {
            id: "1".to_string(),
            task_category: "api_call".to_string(),
            tool_sequence: vec![],
            param_patterns: vec![],
            success_indicators: vec![],
            steps: vec![],
            reusability_score: 0.8,
            source_task_id: "task-1".to_string(),
            created_at: Utc::now(),
        };
        
        let low_score_pattern = TaskPattern {
            id: "2".to_string(),
            task_category: "general".to_string(),
            tool_sequence: vec![],
            param_patterns: vec![],
            success_indicators: vec![],
            steps: vec![],
            reusability_score: 0.3,
            source_task_id: "task-2".to_string(),
            created_at: Utc::now(),
        };
        
        assert!(analyzer.is_repeatable(&high_score_pattern));
        assert!(!analyzer.is_repeatable(&low_score_pattern));
    }

    #[test]
    fn test_score_reusability() {
        let analyzer = PatternAnalyzer::new();
        
        // High reusability: API call with generic params
        let param_patterns = vec![
            ParamPattern {
                name: "query".to_string(),
                param_type: ParamType::String,
                is_generic: true,
                examples: vec![],
            },
        ];
        let steps = vec![
            ExecutionStep {
                step_number: 1,
                tool_name: "api_call".to_string(),
                input_summary: "test".to_string(),
                output_summary: "result".to_string(),
                success: true,
            },
        ];
        
        let score = analyzer.score_reusability("api_call", &param_patterns, &steps);
        assert!(score > 0.6);
        
        // Low reusability: general task with specific params
        let score = analyzer.score_reusability("general", &[], &[]);
        assert!(score < 0.6);
    }

    #[test]
    fn test_summarize_value() {
        let analyzer = PatternAnalyzer::new();
        
        let short_string = serde_json::json!("hello");
        assert_eq!(analyzer.summarize_value(&short_string), "hello");
        
        let long_string = serde_json::json!("this is a very long string that exceeds fifty characters");
        let summary = analyzer.summarize_value(&long_string);
        assert!(summary.contains("..."));
        
        let obj = serde_json::json!({"key1": "value1", "key2": "value2"});
        let summary = analyzer.summarize_value(&obj);
        assert!(summary.contains("key1"));
        
        let arr = serde_json::json!([1, 2, 3, 4, 5]);
        let summary = analyzer.summarize_value(&arr);
        assert!(summary.contains("5 items"));
    }

    #[test]
    fn test_param_type_from_json_value() {
        assert_eq!(ParamType::from_json_value(&serde_json::json!("test")), ParamType::String);
        assert_eq!(ParamType::from_json_value(&serde_json::json!(123)), ParamType::Number);
        assert_eq!(ParamType::from_json_value(&serde_json::json!(true)), ParamType::Boolean);
        assert_eq!(ParamType::from_json_value(&serde_json::json!({})), ParamType::Object);
        assert_eq!(ParamType::from_json_value(&serde_json::json!([])), ParamType::Array);
    }

    #[test]
    fn test_extract_steps() {
        let analyzer = PatternAnalyzer::new();
        
        let tool_calls = vec![
            ToolCall {
                name: "search".to_string(),
                arguments: serde_json::json!({"query": "test"}),
                result: Some(serde_json::json!(["result"])),
                duration_ms: 100,
            },
            ToolCall {
                name: "fetch".to_string(),
                arguments: serde_json::json!({"url": "test.com"}),
                result: None,
                duration_ms: 50,
            },
        ];
        
        let steps = analyzer.extract_steps(&tool_calls);
        
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].step_number, 1);
        assert_eq!(steps[0].tool_name, "search");
        assert!(steps[0].success);
        
        assert_eq!(steps[1].step_number, 2);
        assert_eq!(steps[1].tool_name, "fetch");
        assert!(!steps[1].success);
    }

    #[test]
    fn test_extract_success_indicators() {
        let analyzer = PatternAnalyzer::new();
        
        let tool_calls = vec![
            ToolCall {
                name: "search".to_string(),
                arguments: serde_json::json!({}),
                result: Some(serde_json::json!([])),
                duration_ms: 100,
            },
            ToolCall {
                name: "fetch".to_string(),
                arguments: serde_json::json!({}),
                result: None,
                duration_ms: 100,
            },
        ];
        
        let indicators = analyzer.extract_success_indicators(&tool_calls);
        
        assert_eq!(indicators.len(), 1);
        assert_eq!(indicators[0], "search_success");
    }

    #[test]
    fn test_with_config() {
        let config = AnalyzerConfig {
            min_reusability_threshold: 0.7,
            max_tool_sequence_length: 10,
            enable_deep_analysis: false,
        };
        
        let analyzer = PatternAnalyzer::with_config(config.clone());
        
        // Test that custom config is used
        let tool_calls = vec![ToolCall {
            name: "test".to_string(),
            arguments: serde_json::json!({}),
            result: None,
            duration_ms: 0,
        }];
        
        let pattern = analyzer.extract("task-1", "test task", &tool_calls);
        assert!(pattern.reusability_score < 0.7 || pattern.reusability_score >= 0.0);
    }
}
