//! COSEM Interface Class 3: Register
//!
//! This module implements the Register interface class as defined in the DLMS/COSEM Blue Book.
//!
//! The Register class represents a simple value with associated scaler and unit.
//! It is one of the most commonly used COSEM interface classes for metering values.
//!
//! ## Attributes
//! - Attribute 1: `logical_name` (inherited) - OBIS code identifying the object
//! - Attribute 2: `value` - The register value (numeric Data type)
//! - Attribute 3: `scaler_unit` - Scaler and physical unit (Structure with 2 elements)
//!
//! ## Methods
//! - Method 1: `reset(data)` - Reset the register value to a default value
//!
//! # Example
//! ```
//! use dlms_cosem::cosem::register::Register;
//! use dlms_cosem::cosem::CosemObject;
//! use dlms_cosem::{ObisCode, Data, ScalerUnit, Unit};
//!
//! let register = Register::new(
//!     ObisCode::new(1, 0, 1, 8, 0, 255),  // Active energy import
//!     Data::DoubleLongUnsigned(12345),     // Raw value
//!     ScalerUnit { scaler: -2, unit: Unit::WattHour },       // 10^-2 Wh = 0.01 Wh
//! );
//!
//! assert_eq!(register.class_id(), 3);
//! assert_eq!(register.version(), 0);
//! assert_eq!(register.scaled_value(), 123.45);  // 12345 * 10^-2 = 123.45 Wh
//! ```

use crate::cosem::CosemObject;
use crate::data::Data;
use crate::get::DataAccessResult;
use crate::obis_code::ObisCode;
use crate::unit::ScalerUnit;

#[cfg(feature = "encode")]
use crate::action::ActionResult;

/// Register object - COSEM Interface Class 3
///
/// Represents a metered value with scaler and unit information.
/// The scaler is applied as: actual_value = raw_value × 10^scaler
///
/// Reference: Blue Book 4.3.1
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Register {
    /// Attribute 1: Logical name (OBIS code)
    pub logical_name: ObisCode,
    /// Attribute 2: Current value of the register (must be numeric)
    pub value: Data,
    /// Attribute 3: Scaler and unit
    pub scaler_unit: ScalerUnit,
}

impl Register {
    /// Create a new Register object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this register
    /// * `value` - Initial value (should be numeric Data type)
    /// * `scaler_unit` - Scaler and physical unit
    ///
    /// # Example
    /// ```
    /// use dlms_cosem::cosem::register::Register;
    /// use dlms_cosem::{ObisCode, Data, ScalerUnit, Unit};
    ///
    /// let register = Register::new(
    ///     ObisCode::new(1, 0, 1, 8, 0, 255),
    ///     Data::DoubleLongUnsigned(100000),
    ///     ScalerUnit { scaler: -3, unit: Unit::WattHour },  // kWh
    /// );
    /// ```
    pub fn new(logical_name: ObisCode, value: Data, scaler_unit: ScalerUnit) -> Self {
        Self { logical_name, value, scaler_unit }
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
    /// use dlms_cosem::cosem::register::Register;
    /// use dlms_cosem::{ObisCode, Data, ScalerUnit, Unit};
    ///
    /// let register = Register::new(
    ///     ObisCode::new(1, 0, 1, 8, 0, 255),
    ///     Data::DoubleLongUnsigned(12345),
    ///     ScalerUnit { scaler: -2, unit: Unit::WattHour },
    /// );
    ///
    /// assert_eq!(register.scaled_value(), 123.45);
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

impl CosemObject for Register {
    fn class_id(&self) -> u16 {
        3
    }

    fn version(&self) -> u8 {
        0
    }

    fn logical_name(&self) -> &ObisCode {
        &self.logical_name
    }

    fn get_attribute(&self, id: i8) -> Result<Data, DataAccessResult> {
        match id {
            1 => Ok(Data::OctetString(self.logical_name.encode().to_vec())),
            2 => Ok(self.value.clone()),
            3 => {
                // scaler_unit is encoded as Structure with 2 elements: scaler (Integer) and unit (Enum)
                Ok(Data::Structure(vec![
                    Data::Integer(self.scaler_unit.scaler),
                    Data::Enum(self.scaler_unit.unit.as_i8() as u8),
                ]))
            }
            _ => Err(DataAccessResult::ObjectUndefined),
        }
    }

    fn set_attribute(&mut self, id: i8, value: Data) -> Result<(), DataAccessResult> {
        match id {
            1 => Err(DataAccessResult::ReadWriteDenied), // logical_name is read-only
            2 => {
                // Validate that the value is numeric
                if value.is_numeric() {
                    self.value = value;
                    Ok(())
                } else {
                    Err(DataAccessResult::TypeUnmatched)
                }
            }
            3 => {
                // scaler_unit can be written but must be a valid Structure
                match value {
                    Data::Structure(ref elements) if elements.len() == 2 => {
                        if let (Data::Integer(scaler), Data::Enum(unit_val)) =
                            (&elements[0], &elements[1])
                        {
                            if let Ok(unit) = crate::unit::Unit::try_from(*unit_val) {
                                self.scaler_unit = ScalerUnit { scaler: *scaler, unit };
                                Ok(())
                            } else {
                                Err(DataAccessResult::TypeUnmatched)
                            }
                        } else {
                            Err(DataAccessResult::TypeUnmatched)
                        }
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
                // Method 1: reset(data) - Reset the value to the provided data or to 0
                let reset_value = parameters.unwrap_or(Data::DoubleLongUnsigned(0));
                if reset_value.is_numeric() {
                    self.value = reset_value;
                    Ok(None) // No return value
                } else {
                    Err(ActionResult::TypeUnmatched)
                }
            }
            _ => Err(ActionResult::ObjectUndefined),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unit::Unit;

    #[test]
    fn test_register_new() {
        let obis = ObisCode::new(1, 0, 1, 8, 0, 255);
        let value = Data::DoubleLongUnsigned(12345);
        let scaler_unit = ScalerUnit { scaler: -2, unit: Unit::WattHour };

        let register = Register::new(obis, value.clone(), scaler_unit);

        assert_eq!(register.logical_name, obis);
        assert_eq!(register.value, value);
        assert_eq!(register.scaler_unit.scaler, -2);
        assert_eq!(register.scaler_unit.unit, Unit::WattHour);
    }

    #[test]
    fn test_register_class_id() {
        let register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(0),
            ScalerUnit { scaler: 0, unit: Unit::WattHour },
        );
        assert_eq!(register.class_id(), 3);
    }

    #[test]
    fn test_register_version() {
        let register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(0),
            ScalerUnit { scaler: 0, unit: Unit::WattHour },
        );
        assert_eq!(register.version(), 0);
    }

    #[test]
    fn test_register_logical_name() {
        let obis = ObisCode::new(1, 0, 1, 8, 0, 255);
        let register = Register::new(
            obis,
            Data::DoubleLongUnsigned(0),
            ScalerUnit { scaler: 0, unit: Unit::WattHour },
        );
        assert_eq!(register.logical_name(), &obis);
    }

    #[test]
    fn test_scaled_value_positive_scaler() {
        let register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(123),
            ScalerUnit { scaler: 2, unit: Unit::WattHour }, // 10^2 = 100
        );
        assert_eq!(register.scaled_value(), 12300.0); // 123 * 100
    }

    #[test]
    fn test_scaled_value_negative_scaler() {
        let register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(12345),
            ScalerUnit { scaler: -2, unit: Unit::WattHour }, // 10^-2 = 0.01
        );
        assert_eq!(register.scaled_value(), 123.45); // 12345 * 0.01
    }

    #[test]
    fn test_scaled_value_zero_scaler() {
        let register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(999),
            ScalerUnit { scaler: 0, unit: Unit::WattHour }, // 10^0 = 1
        );
        assert_eq!(register.scaled_value(), 999.0);
    }

    #[test]
    fn test_scaled_value_different_numeric_types() {
        let test_cases = vec![
            (Data::Integer(-10), -2, -0.1),
            (Data::Unsigned(50), -1, 5.0),
            (Data::Long(-1000), -3, -1.0),
            (Data::LongUnsigned(5000), -1, 500.0),
            (Data::DoubleLong(-100000), 2, -10000000.0),
            (Data::DoubleLongUnsigned(100000), -3, 100.0),
            (Data::Long64(-999999), -2, -9999.99),
            (Data::Long64Unsigned(123456), -3, 123.456),
            (Data::Float32(123.45), 0, 123.45),
            (Data::Float64(987.654), -1, 98.7654),
        ];

        for (value, scaler, expected) in test_cases {
            let register = Register::new(
                ObisCode::new(1, 0, 1, 8, 0, 255),
                value,
                ScalerUnit { scaler, unit: Unit::WattHour },
            );
            let result = register.scaled_value();
            assert!(
                (result - expected).abs() < 0.0001,
                "Expected {}, got {} for value {:?}",
                expected,
                result,
                register.value
            );
        }
    }

    #[test]
    fn test_scaled_value_non_numeric() {
        let register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::Utf8String("test".to_string()),
            ScalerUnit { scaler: -2, unit: Unit::WattHour },
        );
        assert_eq!(register.scaled_value(), 0.0);
    }

    #[test]
    fn test_get_attribute_logical_name() {
        let register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(100),
            ScalerUnit { scaler: -2, unit: Unit::WattHour },
        );

        let value = register.get_attribute(1).unwrap();
        assert_eq!(value, Data::OctetString(vec![0x01, 0x00, 0x01, 0x08, 0x00, 0xFF]));
    }

    #[test]
    fn test_get_attribute_value() {
        let register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(12345),
            ScalerUnit { scaler: -2, unit: Unit::WattHour },
        );

        let value = register.get_attribute(2).unwrap();
        assert_eq!(value, Data::DoubleLongUnsigned(12345));
    }

    #[test]
    fn test_get_attribute_scaler_unit() {
        let register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(100),
            ScalerUnit { scaler: -3, unit: Unit::WattHour },
        );

        let value = register.get_attribute(3).unwrap();
        match value {
            Data::Structure(elements) => {
                assert_eq!(elements.len(), 2);
                assert_eq!(elements[0], Data::Integer(-3));
                assert_eq!(elements[1], Data::Enum(30)); // Wh = 30
            }
            _ => panic!("Expected Structure"),
        }
    }

    #[test]
    fn test_get_attribute_invalid() {
        let register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(100),
            ScalerUnit { scaler: 0, unit: Unit::WattHour },
        );

        assert_eq!(register.get_attribute(0).unwrap_err(), DataAccessResult::ObjectUndefined);
        assert_eq!(register.get_attribute(4).unwrap_err(), DataAccessResult::ObjectUndefined);
    }

    #[test]
    fn test_set_attribute_value() {
        let mut register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(100),
            ScalerUnit { scaler: -2, unit: Unit::WattHour },
        );

        assert!(register.set_attribute(2, Data::DoubleLongUnsigned(500)).is_ok());
        assert_eq!(register.value, Data::DoubleLongUnsigned(500));
    }

    #[test]
    fn test_set_attribute_value_type_mismatch() {
        let mut register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(100),
            ScalerUnit { scaler: -2, unit: Unit::WattHour },
        );

        let result = register.set_attribute(2, Data::Utf8String("test".to_string()));
        assert_eq!(result.unwrap_err(), DataAccessResult::TypeUnmatched);
    }

    #[test]
    fn test_set_attribute_scaler_unit() {
        let mut register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(100),
            ScalerUnit { scaler: -2, unit: Unit::WattHour },
        );

        let new_scaler_unit = Data::Structure(vec![Data::Integer(-3), Data::Enum(31)]); // -3, VoltAmpereHour
        assert!(register.set_attribute(3, new_scaler_unit).is_ok());
        assert_eq!(register.scaler_unit.scaler, -3);
        assert_eq!(register.scaler_unit.unit, Unit::VoltAmpereHour);
    }

    #[test]
    fn test_set_attribute_scaler_unit_invalid_structure() {
        let mut register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(100),
            ScalerUnit { scaler: -2, unit: Unit::WattHour },
        );

        // Wrong number of elements
        let result = register.set_attribute(3, Data::Structure(vec![Data::Integer(-3)]));
        assert_eq!(result.unwrap_err(), DataAccessResult::TypeUnmatched);

        // Wrong types
        let result =
            register.set_attribute(3, Data::Structure(vec![Data::Unsigned(3), Data::Unsigned(30)]));
        assert_eq!(result.unwrap_err(), DataAccessResult::TypeUnmatched);
    }

    #[test]
    fn test_set_attribute_logical_name_denied() {
        let mut register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(100),
            ScalerUnit { scaler: -2, unit: Unit::WattHour },
        );

        let result = register.set_attribute(1, Data::OctetString(vec![1, 2, 3, 4, 5, 6]));
        assert_eq!(result.unwrap_err(), DataAccessResult::ReadWriteDenied);
    }

    #[test]
    fn test_set_attribute_invalid() {
        let mut register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(100),
            ScalerUnit { scaler: -2, unit: Unit::WattHour },
        );

        assert_eq!(
            register.set_attribute(0, Data::Null).unwrap_err(),
            DataAccessResult::ObjectUndefined
        );
        assert_eq!(
            register.set_attribute(4, Data::Null).unwrap_err(),
            DataAccessResult::ObjectUndefined
        );
    }

    #[cfg(feature = "encode")]
    #[test]
    fn test_invoke_method_reset_with_value() {
        let mut register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(12345),
            ScalerUnit { scaler: -2, unit: Unit::WattHour },
        );

        let result = register.invoke_method(1, Some(Data::DoubleLongUnsigned(999)));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
        assert_eq!(register.value, Data::DoubleLongUnsigned(999));
    }

    #[cfg(feature = "encode")]
    #[test]
    fn test_invoke_method_reset_without_value() {
        let mut register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(12345),
            ScalerUnit { scaler: -2, unit: Unit::WattHour },
        );

        let result = register.invoke_method(1, None);
        assert!(result.is_ok());
        assert_eq!(register.value, Data::DoubleLongUnsigned(0)); // Default reset value
    }

    #[cfg(feature = "encode")]
    #[test]
    fn test_invoke_method_reset_type_mismatch() {
        let mut register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(12345),
            ScalerUnit { scaler: -2, unit: Unit::WattHour },
        );

        let result = register.invoke_method(1, Some(Data::Utf8String("test".to_string())));
        assert_eq!(result.unwrap_err(), ActionResult::TypeUnmatched);
    }

    #[cfg(feature = "encode")]
    #[test]
    fn test_invoke_method_invalid() {
        let mut register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(100),
            ScalerUnit { scaler: -2, unit: Unit::WattHour },
        );

        assert_eq!(register.invoke_method(0, None).unwrap_err(), ActionResult::ObjectUndefined);
        assert_eq!(register.invoke_method(2, None).unwrap_err(), ActionResult::ObjectUndefined);
    }

    #[test]
    fn test_register_clone() {
        let register1 = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(12345),
            ScalerUnit { scaler: -2, unit: Unit::WattHour },
        );
        let register2 = register1.clone();

        assert_eq!(register1, register2);
        assert_eq!(register1.logical_name, register2.logical_name);
        assert_eq!(register1.value, register2.value);
        assert_eq!(register1.scaler_unit, register2.scaler_unit);
    }

    #[test]
    fn test_register_debug() {
        let register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(12345),
            ScalerUnit { scaler: -2, unit: Unit::WattHour },
        );
        let debug_str = format!("{:?}", register);
        assert!(debug_str.contains("Register"));
        assert!(debug_str.contains("logical_name"));
        assert!(debug_str.contains("value"));
        assert!(debug_str.contains("scaler_unit"));
    }

    #[test]
    fn test_real_world_example_active_energy() {
        // Real-world example: Active energy import (kWh)
        let energy = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),               // 1-0:1.8.0*255
            Data::DoubleLongUnsigned(123456),                // Raw value
            ScalerUnit { scaler: -3, unit: Unit::WattHour }, // 10^-3 Wh = kWh
        );

        assert_eq!(energy.scaled_value(), 123.456); // kWh
        assert_eq!(energy.class_id(), 3);
    }

    #[test]
    fn test_real_world_example_voltage() {
        // Real-world example: Voltage (V)
        let voltage = Register::new(
            ObisCode::new(1, 0, 32, 7, 0, 255),          // 1-0:32.7.0*255
            Data::LongUnsigned(23050),                   // Raw value (230.50 V)
            ScalerUnit { scaler: -2, unit: Unit::Volt }, // 10^-2 V
        );

        assert_eq!(voltage.scaled_value(), 230.50); // V
    }

    #[test]
    fn test_real_world_example_current() {
        // Real-world example: Current (A)
        let current = Register::new(
            ObisCode::new(1, 0, 31, 7, 0, 255),            // 1-0:31.7.0*255
            Data::LongUnsigned(1234),                      // Raw value (12.34 A)
            ScalerUnit { scaler: -2, unit: Unit::Ampere }, // 10^-2 A
        );

        assert_eq!(current.scaled_value(), 12.34); // A
    }

    #[test]
    fn test_round_trip_value_update() {
        let mut register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(0),
            ScalerUnit { scaler: -2, unit: Unit::WattHour },
        );

        // Get initial value
        let value = register.get_attribute(2).unwrap();
        assert_eq!(value, Data::DoubleLongUnsigned(0));

        // Set new value
        register.set_attribute(2, Data::DoubleLongUnsigned(99999)).unwrap();

        // Verify update
        let value = register.get_attribute(2).unwrap();
        assert_eq!(value, Data::DoubleLongUnsigned(99999));
        assert_eq!(register.scaled_value(), 999.99);
    }

    #[test]
    fn test_round_trip_scaler_unit_update() {
        let mut register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(1000),
            ScalerUnit { scaler: -3, unit: Unit::WattHour },
        );

        assert_eq!(register.scaled_value(), 1.0); // 1000 * 10^-3 = 1 Wh

        // Change scaler and unit
        let new_scaler_unit = Data::Structure(vec![Data::Integer(-2), Data::Enum(31)]); // -2, VoltAmpereHour
        register.set_attribute(3, new_scaler_unit).unwrap();

        assert_eq!(register.scaled_value(), 10.0); // 1000 * 10^-2 = 10 VAh
        assert_eq!(register.scaler_unit.unit, Unit::VoltAmpereHour);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_register_serialize() {
        use serde::Serialize;
        let register = Register::new(
            ObisCode::new(1, 0, 1, 8, 0, 255),
            Data::DoubleLongUnsigned(12345),
            ScalerUnit { scaler: -2, unit: Unit::WattHour },
        );

        // Verify the trait is implemented (compile-time check)
        fn assert_serialize<T: Serialize>(_: &T) {}
        assert_serialize(&register);
    }
}
