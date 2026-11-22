//! A minimal KTX2 writer for what `fh6-kit` loads: `R8G8B8A8_UNORM` (linear data: normals,
//! flow, masks) or `R8G8B8A8_SRGB` (colour), a full mip chain, zstd supercompression, and the
//! premultiplied-alpha flag in the data format descriptor.
use crate::img::{renormalize, Img};

/// Vulkan format numbers.
const VK_R8G8B8A8_UNORM: u32 = 37;
const VK_R8G8B8A8_SRGB: u32 = 43;
/// KTX2 supercompression scheme 2 = Zstandard.
const SUPERCOMPRESSION_ZSTD: u32 = 2;
const IDENTIFIER: [u8; 12] = [0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A];

/// How the texels are meant to be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    /// Linear data (normals, flow, masks, offsets).
    Linear,
    /// Linear data holding 0.5-centred unit normals in RGB (mips are renormalised).
    Normal,
    /// sRGB-encoded colour.
    Srgb,
}

/// Encodes `img` (any channel count; expanded to RGBA) as KTX2 bytes.
pub fn encode(img: &Img, enc: Encoding, premultiplied: bool, mips: bool) -> Vec<u8> {
    let base = img.to_channels(4);
    let mut levels: Vec<Img> = vec![base];
    if mips {
        while let Some(last) = levels.last() {
            if last.w == 1 && last.h == 1 {
                break;
            }
            let mut next = last.half();
            if enc == Encoding::Normal {
                renormalize(&mut next);
            }
            levels.push(next);
        }
    }
    let to_bytes = |l: &Img| -> Vec<u8> {
        l.data
            .iter()
            .map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)
            .collect()
    };
    let raw: Vec<Vec<u8>> = levels.iter().map(to_bytes).collect();
    let packed: Vec<Vec<u8>> = raw
        .iter()
        .map(|r| ruzstd::encoding::compress_to_vec(r.as_slice(), ruzstd::encoding::CompressionLevel::Fastest))
        .collect();

    let level_count = levels.len() as u32;
    let dfd = dfd(enc == Encoding::Srgb, premultiplied);
    let kvd = kvd();
    let header_len = 12 + 4 * 9 + 4 * 4 + 8 * 2;
    let level_index_len = 24 * levels.len();
    let dfd_offset = header_len + level_index_len;
    let kvd_offset = dfd_offset + dfd.len();
    let data_start = kvd_offset + kvd.len();

    // Level data is stored smallest mip first; the index lists largest first.
    let mut offsets = vec![0u64; levels.len()];
    let mut cursor = data_start as u64;
    for i in (0..levels.len()).rev() {
        offsets[i] = cursor;
        cursor += packed[i].len() as u64;
    }

    let mut out = Vec::with_capacity(cursor as usize);
    out.extend_from_slice(&IDENTIFIER);
    let vk = if enc == Encoding::Srgb {
        VK_R8G8B8A8_SRGB
    } else {
        VK_R8G8B8A8_UNORM
    };
    for v in [
        vk,
        1, // typeSize
        levels[0].w as u32,
        levels[0].h as u32,
        0, // pixelDepth
        0, // layerCount
        1, // faceCount
        level_count,
        SUPERCOMPRESSION_ZSTD,
    ] {
        out.extend_from_slice(&v.to_le_bytes());
    }
    for v in [dfd_offset as u32, dfd.len() as u32, kvd_offset as u32, kvd.len() as u32] {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out.extend_from_slice(&0u64.to_le_bytes()); // sgdByteOffset
    out.extend_from_slice(&0u64.to_le_bytes()); // sgdByteLength
    for i in 0..levels.len() {
        out.extend_from_slice(&offsets[i].to_le_bytes());
        out.extend_from_slice(&(packed[i].len() as u64).to_le_bytes());
        out.extend_from_slice(&(raw[i].len() as u64).to_le_bytes());
    }
    out.extend_from_slice(&dfd);
    out.extend_from_slice(&kvd);
    for i in (0..levels.len()).rev() {
        out.extend_from_slice(&packed[i]);
    }
    out
}

/// The Khronos basic data format descriptor for 8-bit RGBA.
fn dfd(srgb: bool, premultiplied: bool) -> Vec<u8> {
    const BLOCK: u32 = 24 + 16 * 4;
    let mut d = Vec::with_capacity(4 + BLOCK as usize);
    d.extend_from_slice(&(4 + BLOCK).to_le_bytes()); // dfdTotalSize
    d.extend_from_slice(&0u32.to_le_bytes()); // vendorId 0, descriptorType 0
    d.extend_from_slice(&(2u32 | (BLOCK << 16)).to_le_bytes()); // version 2, block size
    d.push(1); // colorModel RGBSDA
    d.push(1); // colorPrimaries BT709
    d.push(if srgb { 2 } else { 1 }); // transfer: sRGB or linear
    d.push(u8::from(premultiplied)); // flags: alpha premultiplied
    d.extend_from_slice(&[0, 0, 0, 0]); // texelBlockDimension (1x1x1x1, stored minus one)
    d.extend_from_slice(&[0; 8]); // bytesPlane0..7 are zero when supercompressed
    for (i, channel) in [0u8, 1, 2, 15].iter().enumerate() {
        let mut ch = *channel;
        if srgb && *channel == 15 {
            ch |= 0x10; // alpha stays linear in an sRGB texture
        }
        d.extend_from_slice(&((i as u16) * 8).to_le_bytes()); // bitOffset
        d.push(7); // bitLength - 1
        d.push(ch);
        d.extend_from_slice(&[0, 0, 0, 0]); // samplePosition
        d.extend_from_slice(&0u32.to_le_bytes()); // sampleLower
        d.extend_from_slice(&255u32.to_le_bytes()); // sampleUpper
    }
    d
}

/// Key/value data: the writer id (padded to 4 bytes).
fn kvd() -> Vec<u8> {
    let key = b"KTXwriter\0";
    let value = b"FH6 asset-forge\0";
    let len = (key.len() + value.len()) as u32;
    let mut out = Vec::new();
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(key);
    out.extend_from_slice(value);
    while out.len() % 4 != 0 {
        out.push(0);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ktx2_reads_back_with_the_kit_reader() {
        let img = Img::from_fn(16, 8, 4, |x, y| [x as f32 / 15.0, y as f32 / 7.0, 0.5, 1.0]);
        let bytes = encode(&img, Encoding::Linear, true, true);
        let reader = ktx2::Reader::new(bytes.as_slice()).map_err(|e| e.to_string());
        assert!(reader.is_ok(), "{:?}", reader.as_ref().err());
        if let Ok(r) = reader {
            let h = r.header();
            assert_eq!(h.pixel_width, 16);
            assert_eq!(h.pixel_height, 8);
            assert_eq!(h.level_count, 5);
            assert_eq!(h.format, Some(ktx2::Format::R8G8B8A8_UNORM));
            assert_eq!(h.supercompression_scheme, Some(ktx2::SupercompressionScheme::Zstandard));
            assert_eq!(r.is_alpha_premultiplied(), Some(true));
            let level0 = r.levels().next().map(|l| l.data.len()).unwrap_or(0);
            assert!(level0 > 0);
        }
    }
}
