use futures_channel::oneshot;

use crate::backend::buffers::err::BufferMappingError;
use crate::backend::buffers::view::BufferViewMut;
use std::marker::PhantomData;

use crate::backend::buffers::Buffer;
use crate::backend::device::Context;
use crate::backend::traits::BufferType;

use super::view::BufferView;

#[derive(Debug, Copy, Clone)]
pub struct BufferSlice<'a, T: BufferType> {
    pub(crate) buffer: &'a Buffer<T>,
    pub(crate) slice: wgpu::BufferSlice<'a>,
    pub(crate) _phantom: PhantomData<&'a T>,
}

impl<'a, T: BufferType> BufferSlice<'a, T> {
    pub fn map(&self, context: &Context) -> BufferView<'a, T> {
        let map_result = self.try_map(context);

        if let Err(e) = &map_result {
            log::error!("Failed to map result: {}", e);
        }

        map_result.unwrap()
    }

    pub fn try_map(&self, context: &Context) -> Result<BufferView<'a, T>, BufferMappingError<'R'>> {
        if !self.buffer.usage.contains(wgpu::BufferUsages::MAP_READ) {
            return Err(BufferMappingError::InvalidBufferUsage(self.buffer.usage));
        }

        let (sndr, rcvr) = oneshot::channel();
        self.slice
            .map_async(wgpu::MapMode::Read, move |status| match sndr.send(status) {
                Err(_) => log::error!("Could not send map_async result."),
                _ => {}
            });
        context.device().poll(wgpu::MaintainBase::Wait);
        smol::block_on(rcvr)??;

        Ok(BufferView {
            // NOTE: This will throw if map_async failed.
            view: self.slice.get_mapped_range(),
            _phantom: PhantomData,
        })
    }

    pub fn map_mut(&self, context: &Context) -> BufferViewMut<'a, T> {
        let map_mut_result = self.try_map_mut(context);

        if let Err(e) = &map_mut_result {
            log::error!("Failed to map result: {}", e);
        }

        map_mut_result.unwrap()
    }

    pub fn try_map_mut(
        &self,
        context: &Context,
    ) -> Result<BufferViewMut<'a, T>, BufferMappingError<'W'>> {
        if !self.buffer.usage.contains(wgpu::BufferUsages::MAP_WRITE) {
            return Err(BufferMappingError::InvalidBufferUsage(self.buffer.usage));
        }

        let (sndr, rcvr) = oneshot::channel();
        self.slice.map_async(wgpu::MapMode::Write, move |status| {
            match sndr.send(status) {
                Err(_) => log::error!("Could not send map_async result."),
                _ => {}
            }
        });
        context.device().poll(wgpu::MaintainBase::Wait);
        smol::block_on(rcvr)??;

        Ok(BufferViewMut {
            // NOTE: This will throw if map_async failed.
            view: self.slice.get_mapped_range_mut(),
            _phantom: PhantomData,
        })
    }
}

