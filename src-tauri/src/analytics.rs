use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsage {
    pub date: String,
    pub executions: usize,
    pub successes: usize,
    pub failures: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptStat {
    pub name: String,
    pub executions: usize,
    pub avg_duration: f64,
    pub success_rate: f64,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTimeDistribution {
    pub bucket: String, // "0-1s", "1-5s", "5-10s", "10-30s", "30s+"
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    pub category: String,
    pub official_count: usize,
    pub user_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsData {
    pub usage_over_time: Vec<DailyUsage>,
    pub top_scripts: Vec<ScriptStat>,
    pub success_rate: f64,
    pub avg_execution_time: f64,
    pub total_executions: usize,
    pub total_failures: usize,
    pub execution_time_distribution: Vec<ExecutionTimeDistribution>,
    pub category_stats: Vec<CategoryStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub timestamp: i64,
    pub script_name: String,
    pub duration_ms: u64,
    pub success: bool,
    pub category: String,
    pub is_official: bool,
}

// Get analytics data file path
fn get_analytics_file() -> PathBuf {
    let app_data = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ScriptRunner");

    fs::create_dir_all(&app_data).ok();
    app_data.join("analytics.json")
}

// Load execution records from file
fn load_records() -> Vec<ExecutionRecord> {
    let file_path = get_analytics_file();
    if !file_path.exists() {
        return Vec::new();
    }

    match fs::read_to_string(&file_path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| Vec::new()),
        Err(_) => Vec::new(),
    }
}

// Save execution records to file
fn save_records(records: &[ExecutionRecord]) -> Result<(), String> {
    let file_path = get_analytics_file();
    let content = serde_json::to_string_pretty(records)
        .map_err(|e| format!("Failed to serialize records: {}", e))?;

    fs::write(&file_path, content).map_err(|e| format!("Failed to write analytics file: {}", e))
}

// Track a script execution
pub fn track_execution(
    script_name: String,
    duration_ms: u64,
    success: bool,
    category: Option<String>,
    is_official: Option<bool>,
) -> Result<(), String> {
    let mut records = load_records();

    let record = ExecutionRecord {
        timestamp: Utc::now().timestamp(),
        script_name,
        duration_ms,
        success,
        category: category.unwrap_or_else(|| "uncategorized".to_string()),
        is_official: is_official.unwrap_or(false),
    };

    records.push(record);

    // Keep only last 10,000 records to prevent file bloat
    if records.len() > 10_000 {
        records.drain(0..records.len() - 10_000);
    }

    save_records(&records)
}

// Get analytics data for a specific time range
pub fn get_analytics_data(days: Option<u32>) -> Result<AnalyticsData, String> {
    let records = load_records();
    let days = days.unwrap_or(30);

    // Calculate cutoff timestamp
    let cutoff = Utc::now() - Duration::days(days as i64);
    let cutoff_timestamp = cutoff.timestamp();

    // Filter records within time range
    let filtered: Vec<_> = records
        .iter()
        .filter(|r| r.timestamp >= cutoff_timestamp)
        .collect();

    if filtered.is_empty() {
        return Ok(AnalyticsData {
            usage_over_time: Vec::new(),
            top_scripts: Vec::new(),
            success_rate: 0.0,
            avg_execution_time: 0.0,
            total_executions: 0,
            total_failures: 0,
            execution_time_distribution: Vec::new(),
            category_stats: Vec::new(),
        });
    }

    // Calculate daily usage
    let mut daily_map: HashMap<String, (usize, usize, usize)> = HashMap::new();
    for record in &filtered {
        let date = DateTime::from_timestamp(record.timestamp, 0)
            .unwrap_or_else(Utc::now)
            .format("%Y-%m-%d")
            .to_string();

        let entry = daily_map.entry(date).or_insert((0, 0, 0));
        entry.0 += 1; // total executions
        if record.success {
            entry.1 += 1; // successes
        } else {
            entry.2 += 1; // failures
        }
    }

    let mut usage_over_time: Vec<DailyUsage> = daily_map
        .into_iter()
        .map(|(date, (executions, successes, failures))| DailyUsage {
            date,
            executions,
            successes,
            failures,
        })
        .collect();

    usage_over_time.sort_by(|a, b| a.date.cmp(&b.date));

    // Calculate script stats
    let mut script_map: HashMap<String, (usize, u64, usize, String)> = HashMap::new();
    for record in &filtered {
        let entry = script_map.entry(record.script_name.clone()).or_insert((
            0,
            0,
            0,
            record.category.clone(),
        ));

        entry.0 += 1; // executions
        entry.1 += record.duration_ms; // total duration
        if record.success {
            entry.2 += 1; // successes
        }
    }

    let mut top_scripts: Vec<ScriptStat> = script_map
        .into_iter()
        .map(
            |(name, (executions, total_duration, successes, category))| ScriptStat {
                name,
                executions,
                avg_duration: total_duration as f64 / executions as f64,
                success_rate: (successes as f64 / executions as f64) * 100.0,
                category,
            },
        )
        .collect();

    top_scripts.sort_by(|a, b| b.executions.cmp(&a.executions));
    top_scripts.truncate(10);

    // Calculate overall stats
    let total_executions = filtered.len();
    let total_successes = filtered.iter().filter(|r| r.success).count();
    let total_failures = total_executions - total_successes;
    let success_rate = (total_successes as f64 / total_executions as f64) * 100.0;

    let total_duration: u64 = filtered.iter().map(|r| r.duration_ms).sum();
    let avg_execution_time = total_duration as f64 / total_executions as f64;

    // Calculate execution time distribution
    let mut time_buckets = HashMap::new();
    for record in &filtered {
        let bucket = match record.duration_ms {
            0..=1000 => "0-1s",
            1001..=5000 => "1-5s",
            5001..=10000 => "5-10s",
            10001..=30000 => "10-30s",
            _ => "30s+",
        };
        *time_buckets.entry(bucket).or_insert(0) += 1;
    }

    let execution_time_distribution: Vec<ExecutionTimeDistribution> = time_buckets
        .into_iter()
        .map(|(bucket, count)| ExecutionTimeDistribution {
            bucket: bucket.to_string(),
            count,
        })
        .collect();

    // Calculate category stats
    let mut category_map: HashMap<String, (usize, usize)> = HashMap::new();
    for record in &filtered {
        let entry = category_map
            .entry(record.category.clone())
            .or_insert((0, 0));

        if record.is_official {
            entry.0 += 1;
        } else {
            entry.1 += 1;
        }
    }

    let category_stats: Vec<CategoryStats> = category_map
        .into_iter()
        .map(|(category, (official_count, user_count))| CategoryStats {
            category,
            official_count,
            user_count,
        })
        .collect();

    Ok(AnalyticsData {
        usage_over_time,
        top_scripts,
        success_rate,
        avg_execution_time,
        total_executions,
        total_failures,
        execution_time_distribution,
        category_stats,
    })
}

// Export analytics data
pub fn export_analytics(format: String, days: Option<u32>) -> Result<String, String> {
    let data = get_analytics_data(days)?;

    match format.as_str() {
        "json" => serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Failed to serialize to JSON: {}", e)),
        "csv" => {
            let mut csv = String::from("Date,Executions,Successes,Failures\n");
            for usage in &data.usage_over_time {
                csv.push_str(&format!(
                    "{},{},{},{}\n",
                    usage.date, usage.executions, usage.successes, usage.failures
                ));
            }

            csv.push_str("\n\nTop Scripts\n");
            csv.push_str("Script,Executions,Avg Duration (ms),Success Rate (%),Category\n");
            for script in &data.top_scripts {
                csv.push_str(&format!(
                    "{},{},{:.2},{:.2},{}\n",
                    script.name,
                    script.executions,
                    script.avg_duration,
                    script.success_rate,
                    script.category
                ));
            }

            Ok(csv)
        }
        _ => Err(format!("Unsupported format: {}", format)),
    }
}

// Clear all analytics data
pub fn clear_analytics_data() -> Result<(), String> {
    let file_path = get_analytics_file();
    if file_path.exists() {
        fs::remove_file(&file_path).map_err(|e| format!("Failed to delete analytics file: {}", e))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_and_retrieve() {
        clear_analytics_data().ok();

        track_execution(
            "test_script.py".to_string(),
            1500,
            true,
            Some("test".to_string()),
            Some(false),
        )
        .unwrap();

        let data = get_analytics_data(Some(7)).unwrap();
        assert_eq!(data.total_executions, 1);
        assert_eq!(data.success_rate, 100.0);
    }
}
