use crate::ucsi::cci::Cci;
use crate::ucsi::{CommandHeader, CommandType};

pub mod ack_cc_ci;
pub mod cancel;
pub mod get_capability;
pub mod ppm_reset;
pub mod set_notification_enable;

use bincode::de::{Decode, Decoder};
use bincode::enc::{Encode, Encoder};
use bincode::error::{AllowedEnumVariants, DecodeError, EncodeError};

/// Commands that only affect the PPM level and don't need to be sent to an LPM
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Command {
    PpmReset,
    Cancel,
    AckCcCi(ack_cc_ci::Args),
    SetNotificationEnable(set_notification_enable::Args),
    GetCapability,
}

impl Encode for Command {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            Command::PpmReset => ppm_reset::Args.encode(encoder),
            Command::Cancel => cancel::Args.encode(encoder),
            Command::AckCcCi(args) => args.encode(encoder),
            Command::SetNotificationEnable(args) => args.encode(encoder),
            Command::GetCapability => get_capability::Args.encode(encoder),
        }
    }
}

impl Decode<CommandHeader> for Command {
    fn decode<D: Decoder<Context = CommandHeader>>(decoder: &mut D) -> Result<Self, DecodeError> {
        match decoder.context().command() {
            CommandType::PpmReset => Ok(Command::PpmReset),
            CommandType::Cancel => Ok(Command::Cancel),
            CommandType::AckCcCi => Ok(Command::AckCcCi(ack_cc_ci::Args::decode(decoder)?)),
            CommandType::SetNotificationEnable => Ok(Command::SetNotificationEnable(
                set_notification_enable::Args::decode(decoder)?,
            )),
            CommandType::GetCapability => Ok(Command::GetCapability),
            command_type => Err(DecodeError::UnexpectedVariant {
                type_name: "CommandType",
                allowed: &AllowedEnumVariants::Allowed(&[
                    CommandType::PpmReset as u32,
                    CommandType::Cancel as u32,
                    CommandType::AckCcCi as u32,
                    CommandType::SetNotificationEnable as u32,
                    CommandType::GetCapability as u32,
                ]),
                found: command_type as u32,
            }),
        }
    }
}

impl Decode<()> for Command {
    fn decode<D: Decoder<Context = ()>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let header = CommandHeader::decode(decoder)?;
        Command::decode(&mut decoder.with_context(header))
    }
}

/// PPM command response data
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ResponseData {
    GetCapability(get_capability::ResponseData),
}

/// PPM command response
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Response {
    /// CCI is produced by every command
    pub status: Cci,
    /// Response data for the command
    pub data: Option<ResponseData>,
}

impl Response {
    /// Create a new response with the given status and optional data
    pub const fn new(status: Cci, data: Option<ResponseData>) -> Self {
        Self { status, data }
    }
}

impl From<Cci> for Response {
    fn from(status: Cci) -> Self {
        Self::new(status, None)
    }
}
