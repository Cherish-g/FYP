use serde::Serialize;
use crate::probe_data::{NetworkHealth, HealthStatus};
use std::process::Command;
use sysinfo::{System, SystemExt, ProcessExt};

#[derive(Serialize)]
pub struct NetworkOptimizer {
    current_optimizations: Vec<String>,
    failed_optimizations: Vec<String>,
}

impl NetworkOptimizer {
    pub fn new() -> Self {
        Self {
            current_optimizations: Vec::new(),
            failed_optimizations: Vec::new(),
        }
    }

    fn execute_system_command(&self, command: &str, args: &[&str]) -> Result<(), String> {
        let output = Command::new(command)
            .args(args)
            .output()
            .map_err(|e| format!("Failed to run {command}: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "Command '{command}' failed ({}): {}",
                output.status, stderr
            ));
        }
        Ok(())
    }

    pub fn apply_optimizations(&mut self, health: &NetworkHealth) {
        self.current_optimizations.clear();
        self.failed_optimizations.clear();

        match health.status {
            HealthStatus::Critical => self.handle_critical(health),
            HealthStatus::Poor => self.handle_poor(health),
            HealthStatus::Fair => self.handle_fair(health),
            _ => if let Err(e) = self.maintain_good_state() {
                self.failed_optimizations.push(e);
            },
        }
    }

    fn handle_critical(&mut self, health: &NetworkHealth) {
        if health.averages.packet_loss.unwrap_or(0.0) > 5.0 {
            match self.switch_to_backup_connection() {
                Ok(_) => self.current_optimizations.push(
                    "Switched to backup connection".to_string(),
                ),
                Err(e) => self.failed_optimizations.push(e),
            }
        }
        
        if health.averages.latency.unwrap_or(0.0) > 150.0 {
            match self.enable_aggressive_qos() {
                Ok(_) => self.current_optimizations.push(
                    "Enabled aggressive QoS".to_string(),
                ),
                Err(e) => self.failed_optimizations.push(e),
            }
        }
        
        if let Err(e) = self.restart_network_services() {
            self.failed_optimizations.push(e);
        }
    }

    fn handle_poor(&mut self, health: &NetworkHealth) {
        if health.averages.signal_strength.unwrap_or(100.0) < 50.0 {
            match self.adjust_wireless_power() {
                Ok(_) => self.current_optimizations.push(
                    "Adjusted wireless power".to_string(),
                ),
                Err(e) => self.failed_optimizations.push(e),
            }
        }
        
        if health.averages.download_speed.unwrap_or(0.0) < 10.0 {
            match self.limit_bandwidth_hogs() {
                Ok(_) => self.current_optimizations.push(
                    "Limited bandwidth hogs".to_string(),
                ),
                Err(e) => self.failed_optimizations.push(e),
            }
        }
    }

    fn handle_fair(&mut self, health: &NetworkHealth) {
        if health.averages.jitter.unwrap_or(0.0) > 10.0 {
            match self.enable_jitter_buffering() {
                Ok(_) => self.current_optimizations.push(
                    "Enabled jitter buffering".to_string(),
                ),
                Err(e) => self.failed_optimizations.push(e),
            }
        }
    }

    fn maintain_good_state(&self) -> Result<(), String> {
        self.clean_cache()
    }

    fn switch_to_backup_connection(&self) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        {
            self.execute_system_command("nmcli", &["connection", "up", "backup-connection"])
        }
        #[cfg(not(target_os = "linux"))]
        {
            Ok(())
        }
    }

    fn enable_aggressive_qos(&self) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        {
            self.execute_system_command("tc", &["qdisc", "add", "dev", "eth0", "root", "htb"])
        }
        #[cfg(not(target_os = "linux"))]
        {
            Ok(())
        }
    }

    fn adjust_wireless_power(&self) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        {
            self.execute_system_command("iwconfig", &["wlan0", "txpower", "20"])
        }
        #[cfg(not(target_os = "linux"))]
        {
            Ok(())
        }
    }

    fn limit_bandwidth_hogs(&self) -> Result<(), String> {
        let mut sys = System::new();
        sys.refresh_all();
        
        for (pid, process) in sys.processes() {
            if process.disk_usage().total_read_bytes > 100_000_000 {
                self.execute_system_command("renice", &["19", &pid.to_string()])?;
            }
        }
        Ok(())
    }

    fn enable_jitter_buffering(&self) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        {
            self.execute_system_command("sh", &["-c", "echo 1 > /proc/sys/net/ipv4/tcp_low_latency"])
        }
        #[cfg(not(target_os = "linux"))]
        {
            Ok(())
        }
    }

    fn restart_network_services(&self) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        {
            self.execute_system_command("systemctl", &["restart", "network.service"])
        }
        #[cfg(not(target_os = "linux"))]
        {
            Ok(())
        }
    }

    fn clean_cache(&self) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        {
            self.execute_system_command("systemd-resolve", &["--flush-caches"])
        }
        #[cfg(not(target_os = "linux"))]
        {
            Ok(())
        }
    }

    pub fn get_current_optimizations(&self) -> &Vec<String> {
        &self.current_optimizations
    }

    pub fn get_failed_optimizations(&self) -> &Vec<String> {
        &self.failed_optimizations
    }
}
