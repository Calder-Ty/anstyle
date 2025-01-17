use crate::Lockable;
use crate::RawStream;
use crate::StripStream;
#[cfg(feature = "wincon")]
use crate::WinconStream;

/// [`std::io::Write`] that adapts ANSI escape codes to the underlying `Write`s capabilities
pub struct AutoStream<S: RawStream> {
    inner: StreamInner<S>,
}

enum StreamInner<S: RawStream> {
    PassThrough(S),
    Strip(StripStream<S>),
    #[cfg(feature = "wincon")]
    Wincon(WinconStream<S>),
}

impl<S> AutoStream<S>
where
    S: RawStream,
{
    /// Force ANSI escape codes to be passed through as-is, no matter what the inner `Write`
    /// supports.
    #[inline]
    pub fn always_ansi(raw: S) -> Self {
        #[cfg(feature = "auto")]
        {
            if raw.is_terminal() {
                let _ = concolor_query::windows::enable_ansi_colors();
            }
        }
        Self::always_ansi_(raw)
    }

    #[inline]
    fn always_ansi_(raw: S) -> Self {
        let inner = StreamInner::PassThrough(raw);
        AutoStream { inner }
    }

    /// Force color, no matter what the inner `Write` supports.
    #[inline]
    pub fn always(raw: S) -> Self {
        #[cfg(feature = "wincon")]
        {
            if raw.is_terminal() && !concolor_query::windows::enable_ansi_colors().unwrap_or(true) {
                let console = anstyle_wincon::Console::new(raw);
                Self {
                    inner: StreamInner::Wincon(WinconStream::new(console)),
                }
            } else {
                Self::always_ansi_(raw)
            }
        }
        #[cfg(not(feature = "wincon"))]
        Self::always_ansi(raw)
    }

    /// Only pass printable data to the inner `Write`.
    #[inline]
    pub fn never(raw: S) -> Self {
        let inner = StreamInner::Strip(StripStream::new(raw));
        AutoStream { inner }
    }

    #[cfg(feature = "auto")]
    #[inline]
    pub(crate) fn auto(raw: S) -> Self {
        if raw.is_terminal() {
            Self::always(raw)
        } else {
            Self::never(raw)
        }
    }

    /// Get the wrapped [`RawStream`]
    #[inline]
    pub fn into_inner(self) -> S {
        match self.inner {
            StreamInner::PassThrough(w) => w,
            StreamInner::Strip(w) => w.into_inner(),
            #[cfg(feature = "wincon")]
            StreamInner::Wincon(w) => w.into_inner().into_inner(),
        }
    }
}

impl<S> AutoStream<S>
where
    S: Lockable + RawStream,
    <S as Lockable>::Locked: RawStream,
{
    /// Get exclusive access to the `AutoStream`
    ///
    /// Why?
    /// - Faster performance when writing in a loop
    /// - Avoid other threads interleaving output with the current thread
    #[inline]
    pub fn lock(self) -> <Self as Lockable>::Locked {
        let inner = match self.inner {
            StreamInner::PassThrough(w) => StreamInner::PassThrough(w.lock()),
            StreamInner::Strip(w) => StreamInner::Strip(w.lock()),
            #[cfg(feature = "wincon")]
            StreamInner::Wincon(w) => StreamInner::Wincon(w.lock()),
        };
        AutoStream { inner }
    }
}

impl<S> std::io::Write for AutoStream<S>
where
    S: RawStream,
{
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match &mut self.inner {
            StreamInner::PassThrough(w) => w.write(buf),
            StreamInner::Strip(w) => w.write(buf),
            #[cfg(feature = "wincon")]
            StreamInner::Wincon(w) => w.write(buf),
        }
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.inner {
            StreamInner::PassThrough(w) => w.flush(),
            StreamInner::Strip(w) => w.flush(),
            #[cfg(feature = "wincon")]
            StreamInner::Wincon(w) => w.flush(),
        }
    }

    // Provide explicit implementations of trait methods
    // - To reduce bookkeeping
    // - Avoid acquiring / releasing locks in a loop

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        match &mut self.inner {
            StreamInner::PassThrough(w) => w.write_all(buf),
            StreamInner::Strip(w) => w.write_all(buf),
            #[cfg(feature = "wincon")]
            StreamInner::Wincon(w) => w.write_all(buf),
        }
    }

    // Not bothering with `write_fmt` as it just calls `write_all`
}

impl<S> Lockable for AutoStream<S>
where
    S: Lockable + RawStream,
    <S as Lockable>::Locked: RawStream,
{
    type Locked = AutoStream<<S as Lockable>::Locked>;

    #[inline]
    fn lock(self) -> Self::Locked {
        self.lock()
    }
}
