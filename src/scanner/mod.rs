mod macos;

use crate::types::PortEntry;
use anyhow::Result;

pub use macos::MacOsScanner;

pub trait PortScanner: Send {
    fn scan(&self) -> Result<Vec<PortEntry>>;
}

pub fn create_scanner() -> Box<dyn PortScanner> {
    Box::new(MacOsScanner::new())
}
