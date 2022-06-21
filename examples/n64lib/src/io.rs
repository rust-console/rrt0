#[cfg(feature = "io-isviewer64")]
pub mod isviewer;

/// Specify which I/O backend is automatically chosen by [`init`].
#[derive(Debug)]
pub enum IoBackend {
    /// No suitable I/O backend detected.
    None,

    /// Intelligent Systems Viewer 64.
    IsViewer64,
}

/// Initialize basic I/O.
pub fn init() -> IoBackend {
    #[cfg(feature = "io-isviewer64")]
    if isviewer::init() {
        return IoBackend::IsViewer64;
    }

    IoBackend::None
}
