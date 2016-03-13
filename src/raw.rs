use std::io::{Write, Error, ErrorKind, Result as IoResult};
use std::ops::{Deref, DerefMut};

/// A terminal restorer, which keeps the previous state of the terminal, and restores it, when
/// dropped.
#[cfg(target_os = "redox")]
pub struct RawTerminal<W> {
    output: W,
}

#[cfg(target_os = "redox")]
impl<W: Write> Drop for RawTerminal<W> {
    fn drop(&mut self) {
        use TermControl;
        self.csi(b"R");
    }
}

#[cfg(not(target_os = "redox"))]
use termios::Termios;
/// A terminal restorer, which keeps the previous state of the terminal, and restores it, when
/// dropped.
#[cfg(not(target_os = "redox"))]
pub struct RawTerminal<W> {
    prev_ios: Termios,
    output: W,
}

#[cfg(not(target_os = "redox"))]
impl<W> Drop for RawTerminal<W> {
    fn drop(&mut self) {
        use termios::set_terminal_attr;
        set_terminal_attr(&mut self.prev_ios as *mut _);
    }
}

impl<W> Deref for RawTerminal<W> {
    type Target = W;

    fn deref(&self) -> &W {
        &self.output
    }
}
impl<W> DerefMut for RawTerminal<W> {
    fn deref_mut(&mut self) -> &mut W {
        &mut self.output
    }
}

impl<W: Write> Write for RawTerminal<W> {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.output.write(buf)
    }

    fn flush(&mut self) -> IoResult<()> {
        self.output.flush()
    }
}

/// Types which can be converted into "raw mode".
pub trait IntoRawMode: Sized {
    /// Switch to raw mode.
    ///
    /// Raw mode means that stdin won't be printed (it will instead have to be written manually by the
    /// program). Furthermore, the input isn't canonicalised or buffered (that is, you can read from
    /// stdin one byte of a time). The output is neither modified in any way.
    fn into_raw_mode(self) -> IoResult<RawTerminal<Self>>;
}

impl<W: Write> IntoRawMode for W {
    #[cfg(not(target_os = "redox"))]
    fn into_raw_mode(self) -> IoResult<RawTerminal<W>> {
        use termios::{cfmakeraw, get_terminal_attr, set_terminal_attr};

        let (mut ios, exit) = get_terminal_attr();
        let prev_ios = ios.clone();
        if exit != 0 {
            return Err(Error::new(ErrorKind::Other, "Unable to get Termios attribute."));
        }

        unsafe {
            cfmakeraw(&mut ios);
        }

        if set_terminal_attr(&mut ios as *mut _) != 0 {
            Err(Error::new(ErrorKind::Other, "Unable to set Termios attribute."))
        } else {
            Ok(RawTerminal {
                prev_ios: prev_ios,
                output: self,
            })
        }
    }
    #[cfg(target_os = "redox")]
    fn into_raw_mode(self) -> IoResult<RawTerminal<W>> {
        use TermControl;

        self.csi("r").map(|_| RawTerminal {
            output: self,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::{Write, stdout};

    #[test]
    fn test_into_raw_mode() {
        let mut out = stdout().into_raw_mode().unwrap();

        out.write(b"this is a test, muahhahahah").unwrap();
    }
}
