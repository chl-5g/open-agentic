use std::path::Path;
use std::sync::Arc;

use crate::evo::registry::{DynamicSkill, SharedSkillRegistry};
use crate::evo::{DynamicCompiler, ProgrammingLanguage};

#[derive(Debug)]
pub struct SkillManifest {
    pub name: String,
    pub description: String,
    pub language: String,
    pub parameters: Vec<Parameter>,
    pub dependencies: Vec<String>,
    pub code: String,
}

#[derive(Debug)]
pub struct Parameter {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub description: String,
}

pub struct SkillLoader {
    registry: Arc<SharedSkillRegistry>,
    compiler: DynamicCompiler,
}

impl SkillLoader {
    pub fn new(registry: Arc<SharedSkillRegistry>) -> Self {
        Self {
            registry,
            compiler: DynamicCompiler::new(ProgrammingLanguage::Wasm),
        }
    }

    pub async fn load_from_file(&self, path: &Path) -> Result<DynamicSkill, String> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))?;

        self.parse_skill_md(&content)
    }

    pub async fn load_from_directory(&self, dir: &Path) -> Result<Vec<DynamicSkill>, String> {
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut skills = Vec::new();
        
        let mut entries = tokio::fs::read_dir(dir)
            .await
            .map_err(|e| format!("Failed to read dir: {}", e))?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                match self.load_from_file(&path).await {
                    Ok(skill) => skills.push(skill),
                    Err(e) => tracing::warn!("Failed to load {}: {}", path.display(), e),
                }
            }
        }

        Ok(skills)
    }

    pub fn parse_skill_md(&self, content: &str) -> Result<DynamicSkill, String> {
        let manifest = self.parse_manifest(content)?;
        
        let skill = DynamicSkill::new(
            format!("skill_{}", manifest.name),
            manifest.name,
            manifest.code,
            manifest.language,
            "user".to_string(),
        );

        Ok(skill)
    }

    fn parse_manifest(&self, content: &str) -> Result<SkillManifest, String> {
        let lines: Vec<&str> = content.lines().collect();
        
        let mut name = String::new();
        let mut description = String::new();
        let mut language = String::new();
        let mut parameters = Vec::new();
        let mut dependencies = Vec::new();
        let mut in_code_block = false;
        let mut code_lines = Vec::new();
        let mut current_section = String::new();

        for line in lines {
            let trimmed = line.trim();
            
            if trimmed.starts_with("# Skill:") {
                name = trimmed.trim_start_matches("# Skill:").trim().to_string();
            } else if trimmed.starts_with("## Description") {
                current_section = "description".to_string();
            } else if trimmed.starts_with("## Language") {
                current_section = "language".to_string();
            } else if trimmed.starts_with("## Parameters") {
                current_section = "parameters".to_string();
            } else if trimmed.starts_with("## Dependencies") {
                current_section = "dependencies".to_string();
            } else if trimmed.starts_with("## Code") {
                current_section = "code".to_string();
            } else if trimmed.starts_with("```") {
                if in_code_block {
                    in_code_block = false;
                } else {
                    in_code_block = true;
                }
            } else if in_code_block {
                code_lines.push(trimmed);
            } else if !trimmed.is_empty() {
                match current_section.as_str() {
                    "description" => description.push_str(trimmed),
                    "language" => language.push_str(trimmed),
                    "parameters" => {
                        if trimmed.starts_with("- ") {
                            let param_str = trimmed.trim_start_matches("- ");
                            let parts: Vec<&str> = param_str.splitn(2, ':').collect();
                            if parts.len() >= 2 {
                                let name_type = parts[0].trim();
                                let desc = parts[1].trim();
                                let (param_name, param_type) = if name_type.contains('(') {
                                    let idx = name_type.find('(').unwrap();
                                    (
                                        name_type[..idx].trim().to_string(),
                                        name_type[idx+1..name_type.len()-1].trim().to_string(),
                                    )
                                } else {
                                    (name_type.to_string(), "string".to_string())
                                };
                                parameters.push(Parameter {
                                    name: param_name,
                                    param_type,
                                    required: !param_str.contains("optional"),
                                    description: desc.to_string(),
                                });
                            }
                        }
                    }
                    "dependencies" => {
                        if trimmed.starts_with("- ") {
                            dependencies.push(trimmed.trim_start_matches("- ").to_string());
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(SkillManifest {
            name,
            description,
            language,
            parameters,
            dependencies,
            code: code_lines.join("\n"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_skill_md() {
        let content = r#"# Skill: python_calculator

## Description
Execute Python code for mathematical calculations

## Language
python

## Parameters
- code: string (required) - Python code to execute

## Dependencies
- numpy

## Code
```python
def execute(params):
    code = params.get("code", "")
    return eval(code)
```
"#;

        let loader = SkillLoader::new(Arc::new(SharedSkillRegistry::new()));
        let result = loader.parse_skill_md(content);
        
        assert!(result.is_ok());
        let skill = result.unwrap();
        assert_eq!(skill.name, "python_calculator");
        assert_eq!(skill.language, "python");
    }
}
