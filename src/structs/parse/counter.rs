use bincode::{Decode, Encode};
use derive_deref::{Deref, DerefMut};

#[derive(Debug, Deref, DerefMut, Default, Decode, Encode, Clone, Copy)]
pub struct Counter(u32);

impl Counter {
    #[inline(always)]
    pub fn increment(&mut self) {
        self.0 += 1;
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn decrement(&mut self) {
        self.0 -= 1;
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn reset(&mut self) {
        self.0 = 0;
    }

    #[inline(always)]
    pub fn inner(&self) -> u32 {
        self.0
    }
}
