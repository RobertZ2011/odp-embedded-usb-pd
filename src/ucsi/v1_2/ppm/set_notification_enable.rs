use bitfield::bitfield;
use bytemuck::{Pod, Zeroable};
use pack1::U16LE;

use crate::ucsi::v1_2::{CommandHeaderRaw, COMMAND_LEN};

bitfield! {
    /// Argument for SET_NOTIFICATION_ENABLE see USCI spec 6.5.5
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub struct NotificationEnableRaw(u16);
    impl Debug;

    /// Notify on command complete
    pub bool, cmd_complete, set_cmd_complete: 0;
    /// Notify on external supply change
    pub bool, external_supply_change, set_external_supply_change: 1;
    /// Notify on power operation mode change
    pub bool, power_op_mode_change, set_power_op_mode_change: 2;
    /// Notify on provider capabilities change
    pub bool, provider_caps_change, set_provider_caps_change: 5;
    /// Notify on power level change
    pub bool, power_lvl_change, set_power_lvl_change: 6;
    /// Notify on PD reset complete
    pub bool, pd_reset_complete, set_pd_reset_complete: 7;
    /// Notify on connector alternate mode change
    pub bool, cam_change, set_cam_change: 8;
    /// Notify on battery charge change
    pub bool, battery_charge_change, set_battery_charge_change: 9;
    /// Notify on connector partner change
    pub bool, connector_partner_change, set_connector_partner_change: 11;
    /// Notify on power direction change
    pub bool, power_dir_change, set_power_dir_change: 12;
    /// Notify on connect change
    pub bool, connect_change, set_connect_change: 14;
    /// Notify on error
    pub bool, error, set_error: 15;
}

#[cfg(feature = "defmt")]
impl defmt::Format for NotificationEnableRaw {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "NotificationEnableRaw {{ .0: {}, \
            cmd_complete: {}, \
            external_supply_change: {}, \
            power_op_mode_change: {}, \
            provider_caps_change: {}, \
            power_lvl_change: {}, \
            pd_reset_complete: {}, \
            cam_change: {}, \
            battery_charge_change: {}, \
            connector_partner_change: {}, \
            power_dir_change: {}, \
            connect_change: {}, \
            error: {} }}",
            self.0,
            self.cmd_complete(),
            self.external_supply_change(),
            self.power_op_mode_change(),
            self.provider_caps_change(),
            self.power_lvl_change(),
            self.pd_reset_complete(),
            self.cam_change(),
            self.battery_charge_change(),
            self.connector_partner_change(),
            self.power_dir_change(),
            self.connect_change(),
            self.error()
        )
    }
}

/// Higher-level representation of [`NotificationEnableRaw`]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NotificationEnable {
    /// Notify on command complete
    pub cmd_complete: bool,
    /// Notify on external supply change
    pub external_supply_change: bool,
    /// Notify on power operation mode change
    pub power_op_mode_change: bool,
    /// Notify on provider capabilities change
    pub provider_caps_change: bool,
    /// Notify on power level change
    pub power_lvl_change: bool,
    /// Notify on PD reset complete
    pub pd_reset_complete: bool,
    /// Notify on connector alternate mode change
    pub cam_change: bool,
    /// Notify on battery charge change
    pub battery_charge_change: bool,
    /// Notify on connector partner change
    pub connector_partner_change: bool,
    /// Notify on power direction change
    pub power_dir_change: bool,
    /// Notify on connect change
    pub connect_change: bool,
    /// Notify on error
    pub error: bool,
}

impl NotificationEnable {
    /// Returns true if no notification is enabled
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// Returns true if any status change flags are set
    pub fn any(&self) -> bool {
        !self.is_empty()
    }

    /// Returns the union of two notification enable sets
    pub fn union(&self, other: &Self) -> Self {
        Self::from(u16::from(*self) | u16::from(*other))
    }

    /// Returns the intersection of two notification enable sets
    pub fn intersection(&self, other: &Self) -> Self {
        Self::from(u16::from(*self) & u16::from(*other))
    }
}

impl From<NotificationEnableRaw> for NotificationEnable {
    fn from(raw: NotificationEnableRaw) -> Self {
        Self {
            cmd_complete: raw.cmd_complete(),
            external_supply_change: raw.external_supply_change(),
            power_op_mode_change: raw.power_op_mode_change(),
            provider_caps_change: raw.provider_caps_change(),
            power_lvl_change: raw.power_lvl_change(),
            pd_reset_complete: raw.pd_reset_complete(),
            cam_change: raw.cam_change(),
            battery_charge_change: raw.battery_charge_change(),
            connector_partner_change: raw.connector_partner_change(),
            power_dir_change: raw.power_dir_change(),
            connect_change: raw.connect_change(),
            error: raw.error(),
        }
    }
}

impl From<NotificationEnable> for NotificationEnableRaw {
    fn from(enable: NotificationEnable) -> Self {
        let mut raw = NotificationEnableRaw(0);
        raw.set_cmd_complete(enable.cmd_complete);
        raw.set_external_supply_change(enable.external_supply_change);
        raw.set_power_op_mode_change(enable.power_op_mode_change);
        raw.set_provider_caps_change(enable.provider_caps_change);
        raw.set_power_lvl_change(enable.power_lvl_change);
        raw.set_pd_reset_complete(enable.pd_reset_complete);
        raw.set_cam_change(enable.cam_change);
        raw.set_battery_charge_change(enable.battery_charge_change);
        raw.set_connector_partner_change(enable.connector_partner_change);
        raw.set_power_dir_change(enable.power_dir_change);
        raw.set_connect_change(enable.connect_change);
        raw.set_error(enable.error);
        raw
    }
}

impl From<u16> for NotificationEnable {
    fn from(raw: u16) -> Self {
        NotificationEnableRaw(raw).into()
    }
}

impl From<NotificationEnable> for u16 {
    fn from(enable: NotificationEnable) -> Self {
        NotificationEnableRaw::from(enable).0
    }
}

/// Set notification enable command
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Args {
    /// Notification enable flags
    pub notification_enable: NotificationEnable,
}

/// Data length for the SET_NOTIFICATION_ENABLE command response
pub const RESPONSE_DATA_LEN: u8 = 0;
/// Command padding
pub const COMMAND_PADDING: usize = COMMAND_LEN - size_of::<CommandHeaderRaw>() - size_of::<NotificationEnableRaw>();

/// Raw wire format of [`Args`]
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Zeroable, Pod)]
pub struct ArgsRaw {
    /// Notification enable flags
    pub notification_enable: U16LE,
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
        defmt::write!(
            fmt,
            "ArgsRaw {{ notification_enable: {} }}",
            NotificationEnableRaw(self.notification_enable.get())
        )
    }
}

impl From<Args> for ArgsRaw {
    fn from(args: Args) -> Self {
        Self {
            notification_enable: U16LE::new(args.notification_enable.into()),
            ..Default::default()
        }
    }
}

impl From<ArgsRaw> for Args {
    fn from(raw: ArgsRaw) -> Self {
        Self {
            notification_enable: raw.notification_enable.get().into(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Mask of all bits defined by [`NotificationEnableRaw`]
    const DEFINED_BITS: u16 = 0b1101_1011_1110_0111;

    #[test]
    fn test_raw_len() {
        assert_eq!(ArgsRaw::LEN, COMMAND_LEN - size_of::<CommandHeaderRaw>());
    }

    #[test]
    fn test_notification_enable_roundtrip() {
        for bit in 0..u16::BITS {
            let raw = NotificationEnableRaw(1 << bit);
            // Undefined bits are dropped by the roundtrip
            let expected = NotificationEnableRaw(raw.0 & DEFINED_BITS);
            assert_eq!(NotificationEnableRaw::from(NotificationEnable::from(raw)), expected);
        }

        assert_eq!(u16::from(NotificationEnable::from(DEFINED_BITS)), DEFINED_BITS);
    }

    #[test]
    fn test_notification_enable_set_ops() {
        let empty = NotificationEnable::default();
        assert!(empty.is_empty());
        assert!(!empty.any());

        let cmd_complete = NotificationEnable {
            cmd_complete: true,
            ..Default::default()
        };
        let error = NotificationEnable {
            error: true,
            ..Default::default()
        };

        assert!(cmd_complete.any());
        assert_eq!(
            cmd_complete.union(&error),
            NotificationEnable {
                cmd_complete: true,
                error: true,
                ..Default::default()
            }
        );
        assert_eq!(cmd_complete.intersection(&error), empty);
        assert_eq!(cmd_complete.intersection(&cmd_complete), cmd_complete);
    }

    #[test]
    fn test_args_raw_roundtrip() {
        let args = Args {
            notification_enable: NotificationEnable {
                cmd_complete: true,
                error: true,
                ..Default::default()
            },
        };

        let mut expected = [0u8; ArgsRaw::LEN];
        expected[0] = 0x01;
        expected[1] = 0x80;

        let bytes: [u8; ArgsRaw::LEN] = bytemuck::must_cast(ArgsRaw::from(args));
        assert_eq!(bytes, expected);
        assert_eq!(Args::from(bytemuck::must_cast::<_, ArgsRaw>(expected)), args);
    }
}
