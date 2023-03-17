// TODO: Remove this, temporary to reduce visual noise.
use once_cell::sync::Lazy;
use smol::block_on;
use wgpu::{Backends, Device, Instance, Queue};

use crate::buffers::BufferHandlerBuildInit;

macro_rules! panic_and_log {
    ($a:expr, $($b:expr),+) => {
        {
            log::log!($a, $($b),+);
            panic!($($b),+);
        }
    };
}

pub struct ComputeContext {
    device: Device,
    queue: Queue,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateComputeContextError {
    #[error("Could not find a suitable adapter.")]
    NoAdapterFound,

    #[error("Could not find a suitable surface format.")]
    NoSuitableSurfaceFormat,

    #[error("Could not receive Window object: {0}")]
    RecvError(#[from] flume::RecvError),

    #[error("Could not create a surface: {0}")]
    CreateSurfaceError(#[from] wgpu::CreateSurfaceError),

    #[error("Could not request device: {0}")]
    RequestDeviceError(#[from] wgpu::RequestDeviceError),
}

impl ComputeContext {
    pub fn new() -> Self {
        block_on(async {
            match ComputeContext::async_new().await {
                Ok(compute_context) => compute_context,
                Err(err) => panic_and_log!(
                    log::Level::Error,
                    "Could not instantiate compute context: {}",
                    err
                ),
            }
        })
    }

    async fn async_new() -> Result<Self, CreateComputeContextError> {
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false, // TODO: Perhaps this needs to be corrected?
                compatible_surface: None,
            })
            .await
            .ok_or_else(|| CreateComputeContextError::NoAdapterFound)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Arraynforest Device"),
                    ..Default::default()
                },
                None,
            )
            .await?;

        Ok(Self { device, queue })
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn build_buffer_handler<'a>(&'a self) -> BufferHandlerBuildInit {
        BufferHandlerBuildInit::<'a>::new(&self.device)
    }
}
