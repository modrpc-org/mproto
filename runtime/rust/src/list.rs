use crate::{BaseLen, Compatible, Decode, DecodeCursor, DecodeError, DecodeResult, Encode, EncodeCursor, Owned};

#[cfg(any(feature = "std", feature = "alloc"))]
use crate::Lazy;

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: Owned + Compatible<T>> Owned for Vec<T> {
    type Lazy<'a> = ListLazy<'a, T>;

    fn lazy_to_owned(lazy: Self::Lazy<'_>) -> DecodeResult<Self> {
        lazy.try_into()
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a, T: Owned + Compatible<T>> Lazy<'a> for ListLazy<'a, T> {
    type Owned = Vec<T>;
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a, T, U> core::convert::TryFrom<ListLazy<'a, T>> for Vec<U>
where
    T: Owned,
    U: Decode<'a> + Compatible<T>,
{
    type Error = DecodeError;

    fn try_from(other: ListLazy<'a, T>) -> Result<Self, Self::Error> {
        let cursor = DecodeCursor::at_offset(other.buffer, other.offset);
        Decode::decode(&cursor)
    }
}

impl<T: BaseLen> BaseLen for [T] {
    const BASE_LEN: usize = 4 + 4;
}

impl<T: Encode> Encode for [T] {
    fn scratch_len(&self) -> usize {
        self.len() * T::BASE_LEN + self.iter().fold(0, |sum, item| sum + item.scratch_len())
    }

    fn encode(&self, cursor: &mut EncodeCursor) {
        cursor
            .base(4)
            .copy_from_slice(&(self.len() as u32).to_le_bytes());

        cursor.inner_in_scratch(self.len() * T::BASE_LEN, |cursor| {
            for item in self {
                item.encode(cursor);
            }
        });
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T, U: Compatible<T>> Compatible<Vec<T>> for Vec<U> {}
#[cfg(any(feature = "std", feature = "alloc"))]
impl<T, U: Compatible<T>> Compatible<Vec<T>> for [U] {}
#[cfg(any(feature = "std", feature = "alloc"))]
impl<T, U: Compatible<T>> Compatible<[T]> for Vec<U> {}

impl<'a> Decode<'a> for &'a [u8] {
    fn decode(cursor: &DecodeCursor<'a>) -> DecodeResult<Self> {
        let len = u32::from_le_bytes(cursor.base(4).try_into().map_err(|_| DecodeError)?) as usize;

        Ok(cursor.scratch(len))
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: BaseLen> BaseLen for Vec<T> {
    const BASE_LEN: usize = 4 + 4;
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: Encode> Encode for Vec<T> {
    fn scratch_len(&self) -> usize {
        self.len() * T::BASE_LEN + self.iter().fold(0, |sum, item| sum + item.scratch_len())
    }

    fn encode(&self, cursor: &mut EncodeCursor) {
        cursor
            .base(4)
            .copy_from_slice(&(self.len() as u32).to_le_bytes());

        cursor.inner_in_scratch(self.len() * T::BASE_LEN, |cursor| {
            for item in self {
                item.encode(cursor);
            }
        });
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a, T: Decode<'a>> Decode<'a> for Vec<T> {
    fn decode(cursor: &DecodeCursor<'a>) -> DecodeResult<Self> {
        let len = u32::from_le_bytes(cursor.base(4).try_into().map_err(|_| DecodeError)?) as usize;

        cursor.inner_in_scratch(|cursor| {
            let mut vec = Vec::with_capacity(len);
            for _ in 0..len {
                vec.push(T::decode(cursor)?);
            }
            Ok(vec)
        })
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a, T, U> PartialEq<ListLazy<'a, T>> for Vec<U>
where
    T: Owned,
    U: PartialEq<T::Lazy<'a>>,
{
    fn eq(&self, other: &ListLazy<'a, T>) -> bool {
        for (i, item) in self.iter().enumerate() {
            if *item != other.get(i).unwrap() {
                return false;
            }
        }

        true
    }
}

pub struct ListGen<I: ExactSizeIterator>(pub I);

impl<I: ExactSizeIterator> BaseLen for ListGen<I> {
    const BASE_LEN: usize = 4 + 4;
}

impl<I: Clone + ExactSizeIterator> Encode for ListGen<I>
where
    I::Item: Encode,
{
    fn scratch_len(&self) -> usize {
        self.0.len() * I::Item::BASE_LEN
            + self.0.clone().fold(0, |sum, item| sum + item.scratch_len())
    }

    fn encode(&self, cursor: &mut EncodeCursor) {
        cursor
            .base(4)
            .copy_from_slice(&(self.0.len() as u32).to_le_bytes());

        cursor.inner_in_scratch(self.0.len() * I::Item::BASE_LEN, |cursor| {
            for item in self.0.clone() {
                item.encode(cursor);
            }
        });
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: Owned, U: Compatible<T>, I: Clone + ExactSizeIterator<Item = U>> Compatible<Vec<T>>
    for ListGen<I>
{
}

pub struct ListLazy<'a, T> {
    buffer: &'a [u8],
    offset: usize,
    item_ty: core::marker::PhantomData<T>,
}

impl<T: Owned, U: Compatible<T>> Compatible<[U]> for ListLazy<'_, T> {}
impl<T: Owned, U: Compatible<T>> Compatible<ListLazy<'_, T>> for [U] {}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: Owned, U: Compatible<T>> Compatible<Vec<U>> for ListLazy<'_, T> {}
#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: Owned, U: Compatible<T>> Compatible<ListLazy<'_, T>> for Vec<U> {}

impl<'a, T: Owned> BaseLen for ListLazy<'a, T> {
    const BASE_LEN: usize = 4 + 4;
}

impl<'a, T: Owned> Encode for ListLazy<'a, T> {
    fn scratch_len(&self) -> usize {
        self.len() * T::BASE_LEN + self.iter().fold(0, |sum, item| sum + item.scratch_len())
    }

    fn encode(&self, cursor: &mut EncodeCursor) {
        cursor
            .base(4)
            .copy_from_slice(&(self.len() as u32).to_le_bytes());

        cursor.inner_in_scratch(self.len() * T::BASE_LEN, |cursor| {
            for item in self {
                item.encode(cursor);
            }
        });
    }
}

impl<'a, T: Owned> ListLazy<'a, T> {
    pub fn len(&self) -> usize {
        u32::from_le_bytes(
            self.buffer[self.offset..self.offset + 4]
                .try_into()
                .unwrap(),
        ) as usize
    }

    pub fn get(&self, index: usize) -> DecodeResult<T::Lazy<'a>> {
        if index >= self.len() {
            return Err(DecodeError);
        }

        let list_scratch_offset = u32::from_le_bytes(
            self.buffer[self.offset + 4..self.offset + 8]
                .try_into()
                .map_err(|_| DecodeError)?,
        ) as usize;

        Decode::decode(&DecodeCursor::at_offset(
            self.buffer,
            list_scratch_offset + (index * T::BASE_LEN),
        ))
    }

    pub fn iter<'s>(&'s self) -> ListLazyIter<'s, 'a, T> {
        let len = self.len();
        ListLazyIter {
            list_lazy: self,
            len,
            cursor: 0,
        }
    }
}

impl<'a, T: Owned> Decode<'a> for ListLazy<'a, T> {
    fn decode(cursor: &DecodeCursor<'a>) -> DecodeResult<Self> {
        let offset = cursor.offset();
        cursor.advance(Self::BASE_LEN);
        Ok(ListLazy {
            buffer: cursor.buffer(),
            offset,
            item_ty: core::marker::PhantomData,
        })
    }
}

impl<'s, 'a, T: Owned> IntoIterator for &'s ListLazy<'a, T> {
    type Item = T::Lazy<'a>;
    type IntoIter = ListLazyIter<'s, 'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        let len = self.len();
        ListLazyIter {
            list_lazy: self,
            len,
            cursor: 0,
        }
    }
}

pub struct ListLazyIter<'s, 'a, T> {
    list_lazy: &'s ListLazy<'a, T>,
    len: usize,
    cursor: usize,
}

impl<'s, 'a, T: Owned> Iterator for ListLazyIter<'s, 'a, T> {
    type Item = T::Lazy<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.cursor;
        self.cursor += 1;

        if i < self.len {
            Some(self.list_lazy.get(i).unwrap())
        } else {
            None
        }
    }
}

impl<'a, T, const N: usize> core::convert::TryInto<[T::Lazy<'a>; N]> for ListLazy<'a, T>
where
    T: Owned + Clone,
    for<'b> T::Lazy<'b>: Sized,
{
    type Error = ();

    fn try_into(self) -> Result<[T::Lazy<'a>; N], Self::Error> {
        use core::mem::MaybeUninit;

        if self.len() != N {
            return Err(());
        }

        let mut out: [MaybeUninit<T::Lazy<'_>>; N] =
            [const { MaybeUninit::<T::Lazy<'_>>::uninit() }; N];
        for (out, item) in out.iter_mut().zip(self.iter()) {
            out.write(item);
        }

        // TODO when stable
        // SAFETY: we just initialized all items in `out` in the above for-loop.
        //Ok(unsafe { MaybeUninit::array_assume_init(out) })
        // SAFETY:
        // - we just initialized all items in `out` in the above for-loop.
        // - MaybeUninit<T> and T are guaranteed to have the same layout.
        // - MaybeUninit<T> does not drop, so there are no double-frees.
        Ok(unsafe { core::mem::transmute_copy(&out) })
    }
}

impl<'a> From<ListLazy<'a, u8>> for &'a [u8] {
    fn from(other: ListLazy<'a, u8>) -> Self {
        let list_scratch_offset = u32::from_le_bytes(
            other.buffer[other.offset + 4..other.offset + 8]
                .try_into()
                .unwrap(),
        ) as usize;
        &other.buffer[list_scratch_offset..list_scratch_offset + other.len()]
    }
}

impl<T: Owned> Copy for ListLazy<'_, T> {}
impl<T: Owned> Clone for ListLazy<'_, T> {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer,
            offset: self.offset,
            item_ty: core::marker::PhantomData,
        }
    }
}
impl<T> core::fmt::Debug for ListLazy<'_, T>
where
    T: Owned,
    for<'a> T::Lazy<'a>: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[")?;
        for i in 0..self.len() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:?}", self.get(i))?;
        }
        write!(f, "]")?;
        Ok(())
    }
}

impl<T> PartialEq for ListLazy<'_, T>
where
    T: Owned,
    for<'a> T::Lazy<'a>: PartialEq,
{
    /// Panics if decoding an item from either `ListLazy` fails.
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for (item, other_item) in self.iter().zip(other.iter()) {
            if !item.eq(&other_item) {
                return false;
            }
        }
        true
    }
}

impl<T> Eq for ListLazy<'_, T>
where
    T: Owned,
    for<'a> T::Lazy<'a>: Eq,
{
}
