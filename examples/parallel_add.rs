use arraynforest::context::ComputeContext;

static PARALLEL_ADD_SHADER: &'static str = include_str!("parallel_add.wgsl");

fn main() {
    let context = ComputeContext::new();

    let buffer_handler = context
        .build_buffer_handler()
        .shader("parallel_add", PARALLEL_ADD_SHADER)
        .compute_pipeline("parallel_add", "add")
        .build();

    let command_buffer = buffer_handler.compute_pass(|buffer_handler, mut compute_pass| {
        let pipeline = buffer_handler.pipeline("add@parallel_add").unwrap();
        let bind_group_layout =
            context
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Bind group layout descriptor"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });
        let bind_group = context
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Bind Group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(&first_buffer),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(&second_buffer),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer(&mut return_buffer),
                    },
                ],
            });
        compute_pass.set_pipeline(&pipeline);
    });
}
