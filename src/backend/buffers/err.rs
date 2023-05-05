use std::ops::RangeBounds;

use futures_channel::oneshot;

#[derive(Debug, Clone, thiserror::Error)]
pub enum BufferCopyError<A: RangeBounds<usize>, B: RangeBounds<usize>> {
    #[error("Unequal reference lengths: Tried to copy a slice from {0} to {1}.")]
    UnequalReferenceLengths(A, B),

    #[error("Invalid Source Buffer Usage: {0:?}")]
    InvalidSourceBuffer(wgpu::BufferUsages),

    #[error("Invalid Destination Buffer Usage: {0:?}")]
    InvalidDestinationBuffer(wgpu::BufferUsages),
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum BufferMappingError<const TYPE: char> {
    #[error("Invalid {TYPE} Buffer Usage: {0:?}")]
    InvalidBufferUsage(wgpu::BufferUsages),

    #[error("Could not receive data: {0}")]
    FailedReceive(#[from] oneshot::Canceled),

    #[error("Failed to map buffer: {0}")]
    BufferAsyncError(#[from] wgpu::BufferAsyncError),
}
