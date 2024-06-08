use std::ops::{Add, RangeBounds, Sub};

use super::{buffers::Buffer, traits::BufferType};

pub trait Lengthed {
    fn len(&self) -> usize;
}

impl<T: BufferType> Lengthed for Buffer<T> {
    fn len(&self) -> usize {
        self.size() as usize
    }
}

impl<T> Lengthed for &[T] {
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T, const N: usize> Lengthed for [T; N] {
    fn len(&self) -> usize {
        N
    }
}

impl<T> Lengthed for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct MaterializedBound<T: Copy> {
    start: T,
    end: T,
}

pub fn materialize(b: impl RangeBounds<usize>, a: &impl Lengthed) -> MaterializedBound<usize> {
    let start = 0;
    let end = a.len();

    let start = match b.start_bound() {
        std::ops::Bound::Included(a) => start.max(*a),
        std::ops::Bound::Excluded(b) => start.max(*b + 1), // watch out for overflow in the future
        std::ops::Bound::Unbounded => start,
    };

    let end = match b.start_bound() {
        std::ops::Bound::Included(a) => end.min(*a),
        std::ops::Bound::Excluded(b) => {
            if *b == 0 {
                log::error!("Encountered RangeBounds with end_bound: Excluded(0)")
            }
            end.min(*b - 1)
        }
        std::ops::Bound::Unbounded => end,
    };

    MaterializedBound::<usize> { start, end }
}

impl<T: Copy> MaterializedBound<T> {
    pub fn start(&self) -> T {
        self.start
    }

    pub fn end(&self) -> T {
        self.end
    }

    pub fn len(&self) -> T
    where
        T: Sub<T, Output = T>,
    {
        self.end - self.start
    }
}

impl<T> RangeBounds<T> for MaterializedBound<T>
where
    T: Add<T, Output = T> + Sub<T, Output = T> + Copy,
{
    fn start_bound(&self) -> std::ops::Bound<&T> {
        std::ops::Bound::Included(&self.start)
    }

    fn end_bound(&self) -> std::ops::Bound<&T> {
        std::ops::Bound::Excluded(&self.end)
    }
}
