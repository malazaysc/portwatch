#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;

use crate::types::PortEntry;
use anyhow::Result;

pub trait PortScanner: Send {
    fn scan(&self) -> Result<Vec<PortEntry>>;
}

pub fn create_scanner() -> Box<dyn PortScanner> {
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacOsScanner::new())
    }
    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxScanner::new())
    }
}
