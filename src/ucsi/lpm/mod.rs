use bincode::de::Decoder;
use bincode::enc::Encoder;
use bincode::error::{AllowedEnumVariants, DecodeError, EncodeError};
use bincode::{Decode, Encode};
use bitfield::bitfield;

use crate::ucsi::{CommandHeader, CommandType};
use crate::GlobalPortId;

pub mod get_connector_status;

/// Connector reset types
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ResetType {
    Hard,
    Data,
}

/// LPM command data
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CommandData {
    ConnectorReset(ResetType),
    GetConnectorStatus,
}

impl CommandData {
    /// Returns the command type for this command
    pub const fn command_type(&self) -> CommandType {
        match self {
            CommandData::ConnectorReset(_) => CommandType::ConnectorReset,
            CommandData::GetConnectorStatus => CommandType::GetConnectorStatus,
        }
    }
}

/// LPM commands
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Command {
    pub port: GlobalPortId,
    pub operation: CommandData,
}

impl Command {
    /// Returns the command type for this command
    pub const fn command_type(&self) -> CommandType {
        self.operation.command_type()
    }
}

impl Encode for Command {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        CommandHeader::new(self.command_type(), 0).encode(encoder)?;
        match self.operation {
            CommandData::GetConnectorStatus => {
                self.port.0.encode(encoder)?;
                get_connector_status::Args.encode(encoder)
            }
            _ => Err(EncodeError::Other("Unsupported command")),
        }
    }
}

impl Decode<CommandHeader> for Command {
    fn decode<D: Decoder<Context = CommandHeader>>(decoder: &mut D) -> Result<Self, DecodeError> {
        match decoder.context().command() {
            CommandType::GetConnectorStatus => {
                let connector_number = ConnectorNumberRaw::decode(decoder)?.connector_number();
                // Don't actually have any args, but need to consume command padding
                let _args = get_connector_status::Args::decode(decoder)?;
                Ok(Command {
                    port: GlobalPortId(connector_number),
                    operation: CommandData::GetConnectorStatus,
                })
            }
            command_type => Err(DecodeError::UnexpectedVariant {
                type_name: "CommandType",
                allowed: &AllowedEnumVariants::Allowed(&[CommandType::GetConnectorStatus as u32]),
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

/// LPM response data
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Response {
    GetConnectorStatus(get_connector_status::ResponseData),
}

impl Encode for Response {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            Response::GetConnectorStatus(data) => data.encode(encoder),
        }
    }
}

impl Decode<CommandType> for Response {
    fn decode<D: Decoder<Context = CommandType>>(decoder: &mut D) -> Result<Self, DecodeError> {
        match decoder.context() {
            CommandType::GetConnectorStatus => Ok(Response::GetConnectorStatus(
                get_connector_status::ResponseData::decode(decoder)?,
            )),
            command_type => Err(DecodeError::UnexpectedVariant {
                type_name: "CommandType",
                allowed: &AllowedEnumVariants::Allowed(&[CommandType::GetConnectorStatus as u32]),
                found: *command_type as u32,
            }),
        }
    }
}

bitfield! {
    /// Raw connector number
    #[derive(Copy, Clone, Default)]
    pub(self) struct ConnectorNumberRaw(u8);
    impl Debug;

    // Connector number
    pub u8, connector_number, set_connector_number: 6, 0;
    // Only 7-bits used for the connector number, some commands use this bit as part of their arguments
    pub bool, high_bit, set_high_bit: 7;
}

impl Encode for ConnectorNumberRaw {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.0.encode(encoder)
    }
}

impl Decode<CommandHeader> for ConnectorNumberRaw {
    fn decode<D: Decoder<Context = CommandHeader>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let raw = u8::decode(decoder)?;
        Ok(ConnectorNumberRaw(raw))
    }
}

#[cfg(test)]
mod tests {
    use bincode::config::standard;
    use bincode::decode_from_slice;

    use super::*;
    use crate::ucsi::COMMAND_LEN;

    #[test]
    fn test_decode_get_connector_status() {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = CommandType::GetConnectorStatus as u8;
        bytes[2] = 0x1;

        let (get_connector_status, consumed): (Command, usize) =
            decode_from_slice(&bytes, standard().with_fixed_int_encoding()).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(
            get_connector_status,
            Command {
                port: GlobalPortId(1),
                operation: CommandData::GetConnectorStatus,
            }
        );
    }
}
