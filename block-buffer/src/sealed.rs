extern crate alloc;
use super::{ArrayLength, Block};
use alloc::vec::Vec;
use core::slice;

/// Sealed trait for buffer kinds.
pub trait Sealed {
    /// Invariant guaranteed by a buffer kind, i.e. with correct
    /// buffer code this function always returns true.
    fn invariant(pos: usize, block_size: usize) -> bool;

    /// Split input data into slice of blocks and tail.
    fn split_blocks<N: ArrayLength<u8>>(data: &[u8]) -> (&[Block<N>], &[u8]);
}

impl Sealed for super::Eager {
    #[inline(always)]
    fn invariant(pos: usize, block_size: usize) -> bool {
        pos < block_size
    }

    #[inline(always)]
    fn split_blocks<N: ArrayLength<u8>>(data: &[u8]) -> (&[Block<N>], &[u8]) {
        let nb = data.len() / N::USIZE;
        let blocks_len = nb * N::USIZE;
        let tail_len = data.len() - blocks_len;
        // SAFETY: we guarantee that created slices do not point
        // outside of `data`
        unsafe {
            let blocks_ptr = blocks_from_bytes(data).as_ptr();
            let tail_ptr = data.as_ptr().add(blocks_len);
            (
                slice::from_raw_parts(blocks_ptr, nb),
                slice::from_raw_parts(tail_ptr, tail_len),
            )
        }
    }
}

impl Sealed for super::Lazy {
    #[inline(always)]
    fn invariant(pos: usize, block_size: usize) -> bool {
        pos <= block_size
    }

    #[inline(always)]
    fn split_blocks<N: ArrayLength<u8>>(data: &[u8]) -> (&[Block<N>], &[u8]) {
        if data.is_empty() {
            return (&[], &[]);
        }
        let (nb, tail_len) = if data.len() % N::USIZE == 0 {
            (data.len() / N::USIZE - 1, N::USIZE)
        } else {
            let nb = data.len() / N::USIZE;
            (nb, data.len() - nb * N::USIZE)
        };
        let blocks_len = nb * N::USIZE;
        // SAFETY: we guarantee that created slices do not point
        // outside of `data`
        unsafe {
            let blocks_ptr = blocks_from_bytes(data).as_ptr();
            let tail_ptr = data.as_ptr().add(blocks_len);
            (
                slice::from_raw_parts(blocks_ptr, nb),
                slice::from_raw_parts(tail_ptr, tail_len),
            )
        }
    }
}

/// Convert a slice of bytes to a slice of Blocks. PRECONDITION: the argument
/// slice's length must be a multiple of `N`. If this is not the case, this
/// function will panic.
///
/// This is essentially a much hackier version of `slice::align_to` that does
/// not preserve aliasing between the input and output slices. The only reason
/// this exists is to make it easier for Crucible to simulate.
fn blocks_from_bytes<'a, N: ArrayLength<u8>>(s: &'a [u8]) -> &'a [Block<N>] {
    s.chunks_exact(N::USIZE)
        .map(|chunk: &[u8]| {
            let br: &Block<N> = chunk.into();
            br.clone()
        })
        .collect::<Vec<Block<N>>>()
        .leak()
}
