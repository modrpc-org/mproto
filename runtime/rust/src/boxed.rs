use crate::{BaseLen, Decode, DecodeCursor, DecodeResult, Encode, EncodeCursor, Owned};

#[cfg(any(feature = "std", feature = "alloc"))]
use crate::{Compatible, Lazy};

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: Owned> Owned for Box<T> {
    type Lazy<'a> = BoxLazy<'a, T>;

    fn lazy_to_owned(lazy: Self::Lazy<'_>) -> DecodeResult<Self> {
        Ok(Box::new(T::lazy_to_owned(lazy.get()?)?))
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a, T: Owned> Lazy<'a> for BoxLazy<'a, T> {
    type Owned = Box<T>;
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: BaseLen> BaseLen for Box<T> {
    const BASE_LEN: usize = 4;
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: Encode> Encode for Box<T> {
    fn scratch_len(&self) -> usize {
        use core::ops::Deref;
        T::BASE_LEN + self.deref().scratch_len()
    }

    fn encode(&self, cursor: &mut EncodeCursor) {
        use core::ops::Deref;
        cursor.inner_in_scratch(T::BASE_LEN, |cursor| self.deref().encode(cursor));
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a, T: BaseLen + Decode<'a>> Decode<'a> for Box<T> {
    fn decode(cursor: &DecodeCursor<'a>) -> DecodeResult<Self> {
        let inner = cursor.inner_in_scratch(|cursor| T::decode(cursor))?;
        Ok(Box::new(inner))
    }
}

pub struct BoxLazy<'a, T: Owned> {
    buffer: &'a [u8],
    offset: usize,
    inner_ty: core::marker::PhantomData<T>,
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: Owned> Compatible<Box<T>> for BoxLazy<'_, T> {}
#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: Owned> Compatible<BoxLazy<'_, T>> for Box<T> {}

impl<'a, T: Owned> BaseLen for BoxLazy<'a, T> {
    const BASE_LEN: usize = 4;
}

impl<'a, T: Owned> Encode for BoxLazy<'a, T> {
    fn scratch_len(&self) -> usize {
        T::BASE_LEN
    }

    fn encode(&self, cursor: &mut EncodeCursor) {
        cursor.inner_in_scratch(T::BASE_LEN, |cursor| {
            self.get().unwrap().encode(cursor);
        });
    }
}

impl<'a, T: Owned> BoxLazy<'a, T> {
    pub fn get(&self) -> DecodeResult<T::Lazy<'a>> {
        DecodeCursor::at_offset(self.buffer, self.offset)
            .inner_in_scratch(|cursor| Decode::decode(cursor))
    }
}

impl<'a, T: Owned> Decode<'a> for BoxLazy<'a, T> {
    fn decode(cursor: &DecodeCursor<'a>) -> DecodeResult<Self> {
        Ok(BoxLazy {
            buffer: cursor.buffer(),
            offset: cursor.offset(),
            inner_ty: core::marker::PhantomData,
        })
    }
}

impl<T: Owned> Copy for BoxLazy<'_, T> {}
impl<T: Owned> Clone for BoxLazy<'_, T> {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer,
            offset: self.offset,
            inner_ty: core::marker::PhantomData,
        }
    }
}
impl<T> core::fmt::Debug for BoxLazy<'_, T>
where
    T: Owned,
    for<'a> T::Lazy<'a>: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.get())
    }
}

impl<T: Owned> PartialOrd for BoxLazy<'_, T>
where
    for<'a> T::Lazy<'a>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        let this = self.get().ok()?;
        let other = other.get().ok()?;
        this.partial_cmp(&other)
    }
}

impl<T: Ord> Ord for BoxLazy<'_, T>
where
    T: Owned,
    for<'a> T::Lazy<'a>: Ord,
{
    /// Panics if decoding either `BoxLazy` fails.
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.get().unwrap().cmp(&other.get().unwrap())
    }
}

impl<T> PartialEq for BoxLazy<'_, T>
where
    T: Owned,
    for<'a> T::Lazy<'a>: PartialEq,
{
    /// Panics if decoding either `BoxLazy` fails.
    fn eq(&self, other: &Self) -> bool {
        self.get().unwrap().eq(&other.get().unwrap())
    }
}

impl<T> Eq for BoxLazy<'_, T>
where
    T: Owned,
    for<'a> T::Lazy<'a>: Eq,
{
}
