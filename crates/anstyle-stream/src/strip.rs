use crate::adapter::StripBytes;
use crate::Lockable;
use crate::RawStream;

/// Only pass printable data to the inner `Write`
pub struct StripStream<S> {
    raw: S,
    state: StripBytes,
}

impl<S> StripStream<S>
where
    S: RawStream,
{
    /// Only pass printable data to the inner `Write`
    #[inline]
    pub fn new(raw: S) -> Self {
        Self {
            raw,
            state: Default::default(),
        }
    }

    /// Get the wrapped [`RawStream`]
    #[inline]
    pub fn into_inner(self) -> S {
        self.raw
    }
}

impl<S> std::io::Write for StripStream<S>
where
    S: RawStream,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let initial_state = self.state.clone();

        for printable in self.state.strip_next(buf) {
            let possible = printable.len();
            let written = self.raw.write(printable)?;
            if possible != written {
                let divergence = &printable[written..];
                let offset = offset_to(buf, divergence);
                let consumed = &buf[offset..];
                self.state = initial_state;
                self.state.strip_next(consumed).last();
                return Ok(offset);
            }
        }
        Ok(buf.len())
    }
    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        self.raw.flush()
    }

    // Provide explicit implementations of trait methods
    // - To reduce bookkeeping
    // - Avoid acquiring / releasing locks in a loop

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        for printable in self.state.strip_next(buf) {
            self.raw.write_all(printable)?;
        }
        Ok(())
    }

    // Not bothering with `write_fmt` as it just calls `write_all`
}

#[inline]
fn offset_to(total: &[u8], subslice: &[u8]) -> usize {
    let total = total.as_ptr();
    let subslice = subslice.as_ptr();

    debug_assert!(
        total <= subslice,
        "`Offset::offset_to` only accepts slices of `self`"
    );
    subslice as usize - total as usize
}

impl<S> Lockable for StripStream<S>
where
    S: Lockable,
{
    type Locked = StripStream<<S as Lockable>::Locked>;

    #[inline]
    fn lock(self) -> Self::Locked {
        Self::Locked {
            raw: self.raw.lock(),
            state: self.state,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;
    use std::io::Write as _;

    proptest! {
        #[test]
        #[cfg_attr(miri, ignore)]  // See https://github.com/AltSysrq/proptest/issues/253
        fn write_all_no_escapes(s in "\\PC*") {
            let buffer = crate::Buffer::new();
            let mut stream = StripStream::new(buffer);
            stream.write_all(s.as_bytes()).unwrap();
            let buffer = stream.into_inner();
            let actual = std::str::from_utf8(buffer.as_ref()).unwrap();
            assert_eq!(s, actual);
        }

        #[test]
        #[cfg_attr(miri, ignore)]  // See https://github.com/AltSysrq/proptest/issues/253
        fn write_byte_no_escapes(s in "\\PC*") {
            let buffer = crate::Buffer::new();
            let mut stream = StripStream::new(buffer);
            for byte in s.as_bytes() {
                stream.write_all(&[*byte]).unwrap();
            }
            let buffer = stream.into_inner();
            let actual = std::str::from_utf8(buffer.as_ref()).unwrap();
            assert_eq!(s, actual);
        }

        #[test]
        #[cfg_attr(miri, ignore)]  // See https://github.com/AltSysrq/proptest/issues/253
        fn write_all_random(s in any::<Vec<u8>>()) {
            let buffer = crate::Buffer::new();
            let mut stream = StripStream::new(buffer);
            stream.write_all(s.as_slice()).unwrap();
            let buffer = stream.into_inner();
            if let Ok(actual) = std::str::from_utf8(buffer.as_ref()) {
                for char in actual.chars() {
                    assert!(!char.is_ascii() || !char.is_control() || char.is_ascii_whitespace(), "{:?} -> {:?}: {:?}", String::from_utf8_lossy(&s), actual, char);
                }
            }
        }

        #[test]
        #[cfg_attr(miri, ignore)]  // See https://github.com/AltSysrq/proptest/issues/253
        fn write_byte_random(s in any::<Vec<u8>>()) {
            let buffer = crate::Buffer::new();
            let mut stream = StripStream::new(buffer);
            for byte in s.as_slice() {
                stream.write_all(&[*byte]).unwrap();
            }
            let buffer = stream.into_inner();
            if let Ok(actual) = std::str::from_utf8(buffer.as_ref()) {
                for char in actual.chars() {
                    assert!(!char.is_ascii() || !char.is_control() || char.is_ascii_whitespace(), "{:?} -> {:?}: {:?}", String::from_utf8_lossy(&s), actual, char);
                }
            }
        }
    }
}
