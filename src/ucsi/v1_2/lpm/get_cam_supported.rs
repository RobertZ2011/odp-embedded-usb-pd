//! Types for GET_CAM_SUPPORTED command, see UCSI spec 6.5.12
use bytemuck::{Pod, Zeroable};

use crate::ucsi::v1_2::lpm::ConnectorNumberRaw;
use crate::ucsi::v1_2::{CommandHeaderRaw, COMMAND_LEN};

/// Data length for the GET_CAM_SUPPORTED command response
pub const RESPONSE_DATA_LEN: usize = 1;
/// Command padding
pub const COMMAND_PADDING: usize = COMMAND_LEN - size_of::<CommandHeaderRaw>() - size_of::<ConnectorNumberRaw>();
/// Maximum number of alternate modes supported
pub const MAX_ALT_MODES: usize = 8;

/// Raw wire format of the command arguments
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Zeroable, Pod)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ArgsRaw {
    /// Connector number in bits 6:0
    pub connector: u8,
    /// Reserved bytes, filling out the remainder of the command
    _reserved: [u8; COMMAND_PADDING],
}

impl ArgsRaw {
    /// Length of the raw arguments in bytes
    pub const LEN: usize = size_of::<Self>();
}

impl From<u8> for ArgsRaw {
    /// Creates raw arguments for the given connector number
    fn from(connector_number: u8) -> Self {
        let mut connector = ConnectorNumberRaw::default();
        connector.set_connector_number(connector_number);
        Self {
            connector: connector.0,
            ..Default::default()
        }
    }
}

impl From<ArgsRaw> for u8 {
    /// Returns the connector number
    fn from(raw: ArgsRaw) -> Self {
        ConnectorNumberRaw(raw.connector).connector_number()
    }
}

/// GET_CAM_SUPPORTED response data, supports up to [`MAX_ALT_MODES`] alternate modes
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ResponseData {
    /// Support flag for each alternate mode
    pub alt_modes: [bool; MAX_ALT_MODES],
}

impl ResponseData {
    /// Returns true if the alternate mode at `index` is supported
    ///
    /// Returns false for an out of range `index`.
    pub fn alt_mode_supported(&self, index: usize) -> bool {
        self.alt_modes.get(index).copied().unwrap_or(false)
    }

    /// Sets the support flag for the alternate mode at `index`
    ///
    /// Does nothing for an out of range `index`.
    pub fn set_alt_mode_supported(&mut self, index: usize, supported: bool) {
        if let Some(alt_mode) = self.alt_modes.get_mut(index) {
            *alt_mode = supported;
        }
    }
}

/// Raw wire format of [`ResponseData`]
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Zeroable, Pod)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ResponseDataRaw {
    /// Bitmap of supported alternate modes
    pub alt_modes: u8,
}

impl ResponseDataRaw {
    /// Length of the raw response data in bytes
    pub const LEN: usize = size_of::<Self>();
}

impl From<ResponseDataRaw> for ResponseData {
    fn from(raw: ResponseDataRaw) -> Self {
        let mut data = ResponseData::default();
        for (index, alt_mode) in data.alt_modes.iter_mut().enumerate() {
            *alt_mode = (raw.alt_modes & (1 << index)) != 0;
        }
        data
    }
}

impl From<ResponseData> for ResponseDataRaw {
    fn from(data: ResponseData) -> Self {
        let mut alt_modes = 0u8;
        for (index, supported) in data.alt_modes.iter().enumerate() {
            if *supported {
                alt_modes |= 1 << index;
            }
        }
        Self { alt_modes }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_raw_len() {
        assert_eq!(ArgsRaw::LEN, COMMAND_LEN - size_of::<CommandHeaderRaw>());
        assert_eq!(ResponseDataRaw::LEN, RESPONSE_DATA_LEN);
    }

    #[test]
    fn test_args_raw_roundtrip() {
        let encoded: [u8; ArgsRaw::LEN] = [0x03, 0x00, 0x00, 0x00, 0x00, 0x00];
        let raw = ArgsRaw::from(3);

        assert_eq!(bytemuck::must_cast::<_, [u8; ArgsRaw::LEN]>(raw), encoded);
        assert_eq!(u8::from(bytemuck::must_cast::<_, ArgsRaw>(encoded)), 3);
    }

    #[test]
    fn test_response_data_roundtrip() {
        let bytes = [0x12u8; RESPONSE_DATA_LEN];
        let expected = ResponseData {
            alt_modes: [false, true, false, false, true, false, false, false],
        };

        let data = ResponseData::from(bytemuck::must_cast::<_, ResponseDataRaw>(bytes));
        assert_eq!(data, expected);
        assert!(data.alt_mode_supported(1));
        assert!(!data.alt_mode_supported(2));
        assert!(!data.alt_mode_supported(MAX_ALT_MODES));

        let encoded: [u8; RESPONSE_DATA_LEN] = bytemuck::must_cast(ResponseDataRaw::from(expected));
        assert_eq!(encoded, bytes);
    }

    #[test]
    fn test_set_alt_mode_supported_out_of_range() {
        let mut data = ResponseData::default();
        data.set_alt_mode_supported(MAX_ALT_MODES, true);
        assert_eq!(data, ResponseData::default());
    }
}
