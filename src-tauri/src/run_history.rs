use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RunRecord {
    pub id: String,
    pub script_name: String,
    pub start_time: String,
    pub end_time: String,
    pub duration_ms: u64,
    pub status: String, // "success", "error", "aborted"
    pub output: String,
    pub error: Option<String>,
}

pub fn history_file() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        let dir = PathBuf::from(appdata).join("script-runner");
        let _ = fs::create_dir_all(&dir);
        return dir.join("run_history.json");
    }
    PathBuf::from("./run_history.json")
}

pub fn add_record(record: RunRecord) -> Result<(), String> {
    let path = history_file();
    let mut records: Vec<RunRecord> = if path.exists() {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read history: {}", e))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    records.push(record);

    // Keep last 1000 records
    if records.len() > 1000 {
        records.drain(0..records.len() - 1000);
    }

    let content = serde_json::to_string_pretty(&records)
        .map_err(|e| format!("Failed to serialize history: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("Failed to write history: {}", e))?;

    Ok(())
}

pub fn get_records(limit: usize) -> Result<Vec<RunRecord>, String> {
    let path = history_file();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read history: {}", e))?;
    let records: Vec<RunRecord> = serde_json::from_str(&content).unwrap_or_default();

    Ok(records.into_iter().rev().take(limit).collect())
}

pub fn export_as_csv(records: &[RunRecord]) -> String {
    let mut csv = String::from("Skrypt,Status,Czas rozpoczęcia,Czas zakończenia,Czas trwania (ms),Wyjście\n");

    for record in records {
        let output = record.output.replace("\"", "\"\"").replace("\n", " ");
        let error = record.error.as_ref().map(|e| e.replace("\"", "\"\"")).unwrap_or_default();
        let status_output = if record.status == "error" {
            format!("{} - {}", record.status, error)
        } else {
            record.status.clone()
        };
        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",{},\"{}\"\n",
            record.script_name.replace("\"", "\"\""),
            status_output,
            record.start_time,
            record.end_time,
            record.duration_ms,
            output
        ));
    }

    csv
}
