#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

use core::{ops::Deref, pin::Pin};

pub use boxed::BoxLazy;
pub use decode_cursor::DecodeCursor;
pub use encode_cursor::EncodeCursor;
pub use list::{ListGen, ListLazy};

mod boxed;
mod copy_primitives;
mod decode_cursor;
mod encode_cursor;
mod list;
mod option;
mod result;
mod string;
#[cfg(test)]
mod tests;

pub trait BaseLen {
    const BASE_LEN: usize;
}

pub trait Encode: BaseLen {
    fn scratch_len(&self) -> usize;

    fn encode(&self, cursor: &mut EncodeCursor);
}

#[derive(Debug)]
pub struct DecodeError;

impl core::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "failed to decode mproto value")
    }
}

impl core::error::Error for DecodeError {}

pub type DecodeResult<T> = Result<T, DecodeError>;

pub trait Decode<'a>: BaseLen + Sized {
    fn decode(cursor: &DecodeCursor<'a>) -> DecodeResult<Self>;
}

pub trait Compatible<Other: ?Sized>: Encode {}

pub trait Owned: Encode + for<'a> Decode<'a> + Compatible<Self> + Clone + Send + Sync + 'static {
    type Lazy<'a>: Lazy<'a, Owned = Self>;

    fn lazy_to_owned(lazy: Self::Lazy<'_>) -> DecodeResult<Self>;
}

pub trait Lazy<'a>: Encode + Decode<'a> + Copy + Clone + PartialEq + core::fmt::Debug {
    type Owned: Owned<Lazy<'a> = Self>;
}

pub fn encoded_len<T: Encode>(value: T) -> usize {
    T::BASE_LEN + value.scratch_len()
}

pub fn encode_value<E: Encode>(v: E, mut buf: impl AsMut<[u8]>) -> usize {
    let mut cursor = EncodeCursor::new::<E>(buf.as_mut());
    v.encode(&mut cursor);
    cursor.encoded_len()
}

#[cfg(any(feature = "std", feature = "alloc"))]
pub fn encode_value_vec<E: Encode>(v: E) -> Vec<u8> {
    let mut buf = vec![0u8; encoded_len(&v)];
    let mut cursor = EncodeCursor::new::<E>(buf.as_mut());
    v.encode(&mut cursor);
    buf
}

pub fn decode_value<'a, D: Decode<'a>>(buf: &'a [u8]) -> DecodeResult<D> {
    Decode::decode(&DecodeCursor::new(buf))
}

// TODO Consider using Yoke, either directly in apps or wrapping it here.
pub struct LazyBuf<T: Owned, B> {
    buf: Pin<B>,
    lazy: T::Lazy<'static>,
}

mod sealed {
    // workaround for compiler limitation
    // https://github.com/rust-lang/rust/issues/49601#issuecomment-1007884546
    pub trait LazyBufMapFn<T, U>: FnOnce(T) -> U {}
    impl<F, T, U> LazyBufMapFn<T, U> for F where F: FnOnce(T) -> U {}
}

impl<T: Owned, B: Deref<Target = [u8]> + core::marker::Unpin> LazyBuf<T, B> {
    pub fn new(buf: B) -> Self {
        let buf = Pin::new(buf);
        let lazy: T::Lazy<'_> = decode_value(buf.as_ref().get_ref().as_ref()).unwrap();
        // TODO is this actually safe to do?
        // Erase lifetime of lazy value
        let lazy = unsafe { core::mem::transmute(lazy) };
        Self { buf, lazy }
    }

    pub fn get<'a>(&'a self) -> T::Lazy<'a> {
        // TODO is this actually safe to do?
        unsafe { core::mem::transmute(self.lazy.clone()) }
    }

    pub fn map<U: Owned, F>(self, f: F) -> LazyBuf<U, B>
    where
        F: for<'a> sealed::LazyBufMapFn<T::Lazy<'a>, U::Lazy<'a>>,
    {
        let lazy = f(self.lazy);
        // TODO is this actually safe to do?
        // Erase lifetime of lazy value
        let lazy = unsafe { core::mem::transmute(lazy) };
        LazyBuf {
            buf: self.buf,
            lazy,
        }
    }
}

impl<T, U: Compatible<T> + ?Sized> Compatible<T> for &U {}

impl<T: BaseLen + ?Sized> BaseLen for &T {
    const BASE_LEN: usize = T::BASE_LEN;
}

impl<T: Encode + ?Sized> Encode for &T {
    fn scratch_len(&self) -> usize {
        T::scratch_len(self)
    }

    fn encode(&self, cursor: &mut EncodeCursor) {
        T::encode(self, cursor);
    }
}

impl<T: BaseLen + ?Sized> BaseLen for &mut T {
    const BASE_LEN: usize = T::BASE_LEN;
}

impl<T: Encode + ?Sized> Encode for &mut T {
    fn scratch_len(&self) -> usize {
        T::scratch_len(self)
    }

    fn encode(&self, cursor: &mut EncodeCursor) {
        T::encode(self, cursor);
    }
}

pub const fn max(a: usize, b: usize) -> usize {
    [a, b][(a < b) as usize]
}
