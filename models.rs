use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Primary response model for the dashboard
#[derive(Debug, Serialize)]
pub struct NetworkHealthResponse {
    pub status: NetworkStatus,
    pub metrics: NetworkMetrics,
    pub optimizations: Vec<ActiveOptimization>,
    pub timestamp: DateTime<Utc>,
    pub time_range_seconds: u64,
}

/// Detailed network metrics (your 7 core fields)
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub latency_ms: Option<f64>,
    pub jitter_ms: Option<f64>,
    pub packet_loss_percent: Option<f64>,
    pub signal_strength_percent: Option<f64>,
    pub download_speed_mbps: Option<f64>,
    pub upload_speed_mbps: Option<f64>,
    pub gateway_reachable: bool,
}

/// Health status classification
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NetworkStatus {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

/// Currently active optimizations
#[derive(Debug, Serialize)]
pub struct ActiveOptimization {
    pub name: String,
    pub description: String,
    pub implemented_at: DateTime<Utc>,
    pub impact: OptimizationImpact,
}

/// Optimization impact assessment
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OptimizationImpact {
    Positive,
    Neutral,
    Negative,  // In case an optimization makes things worse
}

/// Request model for historical data queries
#[derive(Debug, Deserialize)]
pub struct HistoricalDataRequest {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    #[serde(default = "default_metrics")]
    pub metrics: Vec<String>,  // e.g. ["latency_ms", "packet_loss_percent"]
}

fn default_metrics() -> Vec<String> {
    vec![
        "latency_ms".into(),
        "jitter_ms".into(),
        "packet_loss_percent".into(),
        "signal_strength_percent".into(),
        "download_speed_mbps".into(),
        "upload_speed_mbps".into(),
    ]
}

/// Error response model
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
    pub suggestion: Option<String>,
}
