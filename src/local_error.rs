//! Holds our local Error and Result types.

use std::error::Error;

pub type LocalError = Box<dyn Error + Send + Sync>;
pub type Result<T> = core::result::Result<T, LocalError>;
