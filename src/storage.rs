use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::model::{AppData, Project, Thread};

pub struct StorageManager {
    file_path: PathBuf,
}

impl StorageManager {
    pub fn new() -> Result<Self> {
        let base_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".k7code");

        if !base_dir.exists() {
            fs::create_dir_all(&base_dir)
                .with_context(|| format!("Failed to create storage dir: {:?}", base_dir))?;
        }

        let file_path = base_dir.join("data.json");
        Ok(Self { file_path })
    }

    pub fn load_or_init(&self) -> AppData {
        if self.file_path.exists() {
            match fs::read_to_string(&self.file_path) {
                Ok(content) => match serde_json::from_str::<AppData>(&content) {
                    Ok(mut data) => {
                        if data.projects.is_empty() {
                            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                            let project_name = cwd
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("k7code")
                                .to_string();
                            let proj = Project::new(project_name, cwd);
                            data.active_project_id = Some(proj.id.clone());
                            data.projects.push(proj);
                        }

                        if data.active_project_id.is_none() && !data.projects.is_empty() {
                            data.active_project_id = Some(data.projects[0].id.clone());
                        }

                        if let Some(ref proj_id) = data.active_project_id {
                            let proj_threads: Vec<_> = data.threads.iter().filter(|t| t.project_id == *proj_id).collect();
                            if proj_threads.is_empty() {
                                let new_thread = Thread::new(proj_id, "Welcome to k7code");
                                data.active_thread_id = Some(new_thread.id.clone());
                                data.threads.push(new_thread);
                            } else if data.active_thread_id.is_none() {
                                data.active_thread_id = Some(proj_threads[0].id.clone());
                            }
                        }

                        return data;
                    }
                    Err(e) => {
                        eprintln!("Failed to parse data.json: {e}, initializing fresh state");
                    }
                },
                Err(e) => {
                    eprintln!("Failed to read data.json: {e}, initializing fresh state");
                }
            }
        }

        let mut data = AppData::default();
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let project_name = cwd
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("k7code")
            .to_string();
        let proj = Project::new(project_name, cwd);
        let thread = Thread::new(&proj.id, "Welcome to k7code");

        data.active_project_id = Some(proj.id.clone());
        data.active_thread_id = Some(thread.id.clone());
        data.projects.push(proj);
        data.threads.push(thread);

        let _ = self.save(&data);
        data
    }

    pub fn save(&self, data: &AppData) -> Result<()> {
        let json = serde_json::to_string_pretty(data)
            .context("Failed to serialize AppData to JSON")?;
        fs::write(&self.file_path, json)
            .with_context(|| format!("Failed to write state to {:?}", self.file_path))?;
        Ok(())
    }
}
