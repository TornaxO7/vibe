use cgmath::{MetricSpace, Vector2};

use crate::texture_generation::TextureGenerator;

struct LightPointCtx {
    pos: Vector2<f32>,
    min_dist: f32,
    cluster: usize,
}

impl LightPointCtx {
    pub fn new(pos: Vector2<f32>) -> Self {
        Self {
            pos,
            min_dist: f32::MAX,
            cluster: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TexelData {
    dist: f32,
    cluster_idx: f32,
}

/// Returns a distance-texture (format: `Rg32float`).
///
/// `r` channel:
///   Contains the (shortest distance) to a light source and the more far away the texel is, the higher the value gets.
///   Basically just a distance map.
/// `g` channel:
///   Contains the index (starting from `0.`) of the cluster which the texel belongs to.
///   Natural numbers are stored there.
pub struct LightSources<'a> {
    pub src: &'a image::DynamicImage,

    pub light_threshold: f32,
}

impl<'a> TextureGenerator for LightSources<'a> {
    fn generate(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        let width = self.src.width();
        let height = self.src.height();

        let gray_scaled_img = self.gray_scale_image(width, height);

        let blurred_img = self.apply_gaussian_blur(gray_scaled_img);

        let light_threshold = self.compute_light_thresold(&blurred_img);
        let mut light_ctxs = self.get_light_ctxs(light_threshold, blurred_img);

        // TODO: Generalise

        const AMOUNT_CENTERS: usize = 10;
        const AMOUNT_ITERATIONS: usize = 50;

        let mut cluster_centers =
            vec![self.init_cluster_center(width as f32, height as f32, &light_ctxs)];

        for _ in 1..AMOUNT_CENTERS {
            self.converge(AMOUNT_ITERATIONS, &mut cluster_centers, &mut light_ctxs);
            self.add_center(&mut cluster_centers, &light_ctxs);
        }
        self.converge(AMOUNT_ITERATIONS, &mut cluster_centers, &mut light_ctxs);

        // since we have the centers now, compute the data for each damn pixel
        let mut data: Vec<TexelData> = Vec::new();

        for y in 0..height {
            for x in 0..width {
                let texel_pos = Vector2::new(x as f32, y as f32);

                let mut min_dist = texel_pos.distance2(cluster_centers[0]);
                let mut min_idx = 0;
                for (idx, center) in cluster_centers.iter().enumerate() {
                    let dist = texel_pos.distance2(*center);

                    if dist < min_dist {
                        min_dist = dist;
                        min_idx = idx;
                    }
                }

                data.push(TexelData {
                    dist: min_dist.sqrt(),
                    cluster_idx: min_idx as f32,
                });
            }
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("KMeans texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rg32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            texture.as_image_copy(),
            bytemuck::cast_slice(&data),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(std::mem::size_of::<TexelData>() as u32 * width),
                rows_per_image: Some(height),
            },
            texture.size(),
        );

        texture
    }
}

impl<'a> LightSources<'a> {
    fn gray_scale_image(&self, width: u32, height: u32) -> Box<[Box<[f32]>]> {
        let rgb8 = self.src.to_rgb8();
        let mut buffer =
            vec![vec![0f32; width as usize].into_boxed_slice(); height as usize].into_boxed_slice();

        for (x, y, value) in rgb8.enumerate_pixels() {
            let r = value.0[0] as f32;
            let g = value.0[1] as f32;
            let b = value.0[2] as f32;

            let lum = 0.3 * r + 0.59 * g + 0.11 * b;

            buffer[y as usize][x as usize] = lum;
        }

        buffer
    }

    fn apply_gaussian_blur(&self, src: Box<[Box<[f32]>]>) -> Box<[Box<[f32]>]> {
        let img_width = src[0].len();
        let img_height = src.len();

        let mut img1 = src;
        let mut img2 = img1.clone();

        let kernel_size = 9;
        let kernel_width = (kernel_size as i32).isqrt() as u32;
        let half_kernel_width = (kernel_width / 2) as i32;
        let kernel = generate_kernel(kernel_size, 5.);

        // horizontal_kernel
        for pixel_y in 0..img_height {
            for pixel_x in 0..img_width {
                let mut sum = 0.;
                for i in -half_kernel_width..half_kernel_width {
                    let x = pixel_x as i32 + i;

                    if x < 0 || x >= img_width as i32 {
                        continue;
                    }

                    let pixel = img1[pixel_y][x as usize];
                    let kernel_value = kernel[(i + half_kernel_width) as usize];
                    sum += pixel * kernel_value;
                }

                img2[pixel_y][pixel_x] = sum;
            }
        }

        // vertical
        for pixel_y in 0..img_height {
            for pixel_x in 0..img_width {
                let mut sum = 0.;
                for i in -half_kernel_width..half_kernel_width {
                    let y = pixel_y as i32 + i;

                    if y < 0 || y >= img_height as i32 {
                        continue;
                    }

                    let pixel = img2[y as usize][pixel_x];
                    let kernel_value = kernel[(i + half_kernel_width) as usize];
                    sum += pixel * kernel_value;
                }

                img1[pixel_y][pixel_x] = sum;
            }
        }

        img1
    }

    fn get_light_ctxs(
        &self,
        threshold: f32,
        blurred_img: Box<[Box<[f32]>]>,
    ) -> Box<[LightPointCtx]> {
        let mut positions = Vec::new();

        let height = blurred_img.len();
        let width = blurred_img[0].len();

        for y in 0..height {
            for x in 0..width {
                let value = blurred_img[y][x];
                if value >= threshold {
                    positions.push(LightPointCtx::new(Vector2::new(x as f32, y as f32)));
                }
            }
        }

        positions.into_boxed_slice()
    }

    fn init_cluster_center(
        &self,
        max_x: f32,
        max_y: f32,
        light_ctxs: &Box<[LightPointCtx]>,
    ) -> Vector2<f32> {
        let pos: Vector2<f32> = Vector2 {
            x: fastrand::f32() * max_x,
            y: fastrand::f32() * max_y,
        };

        let mut nearest_point = light_ctxs[0].pos;
        let mut min_dist = pos.distance2(nearest_point);
        for light_pos in light_ctxs.iter() {
            let dist = light_pos.pos.distance2(pos);
            if dist < min_dist {
                min_dist = dist;
                nearest_point = light_pos.pos.clone();
            }
        }

        nearest_point
    }

    fn converge(
        &self,
        iterations: usize,
        centers: &mut Vec<Vector2<f32>>,
        light_positions: &mut Box<[LightPointCtx]>,
    ) {
        for _ in 0..iterations {
            // reset min dist
            for ctx in light_positions.iter_mut() {
                ctx.min_dist = f32::MAX;
            }

            // update min dist
            for ctx in light_positions.iter_mut() {
                for (cluster_idx, center) in centers.iter().enumerate() {
                    let dist = ctx.pos.distance2(*center);

                    if dist < ctx.min_dist {
                        ctx.min_dist = dist;
                        ctx.cluster = cluster_idx;
                    }
                }
            }

            // update position of centers
            for (cluster_idx, center) in centers.iter_mut().enumerate() {
                let mut cluster_sum: Vector2<f32> = Vector2::new(0f32, 0f32);
                let mut cluster_size = 0;

                for point in light_positions.iter() {
                    if point.cluster == cluster_idx {
                        cluster_sum += point.pos;
                        cluster_size += 1;
                    }
                }

                *center = cluster_sum / cluster_size as f32;
            }
        }
    }

    fn add_center(&self, centers: &mut Vec<Vector2<f32>>, light_positions: &Box<[LightPointCtx]>) {
        let mut distances = vec![0f32; light_positions.len()].into_boxed_slice();

        // compute distances from each point to each center
        for (idx, point) in light_positions.iter().enumerate() {
            let mut min_dist = point.pos.distance2(centers[0]);

            for center in centers.iter() {
                let dist = point.pos.distance2(*center);

                if dist < min_dist {
                    min_dist = dist;
                }
            }

            distances[idx] = min_dist;
        }

        let mut min_idx = 0;
        let mut min_dist = distances[0];
        for (idx, dist) in distances.iter().enumerate() {
            if *dist < min_dist {
                min_dist = *dist;
                min_idx = idx;
            }
        }

        centers.push(light_positions[min_idx].pos);
    }

    fn compute_light_thresold(&self, blurred_img: &Box<[Box<[f32]>]>) -> f32 {
        let mut max_brightness = 0.;

        let height = blurred_img.len();
        let width = blurred_img[0].len();

        for y in 0..height {
            for x in 0..width {
                let value = blurred_img[y][x];

                if value >= max_brightness {
                    max_brightness = value;
                }
            }
        }

        max_brightness * self.light_threshold
    }
}

fn generate_kernel(size: usize, sigma: f32) -> Vec<f32> {
    assert!(size % 2 == 1);

    let mut kernel = Vec::with_capacity(size);

    let mut sum = 0.;
    let half_size = (size / 2) as isize;
    for x in (-half_size)..=half_size {
        let value = gauss(sigma, x as f32);
        kernel.push(value);
        sum += value;
    }

    // normamlize kernel
    for value in kernel.iter_mut() {
        *value /= sum;
    }

    kernel
}

fn gauss(sigma: f32, x: f32) -> f32 {
    (1. / (2. * std::f32::consts::PI * sigma * sigma))
        * std::f32::consts::E.powf(-(x * x) / (2. * sigma * sigma))
}
