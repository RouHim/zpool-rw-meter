//! System interface abstractions for testing and development

pub mod cache;
pub mod commands;
pub mod filesystem;

// Re-export commonly used traits
pub use cache::Cache;
pub use commands::CommandExecutor;
pub use filesystem::FilesystemReader;
