use super::device::Context;

pub mod config;

pub struct ComputePipeline {
    compute_pipeline: Option<wgpu::ComputePipeline>,
    layout: Option<wgpu::PipelineLayout>,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum PipelineExecutionError {
    #[error("The pipeline has not yet been initialized. Please run `ComputePipeline::pipeline`")]
    UninitializedPipeline,
}

impl ComputePipeline {
    pub fn construct() -> Self {
        Self {
            compute_pipeline: None,
            layout: None,
        }
    }

    pub fn pipeline_layout(
        &mut self,
        context: &Context,
        layout_desc: &wgpu::PipelineLayoutDescriptor,
    ) {
        self.layout
            .replace(context.device().create_pipeline_layout(layout_desc));
    }

    pub fn pipeline(&mut self, context: &Context, desc: &wgpu::ComputePipelineDescriptor) {
        self.compute_pipeline
            .replace(context.device().create_compute_pipeline(desc));
    }

    pub fn get_pipeline(&mut self) -> Option<wgpu::ComputePipeline> {
        self.compute_pipeline.take()
    }

    pub fn get_layout(&mut self) -> Option<wgpu::PipelineLayout> {
        self.layout.take()
    }
}

#[cfg(test)]
mod tests {
    use crate::backend::{buffers::Buffer, device::Context};

    use super::{config::PipelineConfiguration, ComputePipeline};
    type Use = wgpu::BufferUsages;

    #[test]
    fn test_add_shader() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .init();

        let context = Context::new();

        let add_module = context
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                    "../../shaders/add.wgsl"
                ))),
            });

        let config =
            PipelineConfiguration::load("src/shaders/add.hjson").expect("Failed to load add.hjson");
        let mut pipeline = ComputePipeline::construct();

        let bind_group_layout =
            context
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: config.entries(),
                });

        pipeline.pipeline_layout(
            &context,
            &wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            },
        );

        let layout = pipeline.get_layout();
        let layout = layout.as_ref().expect("Could not get layout. Not initialized?");

        let add_desc = wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&layout),
            module: &add_module,
            entry_point: "add",
        };

        pipeline.pipeline(&context, &add_desc);

        let pipeline = pipeline.get_pipeline();
        let pipeline = pipeline.expect("Could not get pipeline. Not initialized?");

        let x_vec = vec![f32::from_bits(2), f32::from_bits(2), 1., 2., 3., 4.];
        let y_vec = vec![f32::from_bits(2), f32::from_bits(2), 3., 1., 6., 2.];

        let x = Buffer::from_vec(&context, Use::STORAGE, x_vec);
        let y = Buffer::from_vec(&context, Use::STORAGE, y_vec);

        let z = Buffer::<f32>::new(&context, Use::STORAGE | Use::COPY_SRC, x.size());
        let mut w = Buffer::<f32>::new(&context, Use::MAP_READ | Use::COPY_DST, z.size());

        let bind_group = context
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: x.get_resource(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: y.get_resource(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: z.get_resource(),
                    },
                ],
            });

        let mut encoder = context.command_encoder();
        let mut compute_pass = encoder.begin_compute_pass(&Default::default());

        compute_pass.set_pipeline(&pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(1, 1, 1);

        drop(compute_pass);
        let command_buf = encoder.finish();
        context.queue().submit([command_buf]);

        z.copy_to(&context, .., &mut w, ..);

        assert_eq!(w.slice(..).map(&context).to_vec(), vec![f32::from_bits(2), f32::from_bits(2), 4., 3., 9., 6.]);
    }
}
