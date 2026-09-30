//! PPM Reset command

use bytemuck::{Pod, Zeroable};

use crate::ucsi::v1_2::{CommandHeaderRaw, COMMAND_LEN};

/// PPM Reset command structure
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Args;

/// Data length for the PPM_RESET command response
pub const RESPONSE_DATA_LEN: u8 = 0;
/// Command padding
pub const COMMAND_PADDING: usize = COMMAND_LEN - size_of::<CommandHeaderRaw>();

/// Raw wire format of [`Args`]
///
/// PPM_RESET takes no arguments, the entire payload is reserved.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Zeroable, Pod)]
pub struct ArgsRaw {
    /// Reserved bytes, filling out the remainder of the command
    _reserved: [u8; COMMAND_PADDING],
}

impl ArgsRaw {
    /// Length of the raw arguments in bytes
    pub const LEN: usize = size_of::<Self>();
}

#[cfg(feature = "defmt")]
impl defmt::Format for ArgsRaw {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "ArgsRaw {{ }}")
    }
}

impl From<Args> for ArgsRaw {
    fn from(_: Args) -> Self {
        Self::default()
    }
}

impl From<ArgsRaw> for Args {
    fn from(_: ArgsRaw) -> Self {
        Self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_raw_len() {
        assert_eq!(ArgsRaw::LEN, COMMAND_PADDING);
    }

    #[test]
    fn test_args_raw_roundtrip() {
        let bytes: [u8; ArgsRaw::LEN] = bytemuck::must_cast(ArgsRaw::from(Args));
        assert_eq!(bytes, [0u8; ArgsRaw::LEN]);
        assert_eq!(Args::from(bytemuck::must_cast::<_, ArgsRaw>(bytes)), Args);
    }
}
