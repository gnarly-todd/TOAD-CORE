//! Writes ampr_emu.index (AMPRIDX3) and ampr_assets.index (AMPRPAK4) so ampr-emu-ng serves the
//! .pkg as an LZ4-mode pack of RAW and compressed chunks (one per compressed block, upstream index documentation (see CREDITS.md)) from
//! one volume, the .pkg named in the runtime package descriptor (pack_id 0).
//! Layouts follow ampr-emu-ng `ampr-format/src/index.rs`, `ampr-lz4/src/manifest.rs` (the
//! runtime runs the full `Manifest::open` on load) and the dump-forge writers
//! (the vendored fpkg crate's `ampr_index.rs`, `ps5-dump-forge-lz4/src/writer.rs`).
//! Manifest record i is AMPRIDX3 id i + 1, so both list the same files in the same sorted order.
//! Directories are not records: the runtime derives them from the sorted file paths.
use std::io::{Error, ErrorKind, Result};
use std::path::Path;

pub struct IndexFile {
    /// Path under /app0 without a leading slash, e.g. "data/level1.bin".
    pub path: String,
    pub size: u64,
    /// None: a loose file extracted on disk. Some: the file's compressed blocks in the runtime package volume, in
    /// file order, one chunk each.
    pub chunks: Option<Vec<Chunk>>,
}

/// One compressed block of a packed file: `stored_len` bytes at `pkg_off` in the runtime package volume that
/// decode to `raw_len` bytes (the block's logical length; the file's tail block may run past
/// the file's end).
pub struct Chunk {
    pub pkg_off: u64,
    pub stored_len: u32,
    pub raw_len: u32,
    /// None: stored as plain bytes (stored_len == raw_len). Some: a compressed block.
    pub compression: Option<CompressedBlock>,
}

/// The block record fields `decoder::decode_block` needs (`decoder::Desc`).
#[derive(Clone, Copy)]
pub struct CompressedBlock {
    pub even_len: u32,
    pub mode: u8,
}

const APP0: &str = "/app0/";
const PKG_NAME: &str = "kraken.pkg";
const INDEX_NAME: &str = "ampr_emu.index";
const MANIFEST_NAME: &str = "ampr_assets.index";

// AMPRIDX3 (ampr-format index.rs, dump-forge's ampr_index.rs).
const IDX_MAGIC: &[u8; 8] = b"AMPRIDX3";
const IDX_HEADER: usize = 0x30;
const IDX_RECORD: usize = 24;
const IDX_SLOT: usize = 16;
const IDX_SLOT_SHARED: u32 = 1;

// AMPRPAK4 (ampr-lz4 lib.rs / manifest.rs).
const PAK_MAGIC: &[u8; 8] = b"AMPRPAK4";
const PAK_HEADER: usize = 128;
const FILE_RECORD: usize = 48;
const CHUNK_RECORD: usize = 12;
const PACK_RECORD: usize = 32;
const PAK_PAYLOAD_CRC_AT: usize = 112;
const PAK_HEADER_CRC_AT: usize = 116;
const ENDIAN_MARKER: u32 = 0x0102_0304;
const FILE_PACKED: u32 = 1;
const FILE_STREAMING: u32 = 4;
const CHUNK_RAW: u32 = 0;
/// upstream index documentation (see CREDITS.md): codec 2, location = offset (40 bits) | even_len << 40 | mode << 59.
const CHUNK_COMPRESSED: u32 = 2;
const CHUNK_COMPRESSED_FULL: u8 = 0x10;
const CHUNK_STREAMING: u8 = 2;
const CHUNK_PAGE_CONTAINED: u8 = 4;
const CHUNK_PAGE_ALIGNED: u8 = 8;
const PACK_IO_PAGE_LAYOUT: u32 = 2;
/// io_page_size of the volume; the payload starts one page in (payload_offset >= DAT_HEADER).
const PAGE: u64 = 65536; // 64 KiB, measured best (pkg game wait/read: 4 KiB 4.14 ms, 16 KiB 3.33, 64 KiB 2.48-2.62, 256 KiB 2.93-3.31; 256 KiB reads ~35-50% more bytes)
/// One chunk per compressed block: block_shift 18 = 256 KiB (`decoder::BLOCK`).
const BLOCK_SHIFT: u32 = 18;
const BLOCK: u64 = 1 << BLOCK_SHIFT;
const OFFSET_LIMIT: u64 = 1 << 48;
const COMPRESSED_OFFSET_LIMIT: u64 = 1 << 40;
const PATH_MAX: usize = 1024;
const NAME_MAX: usize = 255;

fn bad(msg: String) -> Error {
    Error::new(ErrorKind::InvalidData, msg)
}

fn folded(b: u8) -> u8 {
    match b {
        b'\\' => b'/',
        b'A'..=b'Z' => b + 32,
        _ => b,
    }
}

/// FNV-1a over case-folded bytes; 0 becomes 1 (ampr-format lib.rs `hash`).
fn fnv(path: &[u8], basis: u64) -> u64 {
    let h = path.iter().fold(basis, |h, &b| {
        (h ^ u64::from(folded(b))).wrapping_mul(1_099_511_628_211)
    });
    h.max(1)
}

fn idx_path_hash(path: &[u8]) -> u64 {
    fnv(path, 1_469_598_103_934_665_603)
}

fn pak_path_hash(path: &[u8]) -> u64 {
    fnv(path, 14_695_981_039_346_656_037)
}

/// CRC-32 (IEEE, reflected 0xEDB88320), as ampr-format `crc32_parts`.
pub(crate) fn crc32(parts: &[&[u8]]) -> u32 {
    let mut table = [0u32; 256];
    for (i, t) in table.iter_mut().enumerate() {
        let mut v = i as u32;
        for _ in 0..8 {
            v = (v >> 1) ^ (0xedb8_8320 & 0u32.wrapping_sub(v & 1));
        }
        *t = v;
    }
    let mut c = !0u32;
    for p in parts {
        for &b in *p {
            c = table[((c ^ u32::from(b)) & 0xff) as usize] ^ (c >> 8);
        }
    }
    !c
}

fn put32(b: &mut [u8], at: usize, v: u32) {
    b[at..at + 4].copy_from_slice(&v.to_le_bytes());
}

fn put64(b: &mut [u8], at: usize, v: u64) {
    b[at..at + 8].copy_from_slice(&v.to_le_bytes());
}

/// AMPRIDX3 image of `rows` (already sorted by folded path, no duplicates):
/// records (path off/len u32, size u64, mtime i64), NUL-terminated path blob, 16-aligned
/// linear-probe slot table of next_pow2(2n) slots (hash u64, 1-based id u32, flags u32).
fn build_index(rows: &[(String, &IndexFile)], mtime: i64) -> Vec<u8> {
    let path_bytes: usize = rows.iter().map(|(p, _)| p.len() + 1).sum();
    let count = (rows.len() * 2).next_power_of_two().max(2);
    let mask = count - 1;
    let mut slots = vec![(0u64, 0u32, 0u32); count];
    for (i, (path, _)) in rows.iter().enumerate() {
        let h = idx_path_hash(path.as_bytes());
        let mut at = h as usize & mask;
        let mut shared = false;
        while slots[at].1 != 0 {
            if slots[at].0 == h {
                slots[at].2 |= IDX_SLOT_SHARED;
                shared = true;
            }
            at = (at + 1) & mask;
        }
        slots[at] = (h, i as u32 + 1, if shared { IDX_SLOT_SHARED } else { 0 });
    }
    let slots_at = (IDX_HEADER + rows.len() * IDX_RECORD + path_bytes).next_multiple_of(IDX_SLOT);
    // Write directly into the final image. The old writer built and copied a
    // full records buffer and a separate string buffer on every cache build.
    let mut out = Vec::with_capacity(slots_at + count * IDX_SLOT);
    out.extend_from_slice(IDX_MAGIC);
    out.extend_from_slice(&3u32.to_le_bytes());
    out.extend_from_slice(&(IDX_RECORD as u32).to_le_bytes());
    out.extend_from_slice(&(rows.len() as u64).to_le_bytes());
    out.extend_from_slice(&(path_bytes as u64).to_le_bytes());
    out.extend_from_slice(&(slots_at as u64).to_le_bytes());
    out.extend_from_slice(&(IDX_SLOT as u32).to_le_bytes());
    out.extend_from_slice(&(count as u32).to_le_bytes());
    let mut offset = 0usize;
    for (path, file) in rows {
        out.extend_from_slice(&(offset as u32).to_le_bytes());
        out.extend_from_slice(&(path.len() as u32).to_le_bytes());
        out.extend_from_slice(&file.size.to_le_bytes());
        out.extend_from_slice(&mtime.to_le_bytes());
        offset += path.len() + 1;
    }
    for (path, _) in rows {
        out.extend_from_slice(path.as_bytes());
        out.push(0);
    }
    out.resize(slots_at, 0);
    for (h, id, flags) in slots {
        out.extend_from_slice(&h.to_le_bytes());
        out.extend_from_slice(&id.to_le_bytes());
        out.extend_from_slice(&flags.to_le_bytes());
    }
    out
}

/// One chunk per compressed block at block_shift 18 (manifest.rs `validate_file`: chunk_count =
/// ceil(size / block), chunk i decodes to min(block, size - i * block) bytes). A RAW tail block
/// past the file's end is cut to the file; a compressed one is a full 256 KiB block decoded whole
/// (CHUNK_COMPRESSED_FULL). `vol_size` is the page-rounded volume size.
/// Whether the bundled runtime's pack format can represent this exact block grid.
pub fn streamable(path: &str, size: u64, blocks: &[Chunk], vol_size: u64) -> bool {
    visit_chunks(path, size, blocks, vol_size, |_, _| {}).is_ok()
}

/// Validate/encode chunks without a temporary vector for the first-pass
/// streamability check. The manifest writer collects the same descriptors.
fn visit_chunks(
    path: &str,
    size: u64,
    blocks: &[Chunk],
    vol_size: u64,
    mut emit: impl FnMut(u64, u32),
) -> Result<()> {
    let fail = |at: u64, why: &str| bad(format!("{path}: block at pkg offset {at:#x}: {why}"));
    if blocks.len() as u64 != size.div_ceil(BLOCK) {
        return Err(bad(format!("{path}: {} blocks for {size} bytes, want 256 KiB blocks", blocks.len())));
    }
    for (i, c) in blocks.iter().enumerate() {
        let (at, want) = (c.pkg_off, BLOCK.min(size - i as u64 * BLOCK));
        let raw_len = u64::from(c.raw_len);
        if raw_len < want || raw_len > BLOCK {
            return Err(fail(at, "its length does not fit the 256 KiB block grid"));
        }
        let (stored, codec, mut flags, loc) = match c.compression {
            None => {
                if c.stored_len != c.raw_len {
                    return Err(fail(at, "a raw block's stored length differs from its length"));
                }
                (want, CHUNK_RAW, 0u8, at)
            }
            Some(k) => {
                let stored = u64::from(c.stored_len);
                if stored == 0 || stored > BLOCK || u64::from(k.even_len) > stored || k.mode >= 32 {
                    return Err(fail(at, "compressed block not encodable as a compressed chunk"));
                }
                if raw_len > want && raw_len != BLOCK {
                    return Err(fail(at, "a short compressed block runs past the file's end"));
                }
                let full = if raw_len > want { CHUNK_COMPRESSED_FULL } else { 0 };
                let loc = at | u64::from(k.even_len) << 40 | u64::from(k.mode) << 59;
                (stored, CHUNK_COMPRESSED, full, loc)
            }
        };
        let end = at + stored;
        let limit = if codec == CHUNK_COMPRESSED { COMPRESSED_OFFSET_LIMIT } else { OFFSET_LIMIT };
        if at < PAGE || end > vol_size || end >= limit {
            return Err(fail(at, "outside the packable volume range"));
        }
        let contained = stored <= PAGE && at / PAGE == (end - 1) / PAGE;
        let aligned = at % PAGE == 0 && (stored > PAGE || contained);
        // compressed and RAW chunks are both exempt from the page-layout rules in our ng fork
        // (upstream index documentation (see CREDITS.md)): the runtime reads the covering page window at any offset. Not marking
        // files STREAMING keeps ng's decoded cache on for them (caches.rs refuses STREAMING).
        flags |= if contained { CHUNK_PAGE_CONTAINED } else { 0 } | if aligned { CHUNK_PAGE_ALIGNED } else { 0 };
        emit(loc, (stored as u32 - 1) | (codec << 20) | (u32::from(flags) << 22));
    }
    Ok(())
}

/// AMPRPAK4 image: 128-byte header, file records, chunk records, one pack record, strings
/// (paths, then the volume name). `rows` pairs each sorted logical path with its file.
fn build_manifest(rows: &[(String, &IndexFile)], mtime: i64, pkg_len: u64) -> Result<Vec<u8>> {
    let vol_size = pkg_len.next_multiple_of(PAGE).max(2 * PAGE);
    let mut strings = Vec::new();
    let mut add = |s: &str| {
        let at = strings.len() as u32;
        strings.extend_from_slice(s.as_bytes());
        strings.push(0);
        (at, s.len() as u32)
    };
    let mut files = vec![0u8; rows.len() * FILE_RECORD];
    let chunk_capacity: usize = rows.iter().map(|(_, f)| f.chunks.as_ref().map_or(0, |b| b.len() * CHUNK_RECORD)).sum();
    let mut chunks = Vec::with_capacity(chunk_capacity);
    for (i, (path, f)) in rows.iter().enumerate() {
        let r = &mut files[i * FILE_RECORD..(i + 1) * FILE_RECORD];
        put64(r, 0, pak_path_hash(path.as_bytes()));
        put64(r, 8, f.size);
        put64(r, 16, mtime as u64);
        let (at, len) = add(path);
        put32(r, 32, at);
        put32(r, 36, len);
        let Some(blocks) = &f.chunks else { continue };
        put32(r, 24, (chunks.len() / CHUNK_RECORD) as u32);
        put32(r, 28, blocks.len() as u32);
        put32(r, 40, FILE_PACKED);
        r[44] = BLOCK_SHIFT as u8;
        // Encode into the final chunk table without a per-file temporary Plan.
        visit_chunks(path, f.size, blocks, vol_size, |loc, desc| {
            chunks.extend_from_slice(&loc.to_le_bytes());
            chunks.extend_from_slice(&desc.to_le_bytes());
        })?;
    }
    let mut pack = vec![0u8; PACK_RECORD];
    put64(&mut pack, 0, vol_size - PAGE);
    put64(&mut pack, 8, vol_size);
    let (at, len) = add(PKG_NAME);
    put32(&mut pack, 16, at);
    put32(&mut pack, 20, len);
    put32(&mut pack, 24, PACK_IO_PAGE_LAYOUT);
    put32(&mut pack, 28, PAGE as u32);

    let mut m = vec![0u8; PAK_HEADER];
    m[..8].copy_from_slice(PAK_MAGIC);
    put32(&mut m, 8, 4);
    put32(&mut m, 12, PAK_HEADER as u32);
    put32(&mut m, 20, ENDIAN_MARKER);
    // build_id: any 16 bytes; derived from the content so a rebuild of the same pkg matches.
    let seed = crc32(&[&files, &chunks, &strings]);
    for (k, w) in m[24..40].chunks_mut(4).enumerate() {
        w.copy_from_slice(&crc32(&[&seed.to_le_bytes(), &pkg_len.to_le_bytes(), &[k as u8]]).to_le_bytes());
    }
    put64(&mut m, 40, rows.len() as u64);
    put64(&mut m, 48, (chunks.len() / CHUNK_RECORD) as u64);
    put32(&mut m, 56, 1);
    put32(&mut m, 60, FILE_RECORD as u32);
    put32(&mut m, 64, CHUNK_RECORD as u32);
    put32(&mut m, 68, PACK_RECORD as u32);
    let mut at = PAK_HEADER as u64;
    for (off, len) in [(72, files.len()), (80, chunks.len()), (88, pack.len()), (96, strings.len())] {
        put64(&mut m, off, at);
        at += len as u64;
    }
    put64(&mut m, 104, strings.len() as u64);
    if (chunks.len() / CHUNK_RECORD) as u64 > u64::from(u32::MAX) {
        return Err(bad("too many chunks".into()));
    }
    for part in [&files, &chunks, &pack, &strings] {
        m.extend_from_slice(part);
    }
    let payload = crc32(&[&m[PAK_HEADER..]]);
    put32(&mut m, PAK_PAYLOAD_CRC_AT, payload);
    let header = crc32(&[&m[..PAK_HEADER_CRC_AT], &[0; 4], &m[PAK_HEADER_CRC_AT + 4..PAK_HEADER]]);
    put32(&mut m, PAK_HEADER_CRC_AT, header);
    Ok(m)
}

/// The AMPRIDX3 sort key: the folded path bytes (dump-forge's ampr_index.rs `key`).
fn key(path: &str) -> Vec<u8> {
    path.bytes().map(folded).collect()
}

/// `logical_path` + `kernel_path_limits` of ampr-format, for a path without the /app0/ prefix.
fn check_path(rel: &str) -> Result<()> {
    let full_len = APP0.len() + rel.len();
    let ok = !rel.is_empty()
        && full_len < PATH_MAX
        && rel.split('/').all(|c| {
            !c.is_empty() && c != "." && c != ".." && c.len() <= NAME_MAX && !c.contains(['\0', '\\'])
        });
    if ok { Ok(()) } else { Err(bad(format!("unindexable path {rel:?}"))) }
}

fn write_atomic(dir: &Path, name: &str, data: &[u8]) -> Result<()> {
    let tmp = dir.join(format!("{name}.tmp"));
    std::fs::write(&tmp, data)?;
    std::fs::rename(&tmp, dir.join(name))
}

/// Cheap integrity guard used by the scan: verify the required generated files
/// exist, are at least header length and have the expected magic. Do not run
/// full-file CRCs on the critical path or read game-owned cache files.
pub fn outputs_present(dir: &Path) -> bool {
    use std::io::Read;
    fn has_header(path: &Path, magic: &[u8; 8], min_len: usize) -> bool {
        let Ok(mut file) = std::fs::File::open(path) else { return false };
        if !file.metadata().is_ok_and(|m| m.is_file() && m.len() >= min_len as u64) {
            return false;
        }
        let mut bytes = [0u8; 8];
        file.read_exact(&mut bytes).is_ok() && &bytes == magic
    }
    has_header(&dir.join(INDEX_NAME), IDX_MAGIC, IDX_HEADER)
        && has_header(&dir.join(MANIFEST_NAME), PAK_MAGIC, PAK_HEADER)
}

/// Writes both index files into `dir` (the game's cache dir, seen as /app0).
/// `pkg` is the .pkg (ng opens the path in the runtime package descriptor): its size bounds the volume and its mtime goes in
/// every record.
pub fn write(dir: &Path, pkg: &Path, files: &[IndexFile]) -> std::io::Result<()> {
    let meta = std::fs::metadata(pkg)?;
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs() as i64);
    if files.is_empty() {
        return Err(bad("no files to index".into()));
    }
    let mut rows: Vec<(Vec<u8>, String, &IndexFile)> = Vec::with_capacity(files.len());
    for f in files {
        let rel = f.path.trim_start_matches('/');
        check_path(rel)?;
        let full = format!("{APP0}{rel}");
        rows.push((key(&full), full, f));
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    if let Some(w) = rows.windows(2).find(|w| w[0].0 == w[1].0) {
        return Err(bad(format!("{} and {} differ only in case", w[0].1, w[1].1)));
    }
    let pak_rows: Vec<(String, &IndexFile)> = rows.into_iter().map(|r| (r.1, r.2)).collect();
    let manifest = build_manifest(&pak_rows, mtime, meta.len())?;
    let index = build_index(&pak_rows, mtime);
    write_atomic(dir, MANIFEST_NAME, &manifest)?;
    write_atomic(dir, INDEX_NAME, &index)
}

#[cfg(test)]
mod perf_tests {
    use super::*;

    #[test]
    fn crc32_matches_reference_vector() {
        assert_eq!(crc32(&[b"123456789"]), 0xcbf4_3926);
    }

    #[test]
    fn index_and_manifest_keep_expected_layout() {
        let entry = IndexFile {
            path: "data/sample.bin".into(),
            size: BLOCK,
            chunks: Some(vec![Chunk {
                pkg_off: PAGE,
                stored_len: BLOCK as u32,
                raw_len: BLOCK as u32,
                compression: None,
            }]),
        };
        let rows = vec![(format!("{APP0}{}", entry.path), &entry)];
        let index = build_index(&rows, 123);
        assert_eq!(&index[..8], IDX_MAGIC);
        assert_eq!(u64::from_le_bytes(index[16..24].try_into().unwrap()), 1);
        assert_eq!(u64::from_le_bytes(index[24..32].try_into().unwrap()), (rows[0].0.len() + 1) as u64);
        let manifest = build_manifest(&rows, 123, 2 * 1024 * 1024).unwrap();
        assert_eq!(&manifest[..8], PAK_MAGIC);
        assert_eq!(u64::from_le_bytes(manifest[40..48].try_into().unwrap()), 1);
        assert_eq!(u64::from_le_bytes(manifest[48..56].try_into().unwrap()), 1);
        assert_eq!(u32::from_le_bytes(manifest[PAK_PAYLOAD_CRC_AT..PAK_PAYLOAD_CRC_AT + 4].try_into().unwrap()),
            crc32(&[&manifest[PAK_HEADER..]]));
    }
}
