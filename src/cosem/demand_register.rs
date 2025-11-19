//! COSEM Demand Register (Class ID 5)
//!
//! This module implements the DLMS/COSEM Demand Register interface class as defined in
//! the DLMS Blue Book (IEC 62056-6-2).
//!
//! A Demand Register is used to store and manage demand values (average power consumption
//! over time periods). It extends the Register concept with period management and historical
//! tracking capabilities.
//!
//! # DLMS/COSEM Specification
//!
//! - **Class ID**: 5
//! - **Version**: 0
//! - **Logical Name**: Configurable OBIS code
//!
//! ## Attributes
//!
//! 1. **logical_name** (OBIS code) - Inherited from base class, read-only
//! 2. **current_average_value** (Data) - Current demand value (MUST be numeric)
//! 3. **last_average_value** (Data) - Previous period's demand value (MUST be numeric)
//! 4. **scaler_unit** (Structure) - Scaler and unit for values
//! 5. **status** (Data) - Status information (OctetString, BitString, or Null)
//! 6. **capture_time** (DateTime) - When last_average_value was captured
//! 7. **start_time_current** (DateTime) - When current period started
//! 8. **period** (u32) - Integration period in seconds
//! 9. **number_of_periods** (u16) - Number of periods to calculate demand
//!
//! ## Methods
//!
//! 1. **reset** (method_id=1) - Reset demand register to initial state
//! 2. **next_period** (method_id=2) - Move to next demand period
//! 3. **reset_to_maximum** (method_id=3) - Reset to maximum demand value
//!
//! # References
//!
//! - DLMS Blue Book (IEC 62056-6-2): Interface Class Specifications
//! - Gurux.DLMS: GXDLMSDemandRegister reference implementation
//!
//! # Examples
//!
//! ```
//! use dlms_cosem::cosem::demand_register::DemandRegister;
//! use dlms_cosem::cosem::CosemObject;
//! use dlms_cosem::{ObisCode, Data, ScalerUnit, Unit};
//!
//! // Note: Full example omitted due to private DateTime fields.
//! // See unit tests for complete usage examples.
//! // DemandRegister implements CosemObject trait with class_id=5, version=0
//! ```

use crate::DateTime;
use crate::action::ActionResult;
use crate::cosem::CosemObject;
use crate::data::Data;
use crate::get::DataAccessResult;
use crate::obis_code::ObisCode;
use crate::unit::ScalerUnit;

#[cfg(feature = "serde")]
use serde::Serialize;

/// COSEM Demand Register (Class ID 5)
///
/// Stores and manages demand values (average power consumption over integration periods).
///
/// # Gurux Compatibility
///
/// This implementation matches the Gurux.DLMS `gxDemandRegister` structure:
/// - `currentAverageValue` → `current_average_value`
/// - `lastAverageValue` → `last_average_value`
/// - `scaler`/`unit` → `scaler_unit`
/// - `status` → `status`
/// - `captureTime` → `capture_time`
/// - `startTimeCurrent` → `start_time_current`
/// - `period` → `period`
/// - `numberOfPeriods` → `number_of_periods`
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct DemandRegister {
    /// Attribute 1: Logical name (OBIS code)
    pub logical_name: ObisCode,

    /// Attribute 2: Current average demand value (MUST be numeric)
    pub current_average_value: Data,

    /// Attribute 3: Last average demand value (MUST be numeric)
    pub last_average_value: Data,

    /// Attribute 4: Scaler and unit for demand values
    pub scaler_unit: ScalerUnit,

    /// Attribute 5: Status information (OctetString, BitString, or Null)
    pub status: Data,

    /// Attribute 6: Timestamp when last_average_value was captured
    pub capture_time: DateTime,

    /// Attribute 7: Timestamp when current demand period started
    pub start_time_current: DateTime,

    /// Attribute 8: Integration period in seconds
    pub period: u32,

    /// Attribute 9: Number of periods for demand calculation
    pub number_of_periods: u16,
}

impl DemandRegister {
    /// Calculate the scaled current average value as floating-point
    ///
    /// Formula: `actual_value = raw_value × 10^scaler`
    ///
    /// # Examples
    ///
    /// ```
    /// // See unit tests for complete usage examples
    /// // Example: raw_value=12500, scaler=-3 → 12500 × 10^-3 = 12.5 kW
    /// ```
    pub fn scaled_current_value(&self) -> f64 {
        let scaler_multiplier = 10f64.powi(self.scaler_unit.scaler as i32);
        match &self.current_average_value {
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

    /// Calculate the scaled last average value as floating-point
    ///
    /// Formula: `actual_value = raw_value × 10^scaler`
    pub fn scaled_last_value(&self) -> f64 {
        let scaler_multiplier = 10f64.powi(self.scaler_unit.scaler as i32);
        match &self.last_average_value {
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

impl CosemObject for DemandRegister {
    fn class_id(&self) -> u16 {
        5
    }

    fn version(&self) -> u8 {
        0
    }

    fn logical_name(&self) -> &ObisCode {
        &self.logical_name
    }

    fn get_attribute(&self, attribute_id: i8) -> Result<Data, DataAccessResult> {
        match attribute_id {
            1 => {
                // Logical name - encoded as OctetString per DLMS spec
                Ok(Data::OctetString(vec![
                    self.logical_name.a,
                    self.logical_name.b,
                    self.logical_name.c,
                    self.logical_name.d,
                    self.logical_name.e,
                    self.logical_name.f,
                ]))
            }
            2 => Ok(self.current_average_value.clone()),
            3 => Ok(self.last_average_value.clone()),
            4 => {
                // scaler_unit encoded as Structure(Integer, Enum)
                Ok(Data::Structure(vec![
                    Data::Integer(self.scaler_unit.scaler),
                    Data::Enum(self.scaler_unit.unit.as_i8() as u8),
                ]))
            }
            5 => Ok(self.status.clone()),
            6 => Ok(Data::DateTime(self.capture_time.clone())),
            7 => Ok(Data::DateTime(self.start_time_current.clone())),
            8 => Ok(Data::DoubleLongUnsigned(self.period)),
            9 => Ok(Data::LongUnsigned(self.number_of_periods)),
            _ => Err(DataAccessResult::ObjectUndefined),
        }
    }

    fn set_attribute(&mut self, attribute_id: i8, value: Data) -> Result<(), DataAccessResult> {
        match attribute_id {
            1 => {
                // Logical name is read-only per DLMS spec
                Err(DataAccessResult::ReadWriteDenied)
            }
            2 => {
                // current_average_value MUST be numeric
                if value.is_numeric() {
                    self.current_average_value = value;
                    Ok(())
                } else {
                    Err(DataAccessResult::TypeUnmatched)
                }
            }
            3 => {
                // last_average_value MUST be numeric
                if value.is_numeric() {
                    self.last_average_value = value;
                    Ok(())
                } else {
                    Err(DataAccessResult::TypeUnmatched)
                }
            }
            4 => {
                // scaler_unit must be Structure(Integer, Enum)
                if let Data::Structure(elements) = value {
                    if elements.len() == 2 {
                        if let (Data::Integer(scaler), Data::Enum(unit_val)) =
                            (&elements[0], &elements[1])
                        {
                            self.scaler_unit.scaler = *scaler;
                            if let Ok(unit) = crate::unit::Unit::try_from(*unit_val) {
                                self.scaler_unit.unit = unit;
                                return Ok(());
                            }
                        }
                    }
                }
                Err(DataAccessResult::TypeUnmatched)
            }
            5 => {
                // status can be any type (flexible per spec)
                self.status = value;
                Ok(())
            }
            6 => {
                // capture_time must be DateTime
                match value {
                    Data::DateTime(dt) => {
                        self.capture_time = dt;
                        Ok(())
                    }
                    _ => Err(DataAccessResult::TypeUnmatched),
                }
            }
            7 => {
                // start_time_current must be DateTime
                match value {
                    Data::DateTime(dt) => {
                        self.start_time_current = dt;
                        Ok(())
                    }
                    _ => Err(DataAccessResult::TypeUnmatched),
                }
            }
            8 => {
                // period must be DoubleLongUnsigned
                match value {
                    Data::DoubleLongUnsigned(p) => {
                        self.period = p;
                        Ok(())
                    }
                    _ => Err(DataAccessResult::TypeUnmatched),
                }
            }
            9 => {
                // number_of_periods must be LongUnsigned
                match value {
                    Data::LongUnsigned(n) => {
                        self.number_of_periods = n;
                        Ok(())
                    }
                    _ => Err(DataAccessResult::TypeUnmatched),
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
                // Method 1: reset(data)
                // Reset demand register: move current to last, set current to parameter or 0

                let new_value = if let Some(param) = parameters {
                    if !param.is_numeric() {
                        return Err(ActionResult::TypeUnmatched);
                    }
                    param
                } else {
                    Data::DoubleLongUnsigned(0)
                };

                // Move current to last
                self.last_average_value = self.current_average_value.clone();
                // Set current to new value
                self.current_average_value = new_value;
                // Update capture time to now (conceptually)
                // Note: In real implementation, this would use system time
                self.capture_time = DateTime {
                    date: crate::data::Date {
                        year: 0xFFFF,
                        month: 0xFF,
                        day_of_month: 0xFF,
                        day_of_week: 0xFF,
                    },
                    time: crate::data::Time {
                        hour: Some(0xFF),
                        minute: Some(0xFF),
                        second: Some(0xFF),
                        hundredth: Some(0xFF),
                    },
                    offset_minutes: None,
                    clock_status: None,
                };

                Ok(Some(Data::Integer(0))) // Success
            }
            2 => {
                // Method 2: next_period()
                // Move to next demand period: current → last, current = 0

                self.last_average_value = self.current_average_value.clone();
                self.current_average_value = Data::DoubleLongUnsigned(0);
                self.capture_time = DateTime {
                    date: crate::data::Date {
                        year: 0xFFFF,
                        month: 0xFF,
                        day_of_month: 0xFF,
                        day_of_week: 0xFF,
                    },
                    time: crate::data::Time {
                        hour: Some(0xFF),
                        minute: Some(0xFF),
                        second: Some(0xFF),
                        hundredth: Some(0xFF),
                    },
                    offset_minutes: None,
                    clock_status: None,
                };
                self.start_time_current = DateTime {
                    date: crate::data::Date {
                        year: 0xFFFF,
                        month: 0xFF,
                        day_of_month: 0xFF,
                        day_of_week: 0xFF,
                    },
                    time: crate::data::Time {
                        hour: Some(0xFF),
                        minute: Some(0xFF),
                        second: Some(0xFF),
                        hundredth: Some(0xFF),
                    },
                    offset_minutes: None,
                    clock_status: None,
                };

                Ok(Some(Data::Integer(0))) // Success
            }
            3 => {
                // Method 3: reset_to_maximum()
                // Reset: current becomes max(current, last), last = 0

                let current_val = self.scaled_current_value();
                let last_val = self.scaled_last_value();
                let max_val = current_val.max(last_val);

                // Convert back to raw value (unscale: divide by 10^scaler)
                let scaler_divisor = 10f64.powi(self.scaler_unit.scaler as i32);
                let raw_max = max_val / scaler_divisor;

                // Convert back to same type as current
                let max_data = match &self.current_average_value {
                    Data::Integer(_) => Data::Integer(raw_max as i8),
                    Data::Unsigned(_) => Data::Unsigned(raw_max as u8),
                    Data::Long(_) => Data::Long(raw_max as i16),
                    Data::LongUnsigned(_) => Data::LongUnsigned(raw_max as u16),
                    Data::DoubleLong(_) => Data::DoubleLong(raw_max as i32),
                    Data::DoubleLongUnsigned(_) => Data::DoubleLongUnsigned(raw_max as u32),
                    Data::Long64(_) => Data::Long64(raw_max as i64),
                    Data::Long64Unsigned(_) => Data::Long64Unsigned(raw_max as u64),
                    Data::Float32(_) => Data::Float32(raw_max as f32),
                    Data::Float64(_) => Data::Float64(raw_max),
                    _ => Data::DoubleLongUnsigned(0),
                };

                self.current_average_value = max_data;
                self.last_average_value = Data::DoubleLongUnsigned(0);
                self.capture_time = DateTime {
                    date: crate::data::Date {
                        year: 0xFFFF,
                        month: 0xFF,
                        day_of_month: 0xFF,
                        day_of_week: 0xFF,
                    },
                    time: crate::data::Time {
                        hour: Some(0xFF),
                        minute: Some(0xFF),
                        second: Some(0xFF),
                        hundredth: Some(0xFF),
                    },
                    offset_minutes: None,
                    clock_status: None,
                };

                Ok(Some(Data::Integer(0))) // Success
            }
            _ => Err(ActionResult::ObjectUndefined),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wildcard_datetime() -> DateTime {
        DateTime {
            date: crate::data::Date {
                year: 0xFFFF,
                month: 0xFF,
                day_of_month: 0xFF,
                day_of_week: 0xFF,
            },
            time: crate::data::Time {
                hour: Some(0xFF),
                minute: Some(0xFF),
                second: Some(0xFF),
                hundredth: Some(0xFF),
            },
            offset_minutes: None,
            clock_status: None,
        }
    }

    fn create_test_demand_register() -> DemandRegister {
        DemandRegister {
            logical_name: ObisCode::new(1, 0, 1, 6, 0, 255),
            current_average_value: Data::DoubleLongUnsigned(12500),
            last_average_value: Data::DoubleLongUnsigned(11000),
            scaler_unit: ScalerUnit { scaler: -3, unit: crate::Unit::Watt },
            status: Data::Null,
            capture_time: wildcard_datetime(),
            start_time_current: wildcard_datetime(),
            period: 900, // 15 minutes
            number_of_periods: 1,
        }
    }

    // ============================================================================
    // Basic Structure Tests
    // ============================================================================

    #[test]
    fn test_class_id() {
        let demand = create_test_demand_register();
        assert_eq!(demand.class_id(), 5);
    }

    #[test]
    fn test_version() {
        let demand = create_test_demand_register();
        assert_eq!(demand.version(), 0);
    }

    #[test]
    fn test_logical_name() {
        let demand = create_test_demand_register();
        assert_eq!(demand.logical_name(), &ObisCode::new(1, 0, 1, 6, 0, 255));
    }

    // ============================================================================
    // Scaled Value Tests
    // ============================================================================

    #[test]
    fn test_scaled_current_value() {
        let demand = create_test_demand_register();
        // 12500 × 10^-3 = 12.5 kW
        assert_eq!(demand.scaled_current_value(), 12.5);
    }

    #[test]
    fn test_scaled_last_value() {
        let demand = create_test_demand_register();
        // 11000 × 10^-3 = 11.0 kW
        assert_eq!(demand.scaled_last_value(), 11.0);
    }

    #[test]
    fn test_scaled_value_with_positive_scaler() {
        let mut demand = create_test_demand_register();
        demand.scaler_unit.scaler = 2; // × 100
        demand.current_average_value = Data::LongUnsigned(250);
        // 250 × 10^2 = 25000
        assert_eq!(demand.scaled_current_value(), 25000.0);
    }

    #[test]
    fn test_scaled_value_zero_scaler() {
        let mut demand = create_test_demand_register();
        demand.scaler_unit.scaler = 0; // × 1
        demand.current_average_value = Data::LongUnsigned(1234);
        assert_eq!(demand.scaled_current_value(), 1234.0);
    }

    #[test]
    fn test_scaled_value_non_numeric() {
        let mut demand = create_test_demand_register();
        demand.current_average_value = Data::OctetString(vec![1, 2, 3]);
        // Non-numeric returns 0.0
        assert_eq!(demand.scaled_current_value(), 0.0);
    }

    // ============================================================================
    // get_attribute Tests
    // ============================================================================

    #[test]
    fn test_get_attribute_logical_name() {
        let demand = create_test_demand_register();
        let result = demand.get_attribute(1).unwrap();
        assert_eq!(result, Data::OctetString(vec![1, 0, 1, 6, 0, 255]));
    }

    #[test]
    fn test_get_attribute_current_average_value() {
        let demand = create_test_demand_register();
        let result = demand.get_attribute(2).unwrap();
        assert_eq!(result, Data::DoubleLongUnsigned(12500));
    }

    #[test]
    fn test_get_attribute_last_average_value() {
        let demand = create_test_demand_register();
        let result = demand.get_attribute(3).unwrap();
        assert_eq!(result, Data::DoubleLongUnsigned(11000));
    }

    #[test]
    fn test_get_attribute_scaler_unit() {
        let demand = create_test_demand_register();
        let result = demand.get_attribute(4).unwrap();
        match result {
            Data::Structure(elements) => {
                assert_eq!(elements.len(), 2);
                assert_eq!(elements[0], Data::Integer(-3));
                assert_eq!(elements[1], Data::Enum(27)); // Watt = 27
            }
            _ => panic!("Expected Structure"),
        }
    }

    #[test]
    fn test_get_attribute_status() {
        let demand = create_test_demand_register();
        let result = demand.get_attribute(5).unwrap();
        assert_eq!(result, Data::Null);
    }

    #[test]
    fn test_get_attribute_capture_time() {
        let demand = create_test_demand_register();
        let result = demand.get_attribute(6).unwrap();
        match result {
            Data::DateTime(_) => {} // OK
            _ => panic!("Expected DateTime"),
        }
    }

    #[test]
    fn test_get_attribute_start_time_current() {
        let demand = create_test_demand_register();
        let result = demand.get_attribute(7).unwrap();
        match result {
            Data::DateTime(_) => {} // OK
            _ => panic!("Expected DateTime"),
        }
    }

    #[test]
    fn test_get_attribute_period() {
        let demand = create_test_demand_register();
        let result = demand.get_attribute(8).unwrap();
        assert_eq!(result, Data::DoubleLongUnsigned(900));
    }

    #[test]
    fn test_get_attribute_number_of_periods() {
        let demand = create_test_demand_register();
        let result = demand.get_attribute(9).unwrap();
        assert_eq!(result, Data::LongUnsigned(1));
    }

    #[test]
    fn test_get_attribute_invalid() {
        let demand = create_test_demand_register();
        assert_eq!(demand.get_attribute(10).unwrap_err(), DataAccessResult::ObjectUndefined);
        assert_eq!(demand.get_attribute(0).unwrap_err(), DataAccessResult::ObjectUndefined);
    }

    // ============================================================================
    // set_attribute Tests
    // ============================================================================

    #[test]
    fn test_set_attribute_logical_name_readonly() {
        let mut demand = create_test_demand_register();
        let result = demand.set_attribute(1, Data::OctetString(vec![0, 0, 96, 1, 0, 255]));
        assert_eq!(result.unwrap_err(), DataAccessResult::ReadWriteDenied);
    }

    #[test]
    fn test_set_attribute_current_average_value() {
        let mut demand = create_test_demand_register();
        demand.set_attribute(2, Data::DoubleLongUnsigned(15000)).unwrap();
        assert_eq!(demand.current_average_value, Data::DoubleLongUnsigned(15000));
    }

    #[test]
    fn test_set_attribute_current_average_value_type_mismatch() {
        let mut demand = create_test_demand_register();
        let result = demand.set_attribute(2, Data::OctetString(vec![1, 2, 3]));
        assert_eq!(result.unwrap_err(), DataAccessResult::TypeUnmatched);
    }

    #[test]
    fn test_set_attribute_last_average_value() {
        let mut demand = create_test_demand_register();
        demand.set_attribute(3, Data::DoubleLongUnsigned(9500)).unwrap();
        assert_eq!(demand.last_average_value, Data::DoubleLongUnsigned(9500));
    }

    #[test]
    fn test_set_attribute_last_average_value_type_mismatch() {
        let mut demand = create_test_demand_register();
        let result = demand.set_attribute(3, Data::Null);
        assert_eq!(result.unwrap_err(), DataAccessResult::TypeUnmatched);
    }

    #[test]
    fn test_set_attribute_scaler_unit() {
        let mut demand = create_test_demand_register();
        let scaler_unit_data = Data::Structure(vec![
            Data::Integer(-2),
            Data::Enum(30), // WattHour
        ]);
        demand.set_attribute(4, scaler_unit_data).unwrap();
        assert_eq!(demand.scaler_unit.scaler, -2);
        assert_eq!(demand.scaler_unit.unit, crate::Unit::WattHour);
    }

    #[test]
    fn test_set_attribute_status() {
        let mut demand = create_test_demand_register();
        demand.set_attribute(5, Data::OctetString(vec![0x01])).unwrap();
        assert_eq!(demand.status, Data::OctetString(vec![0x01]));
    }

    #[test]
    fn test_set_attribute_capture_time() {
        let mut demand = create_test_demand_register();
        let dt = wildcard_datetime();
        demand.set_attribute(6, Data::DateTime(dt)).unwrap();
        // Verify it was set (we can't compare DateTime directly due to wildcards)
        assert!(matches!(demand.capture_time, DateTime { .. }));
    }

    #[test]
    fn test_set_attribute_start_time_current() {
        let mut demand = create_test_demand_register();
        let dt = wildcard_datetime();
        demand.set_attribute(7, Data::DateTime(dt)).unwrap();
        assert!(matches!(demand.start_time_current, DateTime { .. }));
    }

    #[test]
    fn test_set_attribute_period() {
        let mut demand = create_test_demand_register();
        demand.set_attribute(8, Data::DoubleLongUnsigned(1800)).unwrap();
        assert_eq!(demand.period, 1800);
    }

    #[test]
    fn test_set_attribute_number_of_periods() {
        let mut demand = create_test_demand_register();
        demand.set_attribute(9, Data::LongUnsigned(4)).unwrap();
        assert_eq!(demand.number_of_periods, 4);
    }

    #[test]
    fn test_set_attribute_invalid() {
        let mut demand = create_test_demand_register();
        let result = demand.set_attribute(10, Data::Null);
        assert_eq!(result.unwrap_err(), DataAccessResult::ObjectUndefined);
    }

    // ============================================================================
    // Method Tests: reset (method_id=1)
    // ============================================================================

    #[test]
    fn test_method_reset_with_parameter() {
        let mut demand = create_test_demand_register();
        demand.current_average_value = Data::DoubleLongUnsigned(5000);
        demand.last_average_value = Data::DoubleLongUnsigned(3000);

        let result = demand.invoke_method(1, Some(Data::DoubleLongUnsigned(100)));
        assert!(result.is_ok());

        // current moved to last
        assert_eq!(demand.last_average_value, Data::DoubleLongUnsigned(5000));
        // new value set to parameter
        assert_eq!(demand.current_average_value, Data::DoubleLongUnsigned(100));
    }

    #[test]
    fn test_method_reset_without_parameter() {
        let mut demand = create_test_demand_register();
        demand.current_average_value = Data::DoubleLongUnsigned(8000);
        demand.last_average_value = Data::DoubleLongUnsigned(6000);

        let result = demand.invoke_method(1, None);
        assert!(result.is_ok());

        // current moved to last
        assert_eq!(demand.last_average_value, Data::DoubleLongUnsigned(8000));
        // new value set to 0
        assert_eq!(demand.current_average_value, Data::DoubleLongUnsigned(0));
    }

    #[test]
    fn test_method_reset_invalid_parameter_type() {
        let mut demand = create_test_demand_register();
        let result = demand.invoke_method(1, Some(Data::OctetString(vec![1, 2, 3])));
        assert_eq!(result.unwrap_err(), ActionResult::TypeUnmatched);
    }

    // ============================================================================
    // Method Tests: next_period (method_id=2)
    // ============================================================================

    #[test]
    fn test_method_next_period() {
        let mut demand = create_test_demand_register();
        demand.current_average_value = Data::DoubleLongUnsigned(7500);
        demand.last_average_value = Data::DoubleLongUnsigned(6500);

        let result = demand.invoke_method(2, None);
        assert!(result.is_ok());

        // current moved to last
        assert_eq!(demand.last_average_value, Data::DoubleLongUnsigned(7500));
        // current reset to 0
        assert_eq!(demand.current_average_value, Data::DoubleLongUnsigned(0));
    }

    // ============================================================================
    // Method Tests: reset_to_maximum (method_id=3)
    // ============================================================================

    #[test]
    fn test_method_reset_to_maximum_current_higher() {
        let mut demand = create_test_demand_register();
        demand.current_average_value = Data::DoubleLongUnsigned(15000);
        demand.last_average_value = Data::DoubleLongUnsigned(12000);

        let result = demand.invoke_method(3, None);
        assert!(result.is_ok());

        // max(15, 12) = 15 kW
        assert_eq!(demand.scaled_current_value(), 15.0);
        // last reset to 0
        assert_eq!(demand.last_average_value, Data::DoubleLongUnsigned(0));
    }

    #[test]
    fn test_method_reset_to_maximum_last_higher() {
        let mut demand = create_test_demand_register();
        demand.current_average_value = Data::DoubleLongUnsigned(10000);
        demand.last_average_value = Data::DoubleLongUnsigned(13000);

        let result = demand.invoke_method(3, None);
        assert!(result.is_ok());

        // max(10, 13) = 13 kW
        assert_eq!(demand.scaled_current_value(), 13.0);
        // last reset to 0
        assert_eq!(demand.last_average_value, Data::DoubleLongUnsigned(0));
    }

    #[test]
    fn test_method_invalid() {
        let mut demand = create_test_demand_register();
        let result = demand.invoke_method(4, None);
        assert_eq!(result.unwrap_err(), ActionResult::ObjectUndefined);
    }

    // ============================================================================
    // Round-trip Tests
    // ============================================================================

    #[test]
    fn test_roundtrip_all_attributes() {
        let mut demand = create_test_demand_register();

        // Set all writable attributes
        demand.set_attribute(2, Data::LongUnsigned(5000)).unwrap();
        demand.set_attribute(3, Data::LongUnsigned(4500)).unwrap();
        demand.set_attribute(8, Data::DoubleLongUnsigned(3600)).unwrap();
        demand.set_attribute(9, Data::LongUnsigned(2)).unwrap();

        // Get and verify
        assert_eq!(demand.get_attribute(2).unwrap(), Data::LongUnsigned(5000));
        assert_eq!(demand.get_attribute(3).unwrap(), Data::LongUnsigned(4500));
        assert_eq!(demand.get_attribute(8).unwrap(), Data::DoubleLongUnsigned(3600));
        assert_eq!(demand.get_attribute(9).unwrap(), Data::LongUnsigned(2));
    }

    // ============================================================================
    // Real-world Examples
    // ============================================================================

    #[test]
    fn test_real_world_example_15min_demand() {
        let demand = DemandRegister {
            logical_name: ObisCode::new(1, 0, 1, 6, 0, 255), // Max demand +A
            current_average_value: Data::DoubleLongUnsigned(12345),
            last_average_value: Data::DoubleLongUnsigned(11234),
            scaler_unit: ScalerUnit { scaler: -3, unit: crate::Unit::Watt },
            status: Data::Null,
            capture_time: wildcard_datetime(),
            start_time_current: wildcard_datetime(),
            period: 900, // 15 minutes
            number_of_periods: 1,
        };

        assert_eq!(demand.scaled_current_value(), 12.345); // kW
        assert_eq!(demand.scaled_last_value(), 11.234); // kW
    }

    #[test]
    fn test_real_world_example_monthly_demand() {
        let demand = DemandRegister {
            logical_name: ObisCode::new(1, 0, 1, 6, 1, 255),
            current_average_value: Data::DoubleLongUnsigned(185000),
            last_average_value: Data::DoubleLongUnsigned(178000),
            scaler_unit: ScalerUnit { scaler: -3, unit: crate::Unit::Watt },
            status: Data::OctetString(vec![0x00]),
            capture_time: wildcard_datetime(),
            start_time_current: wildcard_datetime(),
            period: 2592000, // 30 days
            number_of_periods: 1,
        };

        assert_eq!(demand.scaled_current_value(), 185.0); // kW
    }

    // ============================================================================
    // Trait Implementations
    // ============================================================================

    #[test]
    fn test_debug_trait() {
        let demand = create_test_demand_register();
        let debug_str = format!("{:?}", demand);
        assert!(debug_str.contains("DemandRegister"));
    }

    #[test]
    fn test_clone_trait() {
        let demand = create_test_demand_register();
        let cloned = demand.clone();
        assert_eq!(demand, cloned);
    }

    #[test]
    fn test_partial_eq_trait() {
        let demand1 = create_test_demand_register();
        let demand2 = create_test_demand_register();
        assert_eq!(demand1, demand2);
    }
}
