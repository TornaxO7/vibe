use super::bounded_ring_buffer::*;

type StartTime = f32;
type Height = f32;

/// The (estimated) max amount of blocks which a column can have.
const MAX_BLOCKS_PER_COLUMN: usize = 128;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default, PartialEq)]
pub struct BlockData {
    start_time: StartTime,
    height: Height,
}

#[derive(Debug)]
pub struct BlockManager {
    blocks: BoundedRingBuffer<BlockData>,
    latest: Box<[Option<BlockData>]>,

    last_time: f32,
}

impl BlockManager {
    pub fn new(amount_columns: usize) -> Self {
        let blocks = BoundedRingBuffer::new(MAX_BLOCKS_PER_COLUMN * amount_columns);
        let latest = vec![None; amount_columns].into_boxed_slice();

        Self {
            blocks,
            latest,
            last_time: 0.,
        }
    }
}
