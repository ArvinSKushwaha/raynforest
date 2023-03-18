use bytemuck::{Zeroable, Pod};

pub trait BufferType: Zeroable + Pod + 'static {
}

impl BufferType for u8 {}
impl BufferType for f32 {}
impl BufferType for f64 {}
