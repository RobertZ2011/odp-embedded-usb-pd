use bitfield::bitfield;

use crate::ucsi::MapPort;
use crate::{GlobalPortId, LocalPortId, PortId};

bitfield! {
    /// Raw command status and connect change indicator, see UCSI spec 4.2
    #[derive(Copy, Clone, Default, PartialEq, Eq)]
    pub struct CciRaw(u32);
    impl Debug;

    /// Connector change on the given port
    pub u8, connector_change, set_connector_change: 7, 1;
    /// Length of returned data
    pub u8, data_len, set_data_len: 15, 8;
    /// Command not supported
    pub bool, not_supported, set_not_supported: 25;
    /// Cancel complete
    pub bool, cancel_complete, set_cancel_complete: 26;
    /// PPM reset complete
    pub bool, reset_complete, set_reset_complete: 27;
    /// Busy
    pub bool, busy, set_busy: 28;
    /// Acknowledgment command
    pub bool, ack_command, set_ack_command: 29;
    /// Command error
    pub bool, error, set_error: 30;
    /// Command complete
    pub bool, cmd_complete, set_cmd_complete: 31;
}

/// Higher-level wrapper around [`CciRaw`]
///
/// Only the bits defined by [`CciRaw`] are preserved, undefined bits are dropped when converting
/// from the raw representation.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Cci<T: PortId> {
    /// Connector change on the given port
    pub connector_change: T,
    /// Length of returned data
    pub data_len: u8,
    /// Command not supported
    pub not_supported: bool,
    /// Cancel complete
    pub cancel_complete: bool,
    /// PPM reset complete
    pub reset_complete: bool,
    /// Busy
    pub busy: bool,
    /// Acknowledgment command
    pub ack_command: bool,
    /// Command error
    pub error: bool,
    /// Command complete
    pub cmd_complete: bool,
}

impl<T: PortId> Cci<T> {
    /// Create a new CCI with command complete set
    pub fn new_cmd_complete() -> Self {
        Self {
            cmd_complete: true,
            ..Default::default()
        }
    }

    /// Create a new CCI with busy set
    pub fn new_busy() -> Self {
        Self {
            busy: true,
            ..Default::default()
        }
    }

    /// Create a new CCI with reset complete set
    pub fn new_reset_complete() -> Self {
        Self {
            reset_complete: true,
            ..Default::default()
        }
    }

    /// Create a new CCI with error and command complete set
    ///
    /// Error can only be set in combination with command complete
    pub fn new_command_error() -> Self {
        Self {
            error: true,
            cmd_complete: true,
            ..Default::default()
        }
    }

    /// Create a new CCI with not supported and command complete set
    ///
    /// Not supported can only be set in combination with command complete
    pub fn new_command_not_supported() -> Self {
        Self {
            not_supported: true,
            cmd_complete: true,
            ..Default::default()
        }
    }

    /// Create a new CCI with cancel complete and command complete set
    ///
    /// Cancel complete can only be set in combination with command complete
    pub fn new_cancel_complete() -> Self {
        Self {
            cancel_complete: true,
            cmd_complete: true,
            ..Default::default()
        }
    }

    /// Create a new CCI with acknowledgment set
    pub fn new_ack_command() -> Self {
        Self {
            ack_command: true,
            ..Default::default()
        }
    }

    /// Checks the CCI structure for mutually exclusive fields
    fn validate(&self) -> Result<(), InvalidFieldCombo> {
        let fields = InvalidFieldCombo {
            connector_change: self.connector_change != T::from(0),
            not_supported: self.not_supported,
            cancel_complete: self.cancel_complete,
            reset_complete: self.reset_complete,
            busy: self.busy,
            ack_command: self.ack_command,
            error: self.error,
            cmd_complete: self.cmd_complete,
            data_len: self.data_len != 0,
        };
        fields.validate()
    }
}

/// Type used to validate and indicate invalid field combinations in the CCI structure
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InvalidFieldCombo {
    /// Connector change set
    pub connector_change: bool,
    /// Command not supported
    pub not_supported: bool,
    /// Cancel complete
    pub cancel_complete: bool,
    /// PPM reset complete
    pub reset_complete: bool,
    /// Busy
    pub busy: bool,
    /// Acknowledgment command
    pub ack_command: bool,
    /// Command error
    pub error: bool,
    /// Command complete
    pub cmd_complete: bool,
    /// Data length is set
    pub data_len: bool,
}

impl InvalidFieldCombo {
    /// Core validation logic for checking mutually exclusive CCI fields
    fn validate(&self) -> Result<(), Self> {
        if self.reset_complete
            && (self.not_supported
                || self.cmd_complete
                || self.cancel_complete
                || self.ack_command
                || self.error
                || self.busy
                || self.connector_change
                || self.data_len)
        {
            // Reset complete cannot be set with any other fields
            Err(*self)
        } else if self.busy
            && (self.not_supported
                || self.cmd_complete
                || self.cancel_complete
                || self.ack_command
                || self.error
                || self.connector_change
                || self.data_len)
        {
            // Busy cannot be set with any other fields
            Err(*self)
        } else if self.ack_command
            && (self.not_supported
                || self.cmd_complete
                || self.cancel_complete
                || self.error
                || self.busy
                || self.data_len)
        {
            // Ack command can only be set with connector change
            Err(*self)
        } else if self.not_supported || self.cancel_complete || self.error {
            let count = self.not_supported as u8 + self.cancel_complete as u8 + self.error as u8;
            if !self.cmd_complete || count > 1 {
                // These can only be set if command complete is set
                // The spec doesn't explicitly forbid multiple of these flags being set,
                // but it doesn't make sense in practice. Reject this since this is likely an error
                Err(*self)
            } else {
                Ok(())
            }
        } else if self.data_len && !self.cmd_complete {
            // The spec doesn't explicitly forbid data length being set without command complete,
            // but it doesn't make sense in practice. Reject this as likely an error.
            Err(*self)
        } else {
            Ok(())
        }
    }
}

impl<T: PortId, Out: PortId> MapPort<T, Out> for Cci<T> {
    type Output = Cci<Out>;

    fn map_port<F>(self, f: F) -> Self::Output
    where
        F: Fn(T) -> Out,
    {
        Cci {
            connector_change: f(self.connector_change),
            not_supported: self.not_supported,
            cancel_complete: self.cancel_complete,
            reset_complete: self.reset_complete,
            busy: self.busy,
            ack_command: self.ack_command,
            error: self.error,
            cmd_complete: self.cmd_complete,
            data_len: self.data_len,
        }
    }
}

impl<T: PortId> TryFrom<CciRaw> for Cci<T> {
    type Error = InvalidFieldCombo;

    fn try_from(raw: CciRaw) -> Result<Self, Self::Error> {
        let flags = InvalidFieldCombo {
            busy: raw.busy(),
            error: raw.error(),
            cmd_complete: raw.cmd_complete(),
            reset_complete: raw.reset_complete(),
            connector_change: raw.connector_change() != 0,
            not_supported: raw.not_supported(),
            cancel_complete: raw.cancel_complete(),
            ack_command: raw.ack_command(),
            data_len: raw.data_len() != 0,
        };
        flags.validate()?;

        Ok(Self {
            connector_change: raw.connector_change().into(),
            data_len: raw.data_len(),
            not_supported: raw.not_supported(),
            cancel_complete: raw.cancel_complete(),
            reset_complete: raw.reset_complete(),
            busy: raw.busy(),
            ack_command: raw.ack_command(),
            error: raw.error(),
            cmd_complete: raw.cmd_complete(),
        })
    }
}

impl<T: PortId> TryFrom<Cci<T>> for CciRaw {
    type Error = InvalidFieldCombo;

    fn try_from(cci: Cci<T>) -> Result<Self, Self::Error> {
        cci.validate()?;
        let mut raw = CciRaw(0);

        raw.set_connector_change(cci.connector_change.into());
        raw.set_data_len(cci.data_len);
        raw.set_not_supported(cci.not_supported);
        raw.set_cancel_complete(cci.cancel_complete);
        raw.set_reset_complete(cci.reset_complete);
        raw.set_busy(cci.busy);
        raw.set_ack_command(cci.ack_command);
        raw.set_error(cci.error);
        raw.set_cmd_complete(cci.cmd_complete);

        Ok(raw)
    }
}

impl From<u32> for CciRaw {
    fn from(value: u32) -> Self {
        CciRaw(value)
    }
}

impl From<CciRaw> for u32 {
    fn from(raw: CciRaw) -> Self {
        raw.0
    }
}

impl<T: PortId> TryFrom<u32> for Cci<T> {
    type Error = InvalidFieldCombo;

    fn try_from(raw: u32) -> Result<Self, Self::Error> {
        Cci::try_from(CciRaw(raw))
    }
}

impl<T: PortId> TryFrom<Cci<T>> for u32 {
    type Error = InvalidFieldCombo;

    fn try_from(cci: Cci<T>) -> Result<Self, Self::Error> {
        Ok(CciRaw::try_from(cci)?.0)
    }
}

impl<T: PortId> Default for Cci<T> {
    fn default() -> Self {
        Cci {
            connector_change: T::from(0),
            data_len: 0,
            not_supported: false,
            cancel_complete: false,
            reset_complete: false,
            busy: false,
            ack_command: false,
            error: false,
            cmd_complete: false,
        }
    }
}

pub type GlobalCci = Cci<GlobalPortId>;
pub type LocalCci = Cci<LocalPortId>;

#[cfg(feature = "defmt")]
impl defmt::Format for CciRaw {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "CciRaw {{ .0: {}, connector_change: {}, data_len: {}, not_supported: {}, cancel_complete: {}, reset_complete: {}, busy: {}, ack_command: {}, error: {}, cmd_complete: {} }}",
            self.0,
            self.connector_change(),
            self.data_len(),
            self.not_supported(),
            self.cancel_complete(),
            self.reset_complete(),
            self.busy(),
            self.ack_command(),
            self.error(),
            self.cmd_complete(),
        )
    }
}

/// A CCI variant without a data len field around [`Cci`].
///
/// The data len field comes from the [`ResponseData`] struct because that works better from a modularity standpoint. But it actually
/// needs to go in the CCI. This struct omits it to provide a compile time check that the data len is not accidentally used from
/// the CCI itself. This struct cannot be converted to/from a CciRaw directly.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CciNoDataLen<T: PortId> {
    /// Connector change on the given port
    pub connector_change: T,
    /// Command not supported
    pub not_supported: bool,
    /// Cancel complete
    pub cancel_complete: bool,
    /// PPM reset complete
    pub reset_complete: bool,
    /// Busy
    pub busy: bool,
    /// Acknowledgment command
    pub ack_command: bool,
    /// Command error
    pub error: bool,
    /// Command complete
    pub cmd_complete: bool,
}

impl<T: PortId> Default for CciNoDataLen<T> {
    fn default() -> Self {
        Self {
            connector_change: T::from(0),
            not_supported: false,
            cancel_complete: false,
            reset_complete: false,
            busy: false,
            ack_command: false,
            error: false,
            cmd_complete: false,
        }
    }
}

impl<T: PortId> CciNoDataLen<T> {
    pub fn into_cci(self, data_len: u8) -> Cci<T> {
        Cci {
            connector_change: self.connector_change,
            not_supported: self.not_supported,
            cancel_complete: self.cancel_complete,
            reset_complete: self.reset_complete,
            busy: self.busy,
            ack_command: self.ack_command,
            error: self.error,
            cmd_complete: self.cmd_complete,
            data_len,
        }
    }
}

impl<T: PortId, Out: PortId> MapPort<T, Out> for CciNoDataLen<T> {
    type Output = CciNoDataLen<Out>;

    fn map_port<F>(self, f: F) -> Self::Output
    where
        F: Fn(T) -> Out,
    {
        CciNoDataLen {
            connector_change: f(self.connector_change),
            not_supported: self.not_supported,
            cancel_complete: self.cancel_complete,
            reset_complete: self.reset_complete,
            busy: self.busy,
            ack_command: self.ack_command,
            error: self.error,
            cmd_complete: self.cmd_complete,
        }
    }
}

impl<T: PortId> From<Cci<T>> for CciNoDataLen<T> {
    fn from(cci: Cci<T>) -> Self {
        Self {
            connector_change: cci.connector_change,
            not_supported: cci.not_supported,
            cancel_complete: cci.cancel_complete,
            reset_complete: cci.reset_complete,
            busy: cci.busy,
            ack_command: cci.ack_command,
            error: cci.error,
            cmd_complete: cci.cmd_complete,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A high-level CCI only round-trips the bits defined by [`CciRaw`]
    #[test]
    fn test_cci_undefined_bits_dropped() {
        // Ack with a connector change and undefined bits set
        let cci = GlobalCci::try_from(0x21FF0003).unwrap();
        assert_eq!(u32::try_from(cci).unwrap(), 0x20000002);
    }

    /// Validates that a CCI structure round-trips correctly through conversion to and from raw bits
    #[test]
    fn test_cci_round_trip() {
        // Not supported with command complete
        const NOT_SUPPORTED: u32 = 0x82000000;
        let round_trip = GlobalCci::try_from(NOT_SUPPORTED).unwrap();
        assert_eq!(
            round_trip,
            GlobalCci {
                not_supported: true,
                cmd_complete: true,
                ..Default::default()
            }
        );
        assert_eq!(u32::try_from(round_trip).unwrap(), NOT_SUPPORTED);

        // Cancel with command complete
        const CANCEL_COMPLETE: u32 = 0x84000000;
        let round_trip = GlobalCci::try_from(CANCEL_COMPLETE).unwrap();
        assert_eq!(
            round_trip,
            GlobalCci {
                cancel_complete: true,
                cmd_complete: true,
                ..Default::default()
            }
        );
        assert_eq!(u32::try_from(round_trip).unwrap(), CANCEL_COMPLETE);

        // Error with command complete
        const ERROR: u32 = 0xC0000000;
        let round_trip = GlobalCci::try_from(ERROR).unwrap();
        assert_eq!(
            round_trip,
            GlobalCci {
                error: true,
                cmd_complete: true,
                ..Default::default()
            }
        );
        assert_eq!(u32::try_from(round_trip).unwrap(), ERROR);

        // Reset
        const RESET_COMPLETE: u32 = 0x08000000;
        let round_trip = GlobalCci::try_from(RESET_COMPLETE).unwrap();
        assert_eq!(
            round_trip,
            GlobalCci {
                reset_complete: true,
                ..Default::default()
            }
        );
        assert_eq!(u32::try_from(round_trip).unwrap(), RESET_COMPLETE);

        // Busy
        const BUSY: u32 = 0x10000000;
        let round_trip = GlobalCci::try_from(BUSY).unwrap();
        assert_eq!(
            round_trip,
            GlobalCci {
                busy: true,
                ..Default::default()
            }
        );
        assert_eq!(u32::try_from(round_trip).unwrap(), BUSY);

        // Ack command
        const ACK_COMMAND: u32 = 0x20000002;
        let round_trip = GlobalCci::try_from(ACK_COMMAND).unwrap();
        assert_eq!(
            round_trip,
            GlobalCci {
                ack_command: true,
                connector_change: GlobalPortId(1),
                ..Default::default()
            }
        );
        assert_eq!(u32::try_from(round_trip).unwrap(), ACK_COMMAND);

        // Command complete
        const CMD_COMPLETE: u32 = 0x8000_0502;
        let round_trip = GlobalCci::try_from(CMD_COMPLETE).unwrap();
        assert_eq!(
            round_trip,
            GlobalCci {
                connector_change: GlobalPortId(1),
                data_len: 5,
                cmd_complete: true,
                ..Default::default()
            }
        );
        assert_eq!(u32::try_from(round_trip).unwrap(), CMD_COMPLETE);
    }

    #[test]
    fn test_cci_constructors() {
        assert!(GlobalCci::new_cmd_complete().cmd_complete);
        assert!(GlobalCci::new_busy().busy);
        assert!(GlobalCci::new_reset_complete().reset_complete);
        assert!(GlobalCci::new_ack_command().ack_command);

        let command_error = GlobalCci::new_command_error();
        assert!(command_error.error && command_error.cmd_complete);

        let cancel_complete = GlobalCci::new_cancel_complete();
        assert!(cancel_complete.cmd_complete && cancel_complete.cancel_complete);

        let command_not_supported = GlobalCci::new_command_not_supported();
        assert!(command_not_supported.cmd_complete && command_not_supported.not_supported);
    }

    /// Tests that all the new constructors produce valid CCI structures
    #[test]
    fn test_new_are_valid() {
        GlobalCci::new_cmd_complete().validate().unwrap();
        GlobalCci::new_busy().validate().unwrap();
        GlobalCci::new_reset_complete().validate().unwrap();
        GlobalCci::new_command_error().validate().unwrap();
        GlobalCci::new_ack_command().validate().unwrap();
        GlobalCci::new_cancel_complete().validate().unwrap();
        GlobalCci::new_command_not_supported().validate().unwrap();
    }

    /// Tests that the command complete dependent fields are correctly validated
    #[test]
    fn test_command_complete_dependent_validation() {
        // Not supported
        assert_eq!(
            GlobalCci::try_from(0x02000000),
            Err(InvalidFieldCombo {
                not_supported: true,
                ..Default::default()
            })
        );
        assert_eq!(
            GlobalCci::try_from(0x82000000),
            Ok(GlobalCci {
                not_supported: true,
                cmd_complete: true,
                ..Default::default()
            })
        );

        // Cancel
        assert_eq!(
            GlobalCci::try_from(0x04000000),
            Err(InvalidFieldCombo {
                cancel_complete: true,
                ..Default::default()
            })
        );
        assert_eq!(
            GlobalCci::try_from(0x84000000),
            Ok(GlobalCci {
                cancel_complete: true,
                cmd_complete: true,
                ..Default::default()
            })
        );

        // Error
        assert_eq!(
            GlobalCci::try_from(0x40000000),
            Err(InvalidFieldCombo {
                error: true,
                ..Default::default()
            })
        );
        assert_eq!(
            GlobalCci::try_from(0xC0000000),
            Ok(GlobalCci {
                error: true,
                cmd_complete: true,
                ..Default::default()
            })
        );

        // The spec doesn't explicitly forbid combining these flags, but in practice it doesn't make sense
        assert_eq!(
            GlobalCci::try_from(0xC6000000),
            Err(InvalidFieldCombo {
                error: true,
                cmd_complete: true,
                cancel_complete: true,
                not_supported: true,
                ..Default::default()
            })
        );
    }

    #[test]
    fn test_reset_validation() {
        // Reset by itself should work
        assert_eq!(
            GlobalCci::try_from(0x08000000),
            Ok(GlobalCci {
                reset_complete: true,
                ..Default::default()
            })
        );

        // With not supported should fail
        assert_eq!(
            GlobalCci::try_from(0x0A000000),
            Err(InvalidFieldCombo {
                reset_complete: true,
                not_supported: true,
                ..Default::default()
            })
        );

        // With cancel should fail
        assert_eq!(
            GlobalCci::try_from(0x0C000000),
            Err(InvalidFieldCombo {
                reset_complete: true,
                cancel_complete: true,
                ..Default::default()
            })
        );

        // With error should fail
        assert_eq!(
            GlobalCci::try_from(0x48000000),
            Err(InvalidFieldCombo {
                reset_complete: true,
                error: true,
                ..Default::default()
            })
        );

        // With busy should fail
        assert_eq!(
            GlobalCci::try_from(0x18000000),
            Err(InvalidFieldCombo {
                reset_complete: true,
                busy: true,
                ..Default::default()
            })
        );

        // With ack command should fail
        assert_eq!(
            GlobalCci::try_from(0x28000000),
            Err(InvalidFieldCombo {
                reset_complete: true,
                ack_command: true,
                ..Default::default()
            })
        );

        // With command complete should fail
        assert_eq!(
            GlobalCci::try_from(0x88000000),
            Err(InvalidFieldCombo {
                reset_complete: true,
                cmd_complete: true,
                ..Default::default()
            })
        );

        // With data length should fail
        assert_eq!(
            GlobalCci::try_from(0x08000100),
            Err(InvalidFieldCombo {
                reset_complete: true,
                data_len: true,
                ..Default::default()
            })
        );

        // With connector change should fail
        assert_eq!(
            GlobalCci::try_from(0x08000002),
            Err(InvalidFieldCombo {
                reset_complete: true,
                connector_change: true,
                ..Default::default()
            })
        );

        // With everything should fail
        assert_eq!(
            GlobalCci::try_from(0xFE000102),
            Err(InvalidFieldCombo {
                reset_complete: true,
                not_supported: true,
                cancel_complete: true,
                error: true,
                busy: true,
                ack_command: true,
                cmd_complete: true,
                data_len: true,
                connector_change: true,
            })
        );
    }

    #[test]
    fn test_busy_validation() {
        // Busy by itself should work
        assert_eq!(
            GlobalCci::try_from(0x10000000),
            Ok(GlobalCci {
                busy: true,
                ..Default::default()
            })
        );

        // With not supported should fail
        assert_eq!(
            GlobalCci::try_from(0x12000000),
            Err(InvalidFieldCombo {
                busy: true,
                not_supported: true,
                ..Default::default()
            })
        );

        // With cancel should fail
        assert_eq!(
            GlobalCci::try_from(0x14000000),
            Err(InvalidFieldCombo {
                busy: true,
                cancel_complete: true,
                ..Default::default()
            })
        );

        // With error should fail
        assert_eq!(
            GlobalCci::try_from(0x50000000),
            Err(InvalidFieldCombo {
                busy: true,
                error: true,
                ..Default::default()
            })
        );

        // With reset should fail
        assert_eq!(
            GlobalCci::try_from(0x18000000),
            Err(InvalidFieldCombo {
                reset_complete: true,
                busy: true,
                ..Default::default()
            })
        );

        // With ack command should fail
        assert_eq!(
            GlobalCci::try_from(0x30000000),
            Err(InvalidFieldCombo {
                busy: true,
                ack_command: true,
                ..Default::default()
            })
        );

        // With command complete should fail
        assert_eq!(
            GlobalCci::try_from(0x90000000),
            Err(InvalidFieldCombo {
                busy: true,
                cmd_complete: true,
                ..Default::default()
            })
        );

        // With data length should fail
        assert_eq!(
            GlobalCci::try_from(0x10000100),
            Err(InvalidFieldCombo {
                busy: true,
                data_len: true,
                ..Default::default()
            })
        );

        // With connector change should fail
        assert_eq!(
            GlobalCci::try_from(0x10000002),
            Err(InvalidFieldCombo {
                busy: true,
                connector_change: true,
                ..Default::default()
            })
        );

        // With everything should fail
        assert_eq!(
            GlobalCci::try_from(0xFE000102),
            Err(InvalidFieldCombo {
                reset_complete: true,
                not_supported: true,
                cancel_complete: true,
                error: true,
                busy: true,
                ack_command: true,
                cmd_complete: true,
                data_len: true,
                connector_change: true,
            })
        );
    }

    #[test]
    fn test_ack_validation() {
        // Ack by itself should work
        assert_eq!(
            GlobalCci::try_from(0x20000000),
            Ok(GlobalCci {
                ack_command: true,
                ..Default::default()
            })
        );

        // With not supported should fail
        assert_eq!(
            GlobalCci::try_from(0x22000000),
            Err(InvalidFieldCombo {
                ack_command: true,
                not_supported: true,
                ..Default::default()
            })
        );

        // With cancel should fail
        assert_eq!(
            GlobalCci::try_from(0x24000000),
            Err(InvalidFieldCombo {
                ack_command: true,
                cancel_complete: true,
                ..Default::default()
            })
        );

        // With error should fail
        assert_eq!(
            GlobalCci::try_from(0x60000000),
            Err(InvalidFieldCombo {
                ack_command: true,
                error: true,
                ..Default::default()
            })
        );

        // With reset should fail
        assert_eq!(
            GlobalCci::try_from(0x28000000),
            Err(InvalidFieldCombo {
                reset_complete: true,
                ack_command: true,
                ..Default::default()
            })
        );

        // With command complete should fail
        assert_eq!(
            GlobalCci::try_from(0xA0000000),
            Err(InvalidFieldCombo {
                ack_command: true,
                cmd_complete: true,
                ..Default::default()
            })
        );

        // With data length should fail
        assert_eq!(
            GlobalCci::try_from(0x20000100),
            Err(InvalidFieldCombo {
                ack_command: true,
                data_len: true,
                ..Default::default()
            })
        );

        // With connector change should succeed
        assert_eq!(
            GlobalCci::try_from(0x20000002),
            Ok(GlobalCci {
                ack_command: true,
                connector_change: GlobalPortId(1),
                ..Default::default()
            })
        );

        // With everything should fail
        assert_eq!(
            GlobalCci::try_from(0xFE000102),
            Err(InvalidFieldCombo {
                reset_complete: true,
                not_supported: true,
                cancel_complete: true,
                error: true,
                busy: true,
                ack_command: true,
                cmd_complete: true,
                data_len: true,
                connector_change: true,
            })
        );
    }

    /// Test that data length without command complete is rejected
    #[test]
    fn test_data_len() {
        assert_eq!(
            GlobalCci::try_from(0x80000100),
            Ok(GlobalCci {
                cmd_complete: true,
                data_len: 1,
                ..Default::default()
            })
        );

        assert_eq!(
            GlobalCci::try_from(0x20000100),
            Err(InvalidFieldCombo {
                ack_command: true,
                data_len: true,
                ..Default::default()
            })
        );

        assert_eq!(
            GlobalCci::try_from(0x00000100),
            Err(InvalidFieldCombo {
                data_len: true,
                ..Default::default()
            })
        );
    }
}
