extern crate waveshare_barcodescanner;

use anyhow::Result;
use tracing::info;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};
use waveshare_barcodescanner::interface::BarcodeScanner;

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
    let hw_version = interface.get_hw_version()?;
    let sw_version = interface.get_sw_version()?;
    println!("version HW {}, SW {}", hw_version, sw_version);
    let sw_date = interface.get_sw_date()?;
    println!("SW build date {}", sw_date.format("%Y-%m-%d"));

    Ok(())
}
