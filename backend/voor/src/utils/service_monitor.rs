use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;

const MAX_LOG_ENTRIES: usize = 250;

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn as_str(self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub timestamp_ms: u128,
    pub service: String,
    pub level: String,
    pub event: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceStatus {
    pub service: String,
    pub health: String,
    pub status: String,
    pub last_message: String,
    pub updated_at_ms: u128,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthReport {
    pub overall_status: String,
    pub uptime_ms: u128,
    pub services: Vec<ServiceStatus>,
    pub recent_logs: Vec<LogEntry>,
}

struct MonitorState {
    started_at: SystemTime,
    services: HashMap<String, ServiceStatus>,
    logs: VecDeque<LogEntry>,
}

#[derive(Clone)]
pub struct ServiceMonitor {
    state: Arc<Mutex<MonitorState>>,
}

impl ServiceMonitor {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MonitorState {
                started_at: SystemTime::now(),
                services: HashMap::new(),
                logs: VecDeque::with_capacity(MAX_LOG_ENTRIES),
            })),
        }
    }

    pub fn register_service(&self, service: &str, health: &str, status: &str, message: &str) {
        self.update_service(service, health, status, message);
    }

    pub fn update_service(&self, service: &str, health: &str, status: &str, message: &str) {
        let now = timestamp_ms();
        {
            let mut state = self.state.lock().expect("service monitor poisoned");
            state.services.insert(
                service.to_string(),
                ServiceStatus {
                    service: service.to_string(),
                    health: health.to_string(),
                    status: status.to_string(),
                    last_message: message.to_string(),
                    updated_at_ms: now,
                },
            );
        }

        self.log(LogLevel::Info, service, "status-update", message);
    }

    pub fn log(&self, level: LogLevel, service: &str, event: &str, message: &str) {
        let entry = LogEntry {
            timestamp_ms: timestamp_ms(),
            service: service.to_string(),
            level: level.as_str().to_string(),
            event: event.to_string(),
            message: message.to_string(),
        };

        println!(
            "[{}][{}][{}] {}",
            entry.level, entry.service, entry.event, entry.message
        );

        let mut state = self.state.lock().expect("service monitor poisoned");
        if state.logs.len() == MAX_LOG_ENTRIES {
            state.logs.pop_front();
        }
        state.logs.push_back(entry);
    }

    pub fn health_report(&self) -> HealthReport {
        let state = self.state.lock().expect("service monitor poisoned");
        let uptime_ms = state
            .started_at
            .elapsed()
            .unwrap_or(Duration::from_secs(0))
            .as_millis();

        let mut services: Vec<ServiceStatus> = state.services.values().cloned().collect();
        services.sort_by(|left, right| left.service.cmp(&right.service));

        let overall_status = if services.iter().any(|service| service.health == "down") {
            "degraded"
        } else if services.iter().any(|service| service.health == "warning") {
            "warning"
        } else {
            "healthy"
        };

        let recent_logs = state.logs.iter().rev().take(25).cloned().collect();

        HealthReport {
            overall_status: overall_status.to_string(),
            uptime_ms,
            services,
            recent_logs,
        }
    }
}

fn timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_millis()
}
