use std::fmt;

/// Errors that can occur during ZFS statistics collection and parsing
#[derive(Debug)]
pub enum ZfsError {
    /// Command execution failed
    CommandError {
        command: String,
        args: Vec<String>,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// File system operation failed
    FilesystemError {
        path: String,
        operation: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Parsing failed for a specific data source
    ParseError {
        data_source: String,
        reason: String,
    },

    /// Invalid or unexpected data format
    InvalidFormat {
        expected: String,
        received: String,
        context: String,
    },

    /// Required ZFS subsystem not available
    SubsystemUnavailable { subsystem: String, reason: String },
}

impl fmt::Display for ZfsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZfsError::CommandError { command, args, .. } => {
                write!(f, "Command failed: {} {:?}", command, args)
            }
            ZfsError::FilesystemError {
                path, operation, ..
            } => {
                write!(f, "Filesystem {} failed for path: {}", operation, path)
            }
            ZfsError::ParseError {
                data_source,
                reason,
            } => {
                write!(f, "Failed to parse {}: {}", data_source, reason)
            }
            ZfsError::InvalidFormat {
                expected,
                received,
                context,
            } => {
                write!(
                    f,
                    "Invalid format in {}: expected {}, received '{}'",
                    context, expected, received
                )
            }
            ZfsError::SubsystemUnavailable { subsystem, reason } => {
                write!(f, "{} subsystem unavailable: {}", subsystem, reason)
            }
        }
    }
}

impl std::error::Error for ZfsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ZfsError::CommandError { source, .. } => Some(source.as_ref()),
            ZfsError::FilesystemError { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl ZfsError {
    /// Create a command error
    pub fn command_error(command: &str, args: &[&str], message: &str) -> Self {
        ZfsError::CommandError {
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            source: Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                message.to_string(),
            )),
        }
    }

    /// Create a filesystem error
    pub fn filesystem_error(path: &str, operation: &str, message: &str) -> Self {
        ZfsError::FilesystemError {
            path: path.to_string(),
            operation: operation.to_string(),
            source: Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                message.to_string(),
            )),
        }
    }

    /// Create a parse error
    pub fn parse_error(data_source: &str, reason: &str) -> Self {
        ZfsError::ParseError {
            data_source: data_source.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Create an invalid format error
    pub fn invalid_format(expected: &str, received: &str, context: &str) -> Self {
        ZfsError::InvalidFormat {
            expected: expected.to_string(),
            received: received.to_string(),
            context: context.to_string(),
        }
    }

    /// Create a subsystem unavailable error
    pub fn subsystem_unavailable(subsystem: &str, reason: &str) -> Self {
        ZfsError::SubsystemUnavailable {
            subsystem: subsystem.to_string(),
            reason: reason.to_string(),
        }
    }


}

/// Result type alias for ZFS operations
pub type ZfsResult<T> = Result<T, ZfsError>;
