use alloc::boxed::Box;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicPtr, Ordering};

pub struct SyncCell<T>{
    data: AtomicPtr<T>,
}
impl<T> SyncCell<T>{
    pub fn new(value: Option<Box<T>>) -> Self{
        match value{
            None => Self{ data: AtomicPtr::new(null_mut()) },
            Some(value) => Self{ data: AtomicPtr::new(Box::leak(value)) }
        }
    }

    pub fn swap(&self, new: Option<Box<T>>) -> Option<Box<T>>{
        let new = match new{
            None => null_mut(),
            Some(new) => Box::leak(new),
        };
        let taken = self.data.swap(new, Ordering::SeqCst);
        if taken.is_null(){
            None
        }
        else{
            let taken = unsafe {Box::from_raw(taken)};
            Some(taken)
        }
    }
}
impl<T> Drop for SyncCell<T>{
    fn drop(&mut self) {
        self.swap(None);
    }
}
impl<T> Default for SyncCell<T>{
    fn default() -> Self {
        Self::new(None)
    }
}
unsafe impl<T> Sync for SyncCell<T> where T: Send{}
impl<T> From<T> for SyncCell<T>{
    fn from(from: T) -> Self {
        Self::from(Box::new(from))
    }
}
impl<T> From<Box<T>> for SyncCell<T>{
    fn from(from: Box<T>) -> Self {
        Self::from(Some(from))
    }
}
impl<T> From<Option<Box<T>>> for SyncCell<T>{
    fn from(from: Option<Box<T>>) -> Self {
        Self::new(from)
    }
}
