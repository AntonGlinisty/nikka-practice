use core::{
    mem,
    ops::Add,
};

use ku::error::{
    Error::{
        Medium,
        NoDisk,
    },
    Result,
};

use super::{
    BLOCK_SIZE,
    FIRST_BITMAP_BLOCK,
    block_cache::BlockCache,
};

#[derive(Debug)]
pub(super) struct Bitmap {
    bitmap: &'static mut [u64],
    block_count: usize,
    cursor: usize,
}

impl Bitmap {
    pub(super) fn new(block_count: usize) -> Result<Self> {
        let bitmap = Self::new_unchecked(block_count);

        bitmap.validate()
    }

    pub(super) fn format(block_count: usize) -> Result<()> {
        let mut bitmap = Self::new_unchecked(block_count);

        bitmap.bitmap.fill(0);

        for reserved_element in 0 .. bitmap.reserved_block_count() {
            assert_eq!(
                bitmap.allocate().expect("the bitmap has been cleared just yet"),
                reserved_element,
            );
        }

        if block_count < bitmap.bitmap.len() * Self::BITS_PER_ENTRY {
            bitmap.bitmap[block_count / Self::BITS_PER_ENTRY] =
                !0 << (block_count % Self::BITS_PER_ENTRY);
        }

        bitmap.validate()?;

        Ok(())
    }

    pub(super) fn is_free(
        &self,
        number: usize,
    ) -> bool {
        assert!(number < self.block_count);
        self.bitmap[number / Self::BITS_PER_ENTRY] & (1 << (number % Self::BITS_PER_ENTRY)) == 0
    }

    pub(super) fn set_free(
        &mut self,
        number: usize,
    ) {
        assert!(self.reserved_block_count() <= number && number < self.block_count);
        assert!(!self.is_free(number));
        self.bitmap[number / Self::BITS_PER_ENTRY] &= !(1 << (number % Self::BITS_PER_ENTRY));
    }

    pub(super) fn allocate(&mut self) -> Result<usize> {
        // TODO: your code here.
        unimplemented!();
    }

    pub(super) fn free_block_count(&self) -> usize {
        self.bitmap
            .iter()
            .map(|bits| bits.count_zeros().try_into().unwrap())
            .reduce(Add::add)
            .unwrap_or(0)
    }

    pub(super) fn reserved_block_count(&self) -> usize {
        FIRST_BITMAP_BLOCK + self.size_in_blocks()
    }

    fn new_unchecked(block_count: usize) -> Self {
        Self {
            bitmap: Self::bitmap(block_count),
            block_count,
            cursor: 0,
        }
    }

    fn validate(self) -> Result<Self> {
        let bit_count = self.bitmap.len() * Self::BITS_PER_ENTRY;
        if bit_count < self.block_count || self.block_count < self.reserved_block_count() {
            return Err(Medium);
        }

        let unused_elements = self.block_count != self.bitmap.len() * Self::BITS_PER_ENTRY;
        if unused_elements {
            let unused_elements_expected_bitmap = !0 << (self.block_count % Self::BITS_PER_ENTRY);
            let unused_elements_actual_bitmap =
                self.bitmap[self.bitmap.len() - 1] & unused_elements_expected_bitmap;
            if unused_elements_actual_bitmap != unused_elements_expected_bitmap {
                return Err(Medium);
            }
        }

        if (0 .. self.reserved_block_count())
            .all(|reserved_element| !self.is_free(reserved_element))
        {
            Ok(self)
        } else {
            Err(Medium)
        }
    }

    fn size_in_blocks(&self) -> usize {
        mem::size_of_val(self.bitmap).div_ceil(BLOCK_SIZE)
    }

    fn bitmap(block_count: usize) -> &'static mut [u64] {
        unsafe {
            BlockCache::block(FIRST_BITMAP_BLOCK)
                .unwrap()
                .start_address()
                .try_into_mut_slice::<u64>(block_count.div_ceil(Self::BITS_PER_ENTRY))
                .unwrap()
        }
    }

    const BITS_PER_ENTRY: usize = u64::BITS as usize;
}

impl Clone for Bitmap {
    fn clone(&self) -> Self {
        Self {
            bitmap: Self::bitmap(self.block_count),
            block_count: self.block_count,
            cursor: self.cursor,
        }
    }
}
