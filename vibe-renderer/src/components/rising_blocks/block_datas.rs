use std::time::Instant;

const MAX_AMOUNT_RISING_BLOCKS: usize = 1024;

type StartTime = f32;
type Height = f32;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default, PartialEq)]
pub struct BlockData {
    start_time: StartTime,
    height: Height,
}

#[derive(Debug)]
pub struct BlockDatasDescriptor {
    pub threshold: f32,
    pub max_amount_blocks: usize,
}

impl Default for BlockDatasDescriptor {
    fn default() -> Self {
        Self {
            threshold: 0.5,
            max_amount_blocks: MAX_AMOUNT_RISING_BLOCKS,
        }
    }
}

#[derive(Debug)]
pub struct BlockDatas {
    data: Box<[BlockData]>,

    len: usize,

    threshold: f32,
    last_time: f32,
}

impl BlockDatas {
    pub fn new(desc: BlockDatasDescriptor) -> Self {
        let data = vec![BlockData::default(); desc.max_amount_blocks].into_boxed_slice();

        // let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("Rising blocks: Block data buffer"),
        //     size: (std::mem::size_of::<BlockData>() * MAX_AMOUNT_RISING_BLOCKS)
        //         as wgpu::BufferAddress,
        //     usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        //     mapped_at_creation: false,
        // });

        Self {
            data,
            len: 0,
            threshold: desc.threshold,
            last_time: 0.,
        }
    }

    // pub fn get_buffer(&self) -> wgpu::BufferSlice<'_> {
    //     self.buffer.slice(0..self.len as u64)
    // }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn max_amount_blocks(&self) -> usize {
        self.data.len()
    }
}

/// Updating relevant functions
impl BlockDatas {
    pub fn update_blocks(&mut self, bars: &[Box<[f32]>]) {
        // self.register_new_blocks(bars);

        // queue.write_buffer(
        //     &self.buffer,
        //     0,
        //     bytemuck::cast_slice(&self.data[0..self.len]),
        // )
    }

    pub fn discard_expired_blocks(&mut self, new_time: f32) {
        self.last_time = new_time;

        // TODO: Make that configureable
        let time_window = 5.;
        let t = new_time - time_window;

        if let Some(idx) = binary_search_less_equal_than(&self.data[0..self.len], t) {
            self.data.copy_within(idx.., 0);
            self.len -= idx + 1;
        }
    }

    fn register_new_blocks(&mut self, bars: &[Box<[f32]>]) {
        let is_full = self.len >= self.data.len();
        if is_full {
            return;
        }

        for channel in bars.iter() {
            for bar_value in channel.iter().cloned() {
                if bar_value >= self.threshold {
                    self.data[self.len] = BlockData {
                        start_time: self.last_time,
                        height: 0.,
                    };
                    self.len += 1;

                    let is_full = self.len >= self.data.len();
                    if is_full {
                        return;
                    }
                }
            }
        }
    }
}

trait BinarySearchable {
    fn value(&self) -> f32;
}

impl BinarySearchable for f32 {
    fn value(&self) -> f32 {
        *self
    }
}

impl BinarySearchable for BlockData {
    fn value(&self) -> f32 {
        self.start_time
    }
}

fn binary_search_less_equal_than<B: BinarySearchable>(
    searchables: &[B],
    value: f32,
) -> Option<usize> {
    if searchables.is_empty() {
        return None;
    }

    let mut idx = searchables.len() / 2;
    let mut search_size = searchables.len().div_ceil(2);
    while search_size > 0 {
        let curr = searchables[idx].value();
        let next = searchables.get(idx + 1);

        match next {
            Some(next) => {
                let next = next.value();

                if curr < value && next < value {
                    idx += search_size - 1;
                } else if value < curr && value < next {
                    idx -= search_size - 1;
                } else {
                    // curr < value < next
                    return Some(idx);
                }
            }
            None => {
                // is at the right-most end
                if curr <= value {
                    return Some(idx);
                } else {
                    return None;
                }
            }
        }

        search_size /= 2;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    mod binary_search {
        use super::*;

        #[test]
        fn empty() {
            assert_eq!(binary_search_less_equal_than::<f32>(&[], 100.), None);
        }

        #[test]
        fn one_item() {
            let arr = [0.];

            assert_eq!(binary_search_less_equal_than(&arr, 1.), Some(arr.len() - 1));
        }

        #[test]
        fn general() {
            assert_eq!(
                binary_search_less_equal_than(&[0., 1., 2., 3.], 2.5),
                Some(2)
            );
        }

        #[test]
        fn last_item() {
            let arr = [0., 1., 2., 3.];
            assert_eq!(binary_search_less_equal_than(&arr, 4.), Some(arr.len() - 1))
        }

        #[test]
        fn no_item() {
            let arr = [1., 2., 3., 4.];
            assert_eq!(binary_search_less_equal_than(&arr, 0.), None);
        }
    }

    mod discard_expired_blocks {

        use super::*;

        const DESC: BlockDatasDescriptor = BlockDatasDescriptor {
            threshold: 0.5,
            max_amount_blocks: 10,
        };

        #[test]
        fn empty() {
            let mut datas = BlockDatas::new(DESC);

            datas.discard_expired_blocks(0.);

            assert_eq!(datas.len, 0);
        }

        #[test]
        fn discard_none() {
            let mut datas = BlockDatas::new(DESC);

            let active_blocks = [
                BlockData {
                    start_time: 1.,
                    height: 0.,
                },
                BlockData {
                    start_time: 2.,
                    height: 0.,
                },
                BlockData {
                    start_time: 3.,
                    height: 0.,
                },
            ];

            datas.data[0..3].copy_from_slice(&active_blocks);
            datas.len = 3;
            datas.discard_expired_blocks(0.);

            assert_eq!(datas.len, 3);
            assert_eq!(&datas.data[0..3], &active_blocks);
        }

        #[test]
        fn discard_one() {
            let mut datas = BlockDatas::new(DESC);

            let active_blocks = [
                BlockData {
                    start_time: 1.,
                    height: 0.,
                },
                BlockData {
                    start_time: 2.,
                    height: 0.,
                },
                BlockData {
                    start_time: 3.,
                    height: 0.,
                },
            ];

            datas.data[0..3].copy_from_slice(&active_blocks);
            datas.len = 3;
            datas.discard_expired_blocks(1.5);

            assert_eq!(datas.len, 2);
            assert_eq!(
                &datas.data[0..3],
                &[
                    BlockData {
                        start_time: 2.,
                        height: 0.,
                    },
                    BlockData {
                        start_time: 3.,
                        height: 0.,
                    },
                    BlockData {
                        start_time: 0.,
                        height: 0.,
                    },
                ]
            );
        }
    }
}
