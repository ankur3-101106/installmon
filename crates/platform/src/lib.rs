//! InstallMon Platform - Runtime environment and capability detection for Linux systems.

pub mod detector;
pub mod paths;
pub mod service;

pub use detector::detect_platform_capabilities;
pub use paths::*;
pub use service::*;
