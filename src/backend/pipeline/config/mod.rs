use std::io::Read;

use self::inner_types::BindGroupLayoutEntry;
use serde::Deserialize;

mod inner_types;

#[derive(Debug, thiserror::Error)]
pub enum PipelineLoadingError {
    #[error("IOError: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Failed to Deserialize: {0}")]
    DeserializeError(#[from] deser_hjson::Error),
}

#[derive(Debug, Clone)]
pub struct PipelineConfiguration {
    layout_entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl<'a> Deserialize<'a> for PipelineConfiguration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let layout_entries = Vec::<BindGroupLayoutEntry>::deserialize(deserializer)?
            .into_iter()
            .map(Into::<wgpu::BindGroupLayoutEntry>::into)
            .collect();

        Ok(Self { layout_entries })
    }
}

impl PipelineConfiguration {
    pub fn load(config_file: &str) -> Result<Self, PipelineLoadingError> {
        let mut buf = String::new();
        std::fs::File::open(config_file)?.read_to_string(&mut buf)?;
        let config = deser_hjson::from_str(&buf)?;

        Ok(config)
    }

    pub fn entries(&self) -> &[wgpu::BindGroupLayoutEntry] {
        &self.layout_entries
    }
}
