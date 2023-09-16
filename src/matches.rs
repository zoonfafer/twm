use crate::config::TwmGlobal;
use anyhow::Result;
use std::{collections::HashMap, path::Path};

use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SafePath {
    pub path: String,
}

impl TryFrom<&Path> for SafePath {
    type Error = anyhow::Error;

    fn try_from(path: &Path) -> Result<Self> {
        match path.to_str() {
            Some(s) => Ok(Self {
                path: s.to_string(),
            }),
            None => anyhow::bail!("Path is not valid UTF-8"),
        }
    }
}

impl SafePath {
    #[must_use]
    pub fn new(str: &str) -> Self {
        Self {
            path: str.to_string(),
        }
    }
}

pub fn find_workspaces_in_dir<'a>(
    dir: &str,
    config: &'a TwmGlobal,
    workspaces: &mut HashMap<SafePath, &'a str>,
) {
    let is_excluded = |entry: &DirEntry| -> bool {
        match entry
            .path()
            .components()
            .last()
            .expect("Surely there is always a last component?")
            .as_os_str()
            .to_str()
        {
            Some(s) => config.exclude_path_components.iter().any(|e| s == e),
            None => true,
        }
    };

    let walker = WalkDir::new(dir)
        .max_depth(config.max_search_depth)
        .into_iter()
        .filter_entry(|e| e.file_type().is_dir() && !is_excluded(e))
        .filter_map(std::result::Result::ok);

    for entry in walker {
        let path = match SafePath::try_from(entry.path()) {
            Ok(p) => p,
            Err(_) => continue,
        };

        let mut workspace_type: Option<&'a str> = None;
        let mut has_all_files: bool = true;

        for (_, workspace_definition) in &config.workspace_definitions {
            match &workspace_definition.has_all_files {
                Some(filenames) => {
                    for file_name in filenames {
                        if !entry.path().join(file_name).exists() {
                            has_all_files = false;
                            break;
                        }
                    }
                }
                None => (),
            }

            if has_all_files {
                for file_name in &workspace_definition.has_any_file {
                    if entry.path().join(file_name).exists() {
                        workspace_type = Some(workspace_definition.name.as_str());
                        break;
                    }
                }
            }
            if workspace_type.is_some() {
                break;
            }
        }
        if let Some(workspace_type) = workspace_type {
            workspaces.insert(path, workspace_type);
        }
    }
}
