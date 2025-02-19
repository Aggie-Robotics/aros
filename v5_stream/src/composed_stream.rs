use core::fmt::Debug;

use v5_traits::stream::*;
use alloc::vec::Vec;
use core::time::Duration;
use v5_traits::UniversalFunctions;

#[derive(Debug)]
pub struct ComposedStream<S, R> where S: SendStream, R: ReceiveStream<RData=S::SData>{
    pub send_stream: S,
    pub receive_stream: R,
}
impl<S, R> ComposedStream<S, R> where S: SendStream, R: ReceiveStream<RData=S::SData>{
    pub fn new(send_stream: S, receive_stream: R) -> Self{
        Self{ send_stream, receive_stream }
    }
}
impl<S, R> SendStream for ComposedStream<S, R> where S: SendStream, R: ReceiveStream<RData=S::SData>{
    type SData = S::SData;

    fn send(&self, val: Self::SData)  {
        self.send_stream.send(val)
    }

    fn send_slice(&self, slice: &[Self::SData]) where Self::SData: Copy {
        self.send_stream.send_slice(slice)
    }

    fn send_vec(&self, data: Vec<Self::SData>) {
        self.send_stream.send_vec(data)
    }
}
impl<S, R> SendTimeoutStream for ComposedStream<S, R> where S: SendTimeoutStream, R: ReceiveStream<RData=S::SData>{
    fn send_timeout(&self, val: Self::SData, timeout: Duration, uf: &impl UniversalFunctions) -> Option<Self::SData> {
        self.send_stream.send_timeout(val, timeout, uf)
    }

    fn send_slice_timeout(&self, slice: &[Self::SData], timeout: Duration, uf: &impl UniversalFunctions) -> usize where Self::SData: Copy {
        self.send_stream.send_slice_timeout(slice, timeout, uf)
    }

    fn send_vec_timeout(&self, data: Vec<Self::SData>, timeout: Duration, uf: &impl UniversalFunctions) -> Option<Vec<Self::SData>> {
        self.send_stream.send_vec_timeout(data, timeout, uf)
    }
}
impl<S, R> ReceiveStream for ComposedStream<S, R> where S: SendStream, R: ReceiveStream<RData=S::SData>{
    type RData = R::RData;

    fn try_receive(&self) -> Option<Self::RData> {
        self.receive_stream.try_receive()
    }

    fn receive(&self) -> Self::RData {
        self.receive_stream.receive()
    }

    fn receive_slice(&self, buffer: &mut [Self::RData]) -> usize {
        self.receive_stream.receive_slice(buffer)
    }

    fn receive_all(&self, buffer: &mut [Self::RData]) {
        self.receive_stream.receive_all(buffer)
    }

    fn receive_vec(&self, limit: usize) -> Vec<Self::RData>{
        self.receive_stream.receive_vec(limit)
    }
}
impl<S, R> ReceiveTimoutStream for ComposedStream<S, R> where S: SendStream, R: ReceiveTimoutStream<RData=S::SData>{
    fn receive_timeout(&self, timeout: Duration, uf: &impl UniversalFunctions) -> Option<Self::RData> {
        self.receive_stream.receive_timeout(timeout, uf)
    }

    fn receive_slice_timeout(&self, buffer: &mut [Self::RData], timeout: Duration, uf: &impl UniversalFunctions) -> usize {
        self.receive_stream.receive_slice_timeout(buffer, timeout, uf)
    }

    fn receive_vec_timeout(&self, limit: usize, timeout: Duration, uf: &impl UniversalFunctions) -> Vec<Self::RData> {
        self.receive_stream.receive_vec_timeout(limit, timeout, uf)
    }
}
impl<S, R> DuplexStream for ComposedStream<S, R> where S: SendStream, R: ReceiveStream<RData=S::SData>{}
impl<S, R> DuplexTimeoutStream for ComposedStream<S, R> where S: SendTimeoutStream, R: ReceiveTimoutStream<RData=S::SData>{}
