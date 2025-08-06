extern crate waveshare_barcodescanner;

use std::time::Duration;

use anyhow::Result;
use tracing::info;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};
use waveshare_barcodescanner::{interface::BarcodeScanner, Barcodes, IlluminationMode, OperationMode, ScanArea, TargetLightMode};

fn main() -> Result<()> {
    // console output
    let console_layer = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_filter(EnvFilter::from_default_env());

    tracing_subscriber::registry().with(console_layer).init();

    let serial_port = "/dev/ttyAMA0";
    info!("using port {}", serial_port);
    let mut interface = BarcodeScanner::new(serial_port)?;

    info!("setting mode");
    interface.set_mode(
        false,
        false,
        TargetLightMode::Standard,
        IlluminationMode::Standard,
        OperationMode::Command,
    )?;

    info!("setting scan area and enabling all barcodes");
    interface.set_scan_area_and_barcodes(ScanArea::Center, Barcodes::EnableAll)?;

    info!("setting scan time to 10s");
    interface.set_scan_timeout(Duration::from_secs(10))?;

    info!("starting scan");
    interface.start_scan()?;

    match interface.read_barcode()? {
        Some(barcode) => println!("{}", barcode),
        _ => println!("no barcode could be identified"),
    }

    Ok(())
}
