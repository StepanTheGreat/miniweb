use core::{
    mem::{MaybeUninit, transmute}, 
    ops::{Index, IndexMut},
};

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

    pub fn as_slice(&self) -> &[T] {
        let len = self.len();
        unsafe { transmute(&self.items[0..len]) }
    } 

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let len = self.len();
        unsafe { transmute(&mut self.items[0..len]) }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_full(&self) -> bool {
        self.len() >= S
    }

    /// Push an item onto the array
    /// 
    /// This will return [Some] if the operation doesn't succeed
    /// 
    /// NOTE: The binary explodes in size when I'm trying to return [Option<T>] instead of [Option<()>],
    /// which is... interesting.
    pub fn push(&mut self, item: T) {
        if !self.is_full() { 
            self.items[self.length].write(item);
            self.length += 1;
        }
    }

    /// Pop the top-most item from the array. This will return [None] if the
    /// length of the array is zero
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        self.length -= 1;
        
        let item = core::mem::replace(
            &mut self.items[self.length], 
            MaybeUninit::uninit()
        );

        Some(unsafe { item.assume_init() })
    }

    pub fn swap_pop(&mut self, index: usize) -> Option<T> {
        if index >= self.length {
            return None;
        }

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
    pub fn set(&mut self, index: usize, value: T) -> Option<T> {
        if index >= self.length {
            return None
        }

        let item = core::mem::replace(
            &mut self.items[index], 
            MaybeUninit::new(value)
        );

        Some(unsafe { item.assume_init() })
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.length {
            return None
        }

        Some(
            unsafe { self.items[index].assume_init_ref() }
        )
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.length {
            return None
        }

        Some(
            unsafe { self.items[index].assume_init_mut() }
        )
    }

    pub fn clear(&mut self) {
        while let Some(item) = self.pop() {
            drop(item);
        }
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
        self.get(index).unwrap()
    }
}

impl<T, const S: usize> IndexMut<usize> for ConstVec<T, S> {    
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
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
            let _ = arr.push(val);
        }
        
        assert_eq!(arr.len(), vals.len());
        
        while !arr.is_empty() {
            assert!(arr.pop().unwrap() == vals[arr.len()]);
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
            let _ = arr.push(Dropping(val));
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
            assert!(arr.pop().unwrap().0 == vals[arr.len()]);
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
            let _ = arr.push(val);
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

    #[test]
    fn test_const_vec_slices() {
        let mut vec: ConstVec<i32, 16> = ConstVec::new();

        assert_eq!(vec.as_slice(), &[]);

        vec.push(5);
        vec.push(5);
        vec.push(25);

        assert_eq!(vec.as_slice(), &[5, 5, 25]);

        vec.as_mut_slice()[1] = 50;

        vec.push(3);

        assert_eq!(vec.as_slice(), &[5, 50, 25, 3]);
    }
}