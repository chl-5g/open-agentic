use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::CompiledSkill;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SkillSource {
    User,
    Evo,
    Imported,
}

impl Default for SkillSource {
    fn default() -> Self {
        SkillSource::User
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicSkill {
    pub id: String,
    pub name: String,
    pub code: String,
    pub language: String,
    pub source: SkillSource,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub version: String,
}

impl DynamicSkill {
    pub fn new(
        id: String,
        name: String,
        code: String,
        language: String,
        created_by: String,
    ) -> Self {
        Self {
            id,
            name,
            code,
            language,
            source: SkillSource::default(),
            created_by,
            created_at: Utc::now(),
            version: "1.0.0".to_string(),
        }
    }

    pub fn with_source(mut self, source: SkillSource) -> Self {
        self.source = source;
        self
    }
}

pub struct SharedSkillRegistry {
    inner: Arc<RwLock<SkillRegistryInner>>,
    compiled_skills: Arc<RwLock<HashMap<String, CompiledSkill>>>,
}

#[derive(Debug, Clone)]
struct SkillRegistryInner {
    skills: Vec<DynamicSkill>,
}

impl Default for SkillRegistryInner {
    fn default() -> Self {
        Self {
            skills: Vec::new(),
        }
    }
}

impl SharedSkillRegistry {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(SkillRegistryInner::default())),
            compiled_skills: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_skill(&self, skill: DynamicSkill) {
        let mut registry = self.inner.write().await;
        registry.skills.push(skill);
    }

    pub async fn register_compiled(&self, skill_id: &str, compiled: CompiledSkill) {
        let mut compiled_map = self.compiled_skills.write().await;
        compiled_map.insert(skill_id.to_string(), compiled);
    }

    pub async fn get_compiled_skill(&self, skill_id: &str) -> Option<CompiledSkill> {
        let compiled_map = self.compiled_skills.read().await;
        compiled_map.get(skill_id).cloned()
    }

    pub async fn get_skill(&self, id: &str) -> Option<DynamicSkill> {
        let registry = self.inner.read().await;
        registry.skills.iter().find(|s| s.id == id).cloned()
    }

    pub async fn get_skill_by_name(&self, name: &str) -> Option<DynamicSkill> {
        let registry = self.inner.read().await;
        registry.skills.iter().find(|s| s.name == name).cloned()
    }

    pub async fn get_all_skills(&self) -> Vec<DynamicSkill> {
        let registry = self.inner.read().await;
        registry.skills.clone()
    }

    pub async fn get_skills_by_source(&self, source: SkillSource) -> Vec<DynamicSkill> {
        let registry = self.inner.read().await;
        registry.skills
            .iter()
            .filter(|s| s.source == source)
            .cloned()
            .collect()
    }

    pub async fn skill_exists(&self, name: &str) -> bool {
        let registry = self.inner.read().await;
        registry.skills.iter().any(|s| s.name == name)
    }

    pub fn clone_arc(&self) -> Arc<RwLock<SkillRegistryInner>> {
        Arc::clone(&self.inner)
    }

    pub fn clone_inner(&self) -> Arc<SharedSkillRegistry> {
        Arc::new(Self {
            inner: Arc::clone(&self.inner),
            compiled_skills: Arc::clone(&self.compiled_skills),
        })
    }
}

impl Default for SharedSkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}
