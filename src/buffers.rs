use std::{borrow::Cow, collections::HashMap, marker::PhantomData};

use wgpu::{
    CommandBuffer, CommandEncoder, ComputePass, ComputePipeline, Device, ShaderModule, ShaderSource,
};

pub trait BufferHandlerBuilderState {}
pub struct PermitShaders;
pub struct CreatedPipeline;
impl BufferHandlerBuilderState for PermitShaders {}
impl BufferHandlerBuilderState for CreatedPipeline {}

pub type BufferHandlerBuildInit<'a> = BufferHandlerBuilder<'a, PermitShaders>;

pub struct BufferHandlerBuilder<'a, S: BufferHandlerBuilderState>
where
    Self: 'a,
{
    shaders: HashMap<&'a str, ShaderModule>,
    device: &'a Device,
    shader_entrypoints: HashMap<&'a str, Vec<&'a str>>, // TODO: Replace with SmallVec in the future.
    _phantom: PhantomData<S>,
}

impl<'a> BufferHandlerBuilder<'a, PermitShaders> {
    pub(crate) fn new(device: &'a Device) -> BufferHandlerBuilder<'a, PermitShaders> {
        BufferHandlerBuilder::<'a, PermitShaders> {
            device,
            shaders: HashMap::new(),
            shader_entrypoints: HashMap::new(),
            _phantom: PhantomData,
        }
    }

    pub fn shader(mut self, shader_label: &'a str, shader_source: &'a str) -> Self {
        let shader_module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(shader_label),
                source: ShaderSource::Wgsl(Cow::from(shader_source)),
            });
        self.shaders.insert(shader_label, shader_module);
        self
    }

    pub fn compute_pipeline(
        mut self,
        label: &'static str,
        entry_point: &'static str,
    ) -> BufferHandlerBuilder<'a, CreatedPipeline> {
        self.shader_entrypoints
            .entry(label)
            .or_insert(vec![])
            .push(entry_point);

        BufferHandlerBuilder::<'a, CreatedPipeline> {
            shaders: self.shaders,
            device: self.device,
            shader_entrypoints: self.shader_entrypoints,
            _phantom: PhantomData,
        }
    }
}

impl<'a> BufferHandlerBuilder<'a, CreatedPipeline> {
    pub fn compute_pipeline(
        mut self,
        label: &'static str,
        entry_point: &'static str,
    ) -> BufferHandlerBuilder<'a, CreatedPipeline> {
        self.shader_entrypoints
            .entry(label)
            .or_insert(vec![])
            .push(entry_point);

        BufferHandlerBuilder::<'a, CreatedPipeline> {
            shaders: self.shaders,
            device: self.device,
            shader_entrypoints: self.shader_entrypoints,
            _phantom: PhantomData,
        }
    }

    pub fn build(self) -> BufferHandler<'a> {
        let Self {
            shaders,
            device,
            shader_entrypoints,
            _phantom,
        } = self;

        let mut compute_pipelines = HashMap::new();
        shader_entrypoints
            .into_iter()
            .flat_map(|(key, vec)| vec.into_iter().zip(std::iter::repeat(key)))
            .map(|(entry_point, shader_name)| {
                let entry = shaders.get(shader_name).and_then(|module| {
                    let label = &*Box::leak(format!("{}@{}", entry_point, shader_name).into_boxed_str()); // HACK: Gotta do something about this...
                    Some((
                        label,
                        self.device
                            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                                label: Some(&label),
                                layout: None,
                                entry_point,
                                module,
                            }),
                    ))
                });

                if entry.is_none() {
                    log::log!(
                        log::Level::Warn,
                        "Could not find shader with name {}",
                        shader_name
                    );
                }

                entry
            })
            .for_each(|entries| match entries {
                Some((label, compute_pipeline)) => {
                    // HACK: There has to be a way to get the lifetimes to work out without
                    // leaking the pipelines...
                    compute_pipelines.insert(label, &*Box::leak(Box::new(compute_pipeline)));
                }
                _ => {}
            });

        BufferHandler {
            shaders,
            device,
            compute_pipelines,
        }
    }
}

pub struct BufferHandler<'a> {
    shaders: HashMap<&'a str, ShaderModule>,
    device: &'a Device,
    compute_pipelines: HashMap<&'a str, &'static ComputePipeline>,
}

impl<'a> BufferHandler<'a> {
    fn encoder(&self) -> CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None })
    }

    pub fn compute_pass(&self, f: impl FnOnce(&Self, ComputePass) -> ()) -> CommandBuffer {
        let mut encoder = self.encoder();
        let compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Arraynforest Compute Pass"),
        });

        f(self, compute_pass);

        encoder.finish()
    }

    pub fn pipeline(&self, label: &str) -> Option<&'static ComputePipeline> {
        self.compute_pipelines.get(label).copied()
    }
}
