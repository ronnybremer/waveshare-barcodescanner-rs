use anyhow::{Result, anyhow};
use crc_any::CRC;
use tracing::trace;

use crate::IGNORED_CHECKSUM;

/// calculate the checksum of a command or reply packet
/// 
/// # Arguments
///
/// * `data` - the data to calculate the checksum on
/// 
/// # Returns
/// 
/// * the checksum as a u16
///
pub fn calculate_crc(data: &[u8]) -> Result<u16> {
    // Use CRC_CCITT polynomial: X16+X12+X5+1, whose coefficients is 0x1021.
    // Initial value is 0, first calculate high bit for single byte without negating.
    let mut crc_calc = CRC::create_crc_u16(0x1021, 16, 0, 0, false);
    crc_calc.digest(data);
    let crc = crc_calc.get_crc();
    trace!("CRC {:04X}", crc);
    Ok(crc.try_into()?)
}

pub fn verify_crc(data: &[u8], expected_checksum: u16) -> Result<()> {
    if expected_checksum == IGNORED_CHECKSUM {
        // the passed in checksum should be ignored (was not calculated in the first place)
        return Ok(());
    }
    let calculated_checksum = calculate_crc(data)?;
    if expected_checksum != calculated_checksum {
        // checksum doesn't match
        return Err(anyhow!(
            "checksums don't match, expected {} received {}",
            expected_checksum,
            calculated_checksum
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_crc() -> Result<()> {
        assert_eq!(calculate_crc(&[0x00, 0x01, 0x00])?, 0x3331);
        assert_eq!(
            calculate_crc(&[0x00, 0x07, 0x01, 0x00, 0x0A, 0x01])?,
            0xee8a
        );
        assert_eq!(calculate_crc(&[0x00, 0x00, 0x01, 0x3E])?, 0xe4ac);
        assert_eq!(
            calculate_crc(&[0x00, 0x08, 0x01, 0x00, 0x0A, 0x3E])?,
            0x4ccf
        );
        Ok(())
    }
}
