use std::mem::transmute;
use core::nonzero::NonZero;
use std::ptr;
use std::ptr::Unique;
use std::marker::PhantomData;
use intrusive_collections::{LinkedList, LinkedListLink, Adapter,
                            SinglyLinkedList, SinglyLinkedListLink,
                            IntrusivePointer};

use util::{Lengthy, OwnedSlice};
use allocator::{OverAllocator, MemoryPool, SplitOff};

// TODO: replace various checks in alloc/release methods with preconditions

// ================================================================================================

/// Compute free list indices from object word counts.
/// # Laws
/// `alloc_index(x) >= free_index(x) | x > 0`
pub trait IndexCalculation {
    /// Get the index of the first freelist that can support allocation of `n` words.
    fn alloc_index(n: NonZero<usize>) -> usize;

    /// Get the index of the freelist where an object of `n` words should be freed to.
    fn free_index(n: NonZero<usize>) -> usize;

    /// The minimum size of nodes under `index`.
    fn min(index: usize) -> usize;

    /// The non-inclusive upper bound of node size under `index`.
    fn max(index: usize) -> usize;
}

// ================================================================================================

struct SizeClass<N> where N: Adapter<Link = SinglyLinkedListLink>
{
    min: usize,
    max: usize,
    list: SinglyLinkedList<N>
}

impl<N> SizeClass<N> where N: Adapter<Link = SinglyLinkedListLink> + Default {
    fn new(min: usize, max: usize) -> Self {
        SizeClass {
            min: min,
            max: max,
            list: SinglyLinkedList::new(N::default())
        }
    }

    fn is_empty(&self) -> bool { self.list.is_empty() }
}

impl<N> OverAllocator for SizeClass<N> where N: Adapter<Link = SinglyLinkedListLink> {
    fn allocate_at_least(&mut self, walign: NonZero<usize>, wsize: NonZero<usize>)
        -> Option<OwnedSlice<usize>>
    {
        if *wsize <= self.min {
            self.list.pop_front()
                .map(|v| OwnedSlice::from_raw_parts(unsafe { Unique::new(v.into_raw() as _) },
                                                    self.min))
        } else {
            None
        }
    }
}

impl<N> MemoryPool for SizeClass<N> where N: Adapter<Link = SinglyLinkedListLink>,
                                          N::Value: Default
{
    unsafe fn try_release(&mut self, oref: Unique<usize>, wsize: NonZero<usize>)
        -> Option<Unique<usize>>
    {
        if *wsize >= self.min && *wsize < self.max {
            let node = transmute::<*mut usize, *mut N::Value>(*oref);
            ptr::write(node, N::Value::default());
            self.list.push_front(N::Pointer::from_raw(node));
            None
        } else {
            Some(oref)
        }
    }
}

// ================================================================================================

/// First-fit freelist
pub struct FirstFit<N> where N: Adapter<Link = LinkedListLink> {
    min: usize,
    list: LinkedList<N>
}

impl<N> FirstFit<N> where N: Adapter<Link = LinkedListLink> + Default
{
    /// Create a new first-fit freelist
    pub fn new(min: usize) -> Self {
        FirstFit {
            min: min,
            list: LinkedList::new(N::default())
        }
    }
}

impl<N> OverAllocator for FirstFit<N> where N: Adapter<Link = LinkedListLink>,
                                            N::Value: Lengthy + SplitOff
{
    fn allocate_at_least(&mut self, walign: NonZero<usize>, wsize: NonZero<usize>)
        -> Option<OwnedSlice<usize>>
    {
        // TODO: observe walign
        let mut cursor = self.list.cursor_mut();
        while let Some(node) = cursor.get() {
            if node.len() >= *wsize {
                let remainder = node.len() - *wsize;
                return if remainder >= self.min {
                    Some(OwnedSlice::from_raw_parts(node.split_off(*wsize), *wsize))
                } else {
                    cursor.remove()
                          .map(|v| OwnedSlice::from_raw_parts(
                              unsafe { Unique::new(v.into_raw() as _) },
                              node.len()))
               }
            }
            cursor.move_next();
        }
        None
    }
}

impl<N> MemoryPool for FirstFit<N> where N: Adapter<Link = LinkedListLink>,
                                         N::Value: Default + Lengthy
{
    unsafe fn try_release(&mut self, oref: Unique<usize>, wsize: NonZero<usize>)
        -> Option<Unique<usize>>
    {
        if *wsize >= self.min {
            let node = transmute::<*mut usize, *mut N::Value>(*oref);
            ptr::write(node, N::Value::default());
            (*node).set_len(*wsize);
            self.list.push_front(N::Pointer::from_raw(node));
            None
        } else {
            Some(oref)
        }
    }
}

// ================================================================================================

// MAYBE: Use typelevel numbers to exchange the Vec for a size-generic array
/// Bucketed freelist
pub struct Bucketed<N, I> where N: Adapter<Link = SinglyLinkedListLink>
{
    buckets: Vec<SizeClass<N>>,
    index_calc: PhantomData<I>
}

impl<N, I> Bucketed<N, I> where N: Adapter<Link = SinglyLinkedListLink> + Default,
                                I: IndexCalculation
{
    /// Create a new bucketed freelist with `n` buckets.
    pub fn new(n: usize) -> Self {
        Bucketed {
            buckets: (0..n).map(|i| SizeClass::new(I::min(i), I::max(i))).collect(),
            index_calc: PhantomData::default()
        }
    }
}

impl<N, I> OverAllocator for Bucketed<N, I> where N: Adapter<Link = SinglyLinkedListLink>
                                                     + Default,
                                                  I: IndexCalculation
{
    fn allocate_at_least(&mut self, walign: NonZero<usize>, wsize: NonZero<usize>)
        -> Option<OwnedSlice<usize>>
    {
        // TODO: observe walign
        let start = I::alloc_index(wsize);
        self.buckets.get_mut(start)
            .and_then(|b| b.allocate_at_least(walign, wsize))
            .or_else(|| self.buckets[start + 1..].iter_mut()
                            .find(|b| !b.is_empty())
                            .and_then(|b| b.allocate_at_least(walign, wsize)))
    }
}

impl<N, I> MemoryPool for Bucketed<N, I> where N: Adapter<Link = SinglyLinkedListLink>,
                                               N::Value: Default,
                                               I: IndexCalculation
{
    unsafe fn try_release(&mut self, oref: Unique<usize>, wsize: NonZero<usize>)
        -> Option<Unique<usize>>
    {
        self.buckets.get_mut(I::free_index(wsize))
            .and_then(|b| b.try_release(oref, wsize))
    }
}

// ================================================================================================

#[cfg(test)]
mod tests {
    use std::mem::size_of;
    use std::ptr::Unique;

    use object_model::GCRef;

    #[test]
    fn unique_is_null_ptr_optimized() {
        assert_eq!(size_of::<Option<Unique<usize>>>(), size_of::<GCRef>());
    }
}