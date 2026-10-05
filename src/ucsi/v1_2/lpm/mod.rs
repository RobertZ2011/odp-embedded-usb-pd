use bitfield::bitfield;

use crate::ucsi::v1_2::{cci, CommandHeaderRaw, CommandType, InvalidCommandType, COMMAND_LEN};
use crate::{GlobalPortId, LocalPortId, PortId};

pub mod connector_reset;
pub mod get_alternate_modes;
pub mod get_cable_property;
pub mod get_cam_supported;
pub mod get_connector_capability;
pub mod get_connector_status;
pub mod get_current_cam;
pub mod get_error_status;
pub mod get_pd_message;
pub mod get_pdos;
pub mod set_ccom;
pub mod set_new_cam;
pub mod set_pdr;
pub mod set_power_level;
pub mod set_uor;

/// Length of an LPM command payload, the command minus its header
pub const COMMAND_PAYLOAD_LEN: usize = COMMAND_LEN - size_of::<CommandHeaderRaw>();

/// Copies `src` into a new buffer, truncating or zero-padding it as needed
fn resize<const SRC: usize, const DST: usize>(src: [u8; SRC]) -> [u8; DST] {
    let mut dst = [0u8; DST];
    dst.iter_mut().zip(src.iter()).for_each(|(dst, src)| *dst = *src);
    dst
}

/// Error returned when a command cannot be converted to or from its payload
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InvalidCommand {
    /// Not an LPM command
    InvalidCommandType(InvalidCommandType),
    /// SET_POWER_LEVEL argument out of range
    Overflow(set_power_level::OverflowError),
    /// Invalid SET_POWER_LEVEL current
    InvalidCurrent(set_power_level::InvalidCurrent),
    /// Invalid GET_PDOS number of PDOs
    InvalidNumPdos(get_pdos::InvalidNumPdos),
    /// Invalid GET_PDOS source capability type
    InvalidSourceCapabilityType(get_pdos::InvalidSourceCapabilityType),
    /// Invalid recipient
    InvalidRecipient(InvalidRecipient),
    /// Invalid GET_PD_MESSAGE arguments
    InvalidPdMessageArgs(get_pd_message::InvalidArgs),
}

impl From<InvalidCommandType> for InvalidCommand {
    fn from(value: InvalidCommandType) -> Self {
        InvalidCommand::InvalidCommandType(value)
    }
}

impl From<set_power_level::OverflowError> for InvalidCommand {
    fn from(value: set_power_level::OverflowError) -> Self {
        InvalidCommand::Overflow(value)
    }
}

impl From<set_power_level::InvalidCurrent> for InvalidCommand {
    fn from(value: set_power_level::InvalidCurrent) -> Self {
        InvalidCommand::InvalidCurrent(value)
    }
}

impl From<get_pdos::InvalidNumPdos> for InvalidCommand {
    fn from(value: get_pdos::InvalidNumPdos) -> Self {
        InvalidCommand::InvalidNumPdos(value)
    }
}

impl From<get_pdos::InvalidSourceCapabilityType> for InvalidCommand {
    fn from(value: get_pdos::InvalidSourceCapabilityType) -> Self {
        InvalidCommand::InvalidSourceCapabilityType(value)
    }
}

impl From<InvalidRecipient> for InvalidCommand {
    fn from(value: InvalidRecipient) -> Self {
        InvalidCommand::InvalidRecipient(value)
    }
}

impl From<get_pd_message::InvalidArgs> for InvalidCommand {
    fn from(value: get_pd_message::InvalidArgs) -> Self {
        InvalidCommand::InvalidPdMessageArgs(value)
    }
}

/// Error returned when response data cannot be reconstructed from its bytes
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InvalidResponseData {
    /// Not an LPM command
    InvalidCommandType(InvalidCommandType),
    /// Invalid GET_CONNECTOR_STATUS response data
    ConnectorStatus(get_connector_status::InvalidResponseData),
}

impl From<InvalidCommandType> for InvalidResponseData {
    fn from(value: InvalidCommandType) -> Self {
        InvalidResponseData::InvalidCommandType(value)
    }
}

impl From<get_connector_status::InvalidResponseData> for InvalidResponseData {
    fn from(value: get_connector_status::InvalidResponseData) -> Self {
        InvalidResponseData::ConnectorStatus(value)
    }
}

/// LPM command data
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CommandData {
    ConnectorReset(connector_reset::Args),
    GetConnectorStatus,
    GetConnectorCapability,
    SetPowerLevel(set_power_level::Args),
    SetNewCam(set_new_cam::Args),
    GetErrorStatus,
    SetCcom(set_ccom::Args),
    SetUor(set_uor::Args),
    SetPdr(set_pdr::Args),
    GetAlternateModes(get_alternate_modes::Args),
    GetCamSupported,
    GetCurrentCam,
    GetPdos(get_pdos::Args),
    GetCableProperty,
    GetPdMessage(get_pd_message::Args),
}

impl CommandData {
    /// Returns the command type for this command
    pub const fn command_type(&self) -> CommandType {
        match self {
            CommandData::ConnectorReset(_) => CommandType::ConnectorReset,
            CommandData::GetConnectorStatus => CommandType::GetConnectorStatus,
            CommandData::GetConnectorCapability => CommandType::GetConnectorCapability,
            CommandData::SetPowerLevel(_) => CommandType::SetPowerLevel,
            CommandData::SetNewCam(_) => CommandType::SetNewCam,
            CommandData::GetErrorStatus => CommandType::GetErrorStatus,
            CommandData::SetCcom(_) => CommandType::SetCcom,
            CommandData::SetUor(_) => CommandType::SetUor,
            CommandData::SetPdr(_) => CommandType::SetPdr,
            CommandData::GetAlternateModes(_) => CommandType::GetAlternateModes,
            CommandData::GetCamSupported => CommandType::GetCamSupported,
            CommandData::GetCurrentCam => CommandType::GetCurrentCam,
            CommandData::GetPdos(_) => CommandType::GetPdos,
            CommandData::GetCableProperty => CommandType::GetCableProperty,
            CommandData::GetPdMessage(_) => CommandType::GetPdMessage,
        }
    }
}

/// LPM commands
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Command<T: PortId> {
    port: T,
    operation: CommandData,
}

impl<T: PortId> Command<T> {
    pub const fn new(port: T, operation: CommandData) -> Self {
        Command { port, operation }
    }

    pub fn port(&self) -> T {
        self.port
    }

    pub fn set_port(&mut self, port: T) -> &mut Self {
        self.port = port;
        // These commands have the connector number as part of their arguments, update them too
        // TODO: Figure out how to remove this
        match self.operation {
            CommandData::ConnectorReset(ref mut args) => {
                args.connector_number = self.port.into();
            }
            CommandData::SetPowerLevel(ref mut args) => {
                args.connector_number = self.port.into();
            }
            CommandData::SetNewCam(ref mut args) => {
                args.connector_number = self.port.into();
            }
            CommandData::SetCcom(ref mut args) => {
                args.connector_number = self.port.into();
            }
            CommandData::SetUor(ref mut args) => {
                args.connector_number = self.port.into();
            }
            CommandData::SetPdr(ref mut args) => {
                args.connector_number = self.port.into();
            }
            CommandData::GetAlternateModes(ref mut args) => {
                args.connector_number = self.port.into();
            }
            CommandData::GetPdos(ref mut args) => {
                args.connector_number = self.port.into();
            }
            CommandData::GetPdMessage(ref mut args) => {
                args.connector_number = self.port.into();
            }
            _ => {}
        }

        self
    }

    pub fn operation(&self) -> CommandData {
        self.operation
    }

    pub fn set_operation(&mut self, operation: CommandData) -> &mut Self {
        self.operation = operation;
        self
    }
}

impl<T: PortId> Command<T> {
    /// Length of an LPM command payload, the command minus its header
    pub const PAYLOAD_LEN: usize = COMMAND_PAYLOAD_LEN;

    /// Returns the command type for this command
    pub const fn command_type(&self) -> CommandType {
        self.operation.command_type()
    }

    /// Converts this command into its raw payload bytes
    ///
    /// Returns an error if the arguments cannot be represented on the wire.
    pub fn to_payload(&self) -> Result<[u8; COMMAND_PAYLOAD_LEN], InvalidCommand> {
        // Commands that combine the connector number with their arguments handle it themselves
        let raw_port: u8 = self.port.into();
        match self.operation {
            CommandData::ConnectorReset(args) => Ok(bytemuck::must_cast(connector_reset::ArgsRaw::from(args))),
            CommandData::GetConnectorStatus => Ok(bytemuck::must_cast(get_connector_status::ArgsRaw::from(raw_port))),
            CommandData::GetConnectorCapability => {
                Ok(bytemuck::must_cast(get_connector_capability::ArgsRaw::from(raw_port)))
            }
            CommandData::SetPowerLevel(args) => Ok(bytemuck::must_cast(set_power_level::ArgsRaw::try_from(args)?)),
            CommandData::SetNewCam(args) => Ok(bytemuck::must_cast(set_new_cam::ArgsRaw::from(args))),
            CommandData::GetErrorStatus => Ok(bytemuck::must_cast(get_error_status::ArgsRaw::from(raw_port))),
            CommandData::SetCcom(args) => Ok(bytemuck::must_cast(set_ccom::ArgsRaw::from(args))),
            CommandData::SetUor(args) => Ok(bytemuck::must_cast(set_uor::ArgsRaw::from(args))),
            CommandData::SetPdr(args) => Ok(bytemuck::must_cast(set_pdr::ArgsRaw::from(args))),
            // This command has a different format without a leading port number
            // TODO: Figure out if this can stay an exception or if each command is responsible for pulling its port number.
            CommandData::GetAlternateModes(args) => Ok(bytemuck::must_cast(get_alternate_modes::ArgsRaw::from(args))),
            CommandData::GetCamSupported => Ok(bytemuck::must_cast(get_cam_supported::ArgsRaw::from(raw_port))),
            CommandData::GetCurrentCam => Ok(bytemuck::must_cast(get_current_cam::ArgsRaw::from(raw_port))),
            CommandData::GetPdos(args) => Ok(bytemuck::must_cast(get_pdos::ArgsRaw::try_from(args)?)),
            CommandData::GetCableProperty => Ok(bytemuck::must_cast(get_cable_property::ArgsRaw::from(raw_port))),
            CommandData::GetPdMessage(args) => Ok(bytemuck::must_cast(get_pd_message::ArgsRaw::from(args))),
        }
    }

    /// Reconstructs a command from its command type and raw payload bytes
    ///
    /// Returns an error if `command_type` is not an LPM command or if the
    /// payload does not contain valid arguments.
    pub fn from_payload(command_type: CommandType, payload: [u8; COMMAND_PAYLOAD_LEN]) -> Result<Self, InvalidCommand> {
        match command_type {
            CommandType::ConnectorReset => {
                let args = connector_reset::Args::from(bytemuck::must_cast::<_, connector_reset::ArgsRaw>(payload));
                Ok(Command {
                    port: From::from(args.connector_number),
                    operation: CommandData::ConnectorReset(args),
                })
            }
            CommandType::GetConnectorStatus => {
                let connector_number = u8::from(bytemuck::must_cast::<_, get_connector_status::ArgsRaw>(payload));
                Ok(Command {
                    port: From::from(connector_number),
                    operation: CommandData::GetConnectorStatus,
                })
            }
            CommandType::GetConnectorCapability => {
                let connector_number = u8::from(bytemuck::must_cast::<_, get_connector_capability::ArgsRaw>(payload));
                Ok(Command {
                    port: From::from(connector_number),
                    operation: CommandData::GetConnectorCapability,
                })
            }
            CommandType::SetPowerLevel => {
                let args =
                    set_power_level::Args::try_from(bytemuck::must_cast::<_, set_power_level::ArgsRaw>(payload))?;
                Ok(Command {
                    port: From::from(args.connector_number),
                    operation: CommandData::SetPowerLevel(args),
                })
            }
            CommandType::SetNewCam => {
                let args = set_new_cam::Args::from(bytemuck::must_cast::<_, set_new_cam::ArgsRaw>(payload));
                Ok(Command {
                    port: From::from(args.connector_number),
                    operation: CommandData::SetNewCam(args),
                })
            }
            CommandType::GetErrorStatus => {
                let connector_number = u8::from(bytemuck::must_cast::<_, get_error_status::ArgsRaw>(payload));
                Ok(Command {
                    port: From::from(connector_number),
                    operation: CommandData::GetErrorStatus,
                })
            }
            CommandType::SetCcom => {
                let args = set_ccom::Args::from(bytemuck::must_cast::<_, set_ccom::ArgsRaw>(payload));
                Ok(Command {
                    port: From::from(args.connector_number),
                    operation: CommandData::SetCcom(args),
                })
            }
            CommandType::SetUor => {
                let args = set_uor::Args::from(bytemuck::must_cast::<_, set_uor::ArgsRaw>(payload));
                Ok(Command {
                    port: From::from(args.connector_number),
                    operation: CommandData::SetUor(args),
                })
            }
            CommandType::SetPdr => {
                let args = set_pdr::Args::from(bytemuck::must_cast::<_, set_pdr::ArgsRaw>(payload));
                Ok(Command {
                    port: From::from(args.connector_number),
                    operation: CommandData::SetPdr(args),
                })
            }
            CommandType::GetAlternateModes => {
                let args = get_alternate_modes::Args::try_from(
                    bytemuck::must_cast::<_, get_alternate_modes::ArgsRaw>(payload),
                )?;
                Ok(Command {
                    port: From::from(args.connector_number),
                    operation: CommandData::GetAlternateModes(args),
                })
            }
            CommandType::GetCamSupported => {
                let connector_number = u8::from(bytemuck::must_cast::<_, get_cam_supported::ArgsRaw>(payload));
                Ok(Command {
                    port: From::from(connector_number),
                    operation: CommandData::GetCamSupported,
                })
            }
            CommandType::GetCurrentCam => {
                let connector_number = u8::from(bytemuck::must_cast::<_, get_current_cam::ArgsRaw>(payload));
                Ok(Command {
                    port: From::from(connector_number),
                    operation: CommandData::GetCurrentCam,
                })
            }
            CommandType::GetPdos => {
                let args = get_pdos::Args::try_from(bytemuck::must_cast::<_, get_pdos::ArgsRaw>(payload))?;
                Ok(Command {
                    port: From::from(args.connector_number),
                    operation: CommandData::GetPdos(args),
                })
            }
            CommandType::GetCableProperty => {
                let connector_number = u8::from(bytemuck::must_cast::<_, get_cable_property::ArgsRaw>(payload));
                Ok(Command {
                    port: From::from(connector_number),
                    operation: CommandData::GetCableProperty,
                })
            }
            CommandType::GetPdMessage => {
                let args = get_pd_message::Args::try_from(bytemuck::must_cast::<_, get_pd_message::ArgsRaw>(payload))?;
                Ok(Command {
                    port: From::from(args.connector_number),
                    operation: CommandData::GetPdMessage(args),
                })
            }
            _ => Err(InvalidCommand::InvalidCommandType(InvalidCommandType(
                command_type as u8,
            ))),
        }
    }
}

pub type GlobalCommand = Command<GlobalPortId>;
pub type LocalCommand = Command<LocalPortId>;

/// LPM response data
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ResponseData {
    ConnectorReset,
    GetConnectorStatus(get_connector_status::ResponseData),
    GetConnectorCapability(get_connector_capability::ResponseData),
    GetErrorStatus(get_error_status::ResponseData),
    GetAlternateModes(get_alternate_modes::ResponseData),
    GetCamSupported(get_cam_supported::ResponseData),
    GetCurrentCam(get_current_cam::ResponseData),
    GetPdos(get_pdos::ResponseData),
    GetCableProperty(get_cable_property::ResponseData),
    GetPdMessage(get_pd_message::ResponseData),
}

impl ResponseData {
    /// Maximum length in bytes of any LPM response data
    pub const MAX_LEN: usize = get_current_cam::ResponseDataRaw::LEN;

    /// Returns the command type that produces this response data
    pub const fn command_type(&self) -> CommandType {
        match self {
            ResponseData::ConnectorReset => CommandType::ConnectorReset,
            ResponseData::GetConnectorStatus(_) => CommandType::GetConnectorStatus,
            ResponseData::GetConnectorCapability(_) => CommandType::GetConnectorCapability,
            ResponseData::GetErrorStatus(_) => CommandType::GetErrorStatus,
            ResponseData::GetAlternateModes(_) => CommandType::GetAlternateModes,
            ResponseData::GetCamSupported(_) => CommandType::GetCamSupported,
            ResponseData::GetCurrentCam(_) => CommandType::GetCurrentCam,
            ResponseData::GetPdos(_) => CommandType::GetPdos,
            ResponseData::GetCableProperty(_) => CommandType::GetCableProperty,
            ResponseData::GetPdMessage(_) => CommandType::GetPdMessage,
        }
    }

    /// Converts this response data into raw bytes
    ///
    /// Returns a [`Self::MAX_LEN`] sized buffer along with the number of valid
    /// bytes at its start.
    pub fn to_bytes(&self) -> ([u8; Self::MAX_LEN], usize) {
        match self {
            // No response data
            ResponseData::ConnectorReset => ([0; Self::MAX_LEN], 0),
            ResponseData::GetConnectorStatus(data) => (
                resize(
                    bytemuck::must_cast::<_, [u8; get_connector_status::ResponseDataRaw::LEN]>(
                        get_connector_status::ResponseDataRaw::from(*data),
                    ),
                ),
                get_connector_status::ResponseDataRaw::LEN,
            ),
            ResponseData::GetConnectorCapability(data) => (
                resize(bytemuck::must_cast::<
                    _,
                    [u8; get_connector_capability::ResponseDataRaw::LEN],
                >(get_connector_capability::ResponseDataRaw::from(
                    *data,
                ))),
                get_connector_capability::ResponseDataRaw::LEN,
            ),
            ResponseData::GetErrorStatus(data) => (
                resize(bytemuck::must_cast::<_, [u8; get_error_status::ResponseDataRaw::LEN]>(
                    get_error_status::ResponseDataRaw::from(*data),
                )),
                get_error_status::ResponseDataRaw::LEN,
            ),
            ResponseData::GetAlternateModes(data) => (
                resize(
                    bytemuck::must_cast::<_, [u8; get_alternate_modes::ResponseDataRaw::LEN]>(
                        get_alternate_modes::ResponseDataRaw::from(*data),
                    ),
                ),
                get_alternate_modes::ResponseDataRaw::LEN,
            ),
            ResponseData::GetCamSupported(data) => (
                resize(bytemuck::must_cast::<_, [u8; get_cam_supported::ResponseDataRaw::LEN]>(
                    get_cam_supported::ResponseDataRaw::from(*data),
                )),
                get_cam_supported::ResponseDataRaw::LEN,
            ),
            ResponseData::GetCurrentCam(data) => (
                resize(bytemuck::must_cast::<_, [u8; get_current_cam::ResponseDataRaw::LEN]>(
                    get_current_cam::ResponseDataRaw::from(*data),
                )),
                get_current_cam::ResponseDataRaw::LEN,
            ),
            // Only the valid PDOs are sent, the response is shorter than the raw type when fewer are present
            ResponseData::GetPdos(data) => (
                resize(bytemuck::must_cast::<_, [u8; get_pdos::ResponseDataRaw::LEN]>(
                    get_pdos::ResponseDataRaw::from(*data),
                )),
                data.iter().len() * size_of::<u32>(),
            ),
            ResponseData::GetCableProperty(data) => (
                resize(
                    bytemuck::must_cast::<_, [u8; get_cable_property::ResponseDataRaw::LEN]>(
                        get_cable_property::ResponseDataRaw::from(*data),
                    ),
                ),
                get_cable_property::ResponseDataRaw::LEN,
            ),
            ResponseData::GetPdMessage(data) => (
                resize(bytemuck::must_cast::<_, [u8; get_pd_message::ResponseDataRaw::LEN]>(
                    get_pd_message::ResponseDataRaw::from(*data),
                )),
                get_pd_message::ResponseDataRaw::LEN,
            ),
        }
    }

    /// Reconstructs response data from its command type and raw bytes
    ///
    /// Returns an error if `command_type` is not an LPM command or if the bytes
    /// do not contain valid response data.
    pub fn from_bytes(command_type: CommandType, bytes: [u8; Self::MAX_LEN]) -> Result<Self, InvalidResponseData> {
        match command_type {
            CommandType::ConnectorReset => Ok(ResponseData::ConnectorReset),
            CommandType::GetConnectorStatus => {
                let raw = bytemuck::must_cast::<_, get_connector_status::ResponseDataRaw>(resize::<
                    { Self::MAX_LEN },
                    { get_connector_status::ResponseDataRaw::LEN },
                >(bytes));
                Ok(ResponseData::GetConnectorStatus(
                    get_connector_status::ResponseData::try_from(raw)?,
                ))
            }
            CommandType::GetConnectorCapability => Ok(ResponseData::GetConnectorCapability(
                bytemuck::must_cast::<_, get_connector_capability::ResponseDataRaw>(resize::<
                    { Self::MAX_LEN },
                    { get_connector_capability::ResponseDataRaw::LEN },
                >(bytes))
                .into(),
            )),
            CommandType::GetErrorStatus => Ok(ResponseData::GetErrorStatus(
                bytemuck::must_cast::<_, get_error_status::ResponseDataRaw>(resize::<
                    { Self::MAX_LEN },
                    { get_error_status::ResponseDataRaw::LEN },
                >(bytes))
                .into(),
            )),
            CommandType::GetAlternateModes => Ok(ResponseData::GetAlternateModes(
                bytemuck::must_cast::<_, get_alternate_modes::ResponseDataRaw>(resize::<
                    { Self::MAX_LEN },
                    { get_alternate_modes::ResponseDataRaw::LEN },
                >(bytes))
                .into(),
            )),
            CommandType::GetCamSupported => Ok(ResponseData::GetCamSupported(
                bytemuck::must_cast::<_, get_cam_supported::ResponseDataRaw>(resize::<
                    { Self::MAX_LEN },
                    { get_cam_supported::ResponseDataRaw::LEN },
                >(bytes))
                .into(),
            )),
            CommandType::GetCurrentCam => Ok(ResponseData::GetCurrentCam(
                bytemuck::must_cast::<_, get_current_cam::ResponseDataRaw>(resize::<
                    { Self::MAX_LEN },
                    { get_current_cam::ResponseDataRaw::LEN },
                >(bytes))
                .into(),
            )),
            CommandType::GetPdos => Ok(ResponseData::GetPdos(
                bytemuck::must_cast::<_, get_pdos::ResponseDataRaw>(resize::<
                    { Self::MAX_LEN },
                    { get_pdos::ResponseDataRaw::LEN },
                >(bytes))
                .into(),
            )),
            CommandType::GetCableProperty => Ok(ResponseData::GetCableProperty(
                bytemuck::must_cast::<_, get_cable_property::ResponseDataRaw>(resize::<
                    { Self::MAX_LEN },
                    { get_cable_property::ResponseDataRaw::LEN },
                >(bytes))
                .into(),
            )),
            CommandType::GetPdMessage => Ok(ResponseData::GetPdMessage(
                bytemuck::must_cast::<_, get_pd_message::ResponseDataRaw>(resize::<
                    { Self::MAX_LEN },
                    { get_pd_message::ResponseDataRaw::LEN },
                >(bytes))
                .into(),
            )),
            _ => Err(InvalidResponseData::InvalidCommandType(InvalidCommandType(
                command_type as u8,
            ))),
        }
    }
}

/// LPM command response
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Response<T: PortId> {
    /// CCI is produced by every command
    pub cci: cci::Cci<T>,
    /// Response data for the command
    pub data: Option<ResponseData>,
}

pub type GlobalResponse = Response<GlobalPortId>;
pub type LocalResponse = Response<LocalPortId>;

bitfield! {
    /// Raw connector number
    #[derive(Copy, Clone, Default, PartialEq, Eq)]
    pub(self) struct ConnectorNumberRaw(u8);
    impl Debug;

    // Connector number
    pub u8, connector_number, set_connector_number: 6, 0;
    // Only 7-bits used for the connector number, some commands use this bit as part of their arguments
    pub bool, high_bit, set_high_bit: 7;
}

/// Common recipient type used by multiple commands
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Recipient {
    /// Connector
    Connector,
    /// SOP
    Sop,
    /// SOP'
    SopP,
    /// SOP''
    SopPp,
}

/// Invalid recipient error, contains the invalid recipient value
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InvalidRecipient(pub u8);

impl TryFrom<u8> for Recipient {
    type Error = InvalidRecipient;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x0 => Ok(Recipient::Connector),
            0x1 => Ok(Recipient::Sop),
            0x2 => Ok(Recipient::SopP),
            0x3 => Ok(Recipient::SopPp),
            v => Err(InvalidRecipient(v)),
        }
    }
}

impl From<Recipient> for u8 {
    fn from(value: Recipient) -> Self {
        match value {
            Recipient::Connector => 0x0,
            Recipient::Sop => 0x1,
            Recipient::SopP => 0x2,
            Recipient::SopPp => 0x3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PowerRole;

    /// Decodes a command from a full command buffer, splitting off the header
    fn decode(bytes: [u8; COMMAND_LEN]) -> Result<GlobalCommand, InvalidCommand> {
        let header = CommandHeaderRaw(u16::from_le_bytes([bytes[0], bytes[1]]));
        let command_type = CommandType::try_from(header.command()).unwrap();
        let mut payload = [0u8; GlobalCommand::PAYLOAD_LEN];
        payload.copy_from_slice(&bytes[size_of::<CommandHeaderRaw>()..]);
        GlobalCommand::from_payload(command_type, payload)
    }

    #[test]
    fn test_decode_connector_reset() {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = CommandType::ConnectorReset as u8;
        bytes[2] = 0x81;

        assert_eq!(
            decode(bytes).unwrap(),
            GlobalCommand {
                port: GlobalPortId(1),
                operation: CommandData::ConnectorReset(connector_reset::Args {
                    connector_number: 1,
                    hard_reset: true,
                }),
            }
        );
    }

    #[test]
    fn test_decode_get_connector_status() {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = CommandType::GetConnectorStatus as u8;
        bytes[2] = 0x1;

        assert_eq!(
            decode(bytes).unwrap(),
            GlobalCommand {
                port: GlobalPortId(1),
                operation: CommandData::GetConnectorStatus,
            }
        );
    }

    #[test]
    fn test_decode_get_connector_capability() {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = CommandType::GetConnectorCapability as u8;
        bytes[2] = 0x1;

        assert_eq!(
            decode(bytes).unwrap(),
            GlobalCommand {
                port: GlobalPortId(1),
                operation: CommandData::GetConnectorCapability,
            }
        );
    }

    #[test]
    fn test_decode_set_power_level() {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = CommandType::SetPowerLevel as u8;
        bytes[2] = 0x81;

        assert_eq!(
            decode(bytes).unwrap(),
            GlobalCommand {
                port: GlobalPortId(1),
                operation: CommandData::SetPowerLevel(set_power_level::Args {
                    connector_number: 1,
                    power_role: PowerRole::Source,
                    ..Default::default()
                })
            }
        )
    }

    #[test]
    fn test_decode_set_power_level_invalid_current() {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = CommandType::SetPowerLevel as u8;
        bytes[2] = 0x01;
        // Invalid type_c_current value (0x4) at bits 18:16
        bytes[4] = 0x04;

        assert_eq!(
            decode(bytes),
            Err(InvalidCommand::InvalidCurrent(set_power_level::InvalidCurrent(0x04)))
        );
    }

    #[test]
    fn test_decode_get_alternate_modes() {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = CommandType::GetAlternateModes as u8;
        bytes[2] = 0x1; // SOP recipient

        assert_eq!(
            decode(bytes).unwrap(),
            GlobalCommand {
                port: GlobalPortId(0),
                operation: CommandData::GetAlternateModes(get_alternate_modes::Args {
                    recipient: Recipient::Sop,
                    ..Default::default()
                }),
            }
        );
    }

    #[test]
    fn test_decode_get_alternate_modes_invalid_recipient() {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = CommandType::GetAlternateModes as u8;
        bytes[2] = 0x7; // Invalid recipient

        assert_eq!(
            decode(bytes),
            Err(InvalidCommand::InvalidRecipient(InvalidRecipient(0x7)))
        );
    }

    #[test]
    fn test_decode_set_ccom() {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = CommandType::SetCcom as u8;
        bytes[2] = 0x81;

        assert_eq!(
            decode(bytes).unwrap(),
            GlobalCommand {
                port: GlobalPortId(1),
                operation: CommandData::SetCcom(set_ccom::Args {
                    connector_number: 1,
                    rp: true,
                    ..Default::default()
                }),
            }
        );
    }

    #[test]
    fn test_decode_set_new_cam() {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = CommandType::SetNewCam as u8;
        bytes[2] = 0x1;

        assert_eq!(
            decode(bytes).unwrap(),
            GlobalCommand {
                port: GlobalPortId(1),
                operation: CommandData::SetNewCam(set_new_cam::Args {
                    connector_number: 1,
                    enter: false,
                    am_offset: 0,
                    am_specific: 0,
                }),
            }
        );
    }

    #[test]
    fn test_decode_set_uor() {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = CommandType::SetUor as u8;
        bytes[2] = 0x81;

        assert_eq!(
            decode(bytes).unwrap(),
            GlobalCommand {
                port: GlobalPortId(1),
                operation: CommandData::SetUor(set_uor::Args {
                    connector_number: 1,
                    dfp: true,
                    ..Default::default()
                }),
            }
        );
    }

    #[test]
    fn test_decode_get_error_status() {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = CommandType::GetErrorStatus as u8;
        bytes[2] = 0x1;

        assert_eq!(
            decode(bytes).unwrap(),
            GlobalCommand {
                port: GlobalPortId(1),
                operation: CommandData::GetErrorStatus,
            }
        );
    }

    #[test]
    fn test_decode_set_pdr() {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = CommandType::SetPdr as u8;
        bytes[2] = 0x81;

        assert_eq!(
            decode(bytes).unwrap(),
            GlobalCommand {
                port: GlobalPortId(1),
                operation: CommandData::SetPdr(set_pdr::Args {
                    connector_number: 1,
                    swap_source: true,
                    ..Default::default()
                }),
            }
        );
    }

    #[test]
    fn test_get_cam_supported() {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = CommandType::GetCamSupported as u8;
        bytes[2] = 0x1;

        assert_eq!(
            decode(bytes).unwrap(),
            GlobalCommand {
                port: GlobalPortId(1),
                operation: CommandData::GetCamSupported,
            }
        );
    }

    #[test]
    fn test_get_current_cam() {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = CommandType::GetCurrentCam as u8;
        bytes[2] = 0x1;

        assert_eq!(
            decode(bytes).unwrap(),
            GlobalCommand {
                port: GlobalPortId(1),
                operation: CommandData::GetCurrentCam,
            }
        );
    }

    #[test]
    fn test_get_pdos() {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = CommandType::GetPdos as u8;
        bytes[2] = 0x81;

        assert_eq!(
            decode(bytes).unwrap(),
            GlobalCommand {
                port: GlobalPortId(1),
                operation: CommandData::GetPdos(get_pdos::Args {
                    connector_number: 1,
                    partner: true,
                    ..Default::default()
                }),
            }
        );
    }

    #[test]
    fn test_decode_get_pdos_invalid_source_capability_type() {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = CommandType::GetPdos as u8;
        bytes[2] = 0x1;
        bytes[4] = 0x18; // Source capability type 0x3, bits 20:19

        assert_eq!(
            decode(bytes),
            Err(InvalidCommand::InvalidSourceCapabilityType(
                get_pdos::InvalidSourceCapabilityType(0x3)
            ))
        );
    }

    #[test]
    fn test_encode_get_pdos_invalid_num_pdos() {
        let command = GlobalCommand {
            port: GlobalPortId(1),
            operation: CommandData::GetPdos(get_pdos::Args {
                num_pdos: 0,
                ..Default::default()
            }),
        };

        assert_eq!(
            command.to_payload(),
            Err(InvalidCommand::InvalidNumPdos(get_pdos::InvalidNumPdos(0)))
        );
    }

    #[test]
    fn test_encode_get_pdos_response_variable_length() {
        let response = ResponseData::GetPdos(get_pdos::ResponseData {
            pdos: [0x11223344, 0x55667788, 0, 0],
        });

        let (bytes, len) = response.to_bytes();
        assert_eq!(len, 2 * size_of::<u32>());
        assert_eq!(bytes[..len], [0x44, 0x33, 0x22, 0x11, 0x88, 0x77, 0x66, 0x55]);
    }

    #[test]
    fn test_get_cable_property() {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = CommandType::GetCableProperty as u8;
        bytes[2] = 0x1;

        assert_eq!(
            decode(bytes).unwrap(),
            GlobalCommand {
                port: GlobalPortId(1),
                operation: CommandData::GetCableProperty,
            }
        );
    }

    #[test]
    fn test_get_pd_message() {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = CommandType::GetPdMessage as u8;
        bytes[2] = 0x83;
        bytes[3] = 0x08;
        bytes[4] = 0x01;
        bytes[5] = 0x02;

        assert_eq!(
            decode(bytes).unwrap(),
            GlobalCommand {
                port: GlobalPortId(3),
                operation: CommandData::GetPdMessage(get_pd_message::Args {
                    connector_number: 3,
                    recipient: Recipient::Sop,
                    message_offset: 2,
                    num_bytes: 1,
                    message_type: get_pd_message::MessageType::BatteryCap,
                }),
            }
        );
    }

    #[test]
    fn test_decode_get_pd_message_invalid_recipient() {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = CommandType::GetPdMessage as u8;
        bytes[2] = 0x83;
        bytes[3] = 0x0B; // Recipient 0x7, bits 9:7 cross the byte boundary
        bytes[4] = 0x01;
        bytes[5] = 0x02;

        assert_eq!(
            decode(bytes),
            Err(InvalidCommand::InvalidPdMessageArgs(
                get_pd_message::InvalidArgs::InvalidRecipient(InvalidRecipient(0x7))
            ))
        );
    }

    #[test]
    fn test_decode_get_pd_message_invalid_message_type() {
        let mut bytes = [0u8; COMMAND_LEN];
        bytes[0] = CommandType::GetPdMessage as u8;
        bytes[2] = 0x83;
        bytes[3] = 0x38;
        bytes[4] = 0x01;
        bytes[5] = 0x0f; // Invalid message type

        assert_eq!(
            decode(bytes),
            Err(InvalidCommand::InvalidPdMessageArgs(
                get_pd_message::InvalidArgs::InvalidMessageType(get_pd_message::InvalidMessageType(0x0f))
            ))
        );
    }
}
