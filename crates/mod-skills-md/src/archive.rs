//! Bộ giải nén và nạp thư mục an toàn cho `skills-md` (P3-08).
//!
//! Hỗ trợ:
//! 1. Chống Zip-Slip (đường dẫn chứa `..`, bắt đầu bằng `/`, `\` hoặc ổ đĩa).
//! 2. Chống bom giải nén (Decompression Bomb): Giới hạn kích thước tệp upload (10 MB),
//!    số tệp tối đa (100) và tổng dung lượng giải nén (20 MB).
//! 3. Hỗ trợ tệp zip định dạng Store (method 0) và Deflate (method 8, RFC 1951).
//! 4. Duyệt thư mục cục bộ an toàn.

#![expect(
    clippy::indexing_slicing,
    reason = "Thuật toán giải mã Huffman RFC 1951 và phân tích cấu trúc byte nhị phân ZIP"
)]
#![expect(
    clippy::too_many_lines,
    reason = "Triển khai máy trạng thái giải nén Deflate và phân tích tệp Zip hoàn chỉnh"
)]

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;
use thiserror::Error;

use crate::frontmatter::parse_skill_md;
use crate::model::{ExtractedFile, ParsedSkill};

/// Giới hạn dung lượng tải lên tối đa (10 MB).
pub const MAX_UPLOAD_BYTES: usize = 10 * 1024 * 1024;

/// Giới hạn số lượng tệp tối đa trong một skill (100 tệp).
pub const MAX_FILES: usize = 100;

/// Giới hạn dung lượng giải nén tối đa (20 MB).
pub const MAX_DECOMPRESSED_BYTES: usize = 20 * 1024 * 1024;

/// Lỗi xử lý kho lưu trữ kỹ năng.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ArchiveError {
    #[error("Tệp zip vượt quá kích thước upload tối đa ({0} > {MAX_UPLOAD_BYTES} bytes)")]
    UploadTooLarge(usize),

    #[error("Số lượng tệp trong skill vượt quá giới hạn ({0} > {MAX_FILES} tệp)")]
    TooManyFiles(usize),

    #[error("Dung lượng giải nén vượt quá giới hạn an toàn ({0} > {MAX_DECOMPRESSED_BYTES} bytes)")]
    DecompressionBombDetected(usize),

    #[error("Phát hiện nguy cơ Zip-Slip tại đường dẫn: '{0}'")]
    ZipSlipDetected(String),

    #[error("Tệp zip bị hỏng hoặc định dạng không hợp lệ: {0}")]
    CorruptZip(String),

    #[error("Không tìm thấy tệp SKILL.md trong thư mục hoặc tệp nén")]
    MissingSkillMd,

    #[error("Đường dẫn thư mục không tồn tại: {0}")]
    DirectoryNotFound(String),

    #[error("Đường dẫn không phải là thư mục hợp lệ: {0}")]
    NotADirectory(String),

    #[error("Lỗi đọc tệp hoặc thư mục: {0}")]
    IoError(String),

    #[error("Lỗi cú pháp SKILL.md: {0}")]
    ParseError(String),
}

/// Kiểm tra tính an toàn của đường dẫn tệp trong archive chống Zip-Slip.
pub fn validate_archive_path(path_str: &str) -> Result<String, ArchiveError> {
    let normalized = path_str.replace('\\', "/");

    if normalized.starts_with('/') || normalized.starts_with('\\') || normalized.contains(':') {
        return Err(ArchiveError::ZipSlipDetected(path_str.to_string()));
    }

    for component in normalized.split('/') {
        if component == ".." {
            return Err(ArchiveError::ZipSlipDetected(path_str.to_string()));
        }
    }

    let trimmed = normalized.trim_matches('/');
    Ok(trimmed.to_string())
}

/// Kiểm tra xem tệp có phải là tập lệnh (script) hay không.
#[must_use]
pub fn is_script_file(rel_path: &str) -> bool {
    let lower = rel_path.to_lowercase();
    if lower.starts_with("scripts/") || lower.contains("/scripts/") {
        return true;
    }
    lower.ends_with(".sh")
        || lower.ends_with(".bash")
        || lower.ends_with(".py")
        || lower.ends_with(".js")
        || lower.ends_with(".ts")
        || lower.ends_with(".bat")
        || lower.ends_with(".ps1")
        || lower.ends_with(".cmd")
        || lower.ends_with(".rb")
}

// ============================================================================
// Bộ giải mã RFC 1951 Deflate an toàn, thuần Rust (Zero External Dependencies)
// ============================================================================

struct BitReader<'a> {
    bytes: &'a [u8],
    bit_buf: u32,
    bits_in_buf: u8,
    pos: usize,
}

impl<'a> BitReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            bit_buf: 0,
            bits_in_buf: 0,
            pos: 0,
        }
    }

    fn ensure_bits(&mut self, n: u8) -> bool {
        while self.bits_in_buf < n {
            if self.pos >= self.bytes.len() {
                return false;
            }
            let b = u32::from(self.bytes[self.pos]);
            self.pos += 1;
            self.bit_buf |= b << self.bits_in_buf;
            self.bits_in_buf += 8;
        }
        true
    }

    fn read_bits(&mut self, n: u8) -> Option<u32> {
        if n == 0 {
            return Some(0);
        }
        if !self.ensure_bits(n) {
            return None;
        }
        let mask = (1u32 << n) - 1;
        let val = self.bit_buf & mask;
        self.bit_buf >>= n;
        self.bits_in_buf -= n;
        Some(val)
    }

    fn drop_to_byte_boundary(&mut self) {
        let remainder = self.bits_in_buf % 8;
        if remainder != 0 {
            self.bit_buf >>= remainder;
            self.bits_in_buf -= remainder;
        }
    }
}

#[derive(Clone, Debug)]
struct HuffmanTree {
    table: Vec<u16>,
    max_bits: u8,
}

impl HuffmanTree {
    fn from_code_lengths(lengths: &[u8]) -> Option<Self> {
        let mut max_bits = 0;
        for &l in lengths {
            if l > max_bits {
                max_bits = l;
            }
        }
        if max_bits == 0 || max_bits > 15 {
            return None;
        }

        let mut bl_count = vec![0u16; (max_bits + 1) as usize];
        for &l in lengths {
            if l > 0 {
                bl_count[l as usize] += 1;
            }
        }

        let mut next_code = vec![0u16; (max_bits + 1) as usize];
        let mut code = 0u16;
        for bits in 1..=max_bits {
            code = (code + bl_count[(bits - 1) as usize]) << 1;
            next_code[bits as usize] = code;
        }

        let table_size = 1usize << max_bits;
        let mut table = vec![0xFFFFu16; table_size];

        for (sym, &len) in lengths.iter().enumerate() {
            if len == 0 {
                continue;
            }
            let cur_code = next_code[len as usize];
            next_code[len as usize] += 1;

            // Đảo ngược bit
            let mut rev_code = 0usize;
            for i in 0..len {
                if (cur_code & (1 << i)) != 0 {
                    rev_code |= 1 << (len - 1 - i);
                }
            }

            let step = 1usize << len;
            for idx in (rev_code..table_size).step_by(step) {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "Symbol và len nằm trong phạm vi u16"
                )]
                let entry = ((sym as u16) << 4) | (u16::from(len));
                table[idx] = entry;
            }
        }

        Some(Self { table, max_bits })
    }

    fn decode(&self, reader: &mut BitReader<'_>) -> Option<u16> {
        if !reader.ensure_bits(self.max_bits) {
            let available = reader.bits_in_buf;
            if available == 0 {
                return None;
            }
            let mask = (1u32 << available) - 1;
            let val = (reader.bit_buf & mask) as usize;
            let entry = self.table.get(val).copied()?;
            let len = (entry & 0x0F) as u8;
            if len <= available && entry != 0xFFFF {
                reader.bit_buf >>= len;
                reader.bits_in_buf -= len;
                return Some(entry >> 4);
            }
            return None;
        }

        let mask = (1u32 << self.max_bits) - 1;
        let val = (reader.bit_buf & mask) as usize;
        let entry = self.table.get(val).copied()?;
        if entry == 0xFFFF {
            return None;
        }
        let len = (entry & 0x0F) as u8;
        reader.bit_buf >>= len;
        reader.bits_in_buf -= len;
        Some(entry >> 4)
    }
}

fn build_fixed_trees() -> Option<(HuffmanTree, HuffmanTree)> {
    let mut lit_lens = vec![8u8; 288];
    for l in lit_lens.iter_mut().take(144) {
        *l = 8;
    }
    for l in lit_lens.iter_mut().take(256).skip(144) {
        *l = 9;
    }
    for l in lit_lens.iter_mut().take(280).skip(256) {
        *l = 7;
    }
    for l in lit_lens.iter_mut().take(288).skip(280) {
        *l = 8;
    }
    let lit_tree = HuffmanTree::from_code_lengths(&lit_lens)?;

    let dist_lens = vec![5u8; 32];
    let dist_tree = HuffmanTree::from_code_lengths(&dist_lens)?;

    Some((lit_tree, dist_tree))
}

const LENGTH_BASE: [usize; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LENGTH_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];

const DIST_BASE: [usize; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

const CL_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

fn inflate_stream(compressed: &[u8], max_decompressed: usize) -> Result<Vec<u8>, ArchiveError> {
    let mut reader = BitReader::new(compressed);
    let mut output = Vec::new();
    let (fixed_lit, fixed_dist) = build_fixed_trees()
        .ok_or_else(|| ArchiveError::CorruptZip("Lỗi tạo cây Huffman cố định".to_string()))?;

    loop {
        let bfinal = reader.read_bits(1).ok_or_else(|| {
            ArchiveError::CorruptZip("Đạt cuối luồng khi đọc header block".to_string())
        })?;
        let btype = reader.read_bits(2).ok_or_else(|| {
            ArchiveError::CorruptZip("Đạt cuối luồng khi đọc loại block".to_string())
        })?;

        match btype {
            0 => {
                // Uncompressed
                reader.drop_to_byte_boundary();
                let len = reader.read_bits(16).ok_or_else(|| {
                    ArchiveError::CorruptZip("Lỗi đọc độ dài uncompressed block".to_string())
                })? as usize;
                let _nlen = reader.read_bits(16).ok_or_else(|| {
                    ArchiveError::CorruptZip("Lỗi đọc nlen uncompressed block".to_string())
                })?;

                for _ in 0..len {
                    let b = reader.read_bits(8).ok_or_else(|| {
                        ArchiveError::CorruptZip("Lỗi đọc byte uncompressed block".to_string())
                    })? as u8;
                    output.push(b);
                    if output.len() > max_decompressed {
                        return Err(ArchiveError::DecompressionBombDetected(output.len()));
                    }
                }
            }
            1 => {
                // Fixed Huffman
                decode_huffman_block(
                    &mut reader,
                    &fixed_lit,
                    &fixed_dist,
                    &mut output,
                    max_decompressed,
                )?;
            }
            2 => {
                // Dynamic Huffman
                let hlit = reader
                    .read_bits(5)
                    .ok_or_else(|| ArchiveError::CorruptZip("Lỗi đọc hlit".to_string()))?
                    as usize
                    + 257;
                let hdist = reader
                    .read_bits(5)
                    .ok_or_else(|| ArchiveError::CorruptZip("Lỗi đọc hdist".to_string()))?
                    as usize
                    + 1;
                let hclen = reader
                    .read_bits(4)
                    .ok_or_else(|| ArchiveError::CorruptZip("Lỗi đọc hclen".to_string()))?
                    as usize
                    + 4;

                let mut cl_lens = vec![0u8; 19];
                for &idx in CL_ORDER.iter().take(hclen) {
                    let val = reader.read_bits(3).ok_or_else(|| {
                        ArchiveError::CorruptZip("Lỗi đọc code length".to_string())
                    })? as u8;
                    cl_lens[idx] = val;
                }

                let cl_tree = HuffmanTree::from_code_lengths(&cl_lens).ok_or_else(|| {
                    ArchiveError::CorruptZip("Không tạo được cây code length".to_string())
                })?;

                let total_codes = hlit + hdist;
                let mut all_lens = Vec::with_capacity(total_codes);

                while all_lens.len() < total_codes {
                    let sym = cl_tree.decode(&mut reader).ok_or_else(|| {
                        ArchiveError::CorruptZip("Lỗi giải mã dynamic code length".to_string())
                    })?;

                    match sym {
                        0..=15 => all_lens.push(sym as u8),
                        16 => {
                            let last = *all_lens.last().ok_or_else(|| {
                                ArchiveError::CorruptZip(
                                    "Repeat mã 16 không có mẫu trước".to_string(),
                                )
                            })?;
                            let repeat = reader.read_bits(2).ok_or_else(|| {
                                ArchiveError::CorruptZip("Lỗi đọc repeat mã 16".to_string())
                            })? as usize
                                + 3;
                            all_lens.resize(all_lens.len() + repeat, last);
                        }
                        17 => {
                            let repeat = reader.read_bits(3).ok_or_else(|| {
                                ArchiveError::CorruptZip("Lỗi đọc repeat mã 17".to_string())
                            })? as usize
                                + 3;
                            all_lens.resize(all_lens.len() + repeat, 0);
                        }
                        18 => {
                            let repeat = reader.read_bits(7).ok_or_else(|| {
                                ArchiveError::CorruptZip("Lỗi đọc repeat mã 18".to_string())
                            })? as usize
                                + 11;
                            all_lens.resize(all_lens.len() + repeat, 0);
                        }
                        _ => {
                            return Err(ArchiveError::CorruptZip(
                                "Mã code length không hợp lệ".to_string(),
                            ));
                        }
                    }
                }

                let dyn_lit =
                    HuffmanTree::from_code_lengths(&all_lens[..hlit]).ok_or_else(|| {
                        ArchiveError::CorruptZip("Không tạo được cây dynamic literal".to_string())
                    })?;
                let dyn_dist =
                    HuffmanTree::from_code_lengths(&all_lens[hlit..]).ok_or_else(|| {
                        ArchiveError::CorruptZip("Không tạo được cây dynamic distance".to_string())
                    })?;

                decode_huffman_block(
                    &mut reader,
                    &dyn_lit,
                    &dyn_dist,
                    &mut output,
                    max_decompressed,
                )?;
            }
            _ => {
                return Err(ArchiveError::CorruptZip(
                    "Loại block deflate không hợp lệ (3)".to_string(),
                ));
            }
        }

        if bfinal == 1 {
            break;
        }
    }

    Ok(output)
}

fn decode_huffman_block(
    reader: &mut BitReader<'_>,
    lit_tree: &HuffmanTree,
    dist_tree: &HuffmanTree,
    output: &mut Vec<u8>,
    max_decompressed: usize,
) -> Result<(), ArchiveError> {
    loop {
        let sym = lit_tree.decode(reader).ok_or_else(|| {
            ArchiveError::CorruptZip("Lỗi giải mã literal/length symbol".to_string())
        })?;

        match sym {
            0..=255 => {
                output.push(sym as u8);
                if output.len() > max_decompressed {
                    return Err(ArchiveError::DecompressionBombDetected(output.len()));
                }
            }
            256 => break, // Kết thúc block
            257..=285 => {
                let len_idx = (sym - 257) as usize;
                let base_len = LENGTH_BASE[len_idx];
                let extra_bits = LENGTH_EXTRA[len_idx];
                let extra = reader.read_bits(extra_bits).ok_or_else(|| {
                    ArchiveError::CorruptZip("Lỗi đọc extra bit của length".to_string())
                })? as usize;
                let length = base_len + extra;

                let dist_sym = dist_tree.decode(reader).ok_or_else(|| {
                    ArchiveError::CorruptZip("Lỗi giải mã distance symbol".to_string())
                })? as usize;
                if dist_sym >= DIST_BASE.len() {
                    return Err(ArchiveError::CorruptZip(
                        "Distance symbol vượt phạm vi".to_string(),
                    ));
                }
                let base_dist = DIST_BASE[dist_sym];
                let dist_extra_bits = DIST_EXTRA[dist_sym];
                let dist_extra = reader.read_bits(dist_extra_bits).ok_or_else(|| {
                    ArchiveError::CorruptZip("Lỗi đọc extra bit của distance".to_string())
                })? as usize;
                let distance = base_dist + dist_extra;

                if distance > output.len() {
                    return Err(ArchiveError::CorruptZip(
                        "Khoảng cách tham chiếu LZ77 vượt quá bộ đệm hiện tại".to_string(),
                    ));
                }

                let start_pos = output.len() - distance;
                for i in 0..length {
                    let b = output[start_pos + i];
                    output.push(b);
                    if output.len() > max_decompressed {
                        return Err(ArchiveError::DecompressionBombDetected(output.len()));
                    }
                }
            }
            _ => {
                return Err(ArchiveError::CorruptZip(
                    "Literal symbol vượt phạm vi cho phép".to_string(),
                ));
            }
        }
    }

    Ok(())
}

// ============================================================================
// Bộ trích xuất tệp Zip từ byte slice
// ============================================================================

/// Giải nén tệp zip trong bộ nhớ với đầy đủ kiểm tra an toàn.
pub fn extract_zip(bytes: &[u8]) -> Result<Vec<ExtractedFile>, ArchiveError> {
    if bytes.len() > MAX_UPLOAD_BYTES {
        return Err(ArchiveError::UploadTooLarge(bytes.len()));
    }

    if bytes.len() < 22 {
        return Err(ArchiveError::CorruptZip("Tệp zip quá nhỏ".to_string()));
    }

    // Tìm End of Central Directory Record (PK\x05\x06)
    let mut eocd_pos = None;
    for i in (0..=bytes.len().saturating_sub(22)).rev() {
        if bytes[i..i + 4] == [0x50, 0x4b, 0x05, 0x06] {
            eocd_pos = Some(i);
            break;
        }
    }

    let eocd = eocd_pos.ok_or_else(|| {
        ArchiveError::CorruptZip("Không tìm thấy End of Central Directory".to_string())
    })?;

    let total_entries = u16::from_le_bytes([bytes[eocd + 10], bytes[eocd + 11]]) as usize;
    if total_entries > MAX_FILES {
        return Err(ArchiveError::TooManyFiles(total_entries));
    }

    let cd_offset = u32::from_le_bytes([
        bytes[eocd + 16],
        bytes[eocd + 17],
        bytes[eocd + 18],
        bytes[eocd + 19],
    ]) as usize;

    let mut cur_cd = cd_offset;
    let mut files = Vec::new();
    let mut total_decompressed = 0usize;
    // Tổng dung lượng THỰC TẾ đã giải nén (không tin kích thước khai báo trong header).
    let mut total_actual = 0usize;

    for _ in 0..total_entries {
        if cur_cd + 46 > bytes.len() {
            return Err(ArchiveError::CorruptZip(
                "Đầu mục Central Directory bị cắt cụt".to_string(),
            ));
        }

        if bytes[cur_cd..cur_cd + 4] != [0x50, 0x4b, 0x01, 0x02] {
            return Err(ArchiveError::CorruptZip(
                "Chữ ký Central Directory không khớp".to_string(),
            ));
        }

        let method = u16::from_le_bytes([bytes[cur_cd + 10], bytes[cur_cd + 11]]);
        let comp_size = u32::from_le_bytes([
            bytes[cur_cd + 20],
            bytes[cur_cd + 21],
            bytes[cur_cd + 22],
            bytes[cur_cd + 23],
        ]) as usize;
        let uncomp_size = u32::from_le_bytes([
            bytes[cur_cd + 24],
            bytes[cur_cd + 25],
            bytes[cur_cd + 26],
            bytes[cur_cd + 27],
        ]) as usize;

        let name_len = u16::from_le_bytes([bytes[cur_cd + 28], bytes[cur_cd + 29]]) as usize;
        let extra_len = u16::from_le_bytes([bytes[cur_cd + 30], bytes[cur_cd + 31]]) as usize;
        let comment_len = u16::from_le_bytes([bytes[cur_cd + 32], bytes[cur_cd + 33]]) as usize;

        let local_header_offset = u32::from_le_bytes([
            bytes[cur_cd + 42],
            bytes[cur_cd + 43],
            bytes[cur_cd + 44],
            bytes[cur_cd + 45],
        ]) as usize;

        let name_start = cur_cd + 46;
        let name_end = name_start + name_len;
        if name_end > bytes.len() {
            return Err(ArchiveError::CorruptZip(
                "Tên tệp trong Central Directory bị cắt cụt".to_string(),
            ));
        }

        let raw_name = String::from_utf8_lossy(&bytes[name_start..name_end]).to_string();
        let safe_name = validate_archive_path(&raw_name)?;

        cur_cd = name_end + extra_len + comment_len;

        // Bỏ qua thư mục rỗng
        if raw_name.ends_with('/') || raw_name.ends_with('\\') || safe_name.is_empty() {
            continue;
        }

        total_decompressed += uncomp_size;
        if total_decompressed > MAX_DECOMPRESSED_BYTES {
            return Err(ArchiveError::DecompressionBombDetected(total_decompressed));
        }

        // Đọc từ Local File Header
        if local_header_offset + 30 > bytes.len() {
            return Err(ArchiveError::CorruptZip(
                "Local file header vượt phạm vi tệp".to_string(),
            ));
        }
        if bytes[local_header_offset..local_header_offset + 4] != [0x50, 0x4b, 0x03, 0x04] {
            return Err(ArchiveError::CorruptZip(
                "Chữ ký Local file header không khớp".to_string(),
            ));
        }

        let loc_name_len = u16::from_le_bytes([
            bytes[local_header_offset + 26],
            bytes[local_header_offset + 27],
        ]) as usize;
        let loc_extra_len = u16::from_le_bytes([
            bytes[local_header_offset + 28],
            bytes[local_header_offset + 29],
        ]) as usize;

        let data_start = local_header_offset + 30 + loc_name_len + loc_extra_len;
        let data_end = data_start + comp_size;

        if data_end > bytes.len() {
            return Err(ArchiveError::CorruptZip(
                "Dữ liệu nén vượt phạm vi tệp".to_string(),
            ));
        }

        let raw_compressed = &bytes[data_start..data_end];

        let content = match method {
            0 => {
                // Stored
                raw_compressed.to_vec()
            }
            8 => {
                // Deflate
                inflate_stream(
                    raw_compressed,
                    MAX_DECOMPRESSED_BYTES.saturating_sub(total_actual),
                )?
            }
            m => {
                return Err(ArchiveError::CorruptZip(format!(
                    "Phương thức nén không được hỗ trợ ({m})"
                )));
            }
        };

        total_actual += content.len();
        if total_actual > MAX_DECOMPRESSED_BYTES {
            return Err(ArchiveError::DecompressionBombDetected(total_actual));
        }

        files.push(ExtractedFile {
            relative_path: safe_name,
            content,
        });

        if files.len() > MAX_FILES {
            return Err(ArchiveError::TooManyFiles(files.len()));
        }
    }

    Ok(files)
}

// ============================================================================
// Duyệt thư mục cục bộ an toàn
// ============================================================================

/// Đọc toàn bộ các tệp trong thư mục cục bộ với các giới hạn an toàn.
pub fn read_directory(dir_path: &Path) -> Result<Vec<ExtractedFile>, ArchiveError> {
    if !dir_path.exists() {
        return Err(ArchiveError::DirectoryNotFound(
            dir_path.to_string_lossy().to_string(),
        ));
    }
    if !dir_path.is_dir() {
        return Err(ArchiveError::NotADirectory(
            dir_path.to_string_lossy().to_string(),
        ));
    }

    let mut files = Vec::new();
    let mut total_bytes = 0usize;

    walk_dir_recursive(dir_path, dir_path, &mut files, &mut total_bytes, 0)?;

    Ok(files)
}

/// Độ sâu thư mục tối đa khi nạp một skill.
const MAX_DIR_DEPTH: usize = 8;

fn walk_dir_recursive(
    base: &Path,
    current: &Path,
    files: &mut Vec<ExtractedFile>,
    total_bytes: &mut usize,
    depth: usize,
) -> Result<(), ArchiveError> {
    if depth > MAX_DIR_DEPTH {
        return Err(ArchiveError::IoError(format!(
            "Thư mục lồng quá sâu (> {MAX_DIR_DEPTH} cấp)"
        )));
    }
    let entries = std::fs::read_dir(current).map_err(|e| {
        ArchiveError::IoError(format!("Không thể duyệt {}: {e}", current.display()))
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| ArchiveError::IoError(e.to_string()))?;
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Bỏ qua các thư mục nội bộ
        if file_name == ".git" || file_name == "node_modules" || file_name == "target" {
            continue;
        }

        // Không theo liên kết tượng trưng/junction: tránh đọc tệp ngoài thư mục skill và vòng lặp vô hạn.
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }

        if file_type.is_dir() {
            walk_dir_recursive(base, &path, files, total_bytes, depth + 1)?;
        } else if file_type.is_file() {
            let rel_path = path
                .strip_prefix(base)
                .map_err(|e| ArchiveError::IoError(e.to_string()))?
                .to_string_lossy()
                .replace('\\', "/");

            let safe_name = validate_archive_path(&rel_path)?;
            let content = std::fs::read(&path).map_err(|e| {
                ArchiveError::IoError(format!("Lỗi đọc tệp {}: {e}", path.display()))
            })?;

            *total_bytes += content.len();
            if *total_bytes > MAX_DECOMPRESSED_BYTES {
                return Err(ArchiveError::DecompressionBombDetected(*total_bytes));
            }

            files.push(ExtractedFile {
                relative_path: safe_name,
                content,
            });

            if files.len() > MAX_FILES {
                return Err(ArchiveError::TooManyFiles(files.len()));
            }
        }
    }

    Ok(())
}

// ============================================================================
// Hợp nhất và chuẩn hóa skill từ danh sách tệp đã giải nén
// ============================================================================

/// Phân tích cú pháp và trích xuất `ParsedSkill` từ danh sách tệp đã giải nén/đọc được.
pub fn parse_skill_from_files(files: &[ExtractedFile]) -> Result<ParsedSkill, ArchiveError> {
    if files.is_empty() {
        return Err(ArchiveError::MissingSkillMd);
    }

    // Tìm tệp SKILL.md (chấp nhận SKILL.md ở gốc hoặc trong thư mục con bậc 1)
    let skill_md_file = files
        .iter()
        .find(|f| {
            let lower = f.relative_path.to_lowercase();
            lower == "skill.md"
                || lower.ends_with("/skill.md")
                || lower == "skills.md"
                || lower.ends_with("/skills.md")
        })
        .ok_or(ArchiveError::MissingSkillMd)?;

    let skill_md_content = String::from_utf8(skill_md_file.content.clone()).map_err(|_| {
        ArchiveError::ParseError("Tệp SKILL.md không phải là UTF-8 hợp lệ".to_string())
    })?;

    let parsed_md =
        parse_skill_md(&skill_md_content).map_err(|e| ArchiveError::ParseError(e.to_string()))?;

    // Nhận diện script
    let mut has_scripts = false;
    let mut script_files = Vec::new();

    for f in files {
        if is_script_file(&f.relative_path) {
            has_scripts = true;
            script_files.push(f.relative_path.clone());
        }
    }
    script_files.sort();

    // Tính toán mã băm SHA-256 nội dung toàn bộ các tệp theo thứ tự đường dẫn
    let mut sorted_files: BTreeMap<&str, &[u8]> = BTreeMap::new();
    for f in files {
        sorted_files.insert(&f.relative_path, &f.content);
    }

    let mut hasher = Sha256::new();
    for (rel_path, content) in sorted_files {
        hasher.update(rel_path.as_bytes());
        hasher.update(content);
    }
    let content_hash = format!("{:x}", hasher.finalize());

    Ok(ParsedSkill {
        name: parsed_md.name,
        description: parsed_md.description,
        version: parsed_md.version,
        author: parsed_md.author,
        tags: parsed_md.tags,
        tools: parsed_md.tools,
        content: parsed_md.markdown_body,
        has_scripts,
        script_files,
        content_hash,
    })
}
