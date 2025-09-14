use crate::system::CommandExecutor;
use std::error::Error;

/// Pool detection and validation
pub struct PoolManager<E: CommandExecutor> {
    command_executor: E,
}

impl<E: CommandExecutor> PoolManager<E> {
    pub fn new(command_executor: E) -> Self {
        Self { command_executor }
    }

    /// Get list of available pools
    pub fn list_pools(&self) -> Result<Vec<String>, Box<dyn Error>> {
        // TODO: Implement pool listing via `zpool list -H -o name`
        // For now, return demo data
        Ok(vec![
            "boot-pool".to_string(),
            "data".to_string(),
            "usb-backup".to_string(),
        ])
    }

    /// Validate that a pool exists
    pub fn validate_pool(&self, pool_name: &str) -> Result<bool, Box<dyn Error>> {
        let pools = self.list_pools()?;
        Ok(pools.contains(&pool_name.to_string()))
    }

    /// Get default pool (first available pool)
    pub fn get_default_pool(&self) -> Result<String, Box<dyn Error>> {
        let pools = self.list_pools()?;
        pools
            .into_iter()
            .next()
            .ok_or_else(|| "No pools found".into())
    }
}
