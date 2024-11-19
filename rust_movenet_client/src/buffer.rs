use std::ptr::NonNull;

pub struct Buffer {
    pub start: NonNull<u8>,
    pub length: usize,
}
