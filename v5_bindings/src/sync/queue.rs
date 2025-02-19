use crate::raw::pros::apix::*;
use core::marker::PhantomData;
use core::mem::{size_of, forget, MaybeUninit};
use core::time::Duration;
use cty::c_void;
use crate::sync::option_to_timeout;
use v5_traits::stream::{SendStream, SendTimeoutStream, ReceiveStream, ReceiveTimoutStream, MessageStreamCreator};
use v5_traits::UniversalFunctions;
use alloc::sync::Arc;

/// A queue that allows the sending of data across thread boundaries
/// Sends data of type T
#[derive(Debug)]
pub struct Queue<T> where T: 'static + Send{
    queue: queue_t,
    max_length: u32,
    phantom: PhantomData<T>,
}
impl<T> Queue<T> where T: 'static + Send{
    /// Creates a new queue that can store up to max_length messages
    pub fn new(max_length: u32) -> Self{
        Self{
            queue: unsafe{queue_create(max_length, size_of::<T>() as u32)},
            max_length,
            phantom: Default::default(),
        }
    }

    /// Prepends data to the front of the queue
    /// Will wait up to timeout for a spot in the queue
    /// Returns Ok if sent or Err if queue full and timeout reached
    pub fn prepend(&self, item: T, timeout: Option<Duration>) -> Result<(), T>{
        if unsafe { queue_prepend(self.queue, &item as *const T as *const c_void, option_to_timeout(timeout)) }{
            forget(item);
            Ok(())
        }
        else{
            Err(item)
        }
    }
    /// Appends to the queue
    /// Will wait up to timeout for a spot in the queue
    /// Returns Ok if sent or Err if queue full and timeout reached
    pub fn append(&self, item: T, timeout: Option<Duration>) -> Result<(), T>{
        if unsafe { queue_append(self.queue, &item as *const T as *const c_void, option_to_timeout(timeout)) }{
            forget(item);
            Ok(())
        }
        else{
            Err(item)
        }
    }

    /// Pulls an item out of the queue
    /// Returns Some if item pulled or None if timeout reached
    pub fn queue_receive(&self, timeout: Option<Duration>) -> Option<T>{
        let mut out = MaybeUninit::uninit();
        if unsafe { queue_recv(self.queue, out.as_mut_ptr() as *mut c_void, option_to_timeout(timeout)) }{
            Some(unsafe { out.assume_init() })
        }
        else{
            None
        }
    }

    /// The amount of items in the queue
    pub fn len(&self) -> u32{
        unsafe { queue_get_waiting(self.queue) }
    }
    /// The maximum items this queue can store
    pub fn max_len(&self) -> u32{
        self.max_length
    }

    /// Clears all items from the queue dropping each
    pub fn clear(&self){
        while let Some(item) = self.queue_receive(Some(Duration::new(0, 0))){
            drop(item);
        }
    }
}
impl<T> Queue<T> where T: 'static + Send + Copy{
    /// Copies the item at the front of the queue if T implements copy
    /// Will wait up to timeout for an item
    /// Returns some with the copied item or None if timeout reached
    pub fn peek(&self, timeout: Option<Duration>) -> Option<T>{
        let mut out = MaybeUninit::uninit();
        if unsafe { queue_peek(self.queue, out.as_mut_ptr() as *mut c_void, option_to_timeout(timeout)) }{
            Some(unsafe { out.assume_init() })
        }
        else{
            None
        }
    }
}
impl<T> Drop for Queue<T> where T: 'static + Send{
    fn drop(&mut self) {
        self.clear();
        unsafe { queue_delete(self.queue) }
    }
}
unsafe impl<T> Send for Queue<T> where T: 'static + Send{}
unsafe impl<T> Sync for Queue<T> where T: 'static + Send{}
impl<T> SendStream for Queue<T> where T: 'static + Send{
    type SData = T;

    fn send(&self, val: T) {
        match self.append(val, None){
            Ok(_) => {},
            Err(_) => unreachable!(),
        }
    }
}
impl<T> SendTimeoutStream for Queue<T> where T: 'static + Send{
    fn send_timeout(&self, val: T, timeout: Duration, _uf: &impl UniversalFunctions) -> Option<T> {
        match self.append(val, Some(timeout)){
            Ok(_) => None,
            Err(error) => Some(error)
        }
    }
}
impl<T> ReceiveStream for Queue<T> where T: 'static + Send{
    type RData = T;

    fn try_receive(&self) -> Option<T> {
        self.queue_receive(Some(Duration::new(0, 0)))
    }

    fn receive(&self) -> T {
        match self.queue_receive(None){
            None => unreachable!("Queue returned none with no timeout"),
            Some(val) => val,
        }
    }
}
impl<T> ReceiveTimoutStream for Queue<T> where T: 'static + Send{
    fn receive_timeout(&self, timeout: Duration, _uf: &impl UniversalFunctions) -> Option<T> {
        self.queue_receive(Some(timeout))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct QueueCreator1k();
impl<T> MessageStreamCreator<T> for QueueCreator1k where T: 'static + Send{
    type Sender = Arc<Queue<T>>;
    type Receiver = Arc<Queue<T>>;

    fn create_stream(&self) -> (Self::Sender, Self::Receiver) {
        let queue = Arc::new(Queue::new(1 << 10 / size_of::<T>() + 1));
        (queue.clone(), queue)
    }
}
#[derive(Copy, Clone, Debug)]
pub struct QueueCreator16k();
impl<T> MessageStreamCreator<T> for QueueCreator16k where T: 'static + Send{
    type Sender = Arc<Queue<T>>;
    type Receiver = Arc<Queue<T>>;

    fn create_stream(&self) -> (Self::Sender, Self::Receiver) {
        let queue = Arc::new(Queue::new(1 << 14 / size_of::<T>() + 1));
        (queue.clone(), queue)
    }
}

#[cfg(feature = "v5_test")]
pub mod test{
    use crate::sync::queue::Queue;
    use crate::test::{assert, TestItem, TestType};
    use alloc::boxed::Box;
    use alloc::string::ToString;
    use core::time::Duration;

    pub fn queue_test() -> TestItem{
        TestItem::new("queue_test".to_string(), TestType::Parallel(Box::new(|| {
            let queue_length = 100;
            let queue = Queue::new(queue_length);
            assert(queue.max_len() == 100, format!("Queue max_length invalid! Should be: {}, is: {}", queue_length, queue.max_len()))?;
            assert(queue.len() == 0, format!("Queue length invalid at initialization! Should be: {}, is {}", 0, queue.len()))?;
            let insert_val = 1424;
            if let Err(_) = queue.append(insert_val, Some(Duration::from_millis(100))){
                return Err(format!("Could not insert {} into queue", insert_val));
            }
            assert(queue.len() == 1, format!("Queue length invalid! Should be: {}, is {}", 1, queue.len()))?;
            let received = queue.queue_receive(Some(Duration::from_millis(100)));
            assert(received.is_some(), format!("Could not pull from queue"))?;
            assert(received.unwrap() == insert_val, format!("Value from queue wrong! Should be: {}, is: {}", insert_val, received.unwrap()))?;
            assert(queue.len() == 0, format!("Queue length invalid after received! Should be: {}, is {}", 0, queue.len()))?;
            Ok(())
        }), Duration::from_secs(1)))
    }
}
