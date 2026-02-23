#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Message {
    /// Server process has been spawned; begin health polling.
    ServerSpawned,
    /// Result of a health check poll.
    HealthCheck(bool),
    /// Server is ready (voice cloner loaded).
    ServerReady,
    /// Server failed to start or crashed.
    ServerError(String),
    /// Elapsed-time tick while loading (every 1 second).
    Tick,
}
