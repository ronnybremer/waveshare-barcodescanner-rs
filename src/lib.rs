use std::fmt::Display;

pub mod crc;
pub mod interface;

// indicates, that the checksum was not calculated (checksum will not be validated)
pub const IGNORED_CHECKSUM: u16 = 0xabcd;

/// target light mode to indicate scanning
pub enum TargetLightMode {
    /// target light is disabled
    AlwaysOff,
    /// target light is always on (also when not scanning)
    AlwaysOn,
    /// target light is on during scanning
    Standard,
}

/// light mode for object detection in dark environments
pub enum IlluminationMode {
    /// white LED is disabled (scanning in dark environments might be difficult)
    AlwaysOff,
    /// white LED is always on (also when not scanning)
    AlwaysOn,
    /// white LED is on during scanning
    Standard,
}

/// scanner mode operation
pub enum OperationMode {
    /// push button to scan
    Manual,
    /// send command to scan
    Command,
    /// continuous scanning
    Continuous,
    /// detect ambient brightness change and start scanning
    Sensing,
}

/// scan area for bar codes
pub enum ScanArea {
    /// the entire area of view of the camera is used to detect barcodes
    All,
    /// the center area (default 20%) of the camera is used to detect barcodes
    Center,
}

/// type of barcodes to enable/disable (device dependent)
pub enum Barcodes {
    /// enable all supported barcodes
    EnableAll,
    /// disable all barcodes
    DisableAll,
    /// enable all default barcodes
    Default,
}

/// scanned barcode
pub enum Barcode {
    /// Interleaved 2of5, single line of digits
    Interleaved2of5(String),
    /// International Article Number - EAN13, single line of digits
    EAN13(String),
    /// Code 128, single line of alphanumeric characters
    Code128(String),
    /// Code 39, single line of alphanumeric characters
    Code39(String),
    /// QR code, multiple lines of alphanumeric characters
    QR(Vec<String>),
    /// mini QR code, multiple lines of alphanumeric characters
    MicroQR(Vec<String>),
    /// Dot Matrix code, multiple lines of alphanumeric characters
    DotMatrix(Vec<String>),
}

impl Display for Barcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Barcode::Interleaved2of5(data) => write!(f, "interleaved2of5: {}", data),
            Barcode::EAN13(data) => write!(f, "EAN13: {}", data),
            Barcode::Code39(data) => write!(f, "Code39: {}", data),
            Barcode::Code128(data) => write!(f, "Code128: {}", data),
            Barcode::QR(data) => write!(
                f,
                "QR: {}",
                data.iter()
                    .enumerate()
                    .map(|(i, line)| format!("{}: {}", i, line))
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
            Barcode::MicroQR(data) => write!(
                f,
                "Micro QR: {}",
                data.iter()
                    .enumerate()
                    .map(|(i, line)| format!("{}: {}", i, line))
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
            Barcode::DotMatrix(data) => write!(
                f,
                "Dot Matrix: {}",
                data.iter()
                    .enumerate()
                    .map(|(i, line)| format!("{}: {}", i, line))
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
        }
    }
}
