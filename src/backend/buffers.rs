use bytemuck::{try_cast_slice, try_cast_slice_mut};
use futures_channel::oneshot;
use std::ops::{Deref, DerefMut};
use std::{marker::PhantomData, ops::RangeBounds};
use wgpu::util::DeviceExt;
use wgpu::BufferDescriptor;

use wgpu::util::BufferInitDescriptor;

use super::{device::Context, traits::BufferType};

#[derive(Debug, Copy, Clone)]
pub struct CopyInfo {
    first_start: u64,
    second_start: u64,
    size: u64,
}

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
            buffer: &self,
            slice: self.buffer.slice(range),
            _phantom: PhantomData,
        }
    }

    pub fn copy_to(&self, context: &Context, buffer: &Buffer<T>, copy_specs: CopyInfo) {
        self.try_copy_to(context, buffer, copy_specs).unwrap();
    }

    pub fn try_copy_to(
        &self,
        context: &Context,
        buffer: &Buffer<T>,
        copy_specs: CopyInfo,
    ) -> Option<()> {
        if !self.usage.contains(wgpu::BufferUsages::COPY_SRC)
            && !buffer.usage.contains(wgpu::BufferUsages::COPY_DST)
        {
            return None;
        }

        let mut command_encoder = context.command_encoder();
        let CopyInfo {
            first_start,
            second_start,
            size,
        } = copy_specs;

        command_encoder.copy_buffer_to_buffer(
            &self.buffer,
            first_start,
            &buffer.buffer,
            second_start,
            size,
        );
        let command_buffer = command_encoder.finish();

        let idx = context.queue().submit([command_buffer]);
        context
            .device()
            .poll(wgpu::MaintainBase::WaitForSubmissionIndex(idx));

        Some(())
    }

    pub fn unmap(&mut self) {
        self.buffer.unmap();
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BufferSlice<'a, T: BufferType> {
    buffer: &'a Buffer<T>,
    slice: wgpu::BufferSlice<'a>,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T: BufferType> BufferSlice<'a, T> {
    pub fn map(&self, context: &Context) -> BufferView<'a, T> {
        self.try_map(context).unwrap()
    }

    pub fn try_map(&self, context: &Context) -> Option<BufferView<'a, T>> {
        if !self.buffer.usage.contains(wgpu::BufferUsages::MAP_READ) {
            return None;
        }

        let (sndr, rcvr) = oneshot::channel();
        self.slice
            .map_async(wgpu::MapMode::Read, move |status| match sndr.send(status) {
                Err(_) => log::error!("Could not send map_async result."),
                _ => {}
            });
        context.device().poll(wgpu::MaintainBase::Wait);
        smol::block_on(rcvr).ok()?.ok()?;

        Some(BufferView {
            // NOTE: This will throw if map_async failed.
            view: self.slice.get_mapped_range(),
            _phantom: PhantomData,
        })
    }

    pub fn map_mut(&self, context: &Context) -> BufferViewMut<'a, T> {
        self.try_map_mut(context).unwrap()
    }

    pub fn try_map_mut(&self, context: &Context) -> Option<BufferViewMut<'a, T>> {
        if !self.buffer.usage.contains(wgpu::BufferUsages::MAP_WRITE) {
            return None;
        }

        let (sndr, rcvr) = oneshot::channel();
        self.slice.map_async(wgpu::MapMode::Write, move |status| {
            match sndr.send(status) {
                Err(_) => log::error!("Could not send map_async result."),
                _ => {}
            }
        });
        context.device().poll(wgpu::MaintainBase::Wait);
        smol::block_on(rcvr).ok()?.ok()?;

        Some(BufferViewMut {
            // NOTE: This will throw if map_async failed.
            view: self.slice.get_mapped_range_mut(),
            _phantom: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct BufferView<'a, T: BufferType> {
    view: wgpu::BufferView<'a>,
    _phantom: PhantomData<&'a T>,
}

impl<T: BufferType> BufferView<'_, T> {
    fn try_deref(&self) -> Result<&[T], bytemuck::PodCastError> {
        try_cast_slice(self.view.deref())
    }
}

impl<T: BufferType> Deref for BufferView<'_, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.try_deref().unwrap()
    }
}

#[derive(Debug)]
pub struct BufferViewMut<'a, T: BufferType> {
    view: wgpu::BufferViewMut<'a>,
    _phantom: PhantomData<&'a mut T>,
}

impl<T: BufferType> BufferViewMut<'_, T> {
    fn try_deref(&self) -> Result<&[T], bytemuck::PodCastError> {
        try_cast_slice(self.view.deref())
    }

    fn try_deref_mut(&mut self) -> Result<&mut [T], bytemuck::PodCastError> {
        try_cast_slice_mut(self.view.deref_mut())
    }
}

impl<T: BufferType> Deref for BufferViewMut<'_, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.try_deref().unwrap()
    }
}

impl<T: BufferType> DerefMut for BufferViewMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.try_deref_mut().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::backend::{buffers::Buffer, device::Context};

    use super::CopyInfo;

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

        x.copy_to(&context, &y, CopyInfo { first_start: 0, second_start: 0, size: x.size() });
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

        x.copy_to(&context, &y, CopyInfo { first_start: 0, second_start: 0, size: x.size() });
        let y_read = y.slice(..).map(&context);
        
        assert_eq!(&vec, &*y_read);

        drop(y_read);
        y.unmap();
    }
}
