//! COSEM Interface Class 4: Extended Register
//!
//! This module implements the Extended Register interface class as defined in the DLMS/COSEM Blue Book.
//!
//! The Extended Register class extends the basic Register with status and capture time information,
//! typically used for values that need to be timestamped or have status flags.
//!
//! ## Attributes
//! - Attribute 1: `logical_name` (inherited) - OBIS code identifying the object
//! - Attribute 2: `value` - The register value (numeric Data type)
//! - Attribute 3: `scaler_unit` - Scaler and physical unit (Structure with 2 elements)
//! - Attribute 4: `status` - Status information (OctetString, BitString, or Null)
//! - Attribute 5: `capture_time` - Timestamp when the value was captured
//!
//! ## Methods
//! - Method 1: `reset(data)` - Reset the register value to a default value
//!
//! # Example
//! ```
//! use dlms_cosem::cosem::extended_register::ExtendedRegister;
//! use dlms_cosem::cosem::CosemObject;
//! use dlms_cosem::{ObisCode, Data, ScalerUnit, Unit, DateTime, Date, Time};
//!
//! let mut ext_register = ExtendedRegister::new(
//!     ObisCode::new(1, 0, 32, 7, 0, 255),  // Instantaneous voltage L1
//!     Data::LongUnsigned(23050),            // Raw value: 230.50V
//!     ScalerUnit { scaler: -2, unit: Unit::Volt },
//!     Data::Null,                           // No status
//!     DateTime {
//!         date: Date { year: 2025, month: 1, day_of_month: 25, day_of_week: 6 },
//!         time: Time { hour: Some(14), minute: Some(30), second: Some(0), hundredth: Some(0) },
//!         offset_minutes: Some(-60),
//!         clock_status: Some(ClockStatus(0x00)),
//!     },
//! );
//!
//! assert_eq!(ext_register.class_id(), 4);
//! assert_eq!(ext_register.version(), 0);
//! ```

use crate::cosem::CosemObject;
use crate::data::{Data, Date, DateTime, Time};
use crate::get::DataAccessResult;
use crate::obis_code::ObisCode;
use crate::unit::ScalerUnit;

#[cfg(feature = "encode")]
use crate::action::ActionResult;

/// Extended Register object - COSEM Interface Class 4
///
/// Represents a metered value with scaler, unit, status, and capture timestamp.
/// The scaler is applied as: actual_value = raw_value × 10^scaler
///
/// Reference: Blue Book 4.4.1
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExtendedRegister {
    /// Attribute 1: Logical name (OBIS code)
    pub logical_name: ObisCode,
    /// Attribute 2: Current value of the register (must be numeric)
    pub value: Data,
    /// Attribute 3: Scaler and unit
    pub scaler_unit: ScalerUnit,
    /// Attribute 4: Status information (OctetString, BitString, or Null)
    pub status: Data,
    /// Attribute 5: Timestamp when value was captured
    pub capture_time: DateTime,
}

impl ExtendedRegister {
    /// Create a new ExtendedRegister object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this register
    /// * `value` - Initial value (should be numeric Data type)
    /// * `scaler_unit` - Scaler and physical unit
    /// * `status` - Status information (OctetString, BitString, or Null)
    /// * `capture_time` - Timestamp when value was captured
    ///
    /// # Example
    /// ```
    /// use dlms_cosem::cosem::extended_register::ExtendedRegister;
    /// use dlms_cosem::{ObisCode, Data, ScalerUnit, Unit, DateTime, Date, Time};
    ///
    /// let ext_register = ExtendedRegister::new(
    ///     ObisCode::new(1, 0, 32, 7, 0, 255),
    ///     Data::LongUnsigned(23000),
    ///     ScalerUnit { scaler: -2, unit: Unit::Volt },
    ///     Data::OctetString(vec![0x01]),
    ///     DateTime {
    ///         date: Date { year: 2025, month: 1, day_of_month: 25, day_of_week: 6 },
    ///         time: Time { hour: Some(12), minute: Some(0), second: Some(0), hundredth: Some(0) },
    ///         offset_minutes: Some(0),
    ///         clock_status: Some(ClockStatus(0x00)),
    ///     },
    /// );
    /// ```
    pub fn new(
        logical_name: ObisCode,
        value: Data,
        scaler_unit: ScalerUnit,
        status: Data,
        capture_time: DateTime,
    ) -> Self {
        Self { logical_name, value, scaler_unit, status, capture_time }
    }

    /// Get the scaled value as a floating-point number
    ///
    /// Applies the scaler to the raw value: actual_value = raw_value × 10^scaler
    ///
    /// # Returns
    /// - The scaled value as f64
    /// - Returns 0.0 if the value is not numeric
    ///
    /// # Example
    /// ```
    /// use dlms_cosem::cosem::extended_register::ExtendedRegister;
    /// use dlms_cosem::{ObisCode, Data, ScalerUnit, Unit, DateTime, Date, Time};
    ///
    /// let ext_register = ExtendedRegister::new(
    ///     ObisCode::new(1, 0, 32, 7, 0, 255),
    ///     Data::LongUnsigned(23050),
    ///     ScalerUnit { scaler: -2, unit: Unit::Volt },
    ///     Data::Null,
    ///     DateTime {
    ///         date: Date { year: 2025, month: 1, day_of_month: 25, day_of_week: 6 },
    ///         time: Time { hour: Some(12), minute: Some(0), second: Some(0), hundredth: Some(0) },
    ///         offset_minutes: Some(0),
    ///         clock_status: Some(ClockStatus(0x00)),
    ///     },
    /// );
    ///
    /// assert_eq!(ext_register.scaled_value(), 230.50);  // 23050 * 10^-2 = 230.50 V
    /// ```
    pub fn scaled_value(&self) -> f64 {
        let scaler_multiplier = 10f64.powi(self.scaler_unit.scaler as i32);
        match &self.value {
            Data::Integer(v) => (*v as f64) * scaler_multiplier,
            Data::Unsigned(v) => (*v as f64) * scaler_multiplier,
            Data::Long(v) => (*v as f64) * scaler_multiplier,
            Data::LongUnsigned(v) => (*v as f64) * scaler_multiplier,
            Data::DoubleLong(v) => (*v as f64) * scaler_multiplier,
            Data::DoubleLongUnsigned(v) => (*v as f64) * scaler_multiplier,
            Data::Long64(v) => (*v as f64) * scaler_multiplier,
            Data::Long64Unsigned(v) => (*v as f64) * scaler_multiplier,
            Data::Float32(v) => (*v as f64) * scaler_multiplier,
            Data::Float64(v) => *v * scaler_multiplier,
            _ => 0.0, // Non-numeric types return 0
        }
    }
}

impl CosemObject for ExtendedRegister {
    fn class_id(&self) -> u16 {
        4
    }

    fn version(&self) -> u8 {
        0
    }

    fn logical_name(&self) -> &ObisCode {
        &self.logical_name
    }

    fn get_attribute(&self, attribute_id: i8) -> Result<Data, DataAccessResult> {
        match attribute_id {
            1 => Ok(Data::OctetString(self.logical_name.encode().to_vec())),
            2 => Ok(self.value.clone()),
            3 => {
                // Return scaler_unit as Structure(Integer, Enum)
                Ok(Data::Structure(vec![
                    Data::Integer(self.scaler_unit.scaler),
                    Data::Enum(self.scaler_unit.unit.as_i8() as u8),
                ]))
            }
            4 => Ok(self.status.clone()),
            5 => Ok(Data::DateTime(self.capture_time.clone())),
            _ => Err(DataAccessResult::ObjectUndefined),
        }
    }

    fn set_attribute(&mut self, attribute_id: i8, value: Data) -> Result<(), DataAccessResult> {
        match attribute_id {
            1 => {
                // logical_name is read-only
                Err(DataAccessResult::ReadWriteDenied)
            }
            2 => {
                // Validate that value is numeric
                if value.is_numeric() {
                    self.value = value;
                    Ok(())
                } else {
                    Err(DataAccessResult::TypeUnmatched)
                }
            }
            3 => {
                // Parse scaler_unit from Structure(Integer, Enum)
                if let Data::Structure(elements) = value {
                    if elements.len() == 2 {
                        if let (Data::Integer(scaler), Data::Enum(unit_val)) =
                            (&elements[0], &elements[1])
                        {
                            self.scaler_unit.scaler = *scaler;
                            if let Ok(unit) = crate::unit::Unit::try_from(*unit_val) {
                                self.scaler_unit.unit = unit;
                            } else {
                                return Err(DataAccessResult::TypeUnmatched);
                            }
                            return Ok(());
                        }
                    }
                }
                Err(DataAccessResult::TypeUnmatched)
            }
            4 => {
                // Status can be any data type per DLMS spec (matches Gurux behavior)
                self.status = value;
                Ok(())
            }
            5 => {
                // Validate capture_time is DateTime
                if let Data::DateTime(dt) = value {
                    self.capture_time = dt;
                    Ok(())
                } else {
                    Err(DataAccessResult::TypeUnmatched)
                }
            }
            _ => Err(DataAccessResult::ObjectUndefined),
        }
    }

    #[cfg(feature = "encode")]
    fn invoke_method(
        &mut self,
        method_id: i8,
        parameters: Option<Data>,
    ) -> Result<Option<Data>, ActionResult> {
        match method_id {
            1 => {
                // Method 1: reset(data) - reset value to provided value or default
                let reset_value = parameters.unwrap_or(Data::DoubleLongUnsigned(0));
                if reset_value.is_numeric() {
                    self.value = reset_value;
                    // Update capture_time to reflect the reset (matches Gurux behavior)
                    // Use wildcard/cleared DateTime (all fields unspecified)
                    self.capture_time = DateTime {
                        date: Date {
                            year: 0xFFFF,
                            month: 0xFF,
                            day_of_month: 0xFF,
                            day_of_week: 0xFF,
                        },
                        time: Time { hour: None, minute: None, second: None, hundredth: None },
                        offset_minutes: None,
                        clock_status: None,
                    };
                    Ok(None) // No return value
                } else {
                    Err(ActionResult::TypeUnmatched)
                }
            }
            _ => Err(ActionResult::ObjectUndefined),
        }
    }

    #[cfg(not(feature = "encode"))]
    fn invoke_method(
        &mut self,
        _method_id: i8,
        _parameters: Option<Data>,
    ) -> Result<Option<Data>, ActionResult> {
        Err(ActionResult::ObjectUndefined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{ClockStatus, Date, Time};
    use crate::unit::Unit;

    fn test_datetime() -> DateTime {
        DateTime {
            date: Date { year: 2025, month: 1, day_of_month: 25, day_of_week: 6 },
            time: Time { hour: Some(14), minute: Some(30), second: Some(0), hundredth: Some(0) },
            offset_minutes: Some(-60),
            clock_status: Some(ClockStatus(0x00)),
        }
    }

    #[test]
    fn test_extended_register_new() {
        let ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),
            Data::LongUnsigned(23050),
            ScalerUnit { scaler: -2, unit: Unit::Volt },
            Data::OctetString(vec![0x01]),
            test_datetime(),
        );

        assert_eq!(ext_reg.logical_name, ObisCode::new(1, 0, 32, 7, 0, 255));
        assert_eq!(ext_reg.value, Data::LongUnsigned(23050));
        assert_eq!(ext_reg.scaler_unit.scaler, -2);
        assert_eq!(ext_reg.scaler_unit.unit, Unit::Volt);
        assert_eq!(ext_reg.status, Data::OctetString(vec![0x01]));
        assert_eq!(ext_reg.capture_time, test_datetime());
    }

    #[test]
    fn test_extended_register_class_id() {
        let ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(0),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );
        assert_eq!(ext_reg.class_id(), 4);
    }

    #[test]
    fn test_extended_register_version() {
        let ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(0),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );
        assert_eq!(ext_reg.version(), 0);
    }

    #[test]
    fn test_extended_register_logical_name() {
        let obis = ObisCode::new(1, 0, 32, 7, 0, 255);
        let ext_reg = ExtendedRegister::new(
            obis,
            Data::LongUnsigned(0),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );
        assert_eq!(ext_reg.logical_name(), &obis);
    }

    #[test]
    fn test_scaled_value_positive_scaler() {
        let ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::Long(123),
            ScalerUnit { scaler: 2, unit: Unit::WattHour },
            Data::Null,
            test_datetime(),
        );
        assert_eq!(ext_reg.scaled_value(), 12300.0);
    }

    #[test]
    fn test_scaled_value_negative_scaler() {
        let ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),
            Data::LongUnsigned(23050),
            ScalerUnit { scaler: -2, unit: Unit::Volt },
            Data::Null,
            test_datetime(),
        );
        assert_eq!(ext_reg.scaled_value(), 230.50);
    }

    #[test]
    fn test_scaled_value_zero_scaler() {
        let ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 31, 7, 0, 255),
            Data::LongUnsigned(1500),
            ScalerUnit { scaler: 0, unit: Unit::Ampere },
            Data::Null,
            test_datetime(),
        );
        assert_eq!(ext_reg.scaled_value(), 1500.0);
    }

    #[test]
    fn test_scaled_value_different_numeric_types() {
        let test_cases = vec![
            (Data::Integer(-10), -10.0),
            (Data::Unsigned(20), 20.0),
            (Data::Long(-1000), -1000.0),
            (Data::LongUnsigned(2000), 2000.0),
            (Data::DoubleLong(-100000), -100000.0),
            (Data::DoubleLongUnsigned(200000), 200000.0),
            (Data::Long64(-10000000), -10000000.0),
            (Data::Long64Unsigned(20000000), 20000000.0),
            (Data::Float32(123.45), 123.44999694824219),
            (Data::Float64(678.90), 678.90),
        ];

        for (value, expected) in test_cases {
            let ext_reg = ExtendedRegister::new(
                ObisCode::new(1, 0, 1, 8, 0, 255),
                value,
                ScalerUnit { scaler: 0, unit: Unit::Count },
                Data::Null,
                test_datetime(),
            );
            assert_eq!(ext_reg.scaled_value(), expected);
        }
    }

    #[test]
    fn test_scaled_value_non_numeric() {
        let ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::OctetString(vec![1, 2, 3]),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );
        assert_eq!(ext_reg.scaled_value(), 0.0);
    }

    #[test]
    fn test_get_attribute_logical_name() {
        let obis = ObisCode::new(1, 0, 32, 7, 0, 255);
        let ext_reg = ExtendedRegister::new(
            obis,
            Data::LongUnsigned(0),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );
        let result = ext_reg.get_attribute(1);
        assert_eq!(result, Ok(Data::OctetString(obis.encode().to_vec())));
    }

    #[test]
    fn test_get_attribute_value() {
        let ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),
            Data::LongUnsigned(23050),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );
        let result = ext_reg.get_attribute(2);
        assert_eq!(result, Ok(Data::LongUnsigned(23050)));
    }

    #[test]
    fn test_get_attribute_scaler_unit() {
        let ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),
            Data::LongUnsigned(23050),
            ScalerUnit { scaler: -2, unit: Unit::Volt },
            Data::Null,
            test_datetime(),
        );
        let result = ext_reg.get_attribute(3);
        assert_eq!(
            result,
            Ok(Data::Structure(vec![Data::Integer(-2), Data::Enum(Unit::Volt as u8)]))
        );
    }

    #[test]
    fn test_get_attribute_status() {
        let ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),
            Data::LongUnsigned(23050),
            ScalerUnit::default(),
            Data::OctetString(vec![0x01, 0x02]),
            test_datetime(),
        );
        let result = ext_reg.get_attribute(4);
        assert_eq!(result, Ok(Data::OctetString(vec![0x01, 0x02])));
    }

    #[test]
    fn test_get_attribute_capture_time() {
        let dt = test_datetime();
        let ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),
            Data::LongUnsigned(23050),
            ScalerUnit::default(),
            Data::Null,
            dt.clone(),
        );
        let result = ext_reg.get_attribute(5);
        assert_eq!(result, Ok(Data::DateTime(dt)));
    }

    #[test]
    fn test_get_attribute_invalid() {
        let ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(0),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );
        assert_eq!(ext_reg.get_attribute(6), Err(DataAccessResult::ObjectUndefined));
        assert_eq!(ext_reg.get_attribute(0), Err(DataAccessResult::ObjectUndefined));
    }

    #[test]
    fn test_set_attribute_value() {
        let mut ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),
            Data::LongUnsigned(0),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );
        assert_eq!(ext_reg.set_attribute(2, Data::LongUnsigned(23050)), Ok(()));
        assert_eq!(ext_reg.value, Data::LongUnsigned(23050));
    }

    #[test]
    fn test_set_attribute_value_type_mismatch() {
        let mut ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),
            Data::LongUnsigned(0),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );
        assert_eq!(
            ext_reg.set_attribute(2, Data::Utf8String("invalid".to_string())),
            Err(DataAccessResult::TypeUnmatched)
        );
    }

    #[test]
    fn test_set_attribute_scaler_unit() {
        let mut ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),
            Data::LongUnsigned(0),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );
        let new_scaler_unit = Data::Structure(vec![Data::Integer(-2), Data::Enum(35)]);
        assert_eq!(ext_reg.set_attribute(3, new_scaler_unit), Ok(()));
        assert_eq!(ext_reg.scaler_unit.scaler, -2);
        assert_eq!(ext_reg.scaler_unit.unit, Unit::Volt);
    }

    #[test]
    fn test_set_attribute_scaler_unit_invalid_structure() {
        let mut ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),
            Data::LongUnsigned(0),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );
        // Wrong structure - only 1 element
        assert_eq!(
            ext_reg.set_attribute(3, Data::Structure(vec![Data::Integer(-2)])),
            Err(DataAccessResult::TypeUnmatched)
        );
        // Wrong types in structure
        assert_eq!(
            ext_reg.set_attribute(3, Data::Structure(vec![Data::Unsigned(2), Data::Unsigned(35)])),
            Err(DataAccessResult::TypeUnmatched)
        );
    }

    #[test]
    fn test_set_attribute_status_octet_string() {
        let mut ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),
            Data::LongUnsigned(0),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );
        assert_eq!(ext_reg.set_attribute(4, Data::OctetString(vec![0xFF])), Ok(()));
        assert_eq!(ext_reg.status, Data::OctetString(vec![0xFF]));
    }

    #[test]
    fn test_set_attribute_status_bit_string() {
        let mut ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),
            Data::LongUnsigned(0),
            ScalerUnit::default(),
            Data::OctetString(vec![0x01, 0x02]),
            test_datetime(),
        );
        // Update to Null status
        assert_eq!(ext_reg.set_attribute(4, Data::Null), Ok(()));
        assert_eq!(ext_reg.status, Data::Null);
    }

    #[test]
    fn test_set_attribute_status_null() {
        let mut ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),
            Data::LongUnsigned(0),
            ScalerUnit::default(),
            Data::OctetString(vec![0x01]),
            test_datetime(),
        );
        assert_eq!(ext_reg.set_attribute(4, Data::Null), Ok(()));
        assert_eq!(ext_reg.status, Data::Null);
    }

    #[test]
    fn test_set_attribute_status_any_type() {
        let mut ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),
            Data::LongUnsigned(0),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );
        // Status now accepts any Data type per DLMS spec (matches Gurux behavior)
        assert_eq!(ext_reg.set_attribute(4, Data::Unsigned(123)), Ok(()));
        assert_eq!(ext_reg.status, Data::Unsigned(123));

        // BitString is also accepted
        assert_eq!(ext_reg.set_attribute(4, Data::BitString(vec![0x80])), Ok(()));
        assert_eq!(ext_reg.status, Data::BitString(vec![0x80]));
    }

    #[test]
    fn test_set_attribute_capture_time() {
        let mut ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),
            Data::LongUnsigned(0),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );
        let new_time = DateTime {
            date: Date { year: 2025, month: 12, day_of_month: 31, day_of_week: 2 },
            time: Time { hour: Some(23), minute: Some(59), second: Some(59), hundredth: Some(99) },
            offset_minutes: Some(0),
            clock_status: Some(ClockStatus(0x00)),
        };
        assert_eq!(ext_reg.set_attribute(5, Data::DateTime(new_time.clone())), Ok(()));
        assert_eq!(ext_reg.capture_time, new_time);
    }

    #[test]
    fn test_set_attribute_capture_time_invalid() {
        let mut ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),
            Data::LongUnsigned(0),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );
        assert_eq!(
            ext_reg.set_attribute(5, Data::Unsigned(123)),
            Err(DataAccessResult::TypeUnmatched)
        );
    }

    #[test]
    fn test_set_attribute_logical_name_denied() {
        let mut ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),
            Data::LongUnsigned(0),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );
        assert_eq!(
            ext_reg.set_attribute(1, Data::OctetString(vec![1, 0, 1, 8, 0, 255])),
            Err(DataAccessResult::ReadWriteDenied)
        );
    }

    #[test]
    fn test_set_attribute_invalid() {
        let mut ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(0),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );
        assert_eq!(
            ext_reg.set_attribute(6, Data::Unsigned(100)),
            Err(DataAccessResult::ObjectUndefined)
        );
        assert_eq!(
            ext_reg.set_attribute(0, Data::Unsigned(100)),
            Err(DataAccessResult::ObjectUndefined)
        );
        assert_eq!(
            ext_reg.set_attribute(-1, Data::Unsigned(100)),
            Err(DataAccessResult::ObjectUndefined)
        );
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_invoke_method_reset_with_value() {
        let mut ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(12345),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );
        let result = ext_reg.invoke_method(1, Some(Data::DoubleLongUnsigned(0)));
        assert_eq!(result, Ok(None));
        assert_eq!(ext_reg.value, Data::DoubleLongUnsigned(0));
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_invoke_method_reset_without_value() {
        let mut ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::Long(12345),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );
        let result = ext_reg.invoke_method(1, None);
        assert_eq!(result, Ok(None));
        assert_eq!(ext_reg.value, Data::DoubleLongUnsigned(0));
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_invoke_method_reset_type_mismatch() {
        let mut ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(12345),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );
        let result = ext_reg.invoke_method(1, Some(Data::Utf8String("invalid".to_string())));
        assert_eq!(result, Err(ActionResult::TypeUnmatched));
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_invoke_method_invalid() {
        let mut ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(0),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );
        assert_eq!(ext_reg.invoke_method(2, None), Err(ActionResult::ObjectUndefined));
        assert_eq!(ext_reg.invoke_method(0, None), Err(ActionResult::ObjectUndefined));
    }

    #[test]
    fn test_extended_register_clone() {
        let ext_reg1 = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),
            Data::LongUnsigned(23050),
            ScalerUnit { scaler: -2, unit: Unit::Volt },
            Data::OctetString(vec![0x01]),
            test_datetime(),
        );
        let ext_reg2 = ext_reg1.clone();
        assert_eq!(ext_reg1, ext_reg2);
    }

    #[test]
    fn test_extended_register_debug() {
        let ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),
            Data::LongUnsigned(23050),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );
        let debug_str = format!("{:?}", ext_reg);
        assert!(debug_str.contains("ExtendedRegister"));
        assert!(debug_str.contains("LongUnsigned"));
    }

    #[test]
    fn test_real_world_example_instantaneous_voltage() {
        let ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255), // L1 instantaneous voltage
            Data::LongUnsigned(23050),
            ScalerUnit { scaler: -2, unit: Unit::Volt },
            Data::OctetString(vec![0x00]), // Status: OK
            DateTime {
                date: Date { year: 2025, month: 1, day_of_month: 25, day_of_week: 6 },
                time: Time {
                    hour: Some(14),
                    minute: Some(30),
                    second: Some(15),
                    hundredth: Some(0),
                },
                offset_minutes: Some(-60),
                clock_status: Some(ClockStatus(0x00)),
            },
        );
        assert_eq!(ext_reg.scaled_value(), 230.50); // 230.50 V
    }

    #[test]
    fn test_real_world_example_current_with_status() {
        let ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 31, 7, 0, 255), // L1 instantaneous current
            Data::LongUnsigned(1523),
            ScalerUnit { scaler: -2, unit: Unit::Ampere },
            Data::OctetString(vec![0b00000001]), // Bit 0 set = warning
            DateTime {
                date: Date { year: 2025, month: 1, day_of_month: 25, day_of_week: 6 },
                time: Time {
                    hour: Some(14),
                    minute: Some(30),
                    second: Some(15),
                    hundredth: Some(0),
                },
                offset_minutes: Some(-60),
                clock_status: Some(ClockStatus(0x00)),
            },
        );
        assert_eq!(ext_reg.scaled_value(), 15.23); // 15.23 A
    }

    #[test]
    fn test_real_world_example_power_factor() {
        let ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 13, 7, 0, 255), // Power factor
            Data::Long(985),
            ScalerUnit {
                scaler: -3,
                unit: Unit::Count, // Dimensionless
            },
            Data::Null,
            DateTime {
                date: Date { year: 2025, month: 1, day_of_month: 25, day_of_week: 6 },
                time: Time {
                    hour: Some(14),
                    minute: Some(30),
                    second: Some(15),
                    hundredth: Some(0),
                },
                offset_minutes: Some(-60),
                clock_status: Some(ClockStatus(0x00)),
            },
        );
        assert_eq!(ext_reg.scaled_value(), 0.985); // 0.985 (98.5%)
    }

    #[test]
    fn test_round_trip_value_update() {
        let mut ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),
            Data::LongUnsigned(23000),
            ScalerUnit { scaler: -2, unit: Unit::Volt },
            Data::Null,
            test_datetime(),
        );

        // Get initial value
        let initial = ext_reg.get_attribute(2).unwrap();
        assert_eq!(initial, Data::LongUnsigned(23000));

        // Update value
        let new_value = Data::LongUnsigned(23550);
        ext_reg.set_attribute(2, new_value.clone()).unwrap();

        // Get updated value
        let updated = ext_reg.get_attribute(2).unwrap();
        assert_eq!(updated, new_value);
        assert_eq!(ext_reg.scaled_value(), 235.50);
    }

    #[test]
    fn test_round_trip_status_update() {
        let mut ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),
            Data::LongUnsigned(23000),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );

        // Update status
        let new_status = Data::OctetString(vec![0x01, 0x02]);
        ext_reg.set_attribute(4, new_status.clone()).unwrap();

        // Verify
        let retrieved = ext_reg.get_attribute(4).unwrap();
        assert_eq!(retrieved, new_status);
    }

    #[test]
    fn test_round_trip_capture_time_update() {
        let mut ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),
            Data::LongUnsigned(23000),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );

        // Update capture time
        let new_time = DateTime {
            date: Date { year: 2025, month: 6, day_of_month: 15, day_of_week: 1 },
            time: Time { hour: Some(12), minute: Some(0), second: Some(0), hundredth: Some(0) },
            offset_minutes: Some(0),
            clock_status: Some(ClockStatus(0x00)),
        };
        ext_reg.set_attribute(5, Data::DateTime(new_time.clone())).unwrap();

        // Verify
        let retrieved = ext_reg.get_attribute(5).unwrap();
        assert_eq!(retrieved, Data::DateTime(new_time));
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_extended_register_serialize() {
        use serde::Serialize;

        fn assert_serialize<T: Serialize>(_: &T) {}

        let ext_reg = ExtendedRegister::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),
            Data::LongUnsigned(23050),
            ScalerUnit::default(),
            Data::Null,
            test_datetime(),
        );
        assert_serialize(&ext_reg);
    }
}
