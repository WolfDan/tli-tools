use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

pub(crate) fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

pub(crate) fn fixture_path(name: &str) -> PathBuf {
    workspace_root().join("tests").join("fixtures").join(name)
}

pub(crate) fn load_png_rgba8(path: &Path) -> (usize, usize, Vec<u8>) {
    let file =
        File::open(path).unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    let mut decoder = png::Decoder::new(BufReader::new(file));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::ALPHA);
    let mut reader = decoder
        .read_info()
        .unwrap_or_else(|e| panic!("failed to read PNG header in {}: {e}", path.display()));
    let buf_size = reader
        .output_buffer_size()
        .unwrap_or_else(|| panic!("PNG output buffer size overflow in {}", path.display()));
    let mut buf = vec![0u8; buf_size];
    let info = reader
        .next_frame(&mut buf)
        .unwrap_or_else(|e| panic!("failed to decode PNG {}: {e}", path.display()));
    assert_eq!(
        info.color_type,
        png::ColorType::Rgba,
        "expected RGBA8 output for {}",
        path.display()
    );
    assert_eq!(
        info.bit_depth,
        png::BitDepth::Eight,
        "expected 8-bit output for {}",
        path.display()
    );
    buf.truncate(info.buffer_size());
    (info.width as usize, info.height as usize, buf)
}
