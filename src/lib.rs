#![feature(array_chunks)]
#![feature(iter_array_chunks)]
#![feature(iter_advance_by)]
#![feature(array_methods)]
/// This crate is an example implementation of the base-utf8 encoding algorithm.
use std::iter;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("Invalid length: {0} is not a multiple of 8")]
    InvalidLength(usize),
    #[error("Invalid padding: {0} is not a valid padding length")]
    InvalidPadding(usize),
}

pub fn encode(arr: &[u8]) -> String {
    // calculate padding
    let padding = ((7 - ((arr.len() + 1) % 7)) % 7) as u8;
    // allocate space for the result
    let mut ans: Vec<u8> = vec![0u8; (arr.len() + 1 + padding as usize) / 7 * 8];

    // chain values together making full source data
    let arr = 
        // add padding info
        iter::repeat(&padding)
        .take(1)
        // origin data
        .chain(arr.iter())
        // padding
        .chain(iter::repeat(&0).take(padding as usize).into_iter());
    // encode
    for i in arr.array_chunks::<7>().zip(ans.array_chunks_mut::<8>()) {
        encode78(&i.0, i.1);
    }
    String::from_utf8(ans).unwrap()
}

pub fn decode(arr: &str) -> Result<Vec<u8>, DecodeError> {
    let arr = arr.as_bytes();
    // check length
    if arr.len() == 0 {
        return Ok(Vec::new());
    }
    if arr.len() % 8 != 0 {
        return Err(DecodeError::InvalidLength(arr.len()));
    }
    
    // decode first chunk
    let mut firest_chunk = [0u8;7];
    decode87(arr.array_chunks::<8>().next().unwrap(), firest_chunk.each_mut());
    
    // peek padding info
    let padding = firest_chunk[0];
    if padding >= 7 {
        return Err(DecodeError::InvalidPadding(arr.len()));
    }
    // allocate space for the result
    let mut ans: Vec<u8> = vec![0u8; (arr.len() / 8 * 7) - 1 - padding as usize];
    // push first chunk
    for (i, v) in firest_chunk[1..].iter().enumerate() {
        if i >= ans.len() {
            return Ok(ans);
        }
        ans[i] = *v;
    }
    // decode
    // note that we have already decoded the first chunk
    let mut arr_iter = arr.array_chunks::<8>();
    let _ = arr_iter.advance_by(1);
    let mut ans_iter = ans.iter_mut();
    let _ = ans_iter.advance_by(6);
    for i in arr_iter.zip(ans_iter.array_chunks::<7>()) {
        decode87(&i.0, i.1);
    }
    // If we have padding, we need to manually decode the last chunk
    let last_data_len = 7-padding;
    if last_data_len != 0 {
        let mut buffer = [0u8; 7];
        decode87(arr[(arr.len() - 8)..].array_chunks::<8>().next().unwrap(), buffer.each_mut());
        for i in (ans.len() - last_data_len as usize)..ans.len() {
            ans[i] = buffer[i - (ans.len() - last_data_len as usize)];
        }
    }
    Ok(ans)
}

fn encode78(arr: &[&u8; 7], buffer: &mut [u8; 8]) {
    for i in 0..7 {
        buffer[i + 1] = *arr[i] & 0b01111111;
        buffer[0] |= (arr[i] & 0b10000000) >> (i + 1);
    }
}
fn decode87(arr: &[u8; 8], buffer: [&mut u8; 7]) {
    for i in 0..7 {
        *buffer[i] = arr[i + 1] | ((arr[0] << (i + 1)) & 0b10000000);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use rand::{thread_rng, RngCore};
    #[test]
    fn test_normal() {
        let data = b"Hello, world!";
        let encoded = encode(data);
        assert_eq!(encoded.as_bytes(), [0, 0, 72, 101, 108, 108, 111, 44, 0, 32, 119, 111, 114, 108, 100, 33]);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(data, &decoded[..]);
    }
    #[test]
    fn test_low_length() {
        let data = &[0;1];
        let encoded = encode(data);
        assert_eq!(encoded.as_bytes(), [0, 5, 0, 0, 0, 0, 0, 0]);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(data, &decoded[..]);
    }
    #[test]
    fn test_long_random() {
        const MIB: usize = 1024 * 1024;
        let mut rng = thread_rng();
        let data_length = (1*MIB) + (rng.next_u32() as usize % (9*MIB));
        let mut data = vec![0u8; data_length];
        rng.fill_bytes(&mut data);
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(data, &decoded[..]);
    }
}
