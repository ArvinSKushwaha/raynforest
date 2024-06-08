use std::sync::Arc;

pub struct Context {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Context {
    pub fn new() -> Arc<Self> {
        smol::block_on(
            Self::builder()
                .adapter(Default::default())
                .device(Default::default())
                .build(),
        )
    }

    pub fn builder<'a, 'b>() -> ContextBuilder<'a, 'b> {
        ContextBuilder {
            adapter_options: None,
            device_options: None,
        }
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn command_encoder(&self) -> wgpu::CommandEncoder {
        self.device.create_command_encoder(&Default::default())
    }
}

pub struct ContextBuilder<'a, 'b> {
    adapter_options: Option<wgpu::RequestAdapterOptions<'a, 'b>>,
    device_options: Option<wgpu::DeviceDescriptor<'a>>,
}

impl<'a, 'b> ContextBuilder<'a, 'b> {
    pub fn adapter(mut self, options: wgpu::RequestAdapterOptions<'a, 'b>) -> Self {
        self.adapter_options.replace(options);
        self
    }

    pub fn device(mut self, options: wgpu::DeviceDescriptor<'a>) -> Self {
        self.device_options.replace(options);
        self
    }

    pub async fn build(self) -> Arc<Context> {
        let request_device = async {
            let instance = wgpu::Instance::new(Default::default());

            let adapter = instance
                .request_adapter(&self.adapter_options.unwrap_or_default())
                .await?;

            adapter.request_device(&Default::default(), None).await.ok()
        };

        let (device, queue) = match request_device.await {
            Some((device, queue)) => (device, queue),
            None => {
                log::error!("Could not construct device and queue");
                std::process::exit(1);
            }
        };

        Context { device, queue }.into()
    }
}
