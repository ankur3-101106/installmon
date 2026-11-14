//! InstallMon Core - Normalized event model, provenance, argument sanitization,
//! and platform capability interfaces.

pub mod capabilities;
pub mod error;
pub mod event;
pub mod provenance;
pub mod sanitize;

pub use capabilities::*;
pub use error::{CoreError, CoreResult};
pub use event::*;
pub use provenance::*;
pub use sanitize::*;
