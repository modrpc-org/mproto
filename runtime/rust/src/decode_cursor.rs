use core::cell::Cell;

pub struct DecodeCursor<'a> {
    buffer: &'a [u8],
    offset: Cell<usize>,
}

impl<'a> DecodeCursor<'a> {
    #[inline]
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            offset: Cell::new(0),
        }
    }

    #[inline]
    pub fn buffer(&self) -> &'a [u8] {
        self.buffer
    }

    #[inline]
    pub fn offset(&self) -> usize {
        self.offset.get()
    }

    #[inline]
    pub fn at_offset(buffer: &'a [u8], offset: usize) -> Self {
        Self {
            buffer,
            offset: Cell::new(offset),
        }
    }

    #[inline]
    pub fn base(&self, size: usize) -> &'a [u8] {
        let offset = self.offset.get();
        self.offset.set(self.offset.get() + size);
        &self.buffer[offset..offset + size]
    }

    #[inline]
    pub fn scratch(&self, size: usize) -> &'a [u8] {
        // Read the offset of this scratch buffer from the base buffer.
        let offset = u32::from_le_bytes(self.base(4).try_into().unwrap()) as usize;

        &self.buffer[offset..offset + size]
    }

    #[inline]
    pub fn inner_in_scratch<R>(&self, f: impl FnOnce(&Self) -> R) -> R {
        // Read the offset of this scratch buffer from the base buffer.
        let offset = u32::from_le_bytes(self.base(4).try_into().unwrap()) as usize;

        let inner_cursor = Self::at_offset(self.buffer, offset);
        f(&inner_cursor)
    }

    #[inline]
    pub fn advance(&self, offset: usize) {
        self.offset.set(self.offset.get() + offset);
    }

    #[inline]
    pub fn follow_scratch(&self) {
        let offset = self.offset.get();
        let scratch_offset =
            u32::from_le_bytes(self.buffer[offset..offset + 4].try_into().unwrap()) as usize;
        self.offset.set(scratch_offset as usize);
    }
}
