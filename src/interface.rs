use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use chrono::NaiveDate;
use serial2::{CharSize, FlowControl, Parity, SerialPort, Settings, StopBits};
use tracing::{debug, trace};

use crate::{
    Barcode, Barcodes, IlluminationMode, OperationMode, ScanArea, TargetLightMode,
    crc::{calculate_crc, verify_crc},
};

pub struct BarcodeScanner {
    /// serial or USB port to communicate over
    port: SerialPort,
    /// timeout for a single scan (in manual or command mode), default is 5s
    scan_timeout: Duration,
}

impl BarcodeScanner {
    /// open the serial port and initialize the necessary device options for scanning
    ///
    /// # Arguments
    ///
    /// * `serial_port_name` the device name of the serial port to open
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use anyhow::Result;
    /// use waveshare_barcodescanner::interface::BarcodeScanner;
    /// 
    /// fn main() -> Result<()> {
    ///     let scanner = BarcodeScanner::new("/dev/serial0")?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// The device is expected to be in UART mode, with the serial options set to 9600,8,N,1 (factory default).
    pub fn new(serial_port_name: &str) -> Result<Self> {
        let mut port = SerialPort::open(serial_port_name, |mut settings: Settings| {
            settings.set_raw();
            settings.set_baud_rate(9600)?;
            settings.set_char_size(CharSize::Bits8);
            settings.set_stop_bits(StopBits::One);
            settings.set_parity(Parity::None);
            settings.set_flow_control(FlowControl::None);
            Ok(settings)
        })?;
        port.discard_buffers()?;
        port.set_read_timeout(Duration::from_millis(100))?;
        let mut scanner = BarcodeScanner {
            port,
            scan_timeout: Duration::from_secs(5),
        };
        // in order for barcode payload decoding to work corectly, always set the decoding options
        let mut default_barcode_result_options: u8 = 0x00;
        // without protocol
        // CR as end of line
        // without RF
        // without prefix
        // with CodeID
        default_barcode_result_options |= 0x04;
        // without suffix
        // with end character
        default_barcode_result_options |= 0x01;
        scanner.send_write_command(0x0060, &[default_barcode_result_options])?;
        Ok(scanner)
    }

    /// return the hardware version of the attached barcode scanner
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use anyhow::Result;
    /// use waveshare_barcodescanner::interface::BarcodeScanner;
    /// 
    /// fn main() -> Result<()> {
    ///     let mut scanner = BarcodeScanner::new("/dev/serial0")?;
    ///     println!("Detected barcode scanner hardware version: {}", scanner.get_hw_version()?);
    ///     Ok(())
    /// }
    /// ```
    pub fn get_hw_version(&mut self) -> Result<String> {
        let mut buffer: [u8; 1] = [0; 1];
        let bytes_read = self.send_read_command_fixed_reply(0x00E1, &mut buffer)?;
        if bytes_read != 1 {
            return Err(anyhow!(
                "incorrect number of bytes read from device, expected 1 received {}",
                bytes_read
            ));
        }
        match buffer[0] {
            0x64 => Ok(String::from("V1.00")),
            0x6E => Ok(String::from("V1.10")),
            0x78 => Ok(String::from("V1.20")),
            0x82 => Ok(String::from("V1.30")),
            0x8C => Ok(String::from("V1.40")),
            _ => Ok(format!("unknown {:0X}", buffer[0])),
        }
    }

    /// return the software version of the attached barcode scanner
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use anyhow::Result;
    /// use waveshare_barcodescanner::interface::BarcodeScanner;
    /// 
    /// fn main() -> Result<()> {
    ///     let mut scanner = BarcodeScanner::new("/dev/serial0")?;
    ///     println!("Detected barcode scanner software version: {}", scanner.get_sw_version()?);
    ///     Ok(())
    /// }
    /// ```
    pub fn get_sw_version(&mut self) -> Result<String> {
        let mut buffer: [u8; 1] = [0; 1];
        let bytes_read = self.send_read_command_fixed_reply(0x00E2, &mut buffer)?;
        if bytes_read != 1 {
            return Err(anyhow!(
                "incorrect number of bytes read from device, expected 1 received {}",
                bytes_read
            ));
        }
        match buffer[0] {
            0x64 => Ok(String::from("V1.00")),
            0x6E => Ok(String::from("V1.10")),
            0x78 => Ok(String::from("V1.20")),
            0x82 => Ok(String::from("V1.30")),
            0x8C => Ok(String::from("V1.40")),
            _ => Ok(format!("unknown {:0X}", buffer[0])),
        }
    }

    /// return the software date of the attached barcode scanner
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use anyhow::Result;
    /// use waveshare_barcodescanner::interface::BarcodeScanner;
    /// 
    /// fn main() -> Result<()> {
    ///     let mut scanner = BarcodeScanner::new("/dev/serial0")?;
    ///     println!("Detected barcode scanner software build date: {}", scanner.get_sw_date()?.format("%Y/%m/%d"));
    ///     Ok(())
    /// }
    /// ```
    pub fn get_sw_date(&mut self) -> Result<NaiveDate> {
        let mut buffer: [u8; 1] = [0; 1];
        let bytes_read = self.send_read_command_fixed_reply(0x00E3, &mut buffer)?;
        if bytes_read != 1 {
            return Err(anyhow!(
                "incorrect number of bytes read from device, expected 1 received {}",
                bytes_read
            ));
        }
        // only the years past year 2000 are returned
        let year: i32 = buffer[0] as i32 + 2000;
        let bytes_read = self.send_read_command_fixed_reply(0x00E4, &mut buffer)?;
        if bytes_read != 1 {
            return Err(anyhow!(
                "incorrect number of bytes read from device, expected 1 received {}",
                bytes_read
            ));
        }
        let month: u32 = buffer[0] as u32;
        let bytes_read = self.send_read_command_fixed_reply(0x00E5, &mut buffer)?;
        if bytes_read != 1 {
            return Err(anyhow!(
                "incorrect number of bytes read from device, expected 1 received {}",
                bytes_read
            ));
        }
        let day: u32 = buffer[0] as u32;
        let date = match NaiveDate::from_ymd_opt(year, month, day) {
            Some(date) => date,
            None => {
                return Err(anyhow!(
                    "unable to construct date from  year {} month {} day {}",
                    year,
                    month,
                    day
                ));
            }
        };
        Ok(date)
    }

    /// start scanning for barcodes
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use anyhow::Result;
    /// use waveshare_barcodescanner::interface::BarcodeScanner;
    /// 
    /// fn main() -> Result<()> {
    ///     let mut scanner = BarcodeScanner::new("/dev/serial0")?;
    ///     println!("Please scan your badge for identification.");
    ///     scanner.start_scan()?;
    ///     match scanner.read_barcode()? {
    ///         Some(barcode) => println!("data scanned: {}", barcode),
    ///         None => println!("Please try again. Make sure the barcode on your badge is at the center of the green scanning light."),
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn start_scan(&mut self) -> Result<()> {
        self.send_write_command(0x0002, &[0x01])
    }

    /// stop scanning for barcodes
    ///
    /// # Notes
    ///
    /// When a barcode has been scanned, the scanner automatically stops scanning in manual and command mode.
    pub fn stop_scan(&mut self) -> Result<()> {
        self.send_write_command(0x0002, &[0x00])
    }

    /// waits for a barcode payload to be returned from the scanner
    ///
    /// will return immediately when the payload has been read or the `scan_timeout` has been reached
    pub fn read_barcode(&mut self) -> Result<Option<Barcode>> {
        // read the code ID
        let mut codeid_buffer: [u8; 1] = [0x00; 1];
        self.read_from_serial_exact(&mut codeid_buffer, Some(self.scan_timeout))?;
        if codeid_buffer[0] == 0x00 {
            // no data received until the timout was reached
            // 0x00 is an invalid Code ID
            debug!("timeout waiting for barcode data");
            return Ok(None);
        }
        let mut lines: Vec<Vec<u8>> = Vec::new();
        loop {
            trace!("reading next line");
            let (read_line, end_of_data) = self.read_line_from_serial()?;
            if read_line.len() > 0 {
                lines.push(read_line);
            }
            if end_of_data {
                // no more data to read
                trace!("no more data to read");
                break;
            }
        }
        if lines.is_empty() {
            debug!("no barcode data was read from the device");
            return Ok(None);
        }
        debug!(
            "{} line(s) of barcode data was read from the device",
            lines.len()
        );
        match codeid_buffer[0] {
            0x65 => {
                debug!("Interleaved 2of5");
                Ok(Some(Barcode::Interleaved2of5(
                    lines.get(0).unwrap().iter().map(|&c| c as char).collect(),
                )))
            }
            0x62 => {
                debug!("Code39");
                Ok(Some(Barcode::Code39(
                    lines.get(0).unwrap().iter().map(|&c| c as char).collect(),
                )))
            }
            0x64 => {
                debug!("EAN13");
                Ok(Some(Barcode::EAN13(
                    lines.get(0).unwrap().iter().map(|&c| c as char).collect(),
                )))
            }
            0x6A => {
                debug!("Code128");
                Ok(Some(Barcode::Code128(
                    lines.get(0).unwrap().iter().map(|&c| c as char).collect(),
                )))
            }
            0x51 => {
                debug!("QR code");
                Ok(Some(Barcode::QR(
                    lines
                        .iter()
                        .map(|line| line.iter().map(|&c| c as char).collect())
                        .collect(),
                )))
            }
            0x75 => {
                debug!("Dot Matrix code");
                Ok(Some(Barcode::DotMatrix(
                    lines
                        .iter()
                        .map(|line| line.iter().map(|&c| c as char).collect())
                        .collect(),
                )))
            }
            _ => Err(anyhow!(
                "unsupported barcode type received: {:02X}",
                codeid_buffer[0]
            )),
        }
    }

    /// set the mode of operation and light/buzzer parameters
    ///
    /// # Arguments
    ///
    /// * `enable_led_indication_on_successful_scan` if true the LED on the circuit board will light up shortly after a successful scan
    /// * `enable_buzzer` if true a successful scan will be confirmed with a short tone
    /// * `target_light_mode` specifies how the green target light is used
    /// * `illumination_mode` specifies how the white LED light will operate
    /// * `operation_mode` defines the scanning mode
    ///
    /// # Note
    ///
    /// Only manual and command scanning operations are currently supported.
    pub fn set_mode(
        &mut self,
        enable_led_indication_on_successful_scan: bool,
        enable_buzzer: bool,
        target_light_mode: TargetLightMode,
        illumination_mode: IlluminationMode,
        operation_mode: OperationMode,
    ) -> Result<()> {
        let mut operation: u8 = 0x0;
        if enable_led_indication_on_successful_scan {
            operation |= 0x80;
        }
        if enable_buzzer {
            operation |= 0x40;
        }
        match target_light_mode {
            TargetLightMode::AlwaysOff => {}
            TargetLightMode::AlwaysOn => operation |= 0x20,
            TargetLightMode::Standard => operation |= 0x10,
        }
        match illumination_mode {
            IlluminationMode::AlwaysOff => {}
            IlluminationMode::AlwaysOn => operation |= 0x08,
            IlluminationMode::Standard => operation |= 0x04,
        }
        match operation_mode {
            OperationMode::Manual => {}
            OperationMode::Command => operation |= 0x01,
            OperationMode::Continuous => operation |= 0x02,
            OperationMode::Sensing => operation |= 0x03,
        }
        self.send_write_command(0x0000, &[operation])
    }

    /// set the scanning area and barcodes allowed
    ///
    /// # Arguments
    ///
    /// * `scan_area` defines the area of the camera view where barcodes are detected
    /// * `allowed_barcodes` specifies which barcode types are recognized based on the barcode scanner model
    pub fn set_scan_area_and_barcodes(
        &mut self,
        scan_area: ScanArea,
        allowed_barcodes: Barcodes,
    ) -> Result<()> {
        let mut scanner_setting: u8 = 0x0;
        match scan_area {
            ScanArea::All => {}
            ScanArea::Center => scanner_setting |= 0x08,
        }
        match allowed_barcodes {
            Barcodes::EnableAll => scanner_setting |= 0x02,
            Barcodes::DisableAll => {}
            Barcodes::Default => scanner_setting |= 0x04,
        }
        self.send_write_command(0x002C, &[scanner_setting])
    }

    /// set the maximum time for a manual or command scan before the scanner goes inactive again
    ///
    /// # Arguments
    ///
    /// * `scan_timeout` the duration to activate scanning for, allowed range is 1ms to 25.5s
    ///
    /// # Note
    ///
    /// By default the barcode scanner is set to 5s of scanning duration
    pub fn set_scan_timeout(&mut self, scan_timeout: Duration) -> Result<()> {
        if scan_timeout > Duration::from_millis(25500) {
            return Err(anyhow!(
                "timeout is too big, maximum value is 25500 ms (25.5s)"
            ));
        }
        if scan_timeout == Duration::from_millis(0) {
            return Err(anyhow!("this crate does not support indefinite waiting"));
        }
        let scan_timeout_byte: u8 = (scan_timeout.as_millis() / 100).try_into()?;
        self.send_write_command(0x0006, &[scan_timeout_byte])?;
        self.scan_timeout = scan_timeout;
        Ok(())
    }

    /// enable/disable barcode type: EAN13
    pub fn allow_ean13(&mut self, enable: bool) -> Result<()> {
        if enable {
            self.send_write_command(0x002E, &[0x01])
        } else {
            self.send_write_command(0x002E, &[0x00])
        }
    }

    /// enable/disable barcode type: EAN8
    pub fn allow_ean8(&mut self, enable: bool) -> Result<()> {
        if enable {
            self.send_write_command(0x002F, &[0x01])
        } else {
            self.send_write_command(0x002F, &[0x00])
        }
    }

    /// enable/disable barcode type: GS1 Databar Stacked(RSS)
    pub fn allow_rss_stack(&mut self, enable: bool) -> Result<()> {
        if enable {
            self.send_write_command(0x0026, &[0x01])
        } else {
            self.send_write_command(0x0026, &[0x00])
        }
    }

    /// enable/disable barcode type: GS1 Databar(RSS-14)
    pub fn allow_rss14(&mut self, enable: bool) -> Result<()> {
        if enable {
            self.send_write_command(0x004F, &[0x01])
        } else {
            self.send_write_command(0x004F, &[0x00])
        }
    }

    /// enable/disable barcode type: GS1 Databar Limited(RSS)
    pub fn allow_limited_rss(&mut self, enable: bool) -> Result<()> {
        if enable {
            self.send_write_command(0x0050, &[0x01])
        } else {
            self.send_write_command(0x0050, &[0x00])
        }
    }

    /// enable/disable barcode type: GS1 Databar Expanded(RSS)
    ///
    /// optionally specify the minimum and/or maximum number of characters expected in a valid barcode,
    /// all others will be ignored
    pub fn allow_expanded_rss(
        &mut self,
        enable: bool,
        min_length: Option<u8>,
        max_length: Option<u8>,
    ) -> Result<()> {
        if enable {
            if let Some(min_length) = min_length {
                self.send_write_command(0x0052, &[min_length])?;
            }
            if let Some(max_length) = max_length {
                self.send_write_command(0x0053, &[max_length])?;
            }
            self.send_write_command(0x0051, &[0x01])
        } else {
            self.send_write_command(0x0051, &[0x00])
        }
    }

    /// enable/disable barcode type: MSI-Plessey
    ///
    /// optionally specify the minimum and/or maximum number of characters expected in a valid barcode,
    /// all others will be ignored
    pub fn allow_msi(
        &mut self,
        enable: bool,
        min_length: Option<u8>,
        max_length: Option<u8>,
    ) -> Result<()> {
        if enable {
            if let Some(min_length) = min_length {
                self.send_write_command(0x004D, &[min_length])?;
            }
            if let Some(max_length) = max_length {
                self.send_write_command(0x004E, &[max_length])?;
            }
            self.send_write_command(0x004C, &[0x01])
        } else {
            self.send_write_command(0x004C, &[0x00])
        }
    }

    /// enable/disable barcode type: Code11
    ///
    /// optionally specify the minimum and/or maximum number of characters expected in a valid barcode,
    /// all others will be ignored
    pub fn allow_code11(
        &mut self,
        enable: bool,
        min_length: Option<u8>,
        max_length: Option<u8>,
    ) -> Result<()> {
        if enable {
            if let Some(min_length) = min_length {
                self.send_write_command(0x004A, &[min_length])?;
            }
            if let Some(max_length) = max_length {
                self.send_write_command(0x004B, &[max_length])?;
            }
            self.send_write_command(0x0049, &[0x01])
        } else {
            self.send_write_command(0x0049, &[0x00])
        }
    }

    /// enable/disable barcode type: Matrix 2of5
    ///
    /// optionally specify the minimum and/or maximum number of characters expected in a valid barcode,
    /// all others will be ignored
    pub fn allow_matrix2of5(
        &mut self,
        enable: bool,
        min_length: Option<u8>,
        max_length: Option<u8>,
    ) -> Result<()> {
        if enable {
            if let Some(min_length) = min_length {
                self.send_write_command(0x0047, &[min_length])?;
            }
            if let Some(max_length) = max_length {
                self.send_write_command(0x0048, &[max_length])?;
            }
            self.send_write_command(0x0046, &[0x01])
        } else {
            self.send_write_command(0x0046, &[0x00])
        }
    }

    /// enable/disable barcode type: Industrial
    ///
    /// optionally specify the minimum and/or maximum number of characters expected in a valid barcode,
    /// all others will be ignored
    pub fn allow_industrial(
        &mut self,
        enable: bool,
        min_length: Option<u8>,
        max_length: Option<u8>,
    ) -> Result<()> {
        if enable {
            if let Some(min_length) = min_length {
                self.send_write_command(0x0044, &[min_length])?;
            }
            if let Some(max_length) = max_length {
                self.send_write_command(0x0045, &[max_length])?;
            }
            self.send_write_command(0x0043, &[0x01])
        } else {
            self.send_write_command(0x0043, &[0x00])
        }
    }

    /// enable/disable barcode type: CodeBar
    ///
    /// optionally specify the minimum and/or maximum number of characters expected in a valid barcode,
    /// all others will be ignored
    pub fn allow_codebar(
        &mut self,
        enable: bool,
        with_start_stop_character: bool,
        min_length: Option<u8>,
        max_length: Option<u8>,
    ) -> Result<()> {
        if enable {
            if let Some(min_length) = min_length {
                self.send_write_command(0x003D, &[min_length])?;
            }
            if let Some(max_length) = max_length {
                self.send_write_command(0x003E, &[max_length])?;
            }
            let mut mode: u8 = 0x01;
            if with_start_stop_character {
                mode |= 0x02;
            }
            self.send_write_command(0x003C, &[mode])
        } else {
            self.send_write_command(0x003C, &[0x00])
        }
    }

    /// enable/disable barcode type: Code128
    ///
    /// optionally specify the minimum and/or maximum number of characters expected in a valid barcode,
    /// all others will be ignored
    pub fn allow_code128(
        &mut self,
        enable: bool,
        min_length: Option<u8>,
        max_length: Option<u8>,
    ) -> Result<()> {
        if enable {
            if let Some(min_length) = min_length {
                self.send_write_command(0x0034, &[min_length])?;
            }
            if let Some(max_length) = max_length {
                self.send_write_command(0x0035, &[max_length])?;
            }
            self.send_write_command(0x0033, &[0x01])
        } else {
            self.send_write_command(0x0033, &[0x00])
        }
    }

    /// enable/disable barcode type: Code39
    ///
    /// optionally specify the minimum and/or maximum number of characters expected in a valid barcode,
    /// all others will be ignored
    pub fn allow_code39(
        &mut self,
        enable: bool,
        min_length: Option<u8>,
        max_length: Option<u8>,
    ) -> Result<()> {
        if enable {
            if let Some(min_length) = min_length {
                self.send_write_command(0x0037, &[min_length])?;
            }
            if let Some(max_length) = max_length {
                self.send_write_command(0x0038, &[max_length])?;
            }
            self.send_write_command(0x0036, &[0x01])
        } else {
            self.send_write_command(0x0036, &[0x00])
        }
    }

    /// enable/disable barcode type: Code93
    ///
    /// optionally specify the minimum and/or maximum number of characters expected in a valid barcode,
    /// all others will be ignored
    pub fn allow_code93(
        &mut self,
        enable: bool,
        min_length: Option<u8>,
        max_length: Option<u8>,
    ) -> Result<()> {
        if enable {
            if let Some(min_length) = min_length {
                self.send_write_command(0x003A, &[min_length])?;
            }
            if let Some(max_length) = max_length {
                self.send_write_command(0x003B, &[max_length])?;
            }
            self.send_write_command(0x0039, &[0x01])
        } else {
            self.send_write_command(0x0039, &[0x00])
        }
    }

    /// enable/disable barcode type: UPCA
    pub fn allow_upca(&mut self, enable: bool) -> Result<()> {
        if enable {
            self.send_write_command(0x0030, &[0x01])
        } else {
            self.send_write_command(0x0030, &[0x00])
        }
    }

    /// enable/disable barcode type: UPCE0
    pub fn allow_upce0(&mut self, enable: bool) -> Result<()> {
        if enable {
            self.send_write_command(0x0031, &[0x01])
        } else {
            self.send_write_command(0x0031, &[0x00])
        }
    }

    /// enable/disable barcode type: UPCE1
    pub fn allow_upce1(&mut self, enable: bool) -> Result<()> {
        if enable {
            self.send_write_command(0x0032, &[0x01])
        } else {
            self.send_write_command(0x0032, &[0x00])
        }
    }

    /// enable/disable barcode type: PDF417
    pub fn allow_pdf417(&mut self, enable: bool) -> Result<()> {
        if enable {
            self.send_write_command(0x0055, &[0x01])
        } else {
            self.send_write_command(0x0055, &[0x00])
        }
    }

    /// enable/disable barcode type: Micro PDF417
    pub fn allow_micro_pdf417(&mut self, enable: bool) -> Result<()> {
        if enable {
            self.send_write_command(0x0029, &[0x01])
        } else {
            self.send_write_command(0x0029, &[0x00])
        }
    }

    /// enable/disable barcode type: ISBN
    pub fn allow_micro_isbn(&mut self, enable: bool) -> Result<()> {
        if enable {
            self.send_write_command(0x0057, &[0x01])
        } else {
            self.send_write_command(0x0057, &[0x00])
        }
    }

    /// enable/disable barcode type: ISSN
    pub fn allow_micro_issn(&mut self, enable: bool) -> Result<()> {
        if enable {
            self.send_write_command(0x0056, &[0x01])
        } else {
            self.send_write_command(0x0056, &[0x00])
        }
    }

    /// enable/disable barcode type: Dot Matrix code
    pub fn allow_dotmatrix(&mut self, enable: bool) -> Result<()> {
        if enable {
            self.send_write_command(0x0054, &[0x01])
        } else {
            self.send_write_command(0x0054, &[0x00])
        }
    }

    /// enable/disable barcode type: QR code
    pub fn allow_qr(&mut self, enable: bool) -> Result<()> {
        if enable {
            self.send_write_command(0x003F, &[0x01])
        } else {
            self.send_write_command(0x003F, &[0x00])
        }
    }

    /// enable/disable barcode type: Micro QR
    pub fn allow_microqr(&mut self, enable: bool) -> Result<()> {
        if enable {
            self.send_write_command(0x005F, &[0x01])
        } else {
            self.send_write_command(0x005F, &[0x00])
        }
    }

    /// enable/disable barcode type: Interleaved 2 of 5
    ///
    /// optionally specify the minimum and/or maximum number of characters expected in a valid barcode,
    /// all others will be ignored
    pub fn allow_interleaved2of5(
        &mut self,
        enable: bool,
        min_length: Option<u8>,
        max_length: Option<u8>,
    ) -> Result<()> {
        if enable {
            if let Some(min_length) = min_length {
                self.send_write_command(0x0041, &[min_length])?;
            }
            if let Some(max_length) = max_length {
                self.send_write_command(0x0042, &[max_length])?;
            }
            self.send_write_command(0x0040, &[0x01])
        } else {
            self.send_write_command(0x0040, &[0x00])
        }
    }

    /// disable setting changes via barcode scanning (seems like a really good idea for production use)
    pub fn disable_setting_scanning(&mut self) -> Result<()> {
        let mut buffer: [u8; 1] = [0x00; 1];
        let bytes_read = self.send_read_command_fixed_reply(0x0003, &mut buffer)?;
        if bytes_read != 1 {
            return Err(anyhow!(
                "incorrect number of bytes read from device, expected 1 received {}",
                bytes_read
            ));
        }
        buffer[0] &= 0xfe;
        buffer[0] |= 0x02;
        self.send_write_command(0x0003, &buffer)?;
        Ok(())
    }

    /// enable setting changes via barcode scanning
    pub fn enable_setting_scanning(&mut self) -> Result<()> {
        let mut buffer: [u8; 1] = [0x00; 1];
        let bytes_read = self.send_read_command_fixed_reply(0x0003, &mut buffer)?;
        if bytes_read != 1 {
            return Err(anyhow!(
                "incorrect number of bytes read from device, expected 1 received {}",
                bytes_read
            ));
        }
        buffer[0] &= 0xfc;
        self.send_write_command(0x0003, &buffer)?;
        Ok(())
    }

    /// save all pending changes to flash
    pub fn save_to_flash(&mut self) -> Result<()> {
        self.send_to_serial(0x09, 1, 0x0000, None, Some(&[0x00]))?;
        let mut buffer: [u8; 1] = [0x00; 1];
        self.read_from_serial_command_reply(&mut buffer)?;
        Ok(())
    }

    /// set the barcode scanner to factory defaults
    ///
    /// # Note
    ///
    /// Make sure to afterwards enable `UART` mode via the corresponding barcode again.
    pub fn factory_reset(&mut self) -> Result<()> {
        self.send_to_serial(0x08, 1, 0x00D9, None, Some(&[0x50]))?;
        let mut buffer: [u8; 1] = [0x00; 1];
        self.read_from_serial_command_reply(&mut buffer)?;
        Ok(())
    }

    /// send a read command and return the reply from the barcode scanner
    ///
    /// used for an expected payload size upon read
    fn send_read_command_fixed_reply(
        &mut self,
        address: u16,
        return_data: &mut [u8],
    ) -> Result<usize> {
        self.send_to_serial(0x07, 0x01, address, Some(return_data.len()), None)?;
        self.read_from_serial_command_reply(return_data)
    }

    /// send a write command to the barcode scanner
    fn send_write_command(&mut self, address: u16, data: &[u8]) -> Result<()> {
        self.send_to_serial(0x08, data.len().try_into()?, address, None, Some(data))?;
        let mut buffer: [u8; 1] = [0x00; 1];
        self.read_from_serial_command_reply(&mut buffer)?;
        Ok(())
    }

    /// write a command packet to the barcode scanner
    fn send_to_serial(
        &mut self,
        function_type: u8,
        length: u8,
        address: u16,
        return_data_length: Option<usize>,
        write_data: Option<&[u8]>,
    ) -> Result<()> {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.push(0x7e);
        buffer.push(0x00);
        buffer.push(function_type);
        buffer.push(length);
        buffer.append(&mut address.to_be_bytes().to_vec());
        if let Some(return_data_length) = return_data_length {
            // special case, the barcode scanner expects a length of 0 when 256 bytes should be returned
            if return_data_length == 256 {
                buffer.push(0x00);
            } else {
                buffer.push(return_data_length.try_into()?);
            }
        }
        if let Some(write_data) = write_data {
            buffer.append(&mut write_data.to_vec());
        }
        buffer.append(&mut calculate_crc(&buffer[2..])?.to_be_bytes().to_vec());
        debug!("WRITE TO SERIAL {} bytes: {:02X?}", buffer.len(), buffer);
        self.port.write_all(&buffer)?;
        Ok(())
    }

    /// read a command reply packet from the barcode scanner
    ///
    /// the `read_data` array needs to be allocated to the exact size of payload data expected
    fn read_from_serial_command_reply(&mut self, read_data: &mut [u8]) -> Result<usize> {
        debug!("reading {} bytes", read_data.len() + 6);
        let mut buffer: Vec<u8> = (0..read_data.len() + 6).map(|_| 0x00).collect();
        self.port.read_exact(&mut buffer)?;
        debug!("READ FROM SERIAL {} bytes: {:02X?}", buffer.len(), buffer);
        if buffer.len() < 6 {
            // not enough data read
            return Err(anyhow!(
                "read data is shorter than expected, got {} expected {} bytes",
                buffer.len(),
                read_data.len()
            ));
        }
        if buffer[0] != 0x02 || buffer[1] != 0x00 {
            // invalid header
            return Err(anyhow!("invalid header received"));
        }
        if buffer[2] != 0x00 {
            // operation was not successful
            return Err(anyhow!(
                "barcode scanner indicates an unsuccessful operation, rc: {}",
                buffer[2]
            ));
        }
        // verify the checksum
        let received_checksum =
            ((buffer[buffer.len() - 2] as u16) << 8) | buffer[buffer.len() - 1] as u16;
        verify_crc(&buffer[2..buffer.len() - 2], received_checksum)?;
        let mut data_length: usize = buffer[3] as usize;
        // special case, the barcode scanner returns a length of 0 when 256 bytes have been returned
        if data_length == 0 {
            data_length = 256;
        }
        read_data[..data_length as usize].copy_from_slice(&buffer[4..data_length + 4]);
        Ok(data_length)
    }

    /// read a full packet from the barcode scanner
    ///
    /// the `read_data` array needs to be allocated to the exact size of data expected
    ///
    /// wait until the optional `timeout` if specified
    fn read_from_serial_exact(
        &mut self,
        read_data: &mut [u8],
        timeout: Option<Duration>,
    ) -> Result<()> {
        let start_ts = Instant::now();
        loop {
            debug!("reading {} bytes", read_data.len());
            match self.port.read_exact(read_data) {
                Ok(()) => {}
                Err(err) => {
                    if let Some(timeout) = timeout
                        && err.kind() == std::io::ErrorKind::TimedOut
                    {
                        // no data from the serial interface yet, use the passed in timeout
                        if start_ts.elapsed() < timeout {
                            continue;
                        }
                        // timeout reached, return everything read so far
                        return Ok(());
                    }
                    return Err(err.into());
                }
            }
            debug!(
                "READ FROM SERIAL {} bytes: {:02X?}",
                read_data.len(),
                read_data
            );
            return Ok(());
        }
    }

    /// read the next line of recognized data from the barcode scanner
    ///
    /// will wait until EOL or end of data
    fn read_line_from_serial(&mut self) -> Result<(Vec<u8>, bool)> {
        let mut result: Vec<u8> = Vec::with_capacity(256);
        loop {
            let mut buffer: [u8; 1] = [0; 1];
            debug!("reading {} bytes", buffer.len());
            match self.port.read(&mut buffer) {
                Ok(0) => {
                    return Err(anyhow!("no data was read from the device"));
                }
                Ok(read_bytes) => {
                    trace!(
                        "READ FROM SERIAL {} bytes: {:02X?}",
                        read_bytes,
                        &buffer[..read_bytes]
                    );
                    if buffer[0] == 0x0A {
                        trace!("EOL detected, result len {}", result.len());
                        return Ok((result, false));
                    }
                    if buffer[0] == 0x0D {
                        trace!("end of data detected, result len {}", result.len());
                        return Ok((result, true));
                    }
                    result.append(&mut buffer[..read_bytes].to_vec());
                }
                Err(err) => {
                    return Err(err.into());
                }
            }
        }
    }
}
