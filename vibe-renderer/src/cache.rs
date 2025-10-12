use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

use crate::{texture_generation::TextureGenerator, Renderer};

const TEXTURE_INFO_FILE_NAME: &str = "info.toml";
const TEXTURE_FILE_NAME: &str = "texture.raw";
const TEXTURE_CHECKSUM_FILE_NAME: &str = "texture.hash";

/// Information about the cached texture.
#[derive(Debug, Serialize, Deserialize)]
pub struct TextureInfo {
    width: u32,
    height: u32,

    amount_row_padding_bytes: u32,
}

#[derive(thiserror::Error, Debug)]
pub enum CacheError {
    #[error("Couldn't open/close file '{path}' although it exists: {err}")]
    IO { path: String, err: std::io::Error },

    #[error("Couldn't deserialize texture info '{path}': {err}")]
    DeserializeTextureInfo { path: String, err: toml::de::Error },

    #[error("Couldn't serialize texture info '{path}': {err}")]
    SerializeTextureInfo { path: String, err: toml::ser::Error },
}

pub trait Cacheable {
    /// The subpath within the main cache directory of this cacheable texture.
    fn subpath(&self) -> PathBuf;

    /// The checksum of the cached texture.
    fn checksum(&self) -> u64;

    /// The format of the cached texture
    fn format(&self) -> wgpu::TextureFormat;
}

pub fn load<C: Cacheable + TextureGenerator>(
    renderer: &Renderer,
    cacheable: &C,
) -> Result<wgpu::Texture, CacheError> {
    match load_texture(renderer, cacheable)? {
        Some(cached_texture) => {
            info!("Cache hit!");
            Ok(cached_texture)
        }
        None => {
            info!("Cache miss. Generating texture (this may take a while)...");

            let texture = renderer.generate(cacheable);
            store_texture(renderer, cacheable, &texture)?;
            Ok(texture)
        }
    }
}

fn load_texture<C: Cacheable>(
    renderer: &Renderer,
    cacheable: &C,
) -> Result<Option<wgpu::Texture>, CacheError> {
    let device = renderer.device();
    let queue = renderer.queue();

    let dir_path = crate::get_cache_dir(cacheable.subpath());

    let texture_info_file_path =
        PathBuf::from_iter([dir_path.clone(), TEXTURE_INFO_FILE_NAME.into()]);
    let texture_file_path = PathBuf::from_iter([dir_path.clone(), TEXTURE_FILE_NAME.into()]);
    let texture_checksum_path =
        PathBuf::from_iter([dir_path.clone(), TEXTURE_CHECKSUM_FILE_NAME.into()]);

    // check if the cache contains an entry at all
    let cache_exists = {
        let paths = [
            &texture_info_file_path,
            &texture_file_path,
            &texture_checksum_path,
        ];
        paths.iter().all(|path| path.exists())
    };

    if !cache_exists {
        return Ok(None);
    }

    // check if the entry in the cache is up to date
    let cache_is_up_to_date = {
        let mut bytes_buffer = [0u8; 8];
        let current_checksum_bytes =
            std::fs::read(&texture_checksum_path).map_err(|err| CacheError::IO {
                path: texture_checksum_path.to_string_lossy().to_string(),
                err,
            })?;

        bytes_buffer.copy_from_slice(&current_checksum_bytes);

        let current_checksum = u64::from_be_bytes(bytes_buffer);
        current_checksum == cacheable.checksum()
    };

    if !cache_is_up_to_date {
        return Ok(None);
    }

    // load texture from file
    let info: TextureInfo = {
        let path = texture_info_file_path.to_string_lossy().to_string();

        let texture_info_bytes =
            std::fs::read(&texture_info_file_path).map_err(|err| CacheError::IO {
                path: path.clone(),
                err,
            })?;

        toml::from_slice(&texture_info_bytes).map_err(|err| CacheError::DeserializeTextureInfo {
            path: path.clone(),
            err,
        })?
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Cached texture"),
        size: wgpu::Extent3d {
            width: info.width,
            height: info.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: cacheable.format(),
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let texture_bytes = std::fs::read(&texture_file_path).map_err(|err| CacheError::IO {
        path: texture_file_path.to_string_lossy().to_string(),
        err,
    })?;

    queue.write_texture(
        texture.as_image_copy(),
        &texture_bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(
                cacheable
                    .format()
                    .block_copy_size(Some(wgpu::TextureAspect::All))
                    .unwrap()
                    * info.width,
            ),
            rows_per_image: Some(info.height),
        },
        texture.size(),
    );

    Ok(Some(texture))
}

fn store_texture<C: Cacheable>(
    renderer: &Renderer,
    cacheable: &C,
    texture: &wgpu::Texture,
) -> Result<(), CacheError> {
    let device = renderer.device();
    let queue = renderer.queue();

    let dir_path = crate::get_cache_dir(cacheable.subpath());
    let texture_info_file_path =
        PathBuf::from_iter([dir_path.clone(), TEXTURE_INFO_FILE_NAME.into()]);
    let texture_file_path = PathBuf::from_iter([dir_path.clone(), TEXTURE_FILE_NAME.into()]);
    let texture_checksum_path =
        PathBuf::from_iter([dir_path.clone(), TEXTURE_CHECKSUM_FILE_NAME.into()]);

    let texel_byte_size = texture
        .format()
        .block_copy_size(Some(wgpu::TextureAspect::All))
        .unwrap();

    let info = {
        let width = texture.width();
        let height = texture.height();

        TextureInfo {
            width,
            height,
            amount_row_padding_bytes: wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
                - (width * texel_byte_size) % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT,
        }
    };

    // retrieve the bytes within the texture
    let texture_bytes = {
        let unpadded_line_size = info.width * texel_byte_size;
        let padded_line_size = unpadded_line_size + info.amount_row_padding_bytes;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Texture bytes buffer"),
            size: (padded_line_size * info.height) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        {
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            encoder.copy_texture_to_buffer(
                texture.as_image_copy(),
                wgpu::TexelCopyBufferInfo {
                    buffer: &buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(padded_line_size),
                        rows_per_image: Some(info.height),
                    },
                },
                texture.size(),
            );

            queue.submit([encoder.finish()]);
        }

        let texture_bytes = {
            let buffer_slice = buffer.slice(..);

            let (tx, rx) = std::sync::mpsc::channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |result| tx.send(result).unwrap());
            device.poll(wgpu::PollType::Wait).unwrap();
            rx.recv().unwrap().unwrap();

            let mut texture_bytes: Vec<u8> =
                Vec::with_capacity((unpadded_line_size * info.height) as usize);

            let buffer_bytes = buffer.get_mapped_range(..);
            for row in buffer_bytes.chunks(padded_line_size as usize) {
                texture_bytes.extend_from_slice(&row[..unpadded_line_size as usize]);
            }

            texture_bytes
        };

        buffer.unmap();
        texture_bytes
    };

    // save everything to the file
    let info_str = toml::to_string(&info).expect("Serialize texture info");
    std::fs::write(&texture_info_file_path, info_str).map_err(|err| CacheError::IO {
        path: texture_info_file_path.to_string_lossy().to_string(),
        err,
    })?;

    std::fs::write(&texture_file_path, &texture_bytes).map_err(|err| CacheError::IO {
        path: texture_file_path.to_string_lossy().to_string(),
        err,
    })?;

    std::fs::write(&texture_checksum_path, cacheable.checksum().to_be_bytes()).map_err(|err| {
        CacheError::IO {
            path: texture_checksum_path.to_string_lossy().to_string(),
            err,
        }
    })?;

    Ok(())
}
