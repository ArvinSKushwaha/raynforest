use std::{marker::PhantomData, ops::{Deref, DerefMut}};

use bytemuck::checked::{try_cast_slice, try_cast_slice_mut};

use crate::backend::traits::BufferType;



#[derive(Debug)]
pub struct BufferView<'a, T: BufferType> {
    pub(crate) view: wgpu::BufferView<'a>,
    pub(crate) _phantom: PhantomData<&'a T>,
}

impl<T: BufferType> BufferView<'_, T> {
    fn try_deref(&self) -> Result<&[T], bytemuck::checked::CheckedCastError> {
        try_cast_slice(self.view.deref())
    }
}

impl<T: BufferType> Deref for BufferView<'_, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        let deref_result = self.try_deref();

        if let Err(e) = &deref_result {
            log::error!("Failed to deref BufferView: {}", e);
        }

        deref_result.unwrap()
    }
}

#[derive(Debug)]
pub struct BufferViewMut<'a, T: BufferType> {
    pub(crate) view: wgpu::BufferViewMut<'a>,
    pub(crate) _phantom: PhantomData<&'a mut T>,
}

impl<T: BufferType> BufferViewMut<'_, T> {
    fn try_deref(&self) -> Result<&[T], bytemuck::checked::CheckedCastError> {
        try_cast_slice(self.view.deref())
    }

    fn try_deref_mut(&mut self) -> Result<&mut [T], bytemuck::checked::CheckedCastError> {
        try_cast_slice_mut(self.view.deref_mut())
    }
}

impl<T: BufferType> Deref for BufferViewMut<'_, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        let deref_result = self.try_deref();

        if let Err(e) = &deref_result {
            log::error!("Failed to deref BufferViewMut: {}", e);
        }

        deref_result.unwrap()
    }
}

impl<T: BufferType> DerefMut for BufferViewMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let deref_mut_result = self.try_deref_mut();

        if let Err(e) = &deref_mut_result {
            log::error!("Failed to deref BufferViewMut: {}", e);
        }

        deref_mut_result.unwrap()
    }
}

