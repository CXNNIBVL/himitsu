use std::io;
use std::mem;
use crate::errors::blockcipher::BlockCipherError;
use crate::util::{
    readable::Readable,
    buffer::FixedBuffer
};
use crate::traits::cipher::{ 
    BlockCipherPrimitiveEncryption as PrimitiveEncryption,
    BlockCipherPrimitiveDecryption as PrimitiveDecryption,
};

/// CBC Encryption Provider
pub struct CbcEncryption<T: PrimitiveEncryption<BLOCKSIZE>, const BLOCKSIZE: usize> {
    primitive: T,
    buffer: FixedBuffer<u8, BLOCKSIZE>,
    iv: FixedBuffer<u8, BLOCKSIZE>,
    out: Vec<u8>
}

impl<T: PrimitiveEncryption<B>, const B: usize> CbcEncryption<T, B> {

    /// Create a new CBC Encryption instance from a primitive and an IV.
    /// Up to the primitives blocksize of IV contents will be used.
    pub fn new(primitive: T, iv: &[u8]) -> Self {

        let ( buffer, mut iv_buf ) = ( FixedBuffer::new(), FixedBuffer::new() );
        iv_buf.push_slice(iv);

        let out = Vec::new();
        
        Self { primitive, buffer, iv: iv_buf, out }
    }

    fn process_buffer(&mut self) {
        self.primitive.encrypt(self.buffer.as_mut(), Some(self.iv.as_ref()), None);
        let encrypted = mem::replace(&mut self.buffer, FixedBuffer::new());
        
        self.iv.override_contents(encrypted.as_ref(), encrypted.len());
        self.out.extend(encrypted);
    }

    /// Returns a Readable with the processed contents
    pub fn finalize(self) -> Readable<Vec<u8>> {
        Readable::new(self.out)
    }
}

impl<T: PrimitiveEncryption<B>, const B: usize> io::Write for CbcEncryption<T, B> {

    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut written = 0;

        // Push buf until all contents have been written, if necessary, then encrypt buffer
        while written < buf.len() {

            written += self.buffer.push_slice(&buf[written..]);

            if self.buffer.is_full() { self.process_buffer(); }
        }

        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        use io::ErrorKind;
        if !self.buffer.is_full() {
            return Err(io::Error::new(ErrorKind::UnexpectedEof, BlockCipherError::IncompleteBlock(self.buffer.capacity())))
        }

        Ok(())
    }
    
}


pub struct CbcDecryption<T: PrimitiveDecryption<BLOCKSIZE>, const BLOCKSIZE: usize> {
    primitive: T,
    buffer: FixedBuffer<u8, BLOCKSIZE>,
    iv: FixedBuffer<u8, BLOCKSIZE>,
    out: Vec<u8>
}

impl<T: PrimitiveDecryption<B>, const B: usize> CbcDecryption<T, B> {

    pub fn new(primitive: T, iv: &[u8]) -> Self {

        let ( buffer, mut iv_buf ) = ( FixedBuffer::new(), FixedBuffer::new() );
        iv_buf.push_slice(iv);

        let out = Vec::new();
        
        Self { primitive, buffer, iv: iv_buf, out}
    }

    fn process_buffer(&mut self) {

        let new_iv = FixedBuffer::from(self.buffer.as_ref()); 
    
        self.primitive.decrypt(self.buffer.as_mut(), None, Some(self.iv.as_ref()));
        let decrypted = mem::replace(&mut self.buffer, FixedBuffer::new());

        self.iv = new_iv;
        
        self.out.extend(decrypted);
    }

    /// Returns a Readable with the processed contents
    pub fn finalize(self) -> Readable<Vec<u8>> {
        Readable::new(self.out)
    }
}

impl<T: PrimitiveDecryption<B>, const B: usize> io::Write for CbcDecryption<T, B> {

    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut written = 0;

        // Push buf until all contents have been written, if necessary, then encrypt buffer
        while written < buf.len() {

            written += self.buffer.push_slice(&buf[written..]);

            if self.buffer.is_full() { self.process_buffer(); }
        }

        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        use io::ErrorKind;
        if !self.buffer.is_full() {
            return Err(io::Error::new(ErrorKind::UnexpectedEof, BlockCipherError::IncompleteBlock(self.buffer.capacity())))
        }

        Ok(())
    }
    
}