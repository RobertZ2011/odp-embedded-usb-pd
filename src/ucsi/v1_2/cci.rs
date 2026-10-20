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

    /// Create a new CCI with error set
    pub fn new_error() -> Self {
        Self {
            error: true,
            ..Default::default()
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

impl<T: PortId> From<CciRaw> for Cci<T> {
    fn from(raw: CciRaw) -> Self {
        Self {
            connector_change: raw.connector_change().into(),
            data_len: raw.data_len(),
            not_supported: raw.not_supported(),
            cancel_complete: raw.cancel_complete(),
            reset_complete: raw.reset_complete(),
            busy: raw.busy(),
            ack_command: raw.ack_command(),
            error: raw.error(),
            cmd_complete: raw.cmd_complete(),
        }
    }
}

impl<T: PortId> From<Cci<T>> for CciRaw {
    fn from(cci: Cci<T>) -> Self {
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

        raw
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

impl<T: PortId> From<u32> for Cci<T> {
    fn from(raw: u32) -> Self {
        Cci::from(CciRaw(raw))
    }
}

impl<T: PortId> From<Cci<T>> for u32 {
    fn from(cci: Cci<T>) -> Self {
        CciRaw::from(cci).0
    }
}

impl<T: PortId> Default for Cci<T> {
    fn default() -> Self {
        Cci::from(CciRaw(0))
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

    /// All bits defined by [`CciRaw`]
    ///
    /// Bits 0, 24:16 are reserved.
    const DEFINED_BITS: u32 = 0xFE00_FFFE;

    #[test]
    fn test_cci_defined_bits_roundtrip() {
        let cci = GlobalCci::from(DEFINED_BITS);
        assert_eq!(u32::from(cci), DEFINED_BITS);
    }

    /// A high-level CCI only round-trips the bits defined by [`CciRaw`]
    #[test]
    fn test_cci_undefined_bits_dropped() {
        let cci = GlobalCci::from(u32::MAX);
        assert_eq!(u32::from(cci), DEFINED_BITS);
    }

    #[test]
    fn test_cci_fields() {
        let cci = GlobalCci::from(0x8000_0502);
        assert_eq!(cci.connector_change, GlobalPortId(1));
        assert_eq!(cci.data_len, 5);
        assert!(cci.cmd_complete);
        assert!(!cci.error);
    }

    #[test]
    fn test_cci_constructors() {
        assert!(GlobalCci::new_cmd_complete().cmd_complete);
        assert!(GlobalCci::new_busy().busy);
        assert!(GlobalCci::new_reset_complete().reset_complete);
        assert!(GlobalCci::new_error().error);
    }
}
