use std::time::Duration;

pub const DEFAULT_TIMEOUT_MINUTES: u64 = 30;
pub const DEFAULT_SLEEP_BETWEEN_REPOS_SECS: u64 = 60;
pub const DEFAULT_SERVER_PORT_RANGE_START: u16 = 8080;
pub const DEFAULT_SERVER_PORT_RANGE_END: u16 = 8200;
pub const DEFAULT_DASHBOARD_PORT: u16 = 8080;
pub const DEFAULT_HISTORY_DAYS: u32 = 7;
pub const MAX_SESSION_ID_LENGTH: usize = 64;
pub const SERVER_SHUTDOWN_GRACE_PERIOD_MS: u64 = 100;
pub const SESSION_CLEANUP_POLL_INTERVAL_MS: u64 = 500;

pub const ANTHROPIC_API_KEY_ENV: &str = "ANTHROPIC_API_KEY";

pub const SUPPORTED_FILE_EXTENSIONS: &[(&str, &str)] = &[
    ("rs", "rust"),
    ("js", "javascript"),
    ("jsx", "javascript"),
    ("ts", "typescript"),
    ("tsx", "typescript"),
    ("py", "python"),
    ("java", "java"),
    ("cpp", "cpp"),
    ("cc", "cpp"),
    ("cxx", "cpp"),
    ("c", "c"),
    ("h", "c"),
    ("hpp", "c"),
    ("go", "go"),
    ("php", "php"),
    ("rb", "ruby"),
    ("html", "html"),
    ("css", "css"),
    ("json", "json"),
    ("xml", "xml"),
    ("yaml", "yaml"),
    ("yml", "yaml"),
    ("toml", "toml"),
    ("md", "markdown"),
];

pub const DEFAULT_FILE_TYPE: &str = "text";

pub fn timeout_duration(minutes: u64) -> Duration {
    Duration::from_secs(minutes * 60)
}

pub fn sleep_duration_secs(seconds: u64) -> Duration {
    Duration::from_secs(seconds)
}

pub fn sleep_duration_millis(milliseconds: u64) -> Duration {
    Duration::from_millis(milliseconds)
}