use bytes::Bytes;

fn tail_offset(bytes: &Bytes, begin: usize) -> usize {
    assert!(begin <= bytes.len(), "begin must stay within the buffer");

    if begin == bytes.len() {
        who::warn!(
            dependency("bytes").changed_from("1.9.0"),
            "Recheck tokio-rs/bytes#779 and tokio-rs/bytes#780; bytes 1.11.0 guarantees addresses for empty slice() results, so this explicit end-of-buffer branch may be removable."
        );
        return bytes.len();
    }

    let tail = bytes.slice(begin..);
    tail.as_ptr() as usize - bytes.as_ptr() as usize
}

fn main() {
    let bytes = Bytes::from_static(b"Hi");
    let offsets: Vec<_> = (0..=bytes.len()).map(|index| tail_offset(&bytes, index)).collect();

    assert_eq!(offsets, vec![0, 1, 2]);
    println!("tail offsets: {offsets:?}");
}
