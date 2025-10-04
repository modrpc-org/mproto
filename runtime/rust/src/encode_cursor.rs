use crate::Encode;

pub struct EncodeCursor<'a> {
    base_cursor: BufferEncodeCursor<'a>,
    scratch_cursor: BufferEncodeCursor<'a>,
    scratch_offset: u32,
}

impl<'a> EncodeCursor<'a> {
    pub fn new<T: Encode + ?Sized>(buffer: &'a mut [u8]) -> Self {
        let (base_buffer, scratch_buffer) = buffer.split_at_mut(T::BASE_LEN);

        Self {
            base_cursor: BufferEncodeCursor::new(base_buffer),
            scratch_cursor: BufferEncodeCursor::new(scratch_buffer),
            scratch_offset: T::BASE_LEN as u32,
        }
    }

    pub fn encoded_len(&self) -> usize {
        self.scratch_offset as usize
    }

    pub fn base(&mut self, size: usize) -> &'a mut [u8] {
        self.base_cursor.take(size)
    }

    pub fn scratch(&mut self, size: usize) -> &'a mut [u8] {
        // Write the offset of this scratch buffer into the base buffer.
        self.base(4)
            .copy_from_slice(&self.scratch_offset.to_le_bytes());
        self.scratch_offset += size as u32;

        self.scratch_cursor.take(size)
    }

    pub fn inner_in_scratch(&mut self, base_size: usize, f: impl FnOnce(&mut Self)) {
        // https://github.com/alecmocatta/replace_with/blob/1e86e4a3633c133cd3a8f797dc06ac92401bff1e/src/lib.rs#L478
        fn replace_with<T, F: FnOnce(T) -> T>(dest: &mut T, f: F) {
            unsafe {
                let new = f(core::ptr::read(dest));
                core::ptr::write(dest, new);
            }
        }

        replace_with(self, move |s| Self::_inner_in_scratch(s, base_size, f));
    }

    fn _inner_in_scratch(mut self, base_size: usize, f: impl FnOnce(&mut Self)) -> Self {
        let inner_base_buffer = self.scratch(base_size);

        let Self {
            base_cursor,
            scratch_cursor,
            scratch_offset,
        } = self;

        let mut inner = Self {
            base_cursor: BufferEncodeCursor::new(inner_base_buffer),
            scratch_cursor,
            scratch_offset,
        };

        f(&mut inner);

        Self {
            base_cursor,
            scratch_cursor: inner.scratch_cursor,
            scratch_offset: inner.scratch_offset,
        }
    }
}

struct BufferEncodeCursor<'a> {
    buffer: &'a mut [u8],
}

impl<'a> BufferEncodeCursor<'a> {
    fn new(buffer: &'a mut [u8]) -> Self {
        Self { buffer }
    }

    fn take(&mut self, size: usize) -> &'a mut [u8] {
        // Use a little unsafe to make the compiler forget that it got `buffer` from self. This is
        // safe because we immediately replace `self.buffer` with the remaining buffer after
        // `split_at_mut`.
        let buffer: &'a mut [u8] =
            unsafe { core::slice::from_raw_parts_mut(self.buffer.as_mut_ptr(), self.buffer.len()) };
        let (taken_buffer, remaining) = buffer.split_at_mut(size);
        self.buffer = remaining;

        taken_buffer
    }
}
