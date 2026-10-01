//! Types for SET_NEW_CAM command, see UCSI spec 6.5.14

use bytemuck::{Pod, Zeroable};
use pack1::U32LE;

use crate::ucsi::v1_2::lpm::ConnectorNumberRaw;

/// Command arguments
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Args {
    /// Connector number
    pub connector_number: u8,
    /// Enter or exit mode
    pub enter: bool,
    /// Alternate mode offset
    pub am_offset: u8,
    /// Alternate mode specific
    pub am_specific: u32,
}

/// Raw wire format of [`Args`]
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Zeroable, Pod)]
pub struct ArgsRaw {
    /// Connector number in bits 6:0, enter/exit mode in bit 7
    pub connector: u8,
    /// Alternate mode offset
    pub am_offset: u8,
    /// Alternate mode specific
    pub am_specific: U32LE,
}

impl ArgsRaw {
    /// Length of the raw arguments in bytes
    pub const LEN: usize = size_of::<Self>();
}

#[cfg(feature = "defmt")]
impl defmt::Format for ArgsRaw {
    fn format(&self, fmt: defmt::Formatter) {
        let connector = ConnectorNumberRaw(self.connector);
        defmt::write!(
            fmt,
            "ArgsRaw {{ connector_number: {}, enter: {}, am_offset: {}, am_specific: {} }}",
            connector.connector_number(),
            connector.high_bit(),
            self.am_offset,
            self.am_specific.get()
        )
    }
}

impl From<Args> for ArgsRaw {
    fn from(args: Args) -> Self {
        let mut connector = ConnectorNumberRaw::default();
        connector.set_connector_number(args.connector_number);
        connector.set_high_bit(args.enter);
        Self {
            connector: connector.0,
            am_offset: args.am_offset,
            am_specific: U32LE::new(args.am_specific),
        }
    }
}

impl From<ArgsRaw> for Args {
    fn from(raw: ArgsRaw) -> Self {
        let connector = ConnectorNumberRaw(raw.connector);
        Self {
            connector_number: connector.connector_number(),
            enter: connector.high_bit(),
            am_offset: raw.am_offset,
            am_specific: raw.am_specific.get(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ucsi::v1_2::{CommandHeaderRaw, COMMAND_LEN};

    #[test]
    fn test_raw_len() {
        assert_eq!(ArgsRaw::LEN, COMMAND_LEN - size_of::<CommandHeaderRaw>());
    }

    #[test]
    fn test_args_raw_roundtrip() {
        // Enter alt mode at offset 1, on connector 3, with AM-specific 0x12345678
        let encoded: [u8; ArgsRaw::LEN] = [0x83, 0x01, 0x78, 0x56, 0x34, 0x12];
        let expected = Args {
            connector_number: 3,
            enter: true,
            am_offset: 1,
            am_specific: 0x12345678,
        };

        assert_eq!(Args::from(bytemuck::must_cast::<_, ArgsRaw>(encoded)), expected);
        let bytes: [u8; ArgsRaw::LEN] = bytemuck::must_cast(ArgsRaw::from(expected));
        assert_eq!(bytes, encoded);
    }

    #[test]
    fn test_args_raw_exit() {
        // Exit alt mode on connector 127
        let encoded: [u8; ArgsRaw::LEN] = [0x7F, 0x00, 0x00, 0x00, 0x00, 0x00];
        let expected = Args {
            connector_number: 127,
            enter: false,
            ..Default::default()
        };

        assert_eq!(Args::from(bytemuck::must_cast::<_, ArgsRaw>(encoded)), expected);
        let bytes: [u8; ArgsRaw::LEN] = bytemuck::must_cast(ArgsRaw::from(expected));
        assert_eq!(bytes, encoded);
    }
}
