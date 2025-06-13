use actix_web::{get, post, web, App, HttpServer, Responder};
use serde_json::json;
use std::sync::{Arc, Mutex};
use crate::{optimizer::NetworkOptimizer, probe_data};

pub async fn run(optimizer: Arc<Mutex<NetworkOptimizer>>) -> std::io::Result<()> {
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(optimizer.clone()))
            .service(get_network_status)
            .service(analyze_network)
    })
    .bind("127.0.0.1:8080")?;

    println!("API server running on http://localhost:8080");
    server.run().await
}

#[post("/analyze")]
async fn analyze_network(
    optimizer: web::Data<Mutex<NetworkOptimizer>>,
) -> impl Responder {
    // Load and process data
    let data = match probe_data::read_csv("data.csv") {
        Ok(data) => data,
        Err(e) => {
            return web::Json(json!({
                "error": format!("Failed to load data: {}", e),
                "details": "Check if data.csv exists and is properly formatted"
            }))
        }
    };

    // Filter and analyze
    let recent_data = probe_data::filter_last_n_days(&data, 3);
    let averages = probe_data::calculate_averages(&recent_data);
    let health_status = probe_data::determine_health(&averages);

    // Apply optimizations
    let mut optimizer = optimizer.lock().unwrap();
    optimizer.apply_optimizations(&probe_data::NetworkHealth {
        averages: averages.clone(),
        status: health_status,
    });

    // Prepare response
    web::Json(json!({
        "metrics": {
            "latency_ms": averages.latency,
            "jitter_ms": averages.jitter,
            "packet_loss_percent": averages.packet_loss,
            "signal_strength_percent": averages.signal_strength,
            "download_speed_mbps": averages.download_speed,
            "upload_speed_mbps": averages.upload_speed,
        },
        "health_status": format!("{:?}", health_status),
        "optimizations": optimizer.get_current_optimizations(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "time_range_seconds": 3 * 24 * 60 * 60  // 3 days in seconds
    }))
}

#[get("/network-status")]
async fn get_network_status(
    optimizer: web::Data<Mutex<NetworkOptimizer>>,
) -> impl Responder {
    let optimizer = optimizer.lock().unwrap();
    web::Json(json!({
        "active_optimizations": optimizer.get_current_optimizations(),
        "failed_optimizations": optimizer.get_failed_optimizations(),
        "last_updated": chrono::Utc::now().to_rfc3339()
    }))
}
