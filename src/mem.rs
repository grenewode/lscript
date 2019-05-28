use std::borrow::{Borrow, BorrowMut};

pub type Word = u8;
pub type PtrType = u32;
pub type SizeType = u32;

#[derive(Debug)]
pub struct Mem([Word]);

impl Mem {
    pub fn to_buffer(&self) -> MemBuf {
        MemBuf(Vec::from(&self.0))
    }
}

#[derive(Debug, Clone)]
pub struct MemBuf(Vec<Word>);

impl<'a> From<&'a [Word]> for MemBuf {
    fn from(value: &'a [Word]) -> Self {
        MemBuf(Vec::from(value))
    }
}

impl<'a> From<&'a mut [Word]> for MemBuf {
    fn from(value: &'a mut [Word]) -> Self {
        MemBuf(Vec::from(value))
    }
}

pub enum MemError {
    OutOfMemory,
}

impl AsRef<Mem> for [Word] {
    fn as_ref(&self) -> &Mem {
        unsafe { &*(self as *const _ as *const Mem) }
    }
}

impl AsRef<[Word]> for Mem {
    fn as_ref(&self) -> &[Word] {
        &self.0
    }
}

impl AsRef<Mem> for Mem {
    fn as_ref(&self) -> &Mem {
        self
    }
}

impl AsMut<Mem> for [Word] {
    fn as_mut(&mut self) -> &mut Mem {
        unsafe { &mut *(self as *mut _ as *mut Mem) }
    }
}

impl AsMut<[Word]> for Mem {
    fn as_mut(&mut self) -> &mut [Word] {
        &mut self.0
    }
}

impl AsMut<Mem> for Mem {
    fn as_mut(&mut self) -> &mut Mem {
        self
    }
}

impl AsRef<Mem> for MemBuf {
    fn as_ref(&self) -> &Mem {
        self.0.as_slice().as_ref()
    }
}

impl AsRef<[Word]> for MemBuf {
    fn as_ref(&self) -> &[Word] {
        self.0.as_slice()
    }
}

impl AsMut<Mem> for MemBuf {
    fn as_mut(&mut self) -> &mut Mem {
        self.0.as_mut_slice().as_mut()
    }
}

impl AsMut<[Word]> for MemBuf {
    fn as_mut(&mut self) -> &mut [Word] {
        self.0.as_mut_slice()
    }
}

impl Borrow<Mem> for MemBuf {
    fn borrow(&self) -> &Mem {
        self.as_ref()
    }
}

impl BorrowMut<Mem> for MemBuf {
    fn borrow_mut(&mut self) -> &mut Mem {
        self.as_mut()
    }
}

impl ToOwned for Mem {
    type Owned = MemBuf;

    fn to_owned(&self) -> Self::Owned {
        MemBuf(Vec::from(&self.0))
    }
}

macro_rules! array_as_ref {
    ($($size:expr),*) => {
        $(
            impl AsRef<Mem> for [u8; $size] {
                fn as_ref(&self) -> &Mem {
                    (self as &[_]).as_ref()
                }
            }

            impl AsMut<Mem> for [u8; $size] {
                fn as_mut(&mut self) -> &mut Mem {
                    (self as &mut [_]).as_mut()
                }
            }
        )*
    };
}

array_as_ref!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32
);

pub trait MemLike: AsRef<Mem> + AsMut<Mem> {
    fn size(&self) -> SizeType;

    fn load<T>(&self, start: PtrType) -> Result<(PtrType, T), MemError>
    where
        T: Loadable,
    {
        T::load_from(self.as_ref(), start)
    }

    fn store<T>(&mut self, start: PtrType, value: &T) -> Result<PtrType, MemError>
    where
        T: Storable,
    {
        value.store_to(self.as_mut(), start)
    }
}

impl MemLike for Mem {
    fn size(&self) -> SizeType {
        self.0.len() as SizeType
    }
}

pub trait Storable {
    fn store_to(&self, target: &mut Mem, start: PtrType) -> Result<PtrType, MemError>;
}

pub trait Loadable: Sized {
    fn load_from(m: &Mem, start: PtrType) -> Result<(PtrType, Self), MemError>;
}

impl Storable for Mem {
    fn store_to(&self, target: &mut Mem, start: PtrType) -> Result<PtrType, MemError> {
        let start = start as usize;
        let end = start + self.size() as usize;

        target
            .0
            .get_mut(start..end)
            .ok_or(MemError::OutOfMemory)?
            .copy_from_slice(&self.0);

        Ok(end as PtrType)
    }
}

impl Loadable for MemBuf {
    fn load_from(src: &Mem, start: PtrType) -> Result<(PtrType, Self), MemError> {
        let start = start as usize;
        let end = src.size();

        src.0
            .get(start..)
            .map(MemBuf::from)
            .map(|chunk| (end, chunk))
            .ok_or(MemError::OutOfMemory)
    }
}

macro_rules! array_as_ls {
    ($($size:expr),*) => {
        $(
            impl Storable for [u8; $size] {
                fn store_to(&self, dest: &mut Mem, start: PtrType) -> Result<PtrType, MemError> {
                    (self.as_ref() as &Mem).store_to(dest, start)
                }
            }

            impl Loadable for [u8; $size] {
                fn load_from(src: &Mem, start: PtrType) -> Result<(PtrType, Self), MemError> {
                    use std::convert::TryInto;
                    let start = start as usize;
                    let end = start + $size;

                    src.0.get(start..end).ok_or(MemError::OutOfMemory)?.try_into().map(|array| (end as PtrType, array)).map_err(|_| MemError::OutOfMemory)
                }
            }
        )*
    };
}

array_as_ls!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32
);

macro_rules! unumber_as_ls {
    ($($number:ty),*) => {{
        impl Storable for $number {
            fn store(&mut self, start: PtrType, m: &mut impl MemLike) -> Result<PtrType, MemError> {
                self.to_le_bytes().store(start, m)
            }
        }

        impl Loadable for $number {
            fn load(start: PtrType, m: &impl MemLike) -> Result<PtrType, MemError> {
                self.from_le_bytes().store(start, m)
            }
        }
    }};
}