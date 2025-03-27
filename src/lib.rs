#![no_std]

pub mod pdo;
pub mod type_c;

/// Port ID new type
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PortId(pub u8);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// General PD-related errors
pub enum PdError {
    /// Invalid controller
    InvalidController,
    /// Invalid response
    InvalidResponse,
    /// Unrecognized command
    UnrecognizedCommand,
    /// Invalid port
    InvalidPort,
    /// Invalid parameters
    InvalidParams,
    /// Incompatible partner
    IncompatiblePartner,
    /// CC communication error,
    CcCommunication,
    /// Failed due to dead battery condition
    DeadBattery,
    /// Contract negociation failed
    ContractNegociation,
    /// Overcurrent
    Overcurrent,
    /// Swap rejected by port partner
    SwapRejectedPartner,
    /// Hard reset
    HardReset,
    /// Policy conflict
    PolicyConflict,
    /// Swap rejected
    SwapRejected,
    /// Reverse current protection
    ReverseCurrent,
    /// Set sink path rejected
    SetSinkPath,
    /// The requested action has not yet completed
    Busy,
    /// The requested action timed out
    Timeout,
    /// Generic failure
    Failed,
    /// The device is in the incorrect mode
    InvalidMode,
    /// Data serialization error
    Serialize,
    /// Command not yet completed
    InProgress,
}

/// Top-level error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<BE> {
    /// Bus error
    Bus(BE),
    /// General PD error
    Pd(PdError),
}

/// Power role
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PowerRole {
    Sink,
    Source,
}

impl<T, BE> From<PdError> for Result<T, Error<BE>> {
    fn from(err: PdError) -> Self {
        Err(err.into())
    }
}

impl<BE> From<PdError> for Error<BE> {
    fn from(err: PdError) -> Self {
        Error::Pd(err)
    }
}
