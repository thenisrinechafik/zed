use std::path::Path;

use anyhow::Result;

#[derive(Debug)]
pub struct StagedUpdate<'a> {
    pub package: &'a Path,
}

#[derive(Debug)]
pub struct RollbackPoint<'a> {
    pub previous: &'a Path,
}

pub trait PlatformUpdate {
    fn stage_update(&self, pkg: &Path) -> Result<StagedUpdate<'_>>;
    fn apply_update(&self, staged: StagedUpdate<'_>) -> Result<()>;
    fn rollback(&self, previous: RollbackPoint<'_>) -> Result<()>;
}
