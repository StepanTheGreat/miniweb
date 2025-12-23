use core::{mem::MaybeUninit, ops::{Index, IndexMut, Shr}};

#[cfg(test)]
use crate::println;

/// A vector of a constant size. Allows for push/pop operations, but,
/// has an upper limit to its capacity.
pub struct ConstVec<T, const S: usize> {
    items: [MaybeUninit<T>; S],
    length: usize
}

impl<T, const S: usize> ConstVec<T, S> {
    pub const fn new() -> Self {
        Self {
            items: [const { MaybeUninit::uninit() }; S],
            length: 0
        }
    }

    /// Get the current length of the vector
    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_full(&self) -> bool {
        self.len() >= S
    }

    pub fn push(&mut self, item: T) {
        assert!(!self.is_full());

        self.items[self.length].write(item);
        self.length += 1;
    }

    pub fn pop(&mut self) -> T {
        assert!(!self.is_empty());

        self.length -= 1;
        
        let item = core::mem::replace(
            &mut self.items[self.length], 
            MaybeUninit::uninit()
        );

        unsafe { item.assume_init() }
    }

    pub fn swap_pop(&mut self, index: usize) -> T {
        assert!(index < self.length);
        
        if self.length > 1 {
            let [current, last] = self.items
                .get_disjoint_mut([index, self.length-1])
                .unwrap();
    
            core::mem::swap(current, last);
        }

        self.pop()
    }

    // Swap the slot at the current index with a new value.
    //
    // This will return the former value
    pub fn set(&mut self, index: usize, value: T) -> T {
        assert!(index < self.length);

        let item = core::mem::replace(
            &mut self.items[index], 
            MaybeUninit::new(value)
        );

        unsafe { item.assume_init() }
    }

    pub fn get(&self, index: usize) -> &T {
        assert!(index < self.length);

        unsafe { self.items[index].assume_init_ref() }
    }

    pub fn get_mut(&mut self, index: usize) -> &mut T {
        assert!(index < self.length);

        unsafe { self.items[index].assume_init_mut() }
    }

    pub fn clear(&mut self) {
        while !self.is_empty() {
            drop(self.pop());
        }
        self.length = 0;
    }
}

impl<T, const S: usize> Drop for ConstVec<T, S> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T, const S: usize> Index<usize> for ConstVec<T, S> {
    type Output = T;
    
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}

impl<T, const S: usize> IndexMut<usize> for ConstVec<T, S> {    
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index)
    }
}

#[cfg(test)]
mod tests {
    use core::ops::AddAssign;

    use super::*;

    #[test]
    fn test_const_vec() {
        let mut arr: ConstVec<u32, 32> = ConstVec::new();
    
        let vals = [0, 5, 20, 30, 45];
        
        for val in vals.iter().copied() {
            arr.push(val);
        }
        
        assert_eq!(arr.len(), vals.len());
        
        while !arr.is_empty() {
            assert!(arr.pop() == vals[arr.len()]);
        }
    }

    #[test]
    fn test_const_vec_drop() {

        static mut COUNTER: usize = 0;

        #[derive(Debug)]
        struct Dropping(pub i32);

        impl Drop for Dropping {
            fn drop(&mut self) {
                unsafe { COUNTER += 1 };
            }
        }

        let mut arr: ConstVec<Dropping, 32> = ConstVec::new();
    
        let mut vals = [0, 5, 20, 30, 45];
        
        for val in vals.iter().copied() {
            arr.push(Dropping(val));
        }

        for i in 0..vals.len() {
            assert_eq!(vals[i], arr[i].0);
        }

        {
            let new_val = 2000;
            vals[2] = new_val;
            arr.set(2, Dropping(new_val));
        }

        {
            vals[3].add_assign(50);
            arr[3].0.add_assign(50);
        }
        
        assert_eq!(arr.len(), vals.len());
        
        while !arr.is_empty() {
            assert!(arr.pop().0 == vals[arr.len()]);
        }

        arr.push(Dropping(250));
        arr.push(Dropping(100));

        drop(arr);

        assert!( unsafe { COUNTER == vals.len()+1+2 } );

    }

    #[test]
    fn test_const_vec_swap() {
        let mut arr: ConstVec<u32, 32> = ConstVec::new();
    
        let vals = [0, 5, 20, 30, 45];
        
        for val in vals.iter().copied() {
            arr.push(val);
        }

        arr.swap_pop(1);
        arr.swap_pop(2);

        assert_eq!(arr.len(), vals.len()-2);

        let new_vals = [0, 45, 30];
        assert_eq!(arr[1], new_vals[1]);
        assert_eq!(arr[2], new_vals[2]);

        
        for i in 0..arr.len() {
            assert_eq!(arr[i], new_vals[i]);
        }

    }
}

type BlockBitmap = u32;

const BITMAP_BITS: usize = BlockBitmap::BITS as usize;
const PAGE_SIZE: usize = 2usize.pow(16);

const BLOCK_SIZE: usize = 64;
const BLOCKS_LEN: usize = PAGE_SIZE/BLOCK_SIZE;
const BLOCK_BITMAPS: usize = BLOCKS_LEN / BITMAP_BITS;

#[derive(Clone, Copy)]
struct Block {
    blocks: usize,
    index: usize
}

impl Block {
    fn new(index: usize, blocks: usize) -> Self {
        assert!(blocks > 0);

        Self {
            blocks,
            index
        }
    }

    /// Does this block have the exact capacity?
    fn is_exact(&self, blocks: usize) -> bool {
        blocks == self.blocks
    }

    /// Does this block have enough capacity?
    fn can_accomodate(&self, blocks: usize) -> bool {
        blocks <= self.blocks
    }

    /// Consume this block partially and return a new one for use
    fn take_partially(&mut self, blocks: usize) -> Block {
        assert!(blocks < self.blocks);

        let new_index = self.index+self.blocks-blocks;
        self.blocks -= blocks;

        Block::new(new_index, blocks)
    }

    fn can_join_right(&self, block: Block) -> bool {
        self.index+self.blocks == block.index
    }

    /// Join a new block from the right if its position actually is to the left
    fn join_right(&mut self, block: Block) {
        assert!(self.can_join_right(block));

        self.blocks += block.blocks;
    }
}

struct BlockPool<const S: usize> {
    index: usize,
    blocks: ConstVec<Block, S>
}

impl<const S: usize> BlockPool<S> {
    
    // Create a new block pool with a single block initialised to the entire capacity
    fn new(index: usize) -> Self {
        let mut blocks = ConstVec::new();
        blocks.push(Block::new(index, S));

        Self {
            index,
            blocks
        }
    }

    /// Try to get a block that can satisfy the requested amount of blocks
    fn find_block(&self, blocks: usize) -> Option<usize> {
        for ind in 0..self.blocks.len() {

            if self.blocks[ind].can_accomodate(blocks) {
                return Some(ind)
            }
        }

        None
    }

    /// Get a new block from the list (if present)
    /// 
    /// This will either split the first available block, or take an entire one
    fn get_block(&mut self, blocks: usize) -> Option<Block> {
        let ind = self.find_block(blocks)?;

        let ret = if self.blocks[ind].is_exact(blocks) {
            self.blocks.swap_pop(ind)
        } else {
            self.blocks[ind].take_partially(blocks)
        };

        Some(ret)
    }

    fn return_block(&mut self, block: Block) {
        assert!(block.index >= self.index);
        assert!(block.index < self.index+S);

        // In any other case we're just going to push our block alone
        let ind = self.blocks.len();
        self.blocks.push(block);

        // Now do the same, but for our block
        for jind in 0..self.blocks.len() {

            // It must not be ours
            if jind == ind {
                continue;
            }

            #[cfg(test)]
            std::println!("<Block {}, {}>", self.blocks[ind].index, self.blocks[ind].blocks);

            // If we found a block that can actually be joined to ours
            if self.blocks[ind].can_join_right(self.blocks[jind]) {

                // Pop it from the list
                let right_block = self.blocks.swap_pop(jind);

                // Join it to the right to our block
                self.blocks[ind].join_right(right_block);

                break
            }
        }

        for jind in 0..self.blocks.len() {

            // It must not be ours
            if jind == ind {
                continue;
            }

            // If we found a block that can actually be joined to ours
            if self.blocks[jind].can_join_right(self.blocks[ind]) {

                // Pop it from the list
                let right_block = self.blocks.swap_pop(ind);

                // Join it to the right to our block
                self.blocks[jind].join_right(right_block);

                break
            }
        }
    }

    /// The amount of blocks that this pool currently has
    fn len(&self) -> usize {
        self.blocks.len()
    }
}

#[cfg(test)]
mod test_blockpool {
    use super::*;

    #[test]
    fn test_joining() {
        let mut pool: BlockPool<10> = BlockPool::new(0);
        // # # # # # # # # # #

        assert_eq!(pool.len(), 1);

        // Currently, we have 10 blocks. Let's take 3

        let three = pool.get_block(3).unwrap();
        // # # # # # # # + _ _

        // Let's get a few more

        let two = pool.get_block(2).unwrap();
        // # # # # # + _ + _ _

        let four = pool.get_block(4).unwrap();
        // # + _ _ _ + _ + _ _

        assert_eq!(pool.len(), 1);

        // Now, let's put some back

        pool.return_block(three);
        // # + _ _ _ + _ # # #

        assert_eq!(pool.len(), 2);

        pool.return_block(two);
        // # + _ _ _ # # # # #

        assert_eq!(pool.len(), 2);

        pool.return_block(four);
        // # # # # # # # # # #
        
        assert_eq!(pool.len(), 1);
    }
}

struct PageBlock {
    /// The block array. It stores bitmaps of available blocks
    blocks: [BlockBitmap; BLOCK_BITMAPS],

    /// This is a per-block array and it allows tracking how many subsequent blocks are
    /// allocated per block. This is particularly useful when deallocating blocks.
    taken_blocks: [[u8; BITMAP_BITS]; BLOCK_BITMAPS],
    
    /// The amount of freely available blocks
    available_blocks: usize
}

impl PageBlock {

    /// The bit offset for the first block set. Because our page header must fit somewhere - we
    /// must compensate at the first block 
    const BLOCK_OFFSET: usize = (size_of::<Self>()/BLOCK_SIZE)+1;

    pub fn new() -> Self {
        Self {
            blocks: [0; BLOCK_BITMAPS],
            available_blocks: BLOCKS_LEN,
            taken_blocks: [[0; BITMAP_BITS]; BLOCK_BITMAPS]
        }
    }

    /// Create a scanning bitmap for the provided amount of blocks
    /// 
    /// The bitmap will start from the left, so a bitmap of 4 blocks would look like this:
    /// `11110000_00000000_00000000_00000000`
    fn make_block_bitmap(blocks: usize) -> BlockBitmap {
        let mut ret = 0;
        
        for i in 0..blocks {
            ret += 1 << (BITMAP_BITS-1);
            ret = ret >> 1;
        }

        ret
    }

    /// Try find a free, available set of blocks for the provided amount of blocks
    /// (each is a 64 byte chunk).
    /// 
    /// This will return the bitset index and the block index itself
    pub fn get_blocks(&self, blocks: usize) -> Option<(usize, usize)> {
        assert!((1..=BITMAP_BITS).contains(&blocks));

        if blocks > self.available_blocks {
            return None;
        }

        let scan_bitmap: BlockBitmap = Self::make_block_bitmap(blocks);

        for (ind, block_set) in self.blocks.iter().copied().enumerate() {
            
            // Because we have to account for our header, the first block will always
            // have an offset
            let mut bind: usize = if ind == 0 { Self::BLOCK_OFFSET } else { 0 };

            while bind < BITMAP_BITS-blocks {
                let bitmap = scan_bitmap >> bind;

                if (!block_set & bitmap) == bitmap {
                    return Some((ind, bind));
                }

                bind += blocks;
            }
            
        }

        None
    }

    /// Actually take the amount of requested blocks
    pub fn take_blocks(&mut self, blocks: usize) -> Option<usize> {
        let (bs_ind, bind) = self.get_blocks(blocks)?;

        // Mark on this block that it took also these subsequent blocks
        // It's guaranteed that a single blockset can't take more than 32 blocks
        self.taken_blocks[bs_ind][bind] = blocks as u8;

        let taken_bitmap = Self::make_block_bitmap(blocks) >> bind;

        self.blocks[bs_ind] = self.blocks[bs_ind] | taken_bitmap;
        self.available_blocks -= blocks;
        
        Some((bs_ind * BITMAP_BITS) + bind)
    }

    pub fn free_blocks(&mut self, block_ind: usize) {
        let bs_ind = block_ind / BITMAP_BITS;
        let bind = block_ind % BITMAP_BITS;

        let taken_blocks = self.taken_blocks[bs_ind][bind] as usize;
        assert!(taken_blocks > 0, "Deallocating empty blocks");

        // Reset the amount of taken blocks
        self.taken_blocks[bs_ind][bind] = 0;
        let taken_bitmap = Self::make_block_bitmap(taken_blocks) >> bind;

        // Remove taken blocks from the bitmap
        self.blocks[bs_ind] = !(!self.blocks[bs_ind] | taken_bitmap);
        
        self.available_blocks += taken_blocks;
    }
}


#[cfg(test)]
mod tests_block {

    use crate::{alloc::PageBlock, println};
    
    #[test]
    fn test_page_blocks() {
        let mut page = PageBlock::new();

        // Allocate 2 blocks
        let addr = page.take_blocks(2).unwrap();
        
        // Free those blocks
        page.free_blocks(addr);

        // Allocate again (must allocate THE SAME BLOCKS)
        let addr2 = page.take_blocks(2).unwrap();

        // Check the equality
        assert_eq!(addr, addr2);

        let addr3 = page.take_blocks(3).unwrap();

        // The offset between these allocations must be 2

        assert!(addr3 > addr2);

        // Deallocate again our first 2 blocks
        page.free_blocks(addr2);

        // Allocate a bigger block
        let addr4 = page.take_blocks(4).unwrap();
        
        assert_ne!(addr3, addr2);
        assert_ne!(addr4, addr3);
        assert!(addr4 > addr3);
    }
}