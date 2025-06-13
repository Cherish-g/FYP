//use rusqlite::{params, Connection};
use serde::{Deserialize, Deserializer};

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct ProbeData {
    #[serde(rename = "ID")]
    pub id: u32,
    #[serde(rename = "Date")]
    pub date: String,
    #[serde(rename = "Time")]
    pub time: String,
    #[serde(rename = "Router IP")]
    pub router_ip: String,
    #[serde(rename = "Router SSID (Location)")]
    pub router_ssid: String,
    #[serde(rename = "Router MAC")]
    pub router_mac: String,
    #[serde(rename = "Interface")]
    pub interface: String,
    #[serde(rename = "Latency (ms)")]
    pub latency: Option<f64>,
    #[serde(rename = "Jitter (ms)")]
    pub jitter: Option<f64>,
    #[serde(rename = "Packet Loss (%)")]
    pub packet_loss: Option<f64>,
    #[serde(rename = "Signal Strength", deserialize_with = "de_optional_percent")]
    pub signal_strength: Option<f64>,
    #[serde(rename = "Download Speed (Mbps)", deserialize_with = "de_optional_float")]
    pub download_speed: Option<f64>,
    #[serde(rename = "Upload Speed (Mbps)", deserialize_with = "de_optional_float")]
    pub upload_speed: Option<f64>,
    #[serde(rename = "ISP Name")]
    pub isp_name: String,
    #[serde(rename = "Gateway Reachability")]
    pub gateway_reachability: String,
    #[serde(rename = "Interface IP")]
    pub interface_ip: String,
}

#[derive(Debug)]
pub struct NetworkHealth {
    pub averages: Averages,
    pub status: HealthStatus,
}

#[derive(Debug, PartialEq)]
pub enum HealthStatus {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

// Handles things like "45.2", "  ", "n/a", etc.
fn de_optional_float<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let trimmed = s.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("n/a") || trimmed.eq_ignore_ascii_case("unknown") {
        return Ok(None);
    }
    trimmed.parse::<f64>().map(Some).map_err(serde::de::Error::custom)
}

// Handles "85%", "n/a", or ""
fn de_optional_percent<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let trimmed = s.trim().trim_end_matches('%').trim();
    
    if trimmed.is_empty() 
        || trimmed.eq_ignore_ascii_case("n/a") 
        || trimmed.eq_ignore_ascii_case("null")
        || trimmed.eq_ignore_ascii_case("unknown") {
        return Ok(None);
    }
    
    // Handle cases like "31%" -> 31.0
    trimmed.parse::<f64>()
        .map(Some)
        .map_err(|_| serde::de::Error::custom(format!("Invalid percentage value: {}", s)))
}


use chrono::NaiveDate;

pub fn read_csv(path: &str) -> Result<Vec<ProbeData>, Box<dyn std::error::Error>> {
    let mut rdr = csv::Reader::from_path(path)?;
    let mut results = Vec::new();
    for result in rdr.deserialize() {
        let record: ProbeData = result?;
        results.push(record);
    }
    Ok(results)
}

pub fn filter_last_n_days(data: &[ProbeData], days: i64) -> Vec<ProbeData> {
    let today = chrono::Local::now().naive_local().date();
    data.iter()
        .filter_map(|d| NaiveDate::parse_from_str(&d.date, "%Y-%m-%d").ok().map(|record_date| (d, record_date)))
        .filter(|(_, record_date)| {
            *record_date >= today - chrono::Duration::days(days)
        })
        .map(|(d, _)| d.clone())
        .collect()
}

#[derive(Debug)]
pub struct Averages {
    pub latency: Option<f64>,
    pub jitter: Option<f64>,
    pub packet_loss: Option<f64>,
    pub signal_strength: Option<f64>,
    pub download_speed: Option<f64>,
    pub upload_speed: Option<f64>,
}

pub fn calculate_averages(data: &[ProbeData]) -> Averages {
    fn avg<I: Iterator<Item = f64>>(iter: I) -> f64 {
        let (sum, count) = iter.fold((0.0, 0), |(s, c), x| (s + x, c + 1));
        if count > 0 { sum / count as f64 } else { 0.0 }
    }

    fn opt_avg<I: Iterator<Item = Option<f64>>>(iter: I) -> Option<f64> {
        let (sum, count) = iter.fold((0.0, 0), |(s, c), x| {
            if let Some(val) = x {
            (s + val, c + 1)
            } else {
            (s, c)
            }
        });
        if count > 0 { Some(sum / count as f64) } else { None }
    }

    Averages {
        latency: opt_avg(data.iter().map(|d| d.latency)),
        jitter: opt_avg(data.iter().map(|d| d.jitter)),
        packet_loss: opt_avg(data.iter().map(|d| d.packet_loss)),
        signal_strength: opt_avg(data.iter().map(|d| d.signal_strength)),
        download_speed: opt_avg(data.iter().map(|d| d.download_speed)),
        upload_speed: opt_avg(data.iter().map(|d| d.upload_speed)),
    }
}

pub fn determine_health(averages: &Averages) -> HealthStatus {
    // Implement your health determination logic
    if averages.packet_loss.unwrap_or(0.0) > 5.0 || averages.latency.unwrap_or(0.0) > 150.0 {
        HealthStatus::Critical
    } else if averages.signal_strength.unwrap_or(100.0) < 50.0 || averages.download_speed.unwrap_or(0.0) < 10.0 {
        HealthStatus::Poor
    } else if averages.jitter.unwrap_or(0.0) > 10.0 {
        HealthStatus::Fair
    } else {
        HealthStatus::Good
    }
}
