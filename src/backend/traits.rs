use bytemuck::{Pod, Zeroable};

pub trait BufferType: Zeroable + Pod + 'static {}

impl BufferType for u8 {}
impl BufferType for u16 {}
impl BufferType for u32 {}
impl BufferType for u64 {}

impl BufferType for i8 {}
impl BufferType for i16 {}
impl BufferType for i32 {}
impl BufferType for i64 {}

impl BufferType for f32 {}
impl BufferType for f64 {}
