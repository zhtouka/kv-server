use bytes::{BytesMut, BufMut, Buf};


fn main() {
    let mut buf = BytesMut::new();
    buf.put_u32(1);
    buf.put_u32(2);
    let a = buf.get_u64();
    println!("{:?}", a);

    let mut buf = BytesMut::new();
    buf.put_slice(&b"hello world"[..]);
    println!("{:?}", buf);
    buf.advance(10);
    println!("{:?}", buf);
    buf.reserve(11);
    buf.put_slice(&b"hello world"[..]);
    let _ = buf.split_to(2);
    println!("{:?} {:?}", buf.capacity(), buf.len());
}