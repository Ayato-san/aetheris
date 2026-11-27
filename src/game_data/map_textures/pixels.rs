use crate::game_data::binary::invalid;
use std::io;

/// Decode the three non-virtual PC formats present in this client's map assets.
/// RGBA remains unpremultiplied, including BC1's transparent fourth colour.
pub(super) fn decode(
    format: &str,
    width: usize,
    height: usize,
    data: &[u8],
) -> io::Result<Vec<u8>> {
    if width == 0 || height == 0 || width > 4096 || height > 4096 {
        return Err(invalid("map dimensions exceed limits"));
    }
    if format == "PF_B8G8R8A8" {
        if data.len() != width * height * 4 {
            return Err(invalid("invalid BGRA payload length"));
        }
        let mut result = data.to_vec();
        for pixel in result.as_chunks_mut::<4>().0 {
            pixel.swap(0, 2);
        }
        return Ok(result);
    }
    let stride = match format {
        "PF_DXT1" => 8,
        "PF_DXT5" => 16,
        _ => return Err(invalid(format!("unsupported map pixel format {format}"))),
    };
    let blocks_x = width.div_ceil(4);
    let blocks_y = height.div_ceil(4);
    if data.len() != blocks_x * blocks_y * stride {
        return Err(invalid("invalid BC payload length"));
    }
    let mut output = vec![0; width * height * 4];
    for (block_index, block) in data.chunks_exact(stride).enumerate() {
        let color = &block[stride - 8..];
        let c0 = u16::from_le_bytes(color[..2].try_into().unwrap());
        let c1 = u16::from_le_bytes(color[2..4].try_into().unwrap());
        let rgb = |c: u16| {
            let r = (c >> 11) & 31;
            let g = (c >> 5) & 63;
            let b = c & 31;
            [
                (r << 3 | r >> 2) as u8,
                (g << 2 | g >> 4) as u8,
                (b << 3 | b >> 2) as u8,
                255,
            ]
        };
        let mut colors = [rgb(c0), rgb(c1), [0, 0, 0, 255], [0; 4]];
        let first = colors[0];
        let second = colors[1];
        if stride == 16 || c0 > c1 {
            for (i, _) in first[..3].iter().enumerate() {
                colors[2][i] = ((2 * u16::from(first[i]) + u16::from(second[i])) / 3) as u8;
                colors[3][i] = ((u16::from(first[i]) + 2 * u16::from(second[i])) / 3) as u8;
            }
            colors[3][3] = 255;
        } else {
            for (i, _) in first[..3].iter().enumerate() {
                colors[2][i] = ((u16::from(first[i]) + u16::from(second[i])) / 2) as u8;
            }
        }
        let mut alpha = [255u8; 8];
        let mut alpha_indices = 0u64;
        if stride == 16 {
            alpha[0] = block[0];
            alpha[1] = block[1];
            if alpha[0] > alpha[1] {
                for i in 1..=6 {
                    alpha[i + 1] =
                        (((7 - i) * usize::from(alpha[0]) + i * usize::from(alpha[1]) + 3) / 7)
                            as u8;
                }
            } else {
                for i in 1..=4 {
                    alpha[i + 1] =
                        (((5 - i) * usize::from(alpha[0]) + i * usize::from(alpha[1]) + 2) / 5)
                            as u8;
                }
                alpha[6] = 0;
                alpha[7] = 255;
            }
            for (i, b) in block[2..8].iter().enumerate() {
                alpha_indices |= u64::from(*b) << (i * 8);
            }
        }
        let indices = u32::from_le_bytes(color[4..8].try_into().unwrap());
        for pixel in 0..16 {
            let x = block_index % blocks_x * 4 + pixel % 4;
            let y = block_index / blocks_x * 4 + pixel / 4;
            if x >= width || y >= height {
                continue;
            }
            let mut rgba = colors[((indices >> (pixel * 2)) & 3) as usize];
            if stride == 16 {
                rgba[3] = alpha[((alpha_indices >> (pixel * 3)) & 7) as usize];
            }
            output[(y * width + x) * 4..(y * width + x + 1) * 4].copy_from_slice(&rgba);
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bc1_transparency_and_clipped_blocks() {
        let block = [0, 0, 255, 255, 255, 255, 255, 255];
        assert_eq!(decode("PF_DXT1", 1, 2, &block).unwrap(), vec![0; 8]);
        assert!(decode("PF_DXT1", 5, 4, &block).is_err());
        assert!(decode("PF_DXT1", 4097, 1, &[]).is_err());
    }
    #[test]
    fn bc3_interpolated_alpha_and_bgra_channels() {
        let block = [255, 0, 2, 0, 0, 0, 0, 0, 0, 248, 0, 0, 0, 0, 0, 0];
        let data = decode("PF_DXT5", 4, 4, &block).unwrap();
        assert_eq!(&data[..4], &[255, 0, 0, 219]);
        assert_eq!(
            decode("PF_B8G8R8A8", 1, 1, &[1, 2, 3, 4]).unwrap(),
            [3, 2, 1, 4]
        );
    }
}
