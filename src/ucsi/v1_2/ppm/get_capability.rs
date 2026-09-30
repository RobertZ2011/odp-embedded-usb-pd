//! Types for the `GetCapability` command, see USCI spec 6.5

use bitfield::bitfield;
use bytemuck::{Pod, Zeroable};
use pack1::{U16LE, U32LE};

use crate::ucsi::v1_2::{CommandHeaderRaw, COMMAND_LEN};

/// Data length for the GET_CAPABILITY command response
pub const RESPONSE_DATA_LEN: usize = 16;
/// Command padding
pub const COMMAND_PADDING: usize = COMMAND_LEN - size_of::<CommandHeaderRaw>();

/// GetCapability command
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Args;

/// Raw wire format of [`Args`]
///
/// GET_CAPABILITY takes no arguments, the entire payload is reserved.
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

bitfield! {
    /// Optional features bitmap for GetCapability command
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub struct OptionalFeaturesRaw(u32);
    impl Debug;

    /// Supports SET_CCOM
    pub bool, set_ccom_supported, set_set_ccom_supported: 0;
    /// Supports SET_POWER_LEVEL
    pub bool, set_power_level_supported, set_set_power_level_supported: 1;
    /// Supports alternate mode details
    pub bool, altmode_details_supported, set_altmode_details_supported: 2;
    /// Supports alternate mode override
    pub bool, altmode_override_supported, set_altmode_override_supported: 3;
    /// Supports power data object details
    pub bool, pdo_details_supported, set_pdo_details_supported: 4;
    /// Supports cable details
    pub bool, cable_details_supported, set_cable_details_supported: 5;
    /// Supports external supply notification
    pub bool, external_supply_notif_supported, set_external_supply_notif_supported: 6;
    /// Supports PD reset notification
    pub bool, pd_reset_notif_supported, set_pd_reset_notif_supported: 7;
    /// Supports GET_PD_MESSAGE
    pub bool, get_pd_msg_supported, set_get_pd_msg_supported: 8;
}

#[cfg(feature = "defmt")]
impl defmt::Format for OptionalFeaturesRaw {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "OptionalFeaturesRaw {{ .0: {}, set_ccom_supported: {}, set_power_level_supported: {}, altmode_details_supported: {}, altmode_override_supported: {}, pdo_details_supported: {}, cable_details_supported: {}, external_supply_notif_supported: {}, pd_reset_notif_supported: {}, get_pd_msg_supported: {} }}",
            self.0,
            self.set_ccom_supported(),
            self.set_power_level_supported(),
            self.altmode_details_supported(),
            self.altmode_override_supported(),
            self.pdo_details_supported(),
            self.cable_details_supported(),
            self.external_supply_notif_supported(),
            self.pd_reset_notif_supported(),
            self.get_pd_msg_supported()
        )
    }
}

/// Higher-level representation of [`OptionalFeaturesRaw`]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OptionalFeatures {
    /// Supports SET_CCOM
    pub set_ccom_supported: bool,
    /// Supports SET_POWER_LEVEL
    pub set_power_level_supported: bool,
    /// Supports alternate mode details
    pub altmode_details_supported: bool,
    /// Supports alternate mode override
    pub altmode_override_supported: bool,
    /// Supports power data object details
    pub pdo_details_supported: bool,
    /// Supports cable details
    pub cable_details_supported: bool,
    /// Supports external supply notification
    pub external_supply_notif_supported: bool,
    /// Supports PD reset notification
    pub pd_reset_notif_supported: bool,
    /// Supports GET_PD_MESSAGE
    pub get_pd_msg_supported: bool,
}

impl From<OptionalFeaturesRaw> for OptionalFeatures {
    fn from(raw: OptionalFeaturesRaw) -> Self {
        Self {
            set_ccom_supported: raw.set_ccom_supported(),
            set_power_level_supported: raw.set_power_level_supported(),
            altmode_details_supported: raw.altmode_details_supported(),
            altmode_override_supported: raw.altmode_override_supported(),
            pdo_details_supported: raw.pdo_details_supported(),
            cable_details_supported: raw.cable_details_supported(),
            external_supply_notif_supported: raw.external_supply_notif_supported(),
            pd_reset_notif_supported: raw.pd_reset_notif_supported(),
            get_pd_msg_supported: raw.get_pd_msg_supported(),
        }
    }
}

impl From<OptionalFeatures> for OptionalFeaturesRaw {
    fn from(features: OptionalFeatures) -> Self {
        let mut raw = OptionalFeaturesRaw(0);
        raw.set_set_ccom_supported(features.set_ccom_supported);
        raw.set_set_power_level_supported(features.set_power_level_supported);
        raw.set_altmode_details_supported(features.altmode_details_supported);
        raw.set_altmode_override_supported(features.altmode_override_supported);
        raw.set_pdo_details_supported(features.pdo_details_supported);
        raw.set_cable_details_supported(features.cable_details_supported);
        raw.set_external_supply_notif_supported(features.external_supply_notif_supported);
        raw.set_pd_reset_notif_supported(features.pd_reset_notif_supported);
        raw.set_get_pd_msg_supported(features.get_pd_msg_supported);
        raw
    }
}

impl From<u32> for OptionalFeatures {
    fn from(raw: u32) -> Self {
        OptionalFeaturesRaw(raw).into()
    }
}

impl From<OptionalFeatures> for u32 {
    fn from(features: OptionalFeatures) -> Self {
        OptionalFeaturesRaw::from(features).0
    }
}

bitfield! {
    /// Raw power source data for GetCapability command
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub struct PowerSourceRaw(u8);
    impl Debug;

    /// AC supply supported
    pub bool, ac_supply, set_ac_supply: 0;
    /// Other supply supported
    pub bool, other, set_other: 2;
    /// Uses VBUS
    pub bool, use_vbus, set_uses_vbus: 6;
}

#[cfg(feature = "defmt")]
impl defmt::Format for PowerSourceRaw {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "PowerSourceRaw {{ .0: {}, ac_supply: {}, other: {}, use_vbus: {} }}",
            self.0,
            self.ac_supply(),
            self.other(),
            self.use_vbus()
        )
    }
}

/// Higher-level representation of [`PowerSourceRaw`]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PowerSource {
    /// AC supply supported
    pub ac_supply: bool,
    /// Other supply supported
    pub other: bool,
    /// Uses VBUS
    pub use_vbus: bool,
}

impl From<PowerSourceRaw> for PowerSource {
    fn from(raw: PowerSourceRaw) -> Self {
        Self {
            ac_supply: raw.ac_supply(),
            other: raw.other(),
            use_vbus: raw.use_vbus(),
        }
    }
}

impl From<PowerSource> for PowerSourceRaw {
    fn from(power_source: PowerSource) -> Self {
        let mut raw = PowerSourceRaw(0);
        raw.set_ac_supply(power_source.ac_supply);
        raw.set_other(power_source.other);
        raw.set_uses_vbus(power_source.use_vbus);
        raw
    }
}

impl From<u8> for PowerSource {
    fn from(raw: u8) -> Self {
        PowerSourceRaw(raw).into()
    }
}

impl From<PowerSource> for u8 {
    fn from(power_source: PowerSource) -> Self {
        PowerSourceRaw::from(power_source).0
    }
}

bitfield! {
    /// Raw attribute data for GetCapability command
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub struct AttributesRaw(u32);
    impl Debug;

    /// Supports disabled state as defined in Type-C spec
    pub bool, disabled_state_support, set_disabled_state_support: 0;
    /// PPM supports battery charging spec with version given in [`ResponseDataRaw::bcd_battery_charging_spec`]
    pub bool, battery_charging, set_battery_charging: 1;
    /// PPM supports USB PD spec with version given in [`ResponseDataRaw::bcd_usb_pd_spec`]
    pub bool, usb_power_delivery, set_usb_power_delivery: 2;
    /// PPM supports USB Type-C spec with version given in [`ResponseDataRaw::bcd_type_c_spec`]
    pub bool, usb_type_c_current, set_usb_type_c_current: 6;
    /// Supported power sources bitmap
    pub u8, bm_power_source, set_bm_power_source: 15, 8;
}

#[cfg(feature = "defmt")]
impl defmt::Format for AttributesRaw {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "AttributesRaw {{ .0: {}, \
            disabled_state_support: {}, \
            battery_charging: {}, \
            usb_power_delivery: {}, \
            usb_type_c_current: {}, \
            bm_power_source: {} }}",
            self.0,
            self.disabled_state_support(),
            self.battery_charging(),
            self.usb_power_delivery(),
            self.usb_type_c_current(),
            self.bm_power_source()
        )
    }
}

/// Higher-level representation of [`AttributesRaw`]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Attributes {
    /// Supports disabled state as defined in Type-C spec
    pub disabled_state_support: bool,
    /// PPM supports battery charging spec with version given in [`ResponseData::bcd_battery_charging_spec`]
    pub battery_charging: bool,
    /// PPM supports USB PD spec with version given in [`ResponseData::bcd_usb_pd_spec`]
    pub usb_power_delivery: bool,
    /// PPM supports USB Type-C spec with version given in [`ResponseData::bcd_type_c_spec`]
    pub usb_type_c_current: bool,
    /// Supported power sources
    pub power_source: PowerSource,
}

impl From<AttributesRaw> for Attributes {
    fn from(raw: AttributesRaw) -> Self {
        Self {
            disabled_state_support: raw.disabled_state_support(),
            battery_charging: raw.battery_charging(),
            usb_power_delivery: raw.usb_power_delivery(),
            usb_type_c_current: raw.usb_type_c_current(),
            power_source: raw.bm_power_source().into(),
        }
    }
}

impl From<Attributes> for AttributesRaw {
    fn from(attributes: Attributes) -> Self {
        let mut raw = AttributesRaw(0);
        raw.set_disabled_state_support(attributes.disabled_state_support);
        raw.set_battery_charging(attributes.battery_charging);
        raw.set_usb_power_delivery(attributes.usb_power_delivery);
        raw.set_usb_type_c_current(attributes.usb_type_c_current);
        raw.set_bm_power_source(attributes.power_source.into());
        raw
    }
}

impl From<u32> for Attributes {
    fn from(raw: u32) -> Self {
        AttributesRaw(raw).into()
    }
}

impl From<Attributes> for u32 {
    fn from(attributes: Attributes) -> Self {
        AttributesRaw::from(attributes).0
    }
}

/// Get capability response data
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ResponseData {
    /// Attributes
    pub attributes: Attributes,
    /// Number of connectors
    pub num_connectors: u8,
    /// Optional features
    pub optional_features: OptionalFeatures,
    /// Number of supported alternate modes
    pub num_alt_modes: u8,
    /// BCD coded battery charging spec version
    pub bcd_battery_charging_spec: u16,
    /// BCD coded USB PD spec version
    pub bcd_usb_pd_spec: u16,
    /// BCD coded Type-C spec version
    pub bcd_type_c_spec: u16,
}

/// Raw wire format of [`ResponseData`]
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Zeroable, Pod)]
pub struct ResponseDataRaw {
    /// Attributes
    pub attributes: U32LE,
    /// Number of connectors
    pub num_connectors: u8,
    /// Optional features, a 24-bit little-endian bitmap
    pub optional_features: [u8; 3],
    /// Number of supported alternate modes
    pub num_alt_modes: u8,
    /// Reserved byte
    _reserved: u8,
    /// BCD coded battery charging spec version
    pub bcd_battery_charging_spec: U16LE,
    /// BCD coded USB PD spec version
    pub bcd_usb_pd_spec: U16LE,
    /// BCD coded Type-C spec version
    pub bcd_type_c_spec: U16LE,
}

impl ResponseDataRaw {
    /// Length of the raw response data in bytes
    pub const LEN: usize = size_of::<Self>();

    /// Returns the optional features bitmap
    ///
    /// The field is only 24 bits wide, so it is stored as a byte array rather than a `pack1` type.
    const fn optional_features_bits(&self) -> u32 {
        let [low, mid, high] = self.optional_features;
        u32::from_le_bytes([low, mid, high, 0])
    }

    /// Truncates an optional features bitmap to the 24 bits available on the wire
    const fn optional_features_from_bits(bits: u32) -> [u8; 3] {
        let [low, mid, high, _] = bits.to_le_bytes();
        [low, mid, high]
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ResponseDataRaw {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "ResponseDataRaw {{ \
            attributes: {}, \
            num_connectors: {}, \
            optional_features: {}, \
            num_alt_modes: {}, \
            bcd_battery_charging_spec: {}, \
            bcd_usb_pd_spec: {}, \
            bcd_type_c_spec: {} }}",
            AttributesRaw(self.attributes.get()),
            self.num_connectors,
            OptionalFeaturesRaw(self.optional_features_bits()),
            self.num_alt_modes,
            self.bcd_battery_charging_spec.get(),
            self.bcd_usb_pd_spec.get(),
            self.bcd_type_c_spec.get()
        )
    }
}

impl From<ResponseData> for ResponseDataRaw {
    fn from(data: ResponseData) -> Self {
        Self {
            attributes: U32LE::new(data.attributes.into()),
            num_connectors: data.num_connectors,
            optional_features: Self::optional_features_from_bits(data.optional_features.into()),
            num_alt_modes: data.num_alt_modes,
            _reserved: 0,
            bcd_battery_charging_spec: U16LE::new(data.bcd_battery_charging_spec),
            bcd_usb_pd_spec: U16LE::new(data.bcd_usb_pd_spec),
            bcd_type_c_spec: U16LE::new(data.bcd_type_c_spec),
        }
    }
}

impl From<ResponseDataRaw> for ResponseData {
    fn from(raw: ResponseDataRaw) -> Self {
        Self {
            attributes: raw.attributes.get().into(),
            num_connectors: raw.num_connectors,
            optional_features: raw.optional_features_bits().into(),
            num_alt_modes: raw.num_alt_modes,
            bcd_battery_charging_spec: raw.bcd_battery_charging_spec.get(),
            bcd_usb_pd_spec: raw.bcd_usb_pd_spec.get(),
            bcd_type_c_spec: raw.bcd_type_c_spec.get(),
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    /// Create a standard response data value for testing
    pub fn create_response_data() -> (ResponseData, [u8; RESPONSE_DATA_LEN]) {
        let response_data = ResponseData {
            attributes: Attributes::from(0x43),
            num_connectors: 2,
            optional_features: OptionalFeatures::from(0xFF),
            num_alt_modes: 3,
            bcd_battery_charging_spec: 0x0120,
            bcd_usb_pd_spec: 0x0300,
            bcd_type_c_spec: 0x0200,
        };

        let mut bytes = [0u8; RESPONSE_DATA_LEN];

        // Attributes - 4 bytes
        // Disable state support + Battery charging + USB PD + USB Type-C
        bytes[0] = 0x43;

        // Num connectors - 1 byte
        bytes[4] = 2;

        // Optional features - 3 bytes
        bytes[5] = 0xFF; // Support everything

        // Number of support alt modes
        bytes[8] = 3; // Let's just say 3

        // bcdBCVersion - 2 bytes
        // 1.20
        bytes[10] = 0x20;
        bytes[11] = 0x01;

        // bcdPDVersion - 2 bytes
        // 3.00
        bytes[12] = 0x00;
        bytes[13] = 0x03;

        // bcdUSBTypeCVersion - 2 bytes
        // 2.00
        bytes[14] = 0x00;
        bytes[15] = 0x02;

        (response_data, bytes)
    }

    #[test]
    fn test_raw_len() {
        assert_eq!(ArgsRaw::LEN, COMMAND_PADDING);
        assert_eq!(ResponseDataRaw::LEN, RESPONSE_DATA_LEN);
    }

    #[test]
    fn test_response_data_raw_roundtrip() {
        let (expected, bytes) = create_response_data();

        let encoded: [u8; ResponseDataRaw::LEN] = bytemuck::must_cast(ResponseDataRaw::from(expected));
        assert_eq!(encoded, bytes);
        assert_eq!(
            ResponseData::from(bytemuck::must_cast::<_, ResponseDataRaw>(bytes)),
            expected
        );
    }

    #[test]
    fn test_optional_features_24_bit_field() {
        // The wire field is only 24 bits wide, bit 24 and above cannot be represented
        assert_eq!(
            ResponseDataRaw::optional_features_from_bits(0xFFFF_FFFF),
            [0xFF, 0xFF, 0xFF]
        );

        // All nine defined features occupy the low nine bits
        let raw = ResponseDataRaw::from(ResponseData {
            optional_features: OptionalFeatures {
                set_ccom_supported: true,
                set_power_level_supported: true,
                altmode_details_supported: true,
                altmode_override_supported: true,
                pdo_details_supported: true,
                cable_details_supported: true,
                external_supply_notif_supported: true,
                pd_reset_notif_supported: true,
                get_pd_msg_supported: true,
            },
            ..Default::default()
        });
        assert_eq!(raw.optional_features, [0xFF, 0x01, 0x00]);
        assert_eq!(raw.optional_features_bits(), 0x1FF);
    }

    #[test]
    fn test_power_source_roundtrip() {
        let attributes = Attributes {
            usb_power_delivery: true,
            power_source: PowerSource {
                ac_supply: true,
                use_vbus: true,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(Attributes::from(u32::from(attributes)), attributes);
        assert_eq!(
            u32::from(attributes) >> 8 & 0xFF,
            u8::from(attributes.power_source) as u32
        );
    }
}
