use std::io::Read;

pub fn crc32<R: Read>(mut reader: R) -> std::io::Result<u32> {
    let mut hasher = crc32fast::Hasher::new();
    let mut chunk = [0u8; 512];
    loop {
        match reader.read(&mut chunk)? {
            0 => break,
            other => hasher.update(&chunk[0..other])
        }
    }
    Ok(hasher.finalize())
}