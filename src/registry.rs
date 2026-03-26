use crate::models::StoryStructure;
use rust_embed::RustEmbed;
use std::collections::HashMap;

#[derive(RustEmbed)]
#[folder = "structures/"]
struct Asset;

pub struct StructureRegistry {
    pub structures: HashMap<String, StoryStructure>,
}

impl StructureRegistry {
    pub fn new() -> Self {
        let mut registry = StructureRegistry {
            structures: HashMap::new(),
        };
        registry.load_all();
        registry
    }

    fn load_all(&mut self) {
        for file in Asset::iter() {
            if let Some(content) = Asset::get(file.as_ref()) {
                if let Ok(json_str) = std::str::from_utf8(content.data.as_ref()) {
                    if let Ok(structure) = serde_json::from_str::<StoryStructure>(json_str) {
                        self.structures.insert(structure.id.clone(), structure);
                    }
                }
            }
        }
    }

    pub fn get_all(&self) -> Vec<StoryStructure> {
        let mut values: Vec<StoryStructure> = self.structures.values().cloned().collect();
        values.sort_by(|a, b| a.name.cmp(&b.name));
        values
    }

    pub fn get_by_medium(&self, medium: &str) -> Vec<StoryStructure> {
        let mut values: Vec<StoryStructure> = self.structures
            .values()
            .filter(|s| s.mediums.contains(&medium.to_string()))
            .cloned()
            .collect();
        values.sort_by(|a, b| a.name.cmp(&b.name));
        values
    }
}
