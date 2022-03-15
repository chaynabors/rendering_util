use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    NoSuitableAdapter,
    NoSuitableDevice,
    IncompatibleSurface,
    SurfaceLost,
    OutOfMemory,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoSuitableAdapter => write!(f, "Couldn't find a suitable graphics adapter"),
            Self::NoSuitableDevice => write!(f, "Couldn't find a suitable graphics device"),
            Self::IncompatibleSurface => write!(f, "The surface was incompatible with the selected adapter"),
            Self::SurfaceLost => write!(f, "Attempted to retrieve the surface but it was lost"),
            Self::OutOfMemory => write!(f, "Application ran out of memory"),
        }
    }
}

impl std::error::Error for Error {}
