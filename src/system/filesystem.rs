use std::error::Error;

/// Abstraction for filesystem access to enable testing without real files
pub trait FilesystemReader {
    fn read_to_string(&self, path: &str) -> Result<String, Box<dyn Error>>;
    fn exists(&self, path: &str) -> bool;
}

/// Real filesystem reader using std::fs
pub struct RealFilesystemReader;

impl FilesystemReader for RealFilesystemReader {
    fn read_to_string(&self, path: &str) -> Result<String, Box<dyn Error>> {
        Ok(std::fs::read_to_string(path)?)
    }

    fn exists(&self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }
}

/// Demo filesystem reader that returns predefined file contents
pub struct DemoFilesystemReader;

impl DemoFilesystemReader {
    fn get_demo_content(&self, path: &str) -> Option<&'static str> {
        match path {
            "/proc/spl/kstat/zfs/arcstats" => Some(include_str!("../demo/arcstats.txt")),
            _ => None,
        }
    }
}

impl FilesystemReader for DemoFilesystemReader {
    fn read_to_string(&self, path: &str) -> Result<String, Box<dyn Error>> {
        if let Some(content) = self.get_demo_content(path) {
            Ok(content.to_string())
        } else {
            Err(format!("Demo: File not mocked: {}", path).into())
        }
    }

    fn exists(&self, path: &str) -> bool {
        self.get_demo_content(path).is_some()
    }
}
