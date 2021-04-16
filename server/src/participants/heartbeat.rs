use std::time::Duration;

pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);
pub const CLIENT_TIMEOUT: Duration = Duration::from_secs(11);
pub const CLIENT_TERMINATE: Duration = Duration::from_secs(30);