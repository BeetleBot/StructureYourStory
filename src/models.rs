use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Reference {
    pub label: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StructureStep {
    pub id: String,
    pub name: String,
    pub target: String,
    pub prompt: String,
    #[serde(default)]
    pub content: String,
    #[serde(default = "default_status")]
    pub status: String, // empty, draft, done
}

fn default_status() -> String {
    "empty".to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoryStructure {
    pub id: String,
    pub name: String,
    pub mediums: Vec<String>,
    pub author: String,
    #[serde(rename = "type")]
    pub structure_type: String,
    pub complexity: String,
    pub description: String,
    pub best_for: String,
    pub avoid_if: String,
    pub references: Vec<Reference>,
    pub steps: Vec<StructureStep>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectMetadata {
    pub title: String,
    #[serde(default)]
    pub genre: String,
    #[serde(default)]
    pub logline: String,
    #[serde(default)]
    pub estimated_length: String,
    #[serde(default)]
    pub notes: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub id: String,
    pub app_name: String,
    pub app_version: String,
    pub medium: String,
    pub structure_id: String,
    pub structure_name: String,
    pub metadata: ProjectMetadata,
    pub steps: Vec<StructureStep>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}
