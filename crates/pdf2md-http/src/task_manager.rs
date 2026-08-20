use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use pdf2md_core::Converter;
use uuid::Uuid;
use crate::models::{ConversionResponse, TaskStatus, TaskStatusResponse};

#[derive(Debug, Clone)]
struct TaskRecord {
    task_id: String,
    status: TaskStatus,
    result: Option<ConversionResponse>,
    error: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TaskManager {
    tasks: Arc<RwLock<HashMap<String, TaskRecord>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Submits a new asynchronous PDF conversion task and returns the unique task_id
    pub fn submit_task(&self, pdf_bytes: Vec<u8>, converter: Arc<Converter>) -> String {
        let task_id = format!("task_{}", Uuid::new_v4().simple());

        let record = TaskRecord {
            task_id: task_id.clone(),
            status: TaskStatus::Processing,
            result: None,
            error: None,
        };

        if let Ok(mut map) = self.tasks.write() {
            map.insert(task_id.clone(), record);
        }

        let tasks_clone = Arc::clone(&self.tasks);
        let task_id_clone = task_id.clone();

        thread::spawn(move || {
            match converter.convert_bytes(&pdf_bytes) {
                Ok(res) => {
                    let conv_res = ConversionResponse {
                        markdown: res.markdown,
                        confidence: res.diagnostics.overall_confidence,
                        total_pages: res.diagnostics.total_pages,
                        tables_detected: res.diagnostics.tables_detected,
                        images_extracted: res.diagnostics.images_extracted,
                    };

                    if let Ok(mut map) = tasks_clone.write() {
                        if let Some(entry) = map.get_mut(&task_id_clone) {
                            entry.status = TaskStatus::Completed;
                            entry.result = Some(conv_res);
                        }
                    }
                }
                Err(err) => {
                    if let Ok(mut map) = tasks_clone.write() {
                        if let Some(entry) = map.get_mut(&task_id_clone) {
                            entry.status = TaskStatus::Failed;
                            entry.error = Some(err.to_string());
                        }
                    }
                }
            }
        });

        task_id
    }

    /// Retrieves current status of a task by its task_id
    pub fn get_task_status(&self, task_id: &str) -> Option<TaskStatusResponse> {
        if let Ok(map) = self.tasks.read() {
            map.get(task_id).map(|rec| TaskStatusResponse {
                task_id: rec.task_id.clone(),
                status: rec.status.clone(),
                result: rec.result.clone(),
                error: rec.error.clone(),
            })
        } else {
            None
        }
    }
}
