use bytemuck::{try_cast_slice, PodCastError};
use std::{marker::PhantomData, ops::RangeBounds};
use wgpu::util::DeviceExt;
use wgpu::BufferDescriptor;

use wgpu::util::BufferInitDescriptor;

use super::util::materialize;
use super::{device::Context, traits::BufferType};

mod err;
mod slice;
mod view;

pub use self::{err::BufferCopyError, slice::BufferSlice};

#[derive(Debug)]
pub struct Buffer<T: BufferType> {
    buffer: wgpu::Buffer,
    usage: wgpu::BufferUsages,
    _phantom: PhantomData<T>,
}

impl<T: BufferType> Buffer<T> {
    pub fn new(context: &Context, usage: wgpu::BufferUsages, size: u64) -> Buffer<T> {
        let buffer = context.device().create_buffer(&BufferDescriptor {
            label: None,
            size,
            usage,
            mapped_at_creation: false,
        });

        Buffer {
            buffer,
            usage,
            _phantom: PhantomData,
        }
    }

    pub fn from_vec(context: &Context, usage: wgpu::BufferUsages, vec: Vec<T>) -> Buffer<T> {
        let buffer = context.device().create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::checked::cast_slice(&vec),
            usage,
        });

        Buffer {
            buffer,
            usage,
            _phantom: PhantomData,
        }
    }

    pub fn size(&self) -> u64 {
        self.buffer.size()
    }

    pub fn slice(&self, range: impl RangeBounds<u64>) -> BufferSlice<T> {
        BufferSlice {
            buffer: self,
            slice: self.buffer.slice(range),
            _phantom: PhantomData,
        }
    }

    pub fn copy_to<A, B>(
        &self,
        context: &Context,
        src_range: A,
        buffer: &mut Buffer<T>,
        dst_range: B,
    ) where
        A: RangeBounds<usize> + std::fmt::Debug + std::marker::Copy,
        B: RangeBounds<usize> + std::fmt::Debug + std::marker::Copy,
    {
        let copy_to_result = self.try_copy_to(context, src_range, buffer, dst_range);
        if let Err(e) = &copy_to_result {
            log::error!("Failed at Buffer::copy_to: {:?}", e);
        }

        copy_to_result.unwrap();
    }

    pub fn try_copy_to<A, B>(
        &self,
        context: &Context,
        src_range: A,
        buffer: &mut Buffer<T>,
        dst_range: B,
    ) -> Result<(), BufferCopyError<A, B>>
    where
        A: RangeBounds<usize> + std::marker::Copy,
        B: RangeBounds<usize> + std::marker::Copy,
    {
        if !self.usage.contains(wgpu::BufferUsages::COPY_SRC) {
            return Err(BufferCopyError::InvalidSourceBuffer(self.usage));
        }

        if !buffer.usage.contains(wgpu::BufferUsages::COPY_DST) {
            return Err(BufferCopyError::InvalidDestinationBuffer(self.usage));
        }

        let src_bound = materialize(src_range, self);
        let dst_bound = materialize(dst_range, self);

        if src_bound.len() != dst_bound.len() {
            return Err(BufferCopyError::UnequalReferenceLengths(
                src_range, dst_range,
            ));
        }

        let mut command_encoder = context.command_encoder();
        command_encoder.copy_buffer_to_buffer(
            &self.buffer,
            src_bound.start() as u64,
            &buffer.buffer,
            dst_bound.start() as u64,
            src_bound.len() as u64,
        );
        let command_buffer = command_encoder.finish();

        let idx = context.queue().submit([command_buffer]);
        context
            .device()
            .poll(wgpu::MaintainBase::WaitForSubmissionIndex(idx));

        Ok(())
    }

    pub fn try_queue_buffer_write(
        &self,
        context: &Context,
        position: u64,
        data: &[T],
    ) -> Result<(), PodCastError> {
        context
            .queue()
            .write_buffer(&self.buffer, position, try_cast_slice(data)?);
        Ok(())
    }

    pub fn queue_buffer_write(&self, context: &Context, position: u64, data: &[T]) {
        let queue_buffer_write_result = self.try_queue_buffer_write(context, position, data);

        if let Err(e) = &queue_buffer_write_result {
            log::error!("Failed to queue buffer write result: {}", e);
        }
        queue_buffer_write_result.unwrap();
    }

    pub fn unmap(&mut self) {
        self.buffer.unmap();
    }

    pub fn get_resource(&self) -> wgpu::BindingResource {
        self.buffer.as_entire_binding()
    }
}
#[cfg(test)]
mod tests {
    use crate::backend::{buffers::Buffer, device::Context};

    #[test]
    fn test_map() {
        let context = Context::new();

        let vec = vec![0., 0., 0., 0.];
        let mut x = Buffer::from_vec(&context, wgpu::BufferUsages::MAP_READ, vec.clone());
        let y = x.slice(..).map(&context);

        assert_eq!(&vec, &*y);

        drop(y);
        x.unmap();
    }

    #[test]
    fn test_map_mut() {
        let context = Context::new();

        let vec = vec![0., 0., 0., 0.];
        let mut x = Buffer::from_vec(&context, wgpu::BufferUsages::MAP_WRITE, vec.clone());
        let mut y = x.slice(..).map_mut(&context);

        assert_eq!(&vec, &*y);
        y.iter_mut().enumerate().for_each(|(i, v)| {
            *v = i as f32 * 4.;
        });

        drop(y);
        x.unmap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_map() {
        let context = Context::new();

        let vec = vec![0., 0., 0., 0.];
        let mut x = Buffer::from_vec(&context, wgpu::BufferUsages::MAP_WRITE, vec.clone());
        let y = x.slice(..).map(&context);

        assert_eq!(&vec, &*y);

        drop(y);
        x.unmap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_map_mut() {
        let context = Context::new();

        let vec = vec![0., 0., 0., 0.];
        let mut x = Buffer::from_vec(&context, wgpu::BufferUsages::MAP_READ, vec.clone());
        let mut y = x.slice(..).map_mut(&context);

        assert_eq!(&vec, &*y);
        y.iter_mut().enumerate().for_each(|(i, v)| {
            *v = i as f32 * 4.;
        });

        drop(y);
        x.unmap();
    }

    #[test]
    fn test_copy_between() {
        let context = Context::new();

        let vec = vec![1., 2., 3., 4.];
        let x = Buffer::from_vec(&context, wgpu::BufferUsages::COPY_SRC, vec.clone());
        let mut y = Buffer::new(
            &context,
            wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            x.size(),
        );

        x.copy_to(&context, .., &mut y, ..);
        let y_read = y.slice(..).map(&context);

        assert_eq!(&vec, &*y_read);

        drop(y_read);
        y.unmap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_copy_between() {
        let context = Context::new();

        let vec = vec![1., 2., 3., 4.];
        let x = Buffer::from_vec(&context, wgpu::BufferUsages::COPY_DST, vec.clone());
        let mut y = Buffer::new(
            &context,
            wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_READ,
            x.size(),
        );

        x.copy_to(&context, .., &mut y, ..);
        let y_read = y.slice(..).map(&context);

        assert_eq!(&vec, &*y_read);

        drop(y_read);
        y.unmap();
    }
}
