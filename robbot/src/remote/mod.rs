//!
//! Primitives:
//! - bool
//! - u8, u16, u32, u64
//! - i8, i16, i32, i64
//! - f32, f64
//!
//! std-lib types:
//! - Option<T>
//! - Vec<T>
//! - String
//! - Box<T>

use std::io::{self, Read, Write};
use std::string::FromUtf8Error;

use thiserror::Error;

pub trait Encode {
    fn encode<W>(&self, encoder: &mut Encoder<W>) -> Result<()>
    where
        W: Write;
}

pub trait Decode: Sized {
    fn decode<R>(decoder: &mut Decoder<R>) -> Result<Self>
    where
        R: Read;
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),
}

pub struct Encoder<W>
where
    W: Write,
{
    writer: W,
}

impl<W> Encoder<W>
where
    W: Write,
{
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn encode_bool(&mut self, value: bool) -> Result<()> {
        self.encode_u8(match value {
            false => 0,
            true => 1,
        })
    }

    pub fn encode_i8(&mut self, value: i8) -> Result<()> {
        let buf = value.to_be_bytes();
        self.write(&buf)
    }

    pub fn encode_i16(&mut self, value: i16) -> Result<()> {
        let buf = value.to_be_bytes();
        self.write(&buf)
    }

    pub fn encode_i32(&mut self, value: i32) -> Result<()> {
        let buf = value.to_be_bytes();
        self.write(&buf)
    }

    pub fn encode_i64(&mut self, value: i64) -> Result<()> {
        let buf = value.to_be_bytes();
        self.write(&buf)
    }

    pub fn encode_u8(&mut self, value: u8) -> Result<()> {
        self.write(&[value])
    }

    pub fn encode_u16(&mut self, value: u16) -> Result<()> {
        let buf = value.to_be_bytes();
        self.write(&buf)
    }

    pub fn encode_u32(&mut self, value: u32) -> Result<()> {
        let buf = value.to_be_bytes();
        self.write(&buf)
    }

    pub fn encode_u64(&mut self, value: u64) -> Result<()> {
        let buf = value.to_be_bytes();
        self.write(&buf)
    }

    pub fn encode_f32(&mut self, value: f32) -> Result<()> {
        let buf = value.to_be_bytes();
        self.write(&buf)
    }

    pub fn encode_f64(&mut self, value: f64) -> Result<()> {
        let buf = value.to_be_bytes();
        self.write(&buf)
    }

    pub fn encode_bytes(&mut self, value: &[u8]) -> Result<()> {
        self.write(value)
    }

    fn write(&mut self, buf: &[u8]) -> Result<()> {
        self.writer.write_all(buf)?;
        Ok(())
    }
}

pub struct Decoder<R>
where
    R: Read,
{
    reader: R,
}

impl<R> Decoder<R>
where
    R: Read,
{
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn decode_bool(&mut self) -> Result<bool> {
        let value = self.decode_u8()?;

        match value {
            0 => Ok(false),
            1 => Ok(true),
            v => panic!("Invalid decoded boolean value: {}", v),
        }
    }

    pub fn decode_u8(&mut self) -> Result<u8> {
        let mut buf = [0; 1];
        self.read(&mut buf)?;

        Ok(u8::from_be_bytes(buf))
    }

    pub fn decode_u16(&mut self) -> Result<u16> {
        let mut buf = [0; 2];
        self.read(&mut buf)?;

        Ok(u16::from_be_bytes(buf))
    }

    pub fn decode_u32(&mut self) -> Result<u32> {
        let mut buf = [0; 4];
        self.read(&mut buf)?;

        Ok(u32::from_be_bytes(buf))
    }

    pub fn decode_u64(&mut self) -> Result<u64> {
        let mut buf = [0; 8];
        self.read(&mut buf)?;

        Ok(u64::from_be_bytes(buf))
    }

    pub fn decode_i8(&mut self) -> Result<i8> {
        let mut buf = [0; 1];
        self.read(&mut buf)?;

        Ok(i8::from_be_bytes(buf))
    }

    pub fn decode_i16(&mut self) -> Result<i16> {
        let mut buf = [0; 2];
        self.read(&mut buf)?;

        Ok(i16::from_be_bytes(buf))
    }

    pub fn decode_i32(&mut self) -> Result<i32> {
        let mut buf = [0; 4];
        self.read(&mut buf)?;

        Ok(i32::from_be_bytes(buf))
    }

    pub fn decode_i64(&mut self) -> Result<i64> {
        let mut buf = [0; 8];
        self.read(&mut buf)?;

        Ok(i64::from_be_bytes(buf))
    }

    pub fn decode_f32(&mut self) -> Result<f32> {
        let mut buf = [0; 4];
        self.read(&mut buf)?;

        Ok(f32::from_be_bytes(buf))
    }

    pub fn decode_f64(&mut self) -> Result<f64> {
        let mut buf = [0; 8];
        self.read(&mut buf)?;

        Ok(f64::from_be_bytes(buf))
    }

    pub fn decode_bytes(&mut self, buf: &mut [u8]) -> Result<()> {
        self.read(buf)?;
        Ok(())
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<()> {
        self.reader.read_exact(buf)?;
        Ok(())
    }
}

impl Encode for bool {
    fn encode<W>(&self, encoder: &mut Encoder<W>) -> Result<()>
    where
        W: Write,
    {
        encoder.encode_bool(*self)
    }
}

impl Encode for i8 {
    fn encode<W>(&self, encoder: &mut Encoder<W>) -> Result<()>
    where
        W: Write,
    {
        encoder.encode_i8(*self)
    }
}

impl Encode for i16 {
    fn encode<W>(&self, encoder: &mut Encoder<W>) -> Result<()>
    where
        W: Write,
    {
        encoder.encode_i16(*self)
    }
}

impl Encode for i32 {
    fn encode<W>(&self, encoder: &mut Encoder<W>) -> Result<()>
    where
        W: Write,
    {
        encoder.encode_i32(*self)
    }
}

impl Encode for i64 {
    fn encode<W>(&self, encoder: &mut Encoder<W>) -> Result<()>
    where
        W: Write,
    {
        encoder.encode_i64(*self)
    }
}

impl Encode for u8 {
    fn encode<W>(&self, encoder: &mut Encoder<W>) -> Result<()>
    where
        W: Write,
    {
        encoder.encode_u8(*self)
    }
}

impl Encode for u16 {
    fn encode<W>(&self, encoder: &mut Encoder<W>) -> Result<()>
    where
        W: Write,
    {
        encoder.encode_u16(*self)
    }
}

impl Encode for u32 {
    fn encode<W>(&self, encoder: &mut Encoder<W>) -> Result<()>
    where
        W: Write,
    {
        encoder.encode_u32(*self)
    }
}

impl Encode for u64 {
    fn encode<W>(&self, encoder: &mut Encoder<W>) -> Result<()>
    where
        W: Write,
    {
        encoder.encode_u64(*self)
    }
}

impl Encode for f32 {
    fn encode<W>(&self, encoder: &mut Encoder<W>) -> Result<()>
    where
        W: Write,
    {
        encoder.encode_f32(*self)
    }
}

impl Encode for f64 {
    fn encode<W>(&self, encoder: &mut Encoder<W>) -> Result<()>
    where
        W: Write,
    {
        encoder.encode_f64(*self)
    }
}

impl<T> Encode for [T]
where
    T: Encode,
{
    fn encode<W>(&self, encoder: &mut Encoder<W>) -> Result<()>
    where
        W: Write,
    {
        encoder.encode_u64(self.len() as u64)?;

        for item in self {
            item.encode(encoder)?;
        }

        Ok(())
    }
}

impl<T> Encode for Vec<T>
where
    T: Encode,
{
    fn encode<W>(&self, encoder: &mut Encoder<W>) -> Result<()>
    where
        W: Write,
    {
        self.as_slice().encode(encoder)
    }
}

impl Encode for str {
    fn encode<W>(&self, encoder: &mut Encoder<W>) -> Result<()>
    where
        W: Write,
    {
        encoder.encode_u64(self.len() as u64)?;
        encoder.encode_bytes(self.as_bytes())
    }
}

impl Encode for String {
    fn encode<W>(&self, encoder: &mut Encoder<W>) -> Result<()>
    where
        W: Write,
    {
        self.as_str().encode(encoder)
    }
}

impl<T> Encode for Option<T>
where
    T: Encode,
{
    fn encode<W>(&self, encoder: &mut Encoder<W>) -> Result<()>
    where
        W: Write,
    {
        encoder.encode_bool(self.is_some())?;

        if let Some(value) = self {
            value.encode(encoder)?;
        }

        Ok(())
    }
}

impl<T> Encode for Box<T>
where
    T: Encode,
{
    fn encode<W>(&self, encoder: &mut Encoder<W>) -> Result<()>
    where
        W: Write,
    {
        let inner: &T = &*self;

        inner.encode(encoder)
    }
}

impl Decode for bool {
    fn decode<R>(decoder: &mut Decoder<R>) -> Result<Self>
    where
        R: Read,
    {
        decoder.decode_bool()
    }
}

impl Decode for u8 {
    fn decode<R>(decoder: &mut Decoder<R>) -> Result<Self>
    where
        R: Read,
    {
        decoder.decode_u8()
    }
}

impl Decode for u16 {
    fn decode<R>(decoder: &mut Decoder<R>) -> Result<Self>
    where
        R: Read,
    {
        decoder.decode_u16()
    }
}

impl Decode for u32 {
    fn decode<R>(decoder: &mut Decoder<R>) -> Result<Self>
    where
        R: Read,
    {
        decoder.decode_u32()
    }
}

impl Decode for u64 {
    fn decode<R>(decoder: &mut Decoder<R>) -> Result<Self>
    where
        R: Read,
    {
        decoder.decode_u64()
    }
}

impl Decode for i8 {
    fn decode<R>(decoder: &mut Decoder<R>) -> Result<Self>
    where
        R: Read,
    {
        decoder.decode_i8()
    }
}

impl Decode for i16 {
    fn decode<R>(decoder: &mut Decoder<R>) -> Result<Self>
    where
        R: Read,
    {
        decoder.decode_i16()
    }
}

impl Decode for i32 {
    fn decode<R>(decoder: &mut Decoder<R>) -> Result<Self>
    where
        R: Read,
    {
        decoder.decode_i32()
    }
}

impl Decode for i64 {
    fn decode<R>(decoder: &mut Decoder<R>) -> Result<Self>
    where
        R: Read,
    {
        decoder.decode_i64()
    }
}

impl Decode for f32 {
    fn decode<R>(decoder: &mut Decoder<R>) -> Result<Self>
    where
        R: Read,
    {
        decoder.decode_f32()
    }
}

impl Decode for f64 {
    fn decode<R>(decoder: &mut Decoder<R>) -> Result<Self>
    where
        R: Read,
    {
        decoder.decode_f64()
    }
}

impl<T> Decode for Vec<T>
where
    T: Decode,
{
    fn decode<R>(decoder: &mut Decoder<R>) -> Result<Self>
    where
        R: Read,
    {
        let len = decoder.decode_u64()?;
        let mut vec = Vec::with_capacity(len as usize);

        for _ in 0..len {
            let item = T::decode(decoder)?;
            vec.push(item);
        }

        Ok(vec)
    }
}

impl Decode for String {
    fn decode<R>(decoder: &mut Decoder<R>) -> Result<Self>
    where
        R: Read,
    {
        let vec: Vec<u8> = Vec::decode(decoder)?;

        Ok(String::from_utf8(vec)?)
    }
}

impl<T> Decode for Option<T>
where
    T: Decode,
{
    fn decode<R>(decoder: &mut Decoder<R>) -> Result<Self>
    where
        R: Read,
    {
        let is_some = decoder.decode_bool()?;

        Ok(match is_some {
            true => Some(T::decode(decoder)?),
            false => None,
        })
    }
}

impl<T> Decode for Box<T>
where
    T: Decode,
{
    fn decode<R>(decoder: &mut Decoder<R>) -> Result<Self>
    where
        R: Read,
    {
        let value = T::decode(decoder)?;
        Ok(Self::new(value))
    }
}

#[cfg(test)]
mod tests {
    use super::Encoder;

    #[test]
    fn test_encoder_bool() {
        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        encoder.encode_bool(true).unwrap();
        assert_eq!(buf, [0x01]);

        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        encoder.encode_bool(true).unwrap();
        encoder.encode_bool(false).unwrap();
        assert_eq!(buf, [0x01, 0x00]);
    }

    #[test]
    fn test_encoder_u8() {
        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        encoder.encode_u8(0x23).unwrap();
        assert_eq!(buf, [0x23]);

        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        encoder.encode_u8(0x23).unwrap();
        encoder.encode_u8(0x54).unwrap();
        assert_eq!(buf, [0x23, 0x54]);
    }

    #[test]
    fn test_encoder_u16() {
        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        encoder.encode_u16(0x56FF).unwrap();
        assert_eq!(buf, [0x56, 0xFF]);

        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        encoder.encode_u16(0x56FF).unwrap();
        encoder.encode_u16(0x2AF3).unwrap();
        assert_eq!(buf, [0x56, 0xFF, 0x2A, 0xF3]);
    }

    #[test]
    fn test_encoder_u32() {
        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        encoder.encode_u32(0xAB80EE10).unwrap();
        assert_eq!(buf, [0xAB, 0x80, 0xEE, 0x10]);

        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        encoder.encode_u32(0xAB80EE10).unwrap();
        encoder.encode_u32(0x190A90DE).unwrap();
        assert_eq!(buf, [0xAB, 0x80, 0xEE, 0x10, 0x19, 0x0A, 0x90, 0xDE]);
    }

    #[test]
    fn test_encoder_u64() {
        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        encoder.encode_u64(0x0DE0E13CF88ACB61).unwrap();
        assert_eq!(buf, [0x0D, 0xE0, 0xE1, 0x3C, 0xF8, 0x8A, 0xCB, 0x61]);

        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        encoder.encode_u64(0x0DE0E13CF88ACB61).unwrap();
        encoder.encode_u64(0x2D94A6BDBC07CF10).unwrap();
        assert_eq!(
            buf,
            [
                0x0D, 0xE0, 0xE1, 0x3C, 0xF8, 0x8A, 0xCB, 0x61, 0x2D, 0x94, 0xA6, 0xBD, 0xBC, 0x07,
                0xCF, 0x10
            ]
        );
    }

    #[test]
    fn test_encoder_i8() {
        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        encoder.encode_i8(0x74).unwrap();
        assert_eq!(buf, [0x74]);

        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        encoder.encode_i8(0x74).unwrap();
        encoder.encode_i8(-0x43).unwrap();
        assert_eq!(buf, [0x74, 0xBD]);
    }

    #[test]
    fn test_encoder_i16() {
        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        encoder.encode_i16(0x74DE).unwrap();
        assert_eq!(buf, [0x74, 0xDE]);

        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        encoder.encode_i16(0x74DE).unwrap();
        encoder.encode_i16(-0x43AF).unwrap();
        assert_eq!(buf, [0x74, 0xDE, 0xBC, 0x51]);
    }

    #[test]
    fn test_encoder_i32() {
        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        encoder.encode_i32(0x74DEAD13).unwrap();
        assert_eq!(buf, [0x74, 0xDE, 0xAD, 0x13]);

        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        encoder.encode_i32(0x74DEAD13).unwrap();
        encoder.encode_i32(-0x43AFC54E).unwrap();
        assert_eq!(buf, [0x74, 0xDE, 0xAD, 0x13, 0xBC, 0x50, 0x3A, 0xB2]);
    }

    #[test]
    fn test_encoder_i64() {
        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        encoder.encode_i64(0x74DEAD1375634584).unwrap();
        assert_eq!(buf, [0x74, 0xDE, 0xAD, 0x13, 0x75, 0x63, 0x45, 0x84]);

        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        encoder.encode_i64(0x74DEAD1375634584).unwrap();
        encoder.encode_i64(-0x43AFC54EA483E984).unwrap();
        assert_eq!(
            buf,
            [
                0x74, 0xDE, 0xAD, 0x13, 0x75, 0x63, 0x45, 0x84, 0xBC, 0x50, 0x3A, 0xB1, 0x5B, 0x7C,
                0x16, 0x7C
            ]
        );
    }
}
