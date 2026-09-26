#![no_std]
extern crate alloc;

use alloc::{format, string::String, vec::Vec};
use core::{
    any::Any,
    error::Error,
    fmt::{self, Display, Formatter},
    mem::{MaybeUninit, size_of},
    result,
    slice::from_raw_parts_mut,
};

use bytemuck::{AnyBitPattern, NoUninit, bytes_of};

#[derive(Debug)]
pub enum WieError {
    InvalidMemoryAccess(u32),
    AllocationFailure,
    JavaException(u32), // to pass java exception down to rust
    JavaExceptionUnwind { context_base: u32, target: u32, next_pc: u32 },
    Unimplemented(String),
    FatalError(String),
}

impl Display for WieError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            WieError::InvalidMemoryAccess(address) => write!(f, "Invalid memory access; address: {address}"),
            WieError::AllocationFailure => write!(f, "Allocation failure"),
            WieError::JavaException(exception) => write!(f, "Java exception: {exception:#x}"),
            WieError::JavaExceptionUnwind {
                context_base,
                target,
                next_pc,
            } => write!(
                f,
                "Java exception unwind: context_base={context_base:#x}, target={target:#x}, next_pc={next_pc:#x}"
            ),
            WieError::Unimplemented(message) => write!(f, "Unimplemented: {message}"),
            WieError::FatalError(message) => write!(f, "Fatal error: {message}"),
        }
    }
}

impl Error for WieError {}

pub type Result<T> = result::Result<T, WieError>;

pub trait ByteRead {
    fn read_bytes(&self, address: u32, result: &mut [u8]) -> Result<usize>;
}

pub trait ByteWrite {
    fn write_bytes(&mut self, address: u32, data: &[u8]) -> Result<()>;
}

pub fn read_generic<T, R>(reader: &R, address: u32) -> Result<T>
where
    T: Copy + AnyBitPattern + NoUninit,
    R: ?Sized + ByteRead,
{
    if address == 0 {
        return Err(WieError::InvalidMemoryAccess(address));
    }

    let mut destination = MaybeUninit::<T>::uninit();
    let destination_bytes = unsafe { from_raw_parts_mut(destination.as_mut_ptr().cast::<u8>(), size_of::<T>()) };
    let read = reader.read_bytes(address, destination_bytes)?;
    if read != destination_bytes.len() {
        return Err(WieError::FatalError(format!(
            "Short read at {address:#x}: expected {}, got {read}",
            destination_bytes.len()
        )));
    }

    Ok(unsafe { destination.assume_init() })
}

pub fn read_null_terminated_string_bytes<R>(reader: &R, address: u32) -> Result<Vec<u8>>
where
    R: ?Sized + ByteRead,
{
    if address == 0 {
        return Err(WieError::InvalidMemoryAccess(address));
    }

    let mut result = Vec::new();
    let mut cursor = address;
    let mut buffer = [0; 32];
    loop {
        let (size, read) = match reader.read_bytes(cursor, &mut buffer) {
            // The terminator may precede an unmapped byte in this chunk.
            Err(WieError::InvalidMemoryAccess(_)) => (1, reader.read_bytes(cursor, &mut buffer[..1])?),
            result => (buffer.len(), result?),
        };
        if read != size {
            return Err(WieError::FatalError(format!("Short read at {cursor:#x}: expected {size}, got {read}")));
        }

        let bytes = &buffer[..size];
        if let Some(end) = bytes.iter().position(|&byte| byte == 0) {
            result.extend_from_slice(&bytes[..end]);
            break;
        }

        result.extend_from_slice(bytes);
        cursor = cursor.wrapping_add(size as u32);
    }

    Ok(result)
}

/// Longest guest string a diagnostic will quote back into a log.
///
/// `ptr` is a guest pointer and nothing proves it is not a path, so only a short
/// printable token is quoted; anything else yields `None` and the caller still
/// reports the raw pointer. Bounds what is QUOTED, not what is read.
pub const QUOTABLE_TOKEN_MAX: usize = 16;

/// `Some(token)` iff `bytes` is 1..=`QUOTABLE_TOKEN_MAX` printable ASCII bytes.
pub fn quotable_token(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() || bytes.len() > QUOTABLE_TOKEN_MAX || !bytes.iter().all(|b| (0x20..0x7f).contains(b)) {
        return None;
    }
    core::str::from_utf8(bytes).ok().map(String::from)
}

/// Reads the NUL-terminated string at `ptr` and quotes it via `quotable_token`.
/// Failure-tolerant: a null or unreadable pointer is "no token", never an error.
pub fn read_quotable_token<R>(reader: &R, ptr: u32) -> Option<String>
where
    R: ?Sized + ByteRead,
{
    if ptr == 0 {
        return None;
    }
    quotable_token(&read_null_terminated_string_bytes(reader, ptr).ok()?)
}

pub fn write_null_terminated_string_bytes<W>(writer: &mut W, address: u32, bytes: &[u8]) -> Result<()>
where
    W: ?Sized + ByteWrite,
{
    if address == 0 {
        return Err(WieError::InvalidMemoryAccess(address));
    }

    // TODO temp
    writer.write_bytes(address, bytes)?;
    writer.write_bytes(address + bytes.len() as u32, &[0])?;

    Ok(())
}

pub fn write_generic<W, T>(writer: &mut W, address: u32, data: T) -> Result<()>
where
    W: ?Sized + ByteWrite,
    T: NoUninit,
{
    if address == 0 {
        return Err(WieError::InvalidMemoryAccess(address));
    }

    let data_slice = bytes_of(&data);

    writer.write_bytes(address, data_slice)
}

pub fn read_null_terminated_table<R>(reader: &R, base_address: u32) -> Result<Vec<u32>>
where
    R: ?Sized + ByteRead,
{
    if base_address == 0 {
        return Err(WieError::InvalidMemoryAccess(base_address));
    }

    let mut cursor = base_address;
    let mut result = Vec::new();
    loop {
        let item: u32 = read_generic(reader, cursor)?;
        if item == 0 {
            break;
        }
        result.push(item);

        cursor += 4;
    }

    Ok(result)
}

pub fn write_null_terminated_table<W>(writer: &mut W, base_address: u32, items: &[u32]) -> Result<()>
where
    W: ?Sized + ByteWrite,
{
    if base_address == 0 {
        return Err(WieError::InvalidMemoryAccess(base_address));
    }

    let mut cursor = base_address;
    for &item in items {
        write_generic(writer, cursor, item)?;

        cursor += 4;
    }
    write_generic(writer, cursor, 0u32)
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> AsAny for T
where
    T: Any,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    struct StrictMemory {
        memory: Vec<u8>,
    }

    impl ByteRead for StrictMemory {
        fn read_bytes(&self, address: u32, result: &mut [u8]) -> Result<usize> {
            let address = address as usize;
            let end = address + result.len();
            if end > self.memory.len() {
                return Err(WieError::InvalidMemoryAccess(address as u32));
            }

            result.copy_from_slice(&self.memory[address..end]);

            Ok(result.len())
        }
    }

    #[test]
    fn read_generic_reads_into_initialized_storage() {
        let memory = StrictMemory {
            memory: vec![0, 0x78, 0x56, 0x34, 0x12],
        };

        let value: u32 = read_generic(&memory, 1).unwrap();

        assert_eq!(value, 0x1234_5678);
    }

    #[test]
    fn terminated_string_reads_stop_at_the_reader_boundary() {
        for bytes in [b"".as_slice(), b"test", &[0xff, 0x80], &[b'x'; 31], &[b'x'; 32], &[b'x'; 33]] {
            let mut memory = StrictMemory { memory: vec![0] };
            memory.memory.extend_from_slice(bytes);
            memory.memory.push(0);
            assert_eq!(read_null_terminated_string_bytes(&memory, 1).unwrap(), bytes);
            memory.memory.pop();
            assert!(matches!(
                read_null_terminated_string_bytes(&memory, 1),
                Err(WieError::InvalidMemoryAccess(address)) if address == bytes.len() as u32 + 1
            ));
        }
    }

    #[test]
    fn terminated_string_reads_preserve_short_reads_and_errors() {
        struct FailingReader(bool);

        impl ByteRead for FailingReader {
            fn read_bytes(&self, address: u32, result: &mut [u8]) -> Result<usize> {
                assert_eq!(result.len(), 32);
                if address == 1 {
                    result.fill(b'x');
                    Ok(result.len())
                } else if self.0 {
                    Ok(0)
                } else {
                    Err(WieError::AllocationFailure)
                }
            }
        }

        assert!(matches!(
            read_null_terminated_string_bytes(&FailingReader(true), 1),
            Err(WieError::FatalError(message)) if message == "Short read at 0x21: expected 32, got 0"
        ));
        assert!(matches!(
            read_null_terminated_string_bytes(&FailingReader(false), 1),
            Err(WieError::AllocationFailure)
        ));
    }

    #[test]
    fn quotable_token_quotes_only_short_printable_names() {
        // Tokens real guests pass: KTF database slot 8 and the unidentified-table stubs.
        for name in [&b"res"[..], b"ga", b"SaveData", b"SPORTS", b"FG_102"] {
            assert_eq!(quotable_token(name).as_deref(), Some(core::str::from_utf8(name).unwrap()));
        }
        // A literal 17, not `MAX + 1`: widening the constant must redden here.
        assert_eq!(QUOTABLE_TOKEN_MAX, 16);
        assert_eq!(quotable_token(&[b'a'; 17]), None);
        assert_eq!(quotable_token(&[b'a'; 16]).as_deref(), Some("aaaaaaaaaaaaaaaa"));
        assert_eq!(quotable_token(b"ab\x01cd"), None);
        assert_eq!(quotable_token(b"ab\xffcd"), None);
        assert_eq!(quotable_token(b""), None);
    }
}
