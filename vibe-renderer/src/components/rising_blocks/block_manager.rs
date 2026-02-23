use super::bounded_ring_buffer::*;

type StartTime = f32;
type ColumnIdx = u32;

/// The (estimated) max amount of blocks which a column can have.
const MAX_BLOCKS_PER_COLUMN: usize = 16;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default, PartialEq)]
pub struct BlockData {
    start_time: StartTime,
    column_idx: ColumnIdx,
}

impl BlockData {
    pub fn new(start_time: f32, column_idx: u32) -> Self {
        Self {
            column_idx,
            start_time,
        }
    }
}

#[derive(Debug)]
pub struct BlockManager {
    blocks: BoundedRingBuffer<BlockData>,
    prev_beat: Box<[bool]>,

    last_time: f32,
    total_amount_columns: usize,
}

impl BlockManager {
    pub fn new(total_amount_bars: usize) -> Self {
        let blocks = BoundedRingBuffer::new(MAX_BLOCKS_PER_COLUMN * total_amount_bars);

        let prev_beat = vec![false; total_amount_bars].into_boxed_slice();

        Self {
            blocks,
            last_time: 0.,
            prev_beat,
            total_amount_columns: total_amount_bars,
        }
    }

    pub fn process_bars(&mut self, bars: &[Box<[f32]>]) {
        let bar_values = bars.iter().flatten().cloned();

        for ((bar_idx, bar_value), prev_beat) in
            bar_values.enumerate().zip(self.prev_beat.iter_mut())
        {
            const THRESHOLD: f32 = 0.5;
            let is_beat = bar_value > THRESHOLD;
            if is_beat {
                if !*prev_beat {
                    self.blocks
                        .push_back(BlockData::new(self.last_time, bar_idx as u32));
                } else {
                    *prev_beat = false;
                }
            }
            *prev_beat = is_beat;
        }
    }

    pub fn amount_active_blocks(&self) -> usize {
        self.blocks.len()
    }

    pub fn discard_expired_blocks(&mut self, new_time: f32) {
        self.last_time = new_time;

        let time_to_live = 1.;
        self.blocks.pop_while(|block| {
            let run_time = new_time - block.start_time;
            run_time > time_to_live
        });
    }
}

/// WGPU relevant methods.
impl BlockManager {
    pub fn create_block_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let max_amount_blocks = MAX_BLOCKS_PER_COLUMN * self.total_amount_columns;

        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rising blocks: Block data buffer"),
            size: (std::mem::size_of::<BlockData>() * max_amount_blocks) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    pub fn update_wgpu_buffer(&mut self, queue: &wgpu::Queue, buffer: &wgpu::Buffer) {
        let c = self.blocks.contigious();
        let mut offset = 0usize;

        let bs = std::mem::size_of::<BlockData>();

        queue.write_buffer(buffer, offset as u64, bytemuck::cast_slice(c.head));
        offset += c.head.len() * bs;

        queue.write_buffer(buffer, offset as u64, bytemuck::cast_slice(c.tail));
    }
}
