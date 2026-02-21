use super::bounded_ring_buffer::*;

type StartTime = f32;
type Height = f32;

const THRESHOLD: f32 = 0.5;

/// The (estimated) max amount of blocks which a column can have.
const MAX_BLOCKS_PER_COLUMN: usize = 16;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default, PartialEq)]
pub struct BlockData {
    start_time: StartTime,
    height: Height,
}

impl BlockData {
    pub fn new(start_time: f32) -> Self {
        Self {
            start_time,
            height: 0.2,
        }
    }
}

#[derive(Debug)]
pub struct BlockManager {
    blocks: BoundedRingBuffer<BlockData>,
    latest: Box<[Option<BlockData>]>,
    prev_bar_values: Box<[f32]>,

    latest_cache: Box<[BlockData]>,

    last_time: f32,
}

impl BlockManager {
    pub fn new(total_amount_bars: usize) -> Self {
        let blocks = BoundedRingBuffer::new(MAX_BLOCKS_PER_COLUMN * total_amount_bars);
        let latest = vec![None; total_amount_bars].into_boxed_slice();
        let latest_cache = vec![BlockData::default(); total_amount_bars].into_boxed_slice();
        let prev_bar_values = vec![0f32; total_amount_bars].into_boxed_slice();

        Self {
            blocks,
            latest,
            latest_cache,
            last_time: 0.,
            prev_bar_values,
        }
    }

    pub fn process_bars(&mut self, bars: &[Box<[f32]>]) {
        let bar_values = bars.iter().flatten().cloned();

        for ((curr, prev), curr_block) in bar_values
            .zip(self.prev_bar_values.iter_mut())
            .zip(self.latest.iter_mut())
        {
            let gradient = curr / *prev;
            if gradient >= THRESHOLD {
                match curr_block {
                    Some(block) => block.height += 0.2,
                    None => *curr_block = Some(BlockData::new(self.last_time)),
                };
            } else {
                if let Some(block) = curr_block.take() {
                    self.blocks.push_back(block);
                }
            }

            *prev = curr;
        }
    }

    pub fn amount_active_blocks(&self) -> usize {
        let old_active_blocks = self.blocks.len();
        let current_created_blocks = self
            .latest
            .iter()
            .map(|b| match b {
                Some(_) => 1,
                None => 0,
            })
            .sum::<usize>();

        old_active_blocks + current_created_blocks
    }

    pub fn discard_expired_blocks(&mut self, new_time: f32) {
        self.last_time = new_time;

        let time_range = 5.;
        self.blocks
            .pop_while(|block| (new_time - block.start_time) > time_range);
    }
}

/// WGPU relevant methods.
impl BlockManager {
    pub fn create_block_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let total_amount_columns = self.latest.len();

        let max_amount_blocks = (MAX_BLOCKS_PER_COLUMN * total_amount_columns)
                // we don't know how many blocks are also currently produced
                + self.latest_cache.len();

        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rising blocks: Block data buffer"),
            size: (std::mem::size_of::<BlockData>() * max_amount_blocks) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    pub fn update_wgpu_buffer(&mut self, queue: &wgpu::Queue, buffer: &wgpu::Buffer) {
        // refresh cache
        let mut cache_len = 0;
        for latest in self.latest.iter() {
            if let Some(latest) = latest {
                self.latest_cache[cache_len] = *latest;
                cache_len += 1;
            }
        }

        let c = self.blocks.contigious();
        let mut offset = 0usize;
        queue.write_buffer(buffer, offset as u64, bytemuck::cast_slice(c.head));

        offset += std::mem::size_of_val(c.head);
        queue.write_buffer(buffer, offset as u64, bytemuck::cast_slice(c.tail));

        offset += cache_len;
        queue.write_buffer(
            buffer,
            offset as u64,
            bytemuck::cast_slice(&self.latest_cache[..cache_len]),
        );
    }
}
