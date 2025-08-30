use crate::utils::FileLock;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatasetConfig {
    pub version: u32,
    pub index_columns: Option<Vec<String>>,
    pub views: BTreeMap<String, ViewDefinition>,
    pub default_view: String,
}

impl Default for DatasetConfig {
    fn default() -> Self {
        let mut views = BTreeMap::new();
        views.insert("main".to_string(), ViewDefinition::default());
        Self {
            version: 1,
            index_columns: None,
            views,
            default_view: "main".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ViewDefinition {
    pub chart_type: ChartType,
    pub roles: ViewRoles,
    pub style: Option<serde_json::Value>,
    pub filters: Option<Vec<serde_json::Value>>,
}

impl Default for ViewDefinition {
    fn default() -> Self {
        Self {
            chart_type: ChartType::Line,
            roles: ViewRoles::default(),
            style: None,
            filters: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ChartType {
    Line,
    Heatmap,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ViewRoles {
    pub x: String,
    pub y: Vec<String>,
    pub z: Option<String>,
    pub color: Option<String>,
}

impl Default for ViewRoles {
    fn default() -> Self {
        Self {
            x: "".to_string(),
            y: vec![],
            z: None,
            color: None,
        }
    }
}

impl DatasetConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }
        let file = File::open(path)
            .with_context(|| format!("Failed to open config: {}", path.display()))?;
        let cfg = serde_json::from_reader(file)
            .with_context(|| format!("Failed to parse config: {}", path.display()))?;
        Ok(cfg)
    }

    pub fn save_atomic(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        // Acquire an exclusive lock per-config to prevent concurrent writers from
        // clobbering the same tmp file. Lock file: <config_filename>.lock
        let filename = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "config".to_string());
        let lock_path = path.with_file_name(format!("{}.lock", filename));
        let _lock = FileLock::new(&lock_path)
            .with_context(|| format!("Failed to acquire config lock: {}", lock_path.display()))?;

        let tmp = path.with_extension("json.tmp");
        let mut file = File::create(&tmp)
            .with_context(|| format!("Failed to create tmp config: {}", tmp.display()))?;
        serde_json::to_writer_pretty(&file, self)
            .with_context(|| format!("Failed to serialize config to: {}", tmp.display()))?;
        file.flush()
            .with_context(|| format!("Failed to flush tmp config: {}", tmp.display()))?;
        std::fs::rename(&tmp, path)
            .with_context(|| format!("Failed to rename tmp config to: {}", path.display()))?;
        // lock is released when _lock is dropped at end of scope
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        // default_view must exist
        if !self.views.contains_key(&self.default_view) {
            anyhow::bail!("default_view '{}' not found in views", self.default_view);
        }

        // simple validation: heatmap requires z role
        for (name, view) in &self.views {
            if matches!(view.chart_type, ChartType::Heatmap) {
                if view.roles.z.is_none() {
                    anyhow::bail!("view '{}' is heatmap but missing z role", name);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn load_default_when_missing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        let cfg = DatasetConfig::load(&path).unwrap();
        assert_eq!(cfg.version, 1);
        assert!(cfg.views.contains_key("main"));
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        let cfg = DatasetConfig {
            index_columns: Some(vec!["a".to_string(), "b".to_string()]),
            ..DatasetConfig::default()
        };

        cfg.save_atomic(&path).unwrap();

        let loaded = DatasetConfig::load(&path).unwrap();
        assert_eq!(loaded.version, cfg.version);
        assert_eq!(
            loaded.index_columns.unwrap(),
            vec!["a".to_string(), "b".to_string()]
        );
    }

    #[test]
    fn validate_fails_when_default_missing() {
        let cfg = DatasetConfig {
            default_view: "nonexistent".to_string(),
            ..DatasetConfig::default()
        };
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn validate_heatmap_requires_z() {
        let mut cfg = DatasetConfig {
            ..DatasetConfig::default()
        };
        let view = ViewDefinition {
            chart_type: ChartType::Heatmap,
            roles: ViewRoles {
                x: "x".to_string(),
                y: vec![],
                z: None,
                color: None,
            },
            style: None,
            filters: None,
        };
        cfg.views.insert("hm".to_string(), view);
        cfg.default_view = "hm".to_string();
        assert!(cfg.validate().is_err());
    }
}
