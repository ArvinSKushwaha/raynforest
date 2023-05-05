use std::num::{NonZeroU32, NonZeroU64};

use paste::paste;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ShaderStages {
    pub vertex: bool,
    pub fragment: bool,
    pub compute: bool,
}

impl From<ShaderStages> for wgpu::ShaderStages {
    fn from(value: ShaderStages) -> Self {
        let vtx = Self::VERTEX.bits();
        let frg = Self::FRAGMENT.bits();
        let cmp = Self::COMPUTE.bits();
        let bits = Self::from_bits(
            (vtx * (value.vertex as u32))
                | (frg * (value.fragment as u32))
                | (cmp * (value.compute as u32)),
        );

        debug_assert!(bits.is_some());
        bits.unwrap_or(wgpu::ShaderStages::NONE)
    }
}

macro_rules! impl_serdeable_mirror {
    (
        $(#[$m:meta])*
        $v:vis enum $T:ident {
            $($t:ident $({ $($p:ident : $pt:ty),+ $(,)? })? $(($($qt:ty),+ $(,)?))?),+ $(,)?
        }
    ) => {
        $(#[$m])*
        $v enum $T {
            $($t $({ $($p : $pt),+ })? $(($($qt),+))?),+
        }

        paste! {
            impl From<$T> for wgpu::$T {
                fn from(value: $T) -> Self {
                    use $T::*;
                    match value {
                        $($t $({ $($p), + })? $(($([< $qt:snake >]),+))? => wgpu::$T::$t $({ $($p: $p.into()), +})? $(($([< $qt:snake >].into()),+))?),+
                    }
                }
            }
        }
    };
    (
        $(#[$m:meta])*
        $v:vis struct $T:ident {
            $($z:vis $p:ident : $pt:ty),+ $(,)?
        }
    ) => {
        $(#[$m])*
        $v struct $T {
            $($z $p: $pt),+
        }

        impl From<$T> for wgpu::$T {
            fn from(value: $T) -> Self {
                let $T { $($p),+ } = value;
                Self { $($p: $p.into()),+ }
            }
        }
    };
    (
        $(#[$m:meta])*
        $v:vis struct $T:ident($($z:vis $pt:ty),+ $(,)?);
    ) => {
        $(#[$m])*
        $v struct $T($($z $pt),+);

        paste! {
            impl From<$T> for wgpu::$T {
                fn from(value: $T) -> Self {
                    let $T ( $([< $p:snake >]),+ ) = value;
                    Self($([< $p:snake >].into()),+)
                }
            }
        }
    };
}

impl_serdeable_mirror! {
    #[derive(Debug, Clone, Copy, Deserialize)]
    pub enum TextureSampleType {
        Sint,
        Uint,
        Depth,
        Float { filterable: bool },
    }
}

impl_serdeable_mirror! {
    #[derive(Debug, Clone, Copy, Deserialize)]
    pub enum SamplerBindingType {
        Filtering,
        NonFiltering,
        Comparison,
    }
}

impl_serdeable_mirror! {
#[derive(Debug, Clone, Copy, Deserialize)]
pub enum BufferBindingType {
    Storage { read_only: bool },
    Uniform,
}
}

impl_serdeable_mirror! {
#[derive(Clone, Copy, Debug, Deserialize)]
pub enum TextureViewDimension {
    D1,
    D2,
    D3,
    D2Array,
    Cube,
    CubeArray,
}
}

impl_serdeable_mirror! {
#[derive(Clone, Copy, Debug, Deserialize)]
pub enum StorageTextureAccess {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}
}

impl_serdeable_mirror! {
#[derive(Clone, Copy, Debug, Deserialize)]
pub enum TextureFormat {
    R8Unorm,
    R8Snorm,
    R8Uint,
    R8Sint,
    R16Uint,
    R16Sint,
    R16Unorm,
    R16Snorm,
    R16Float,
    Rg8Unorm,
    Rg8Snorm,
    Rg8Uint,
    Rg8Sint,
    R32Uint,
    R32Sint,
    R32Float,
    Rg16Uint,
    Rg16Sint,
    Rg16Unorm,
    Rg16Snorm,
    Rg16Float,
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Rgba8Snorm,
    Rgba8Uint,
    Rgba8Sint,
    Bgra8Unorm,
    Bgra8UnormSrgb,
    Rgb9e5Ufloat,
    Rgb10a2Unorm,
    Rg11b10Float,
    Rg32Uint,
    Rg32Sint,
    Rg32Float,
    Rgba16Uint,
    Rgba16Sint,
    Rgba16Unorm,
    Rgba16Snorm,
    Rgba16Float,
    Rgba32Uint,
    Rgba32Sint,
    Rgba32Float,
    Stencil8,
    Depth16Unorm,
    Depth24Plus,
    Depth24PlusStencil8,
    Depth32Float,
    Depth32FloatStencil8,
    Bc1RgbaUnorm,
    Bc1RgbaUnormSrgb,
    Bc2RgbaUnorm,
    Bc2RgbaUnormSrgb,
    Bc3RgbaUnorm,
    Bc3RgbaUnormSrgb,
    Bc4RUnorm,
    Bc4RSnorm,
    Bc5RgUnorm,
    Bc5RgSnorm,
    Bc6hRgbUfloat,
    Bc6hRgbSfloat,
    Bc7RgbaUnorm,
    Bc7RgbaUnormSrgb,
    Etc2Rgb8Unorm,
    Etc2Rgb8UnormSrgb,
    Etc2Rgb8A1Unorm,
    Etc2Rgb8A1UnormSrgb,
    Etc2Rgba8Unorm,
    Etc2Rgba8UnormSrgb,
    EacR11Unorm,
    EacR11Snorm,
    EacRg11Unorm,
    EacRg11Snorm,
}
}

impl_serdeable_mirror! {
#[derive(Debug, Clone, Deserialize)]
pub enum BindingType {
    Buffer {
        ty: BufferBindingType,
        has_dynamic_offset: bool,
        min_binding_size: Option<NonZeroU64>,
    },
    Sampler(SamplerBindingType),
    Texture {
        sample_type: TextureSampleType,
        view_dimension: TextureViewDimension,
        multisampled: bool,
    },
    StorageTexture {
        access: StorageTextureAccess,
        format: TextureFormat,
        view_dimension: TextureViewDimension,
    },
}
}

impl_serdeable_mirror! {
    #[derive(Clone, Debug, Deserialize)]
    pub struct BindGroupLayoutEntry {
        pub binding: u32,
        pub visibility: ShaderStages,
        pub ty: BindingType,
        pub count: Option<NonZeroU32>,
    }
}
