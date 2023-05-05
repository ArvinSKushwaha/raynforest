pub struct Context {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Context {
    pub fn new() -> Self {
        let request_device = async {
            let instance = wgpu::Instance::new(Default::default());

            let adapter = instance.request_adapter(&wgpu::RequestAdapterOptionsBase{
                // power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            }).await?;

            adapter.request_device(&Default::default(), None).await.ok()
        };

        let (device, queue) = match smol::block_on(request_device) {
            Some((device, queue)) => (device, queue),
            None => {
                log::error!("Could not construct device and queue");
                std::process::exit(1);
            }
        };

        Self { device, queue }
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn command_encoder(&self) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&Default::default())
    }
}
