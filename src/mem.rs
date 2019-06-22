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

#[derive(Debug)]
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

    fn load_and_advance<T>(&self, start: &mut PtrType) -> Result<T, MemError>
    where
        T: Loadable,
    {
        let (end, value) = T::load_from(self.as_ref(), *start)?;
        *start = end;
        Ok(value)
    }

    fn store<T>(&mut self, start: PtrType, value: &T) -> Result<PtrType, MemError>
    where
        T: Storable,
    {
        value.store_to(self.as_mut(), start)
    }

    fn store_and_advance<T>(&mut self, start: &mut PtrType, value: &T) -> Result<(), MemError>
    where
        T: Storable,
    {
        *start = self.store(*start, value)?;
        Ok(())
    }
}

impl MemLike for Mem {
    fn size(&self) -> SizeType {
        self.0.len() as SizeType
    }
}

impl MemLike for MemBuf {
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

macro_rules! number_as_ls {
    ($($number:ty),*) => {$(
        impl Storable for $number {
            fn store_to(&self, dest: &mut Mem, start: PtrType) -> Result<PtrType, MemError> {
                self.to_le_bytes().store_to(dest, start)
            }
        }

        impl Loadable for $number {
            fn load_from(src: &Mem, start: PtrType) -> Result<(PtrType, Self), MemError> {
                <[u8; ::std::mem::size_of::<$number>()]>::load_from(src, start)
                .map(|(end, bytes)| (end, <$number>::from_le_bytes(bytes)))
            }
        }
    )*};
}

number_as_ls!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

#[macro_export]
macro_rules! typedef(
    (enum $name:ident {
        $(
            $variant_name:ident ($variant_value:ty),
        )*
    }) => { typedef!(enum $name { $($variant_name($variant_value)),* }); };
    (enum $name:ident {
        $(
            $variant_name:ident ($variant_value:ty)
        ),*
    }) => {
        enum $name {
            $(
                $variant_name($variant_value)
            ),*
        }

        impl Storable for $name {
            fn store_to(&self, target: &mut Mem, start: PtrType) -> Result<PtrType, MemError> {
                let mut start = start;
                let mut idx = 0u8;
                $(
                    if let $name::$variant_name(value) = self {
                        target.store_and_advance(&mut start, &idx)?;
                        target.store_and_advance(&mut start, value)?;

                        return Ok(start);
                    }

                    idx += 1;
                )*

                unreachable!("we should have hit one of the variants by now");
            }
        }

        impl Loadable for $name {
            fn load_from(m: &Mem, start: PtrType) -> Result<(PtrType, Self), MemError> {
                let mut start = start;
                let mut idx = 0u8;
                let variant: u8 = m.load_and_advance(&mut start)?;
                $(
                    if idx == variant {
                        let value = m.load_and_advance(&mut start)
                            .map($name::$variant_name)?;
                        return Ok((start, value));
                    }
                    idx += 1;
                )*

                unreachable!("we should have hit one of the variants by now");
            }
        }
    };

    (struct $name:ident {
        $(
            $field_name:ident : $field_type:ty
        ),*
    }) => {
        struct $name {
            $(
                pub $field_name:$field_type
            )*
        }

        impl Storable for $name {
            fn store_to(&self, target: &mut Mem, start: PtrType) -> Result<PtrType, MemError> {
                let mut start = start;
                $(
                    target.store_and_advance(&mut start, &self.$field_name)?;
                )*
                Ok(start)
            }
        }

        impl Loadable for $name {
            fn load_from(m: &Mem, start: PtrType) -> Result<(PtrType, Self), MemError> {
                let mut start = start;
                let value = Self { $(
                    $field_name : m.load_and_advance(&mut start)?
                ),*};
                Ok((start, value))
            }
        }
    }
);
