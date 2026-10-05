//! UCSI v1.2 implementation, see spec at https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/usb-type-c-ucsi-spec.pdf
#![allow(missing_docs)]

use bincode::de::{Decode, Decoder};
use bincode::enc::write::Writer;
use bincode::enc::{Encode, Encoder};
use bincode::error::{AllowedEnumVariants, DecodeError, EncodeError};
use bincode::{decode_from_slice, encode_into_slice};
use bytemuck::{Pod, Zeroable};

use crate::{GlobalPortId, LocalPortId, PdError, PortId};

pub mod cci;
pub mod lpm;
pub mod ppm;

/// Standard command length of 64 bits
pub const COMMAND_LEN: usize = 8;

/// Ucsi opcodes, see spec for more detail
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CommandType {
    PpmReset = 0x01,
    Cancel,
    ConnectorReset,
    AckCcCi,
    SetNotificationEnable,
    GetCapability,
    GetConnectorCapability,
    SetCcom,
    SetUor,
    SetPdm,
    SetPdr,
    GetAlternateModes,
    GetCamSupported,
    GetCurrentCam,
    SetNewCam,
    GetPdos,
    GetCableProperty,
    GetConnectorStatus,
    GetErrorStatus,
    SetPowerLevel,
    GetPdMessage,
}

impl CommandType {
    /// Returns true if this command has response data
    pub fn has_response(&self) -> bool {
        // Written as a negative so this function returns true for command not in this list.
        // One of the major uses for this function is to determine if there's a response to serialize.
        // If this function returns true by default then that makes it that a subsequent serialization
        // attempt will fail due to the lack of corresponding response data types. Otherwise the
        // serialization will not be attempted and no error will occur.
        !matches!(
            self,
            CommandType::PpmReset
                | CommandType::Cancel
                | CommandType::ConnectorReset
                | CommandType::AckCcCi
                | CommandType::SetNotificationEnable
                | CommandType::SetCcom
                | CommandType::SetUor
                | CommandType::SetPdr
                | CommandType::SetNewCam
        )
    }
}

/// Invalid command type error
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InvalidCommandType(pub u8);

impl From<InvalidCommandType> for PdError {
    fn from(_: InvalidCommandType) -> Self {
        PdError::InvalidParams
    }
}

impl TryFrom<u8> for CommandType {
    type Error = InvalidCommandType;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(CommandType::PpmReset),
            0x02 => Ok(CommandType::Cancel),
            0x03 => Ok(CommandType::ConnectorReset),
            0x04 => Ok(CommandType::AckCcCi),
            0x05 => Ok(CommandType::SetNotificationEnable),
            0x06 => Ok(CommandType::GetCapability),
            0x07 => Ok(CommandType::GetConnectorCapability),
            0x08 => Ok(CommandType::SetCcom),
            0x09 => Ok(CommandType::SetUor),
            0x0A => Ok(CommandType::SetPdm),
            0x0B => Ok(CommandType::SetPdr),
            0x0C => Ok(CommandType::GetAlternateModes),
            0x0D => Ok(CommandType::GetCamSupported),
            0x0E => Ok(CommandType::GetCurrentCam),
            0x0F => Ok(CommandType::SetNewCam),
            0x10 => Ok(CommandType::GetPdos),
            0x11 => Ok(CommandType::GetCableProperty),
            0x12 => Ok(CommandType::GetConnectorStatus),
            0x13 => Ok(CommandType::GetErrorStatus),
            0x14 => Ok(CommandType::SetPowerLevel),
            0x15 => Ok(CommandType::GetPdMessage),
            _ => Err(InvalidCommandType(value)),
        }
    }
}

impl From<CommandType> for u8 {
    fn from(command: CommandType) -> Self {
        command as u8
    }
}

/// UCSI commands
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Command<T: PortId> {
    PpmCommand(ppm::Command),
    LpmCommand(lpm::Command<T>),
}

pub type GlobalCommand = Command<GlobalPortId>;
pub type LocalCommand = Command<LocalPortId>;

impl<T: PortId> Command<T> {
    /// Returns the command type for this command
    pub const fn command_type(&self) -> CommandType {
        match self {
            Command::PpmCommand(cmd) => cmd.command_type(),
            Command::LpmCommand(cmd) => cmd.command_type(),
        }
    }

    /// Deserialize the a command from a slice
    pub fn decode_from_slice(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        decode_from_slice(bytes, bincode::config::standard().with_fixed_int_encoding())
    }
}

impl<Context, T: PortId> Decode<Context> for Command<T> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let raw = u16::decode(decoder)?;
        let header = CommandHeader::try_from(raw).map_err(|err| DecodeError::UnexpectedVariant {
            type_name: "CommandType",
            allowed: &AllowedEnumVariants::Range { min: 0x01, max: 0x15 },
            found: err.0 as u32,
        })?;
        match header.command {
            // PPM commands
            command_type @ (CommandType::PpmReset
            | CommandType::Cancel
            | CommandType::GetCapability
            | CommandType::AckCcCi
            | CommandType::SetNotificationEnable) => {
                let payload: [u8; ppm::Command::PAYLOAD_LEN] = Decode::decode(decoder)?;
                let command =
                    ppm::Command::from_payload(command_type, payload).map_err(|_| DecodeError::UnexpectedVariant {
                        type_name: "CommandType",
                        allowed: &AllowedEnumVariants::Allowed(&[
                            CommandType::PpmReset as u32,
                            CommandType::Cancel as u32,
                            CommandType::AckCcCi as u32,
                            CommandType::SetNotificationEnable as u32,
                            CommandType::GetCapability as u32,
                        ]),
                        found: command_type as u32,
                    })?;
                Ok(Command::PpmCommand(command))
            }
            // All other commands are LPM commands
            command_type => {
                let payload: [u8; lpm::COMMAND_PAYLOAD_LEN] = Decode::decode(decoder)?;
                let command = lpm::Command::from_payload(command_type, payload)?;
                Ok(Command::LpmCommand(command))
            }
        }
    }
}

impl From<lpm::InvalidRecipient> for DecodeError {
    fn from(value: lpm::InvalidRecipient) -> Self {
        DecodeError::UnexpectedVariant {
            type_name: "Recipient",
            allowed: &AllowedEnumVariants::Allowed(&[
                lpm::Recipient::Connector as u32,
                lpm::Recipient::Sop as u32,
                lpm::Recipient::SopP as u32,
                lpm::Recipient::SopPp as u32,
            ]),
            found: value.0 as u32,
        }
    }
}

impl From<lpm::get_pdos::InvalidSourceCapabilityType> for DecodeError {
    fn from(value: lpm::get_pdos::InvalidSourceCapabilityType) -> Self {
        DecodeError::UnexpectedVariant {
            type_name: "SourceCapabilityType",
            allowed: &AllowedEnumVariants::Allowed(&[
                lpm::get_pdos::SourceCapabilityType::Current as u32,
                lpm::get_pdos::SourceCapabilityType::Advertised as u32,
                lpm::get_pdos::SourceCapabilityType::Maximum as u32,
            ]),
            found: value.0 as u32,
        }
    }
}

impl From<lpm::get_pd_message::InvalidMessageType> for DecodeError {
    fn from(value: lpm::get_pd_message::InvalidMessageType) -> Self {
        DecodeError::UnexpectedVariant {
            type_name: "MessageType",
            allowed: &AllowedEnumVariants::Allowed(&[
                lpm::get_pd_message::MessageType::SinkCapExtended as u32,
                lpm::get_pd_message::MessageType::SourceCapExtended as u32,
                lpm::get_pd_message::MessageType::BatteryCap as u32,
                lpm::get_pd_message::MessageType::BatteryStatus as u32,
                lpm::get_pd_message::MessageType::DiscoverIdentity as u32,
            ]),
            found: value.0 as u32,
        }
    }
}

impl From<lpm::get_pd_message::InvalidArgs> for DecodeError {
    fn from(value: lpm::get_pd_message::InvalidArgs) -> Self {
        match value {
            lpm::get_pd_message::InvalidArgs::InvalidRecipient(err) => err.into(),
            lpm::get_pd_message::InvalidArgs::InvalidMessageType(err) => err.into(),
        }
    }
}

impl From<lpm::InvalidCommand> for DecodeError {
    fn from(value: lpm::InvalidCommand) -> Self {
        match value {
            lpm::InvalidCommand::InvalidCommandType(err) => DecodeError::UnexpectedVariant {
                type_name: "CommandType",
                allowed: &AllowedEnumVariants::Allowed(&[
                    CommandType::ConnectorReset as u32,
                    CommandType::GetConnectorCapability as u32,
                    CommandType::SetCcom as u32,
                    CommandType::SetUor as u32,
                    CommandType::SetPdr as u32,
                    CommandType::GetAlternateModes as u32,
                    CommandType::GetCamSupported as u32,
                    CommandType::GetCurrentCam as u32,
                    CommandType::SetNewCam as u32,
                    CommandType::GetPdos as u32,
                    CommandType::GetCableProperty as u32,
                    CommandType::GetConnectorStatus as u32,
                    CommandType::GetErrorStatus as u32,
                    CommandType::SetPowerLevel as u32,
                    CommandType::GetPdMessage as u32,
                ]),
                found: err.0 as u32,
            },
            lpm::InvalidCommand::InvalidCurrent(err) => DecodeError::UnexpectedVariant {
                type_name: "Current",
                allowed: &AllowedEnumVariants::Range { min: 0, max: 3 },
                found: err.0 as u32,
            },
            lpm::InvalidCommand::InvalidRecipient(err) => err.into(),
            lpm::InvalidCommand::InvalidSourceCapabilityType(err) => err.into(),
            lpm::InvalidCommand::InvalidPdMessageArgs(err) => err.into(),
            // These errors can only be produced when converting a command to its payload
            lpm::InvalidCommand::Overflow(_) => DecodeError::Other("Argument overflow"),
            lpm::InvalidCommand::InvalidNumPdos(_) => DecodeError::Other("Invalid number of PDOs"),
        }
    }
}

/// UCSI command response data
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ResponseData {
    Ppm(ppm::ResponseData),
    Lpm(lpm::ResponseData),
}

impl ResponseData {
    /// Encodes the response into a slice
    pub fn encode_into_slice(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
        encode_into_slice(self, bytes, bincode::config::standard().with_fixed_int_encoding())
    }
}

impl Encode for ResponseData {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            ResponseData::Ppm(resp) => {
                let (bytes, len) = resp.to_bytes();
                encoder
                    .writer()
                    .write(bytes.get(..len).ok_or(EncodeError::UnexpectedEnd)?)
            }
            ResponseData::Lpm(resp) => {
                let (bytes, len) = resp.to_bytes();
                encoder
                    .writer()
                    .write(bytes.get(..len).ok_or(EncodeError::UnexpectedEnd)?)
            }
        }
    }
}

/// UCSI command response
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Response<T: PortId> {
    /// CCI is produced by every command
    pub cci: cci::Cci<T>,
    /// Response data for the command
    pub data: Option<ResponseData>,
}

impl<T: PortId> From<cci::Cci<T>> for Response<T> {
    fn from(cci: cci::Cci<T>) -> Self {
        Self { cci, data: None }
    }
}

impl<T: PortId> From<ppm::Response<T>> for Response<T> {
    fn from(response: ppm::Response<T>) -> Self {
        Self {
            cci: response.cci,
            data: response.data.map(ResponseData::Ppm),
        }
    }
}

impl<T: PortId> From<lpm::Response<T>> for Response<T> {
    fn from(response: lpm::Response<T>) -> Self {
        Self {
            cci: response.cci,
            data: response.data.map(ResponseData::Lpm),
        }
    }
}

pub type GlobalResponse = Response<GlobalPortId>;
pub type LocalResponse = Response<LocalPortId>;

/// Raw wire format of [`CommandHeader`], the common header shared by all UCSI commands
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Zeroable, Pod)]
pub struct CommandHeaderRaw {
    /// Command
    pub command: u8,
    /// Data length
    pub data_len: u8,
}

impl CommandHeaderRaw {
    /// Length of the raw command header in bytes
    pub const LEN: usize = size_of::<Self>();
}

#[cfg(feature = "defmt")]
impl defmt::Format for CommandHeaderRaw {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "CommandHeaderRaw {{ command: {}, data_len: {} }}",
            self.command,
            self.data_len
        )
    }
}

/// Higher-level wrapper around [`CommandHeaderRaw`]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CommandHeader {
    /// Command type
    pub command: CommandType,
    /// Data length
    pub data_len: u8,
}

impl CommandHeader {
    /// Create a new command header
    pub const fn new(command: CommandType, data_len: u8) -> Self {
        Self { command, data_len }
    }
}

impl TryFrom<CommandHeaderRaw> for CommandHeader {
    type Error = InvalidCommandType;

    fn try_from(raw: CommandHeaderRaw) -> Result<Self, Self::Error> {
        Ok(Self {
            command: raw.command.try_into()?,
            data_len: raw.data_len,
        })
    }
}

impl From<CommandHeader> for CommandHeaderRaw {
    fn from(header: CommandHeader) -> Self {
        Self {
            command: header.command.into(),
            data_len: header.data_len,
        }
    }
}

impl TryFrom<u16> for CommandHeader {
    type Error = InvalidCommandType;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        CommandHeader::try_from(bytemuck::must_cast::<_, CommandHeaderRaw>(value.to_le_bytes()))
    }
}

impl From<CommandHeader> for u16 {
    fn from(header: CommandHeader) -> Self {
        u16::from_le_bytes(bytemuck::must_cast(CommandHeaderRaw::from(header)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test PPM command decoding
    ///
    /// Only test one command just to make sure the overall flow works
    #[test]
    fn test_command_decoding_ppm() {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = CommandType::AckCcCi as u8;
        bytes[2] = 0x2; // Set connector change ack

        let (ack_cc_ci, consumed) = Command::decode_from_slice(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(
            ack_cc_ci,
            GlobalCommand::PpmCommand(ppm::Command::AckCcCi(ppm::ack_cc_ci::Args {
                ack: ppm::ack_cc_ci::Ack::from(0x2)
            }))
        );
    }

    /// Test LPM command decoding
    ///
    /// Only test one command just to make sure the overall flow works
    #[test]
    fn test_command_decoding_lpm() {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = CommandType::GetConnectorStatus as u8;
        bytes[2] = 0x1;

        let (get_connector_status, consumed) = Command::decode_from_slice(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(
            get_connector_status,
            Command::LpmCommand(lpm::Command::new(GlobalPortId(1), lpm::CommandData::GetConnectorStatus))
        );
    }

    /// Test PPM response encoding
    ///
    /// Only test one response type just to make sure the overall flow works
    #[test]
    fn test_ppm_response_encoding() {
        let (response_data, bytes) = ppm::get_capability::test::create_response_data();
        let expected = ResponseData::Ppm(ppm::ResponseData::GetCapability(response_data));

        let mut encoded_bytes = [0u8; ppm::get_capability::RESPONSE_DATA_LEN];
        let len = expected.encode_into_slice(&mut encoded_bytes).unwrap();

        assert_eq!(len, ppm::get_capability::RESPONSE_DATA_LEN);
        assert_eq!(encoded_bytes, bytes);
    }

    /// Test LPM response encoding
    ///
    /// Only test one response type just to make sure the overall flow works
    #[test]
    fn test_lpm_response_encoding() {
        let (response_data, bytes) = lpm::get_connector_status::test::create_response_data();
        let expected = ResponseData::Lpm(lpm::ResponseData::GetConnectorStatus(response_data));

        let mut encoded_bytes = [0u8; lpm::get_connector_status::RESPONSE_DATA_LEN];
        let len = expected.encode_into_slice(&mut encoded_bytes).unwrap();

        assert_eq!(len, lpm::get_connector_status::RESPONSE_DATA_LEN);
        assert_eq!(encoded_bytes, bytes);
    }

    /// Every defined command type round-trips through the raw header, preserving the data length
    #[test]
    fn test_command_header_roundtrip() {
        const COMMAND_TYPES: [CommandType; 21] = [
            CommandType::PpmReset,
            CommandType::Cancel,
            CommandType::ConnectorReset,
            CommandType::AckCcCi,
            CommandType::SetNotificationEnable,
            CommandType::GetCapability,
            CommandType::GetConnectorCapability,
            CommandType::SetCcom,
            CommandType::SetUor,
            CommandType::SetPdm,
            CommandType::SetPdr,
            CommandType::GetAlternateModes,
            CommandType::GetCamSupported,
            CommandType::GetCurrentCam,
            CommandType::SetNewCam,
            CommandType::GetPdos,
            CommandType::GetCableProperty,
            CommandType::GetConnectorStatus,
            CommandType::GetErrorStatus,
            CommandType::SetPowerLevel,
            CommandType::GetPdMessage,
        ];

        for command in COMMAND_TYPES {
            let expected = CommandHeader::new(command, 0x06);
            let raw = CommandHeaderRaw::from(expected);

            assert_eq!(raw.command, command as u8);
            assert_eq!(raw.data_len, 0x06);
            assert_eq!(CommandHeader::try_from(raw), Ok(expected));
            assert_eq!(CommandHeader::try_from(u16::from(expected)), Ok(expected));
        }
    }

    #[test]
    fn test_command_header_invalid_command() {
        assert_eq!(
            CommandHeader::try_from(CommandHeaderRaw {
                command: 0x00,
                data_len: 0x06
            }),
            Err(InvalidCommandType(0x00))
        );
        assert_eq!(
            CommandHeader::try_from(CommandHeaderRaw {
                command: 0x16,
                data_len: 0x06
            }),
            Err(InvalidCommandType(0x16))
        );
    }
}
