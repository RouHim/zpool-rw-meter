//! System interface abstractions for testing and development

pub mod commands;
pub mod filesystem;

// Re-export commonly used traits
pub use commands::CommandExecutor;
pub use filesystem::FilesystemReader;
