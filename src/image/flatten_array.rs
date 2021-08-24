use std::ops::{Index, IndexMut};

pub(super) struct FlattenArray<T: Copy> {
    width: usize,
    buf: Vec<T>,
}

impl<T: Copy> FlattenArray<T> {
    pub fn new(width: usize, height: usize, value: T) -> FlattenArray<T> {
        FlattenArray {
            width,
            buf: vec![value; width * height],
        }
    }

    #[inline]
    pub fn from_vec(width: usize, buf: Vec<T>) -> FlattenArray<T> {
        FlattenArray { width, buf }
    }

    #[inline]
    pub fn to_vec(&self) -> Vec<T> {
        self.buf.clone()
    }

    #[inline]
    pub fn into_vec(self) -> Vec<T> {
        self.buf
    }

    #[inline]
    pub fn values(&self, y: usize, x: usize, len: usize) -> &[T] {
        let head = y * self.width + x;
        let tail = y * self.width + (x + len).min(self.width);
        &self.buf[head..tail]
    }
}

impl<T: Copy> Index<(usize, usize)> for FlattenArray<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.buf[index.0 * self.width + index.1]
    }
}

impl<T: Copy> IndexMut<(usize, usize)> for FlattenArray<T> {
    #[inline]
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.buf[index.0 * self.width + index.1]
    }
}
