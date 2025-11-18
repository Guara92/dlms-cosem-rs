//! COSEM Interface Class 1: Data
//!
//! This module implements the Data interface class as defined in the DLMS/COSEM Blue Book.
//!
//! The Data class is the simplest COSEM interface class, providing access to a single data value.
//! It is commonly used for simple parameters and configuration values.
//!
//! ## Attributes
//! - Attribute 1: `logical_name` (inherited) - OBIS code identifying the object
//! - Attribute 2: `value` - The data value (any DLMS Data type)
//!
//! # Example
//! ```
//! use dlms_cosem::cosem::data::DataObject;
//! use dlms_cosem::cosem::CosemObject;
//! use dlms_cosem::{ObisCode, Data};
//!
//! let data_obj = DataObject::new(
//!     ObisCode::new(0, 0, 96, 1, 0, 255),
//!     Data::Unsigned(42),
//! );
//!
//! assert_eq!(data_obj.class_id(), 1);
//! assert_eq!(data_obj.version(), 0);
//! ```

use crate::cosem::CosemObject;
use crate::data::Data;
use crate::obis_code::ObisCode;

#[cfg(feature = "encode")]
use crate::action::ActionResult;

/// Data object - COSEM Interface Class 1
///
/// The Data class provides simple read/write access to a single data value.
/// This is the most basic COSEM interface class.
///
/// Reference: Blue Book 4.1.1
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DataObject {
    /// Attribute 1: Logical name (OBIS code)
    pub logical_name: ObisCode,
    /// Attribute 2: Value of the data object
    pub value: Data,
}

impl DataObject {
    /// Create a new Data object
    ///
    /// # Arguments
    /// * `logical_name` - OBIS code identifying this object
    /// * `value` - Initial value of the data object
    ///
    /// # Example
    /// ```
    /// use dlms_cosem::cosem::data::DataObject;
    /// use dlms_cosem::{ObisCode, Data};
    ///
    /// let obj = DataObject::new(
    ///     ObisCode::new(0, 0, 96, 1, 0, 255),
    ///     Data::Utf8String("Test".to_string()),
    /// );
    /// ```
    pub fn new(logical_name: ObisCode, value: Data) -> Self {
        Self { logical_name, value }
    }
}

impl CosemObject for DataObject {
    fn class_id(&self) -> u16 {
        1
    }

    fn version(&self) -> u8 {
        0
    }

    fn logical_name(&self) -> &ObisCode {
        &self.logical_name
    }

    fn get_attribute(&self, id: i8) -> Result<Data, crate::get::DataAccessResult> {
        match id {
            1 => Ok(Data::OctetString(self.logical_name.encode().to_vec())),
            2 => Ok(self.value.clone()),
            _ => Err(crate::get::DataAccessResult::ObjectUndefined),
        }
    }

    fn set_attribute(&mut self, id: i8, value: Data) -> Result<(), crate::get::DataAccessResult> {
        match id {
            1 => Err(crate::get::DataAccessResult::ReadWriteDenied), // logical_name is read-only
            2 => {
                self.value = value;
                Ok(())
            }
            _ => Err(crate::get::DataAccessResult::ObjectUndefined),
        }
    }

    #[cfg(feature = "encode")]
    fn invoke_method(
        &mut self,
        _id: i8,
        _parameters: Option<Data>,
    ) -> Result<Option<Data>, ActionResult> {
        // Data class has no methods
        Err(ActionResult::ObjectUndefined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_object_new() {
        let obis = ObisCode::new(0, 0, 96, 1, 0, 255);
        let value = Data::Unsigned(42);
        let obj = DataObject::new(obis, value.clone());

        assert_eq!(obj.logical_name, obis);
        assert_eq!(obj.value, value);
    }

    #[test]
    fn test_data_object_class_id() {
        let obj = DataObject::new(ObisCode::new(0, 0, 96, 1, 0, 255), Data::Null);
        assert_eq!(obj.class_id(), 1);
    }

    #[test]
    fn test_data_object_version() {
        let obj = DataObject::new(ObisCode::new(0, 0, 96, 1, 0, 255), Data::Null);
        assert_eq!(obj.version(), 0);
    }

    #[test]
    fn test_data_object_logical_name() {
        let obis = ObisCode::new(0, 0, 96, 1, 0, 255);
        let obj = DataObject::new(obis, Data::Null);
        assert_eq!(obj.logical_name(), &obis);
    }

    #[test]
    fn test_get_attribute_logical_name() {
        let obis = ObisCode::new(0, 0, 96, 1, 0, 255);
        let obj = DataObject::new(obis, Data::Unsigned(42));

        let value = obj.get_attribute(1).unwrap();
        assert_eq!(value, Data::OctetString(vec![0x00, 0x00, 0x60, 0x01, 0x00, 0xFF]));
    }

    #[test]
    fn test_get_attribute_value() {
        let obj = DataObject::new(
            ObisCode::new(0, 0, 96, 1, 0, 255),
            Data::Utf8String("Hello".to_string()),
        );

        let value = obj.get_attribute(2).unwrap();
        assert_eq!(value, Data::Utf8String("Hello".to_string()));
    }

    #[test]
    fn test_get_attribute_invalid() {
        let obj = DataObject::new(ObisCode::new(0, 0, 96, 1, 0, 255), Data::Null);

        let result = obj.get_attribute(0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), crate::get::DataAccessResult::ObjectUndefined);

        let result = obj.get_attribute(3);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), crate::get::DataAccessResult::ObjectUndefined);
    }

    #[test]
    fn test_set_attribute_value() {
        let mut obj = DataObject::new(ObisCode::new(0, 0, 96, 1, 0, 255), Data::Unsigned(42));

        let result = obj.set_attribute(2, Data::Unsigned(100));
        assert!(result.is_ok());
        assert_eq!(obj.value, Data::Unsigned(100));
    }

    #[test]
    fn test_set_attribute_logical_name_denied() {
        let mut obj = DataObject::new(ObisCode::new(0, 0, 96, 1, 0, 255), Data::Null);

        let result =
            obj.set_attribute(1, Data::OctetString(vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06]));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), crate::get::DataAccessResult::ReadWriteDenied);
    }

    #[test]
    fn test_set_attribute_invalid() {
        let mut obj = DataObject::new(ObisCode::new(0, 0, 96, 1, 0, 255), Data::Null);

        let result = obj.set_attribute(0, Data::Null);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), crate::get::DataAccessResult::ObjectUndefined);

        let result = obj.set_attribute(3, Data::Null);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), crate::get::DataAccessResult::ObjectUndefined);
    }

    #[test]
    fn test_data_object_with_different_types() {
        // Test with various Data types
        let test_cases = vec![
            Data::Null,
            Data::Integer(-42),
            Data::Unsigned(255),
            Data::Long(-1000),
            Data::LongUnsigned(65535),
            Data::DoubleLong(-123456),
            Data::DoubleLongUnsigned(4294967295),
            Data::Long64(-123456789),
            Data::Long64Unsigned(4294967295),
            Data::Utf8String("test".to_string()),
            Data::OctetString(vec![0x01, 0x02, 0x03]),
        ];

        for value in test_cases {
            let obj = DataObject::new(ObisCode::new(0, 0, 96, 1, 0, 255), value.clone());
            assert_eq!(obj.value, value);

            let attr_value = obj.get_attribute(2).unwrap();
            assert_eq!(attr_value, value);
        }
    }

    #[cfg(feature = "encode")]
    #[test]
    fn test_invoke_method_not_supported() {
        use crate::action::ActionResult;

        let mut obj = DataObject::new(ObisCode::new(0, 0, 96, 1, 0, 255), Data::Unsigned(42));

        // Data class has no methods
        let result = obj.invoke_method(1, Some(Data::Null));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ActionResult::ObjectUndefined);

        let result = obj.invoke_method(0, None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ActionResult::ObjectUndefined);
    }

    #[test]
    fn test_data_object_clone() {
        let obj1 = DataObject::new(
            ObisCode::new(0, 0, 96, 1, 0, 255),
            Data::Utf8String("test".to_string()),
        );
        let obj2 = obj1.clone();

        assert_eq!(obj1, obj2);
        assert_eq!(obj1.logical_name, obj2.logical_name);
        assert_eq!(obj1.value, obj2.value);
    }

    #[test]
    fn test_data_object_debug() {
        let obj = DataObject::new(ObisCode::new(0, 0, 96, 1, 0, 255), Data::Unsigned(42));
        let debug_str = format!("{:?}", obj);
        assert!(debug_str.contains("DataObject"));
        assert!(debug_str.contains("logical_name"));
        assert!(debug_str.contains("value"));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_data_object_serialize() {
        use serde::Serialize;
        let obj = DataObject::new(ObisCode::new(0, 0, 96, 1, 0, 255), Data::Unsigned(42));

        // Verify the trait is implemented (compile-time check)
        fn assert_serialize<T: Serialize>(_: &T) {}
        assert_serialize(&obj);
    }

    #[test]
    fn test_real_world_example_device_id() {
        // Real-world example: Device ID as UTF-8 string
        let device_id = DataObject::new(
            ObisCode::new(0, 0, 96, 1, 0, 255), // Device ID 1
            Data::Utf8String("METER-12345678".to_string()),
        );

        assert_eq!(device_id.class_id(), 1);
        let attr = device_id.get_attribute(2).unwrap();
        assert_eq!(attr, Data::Utf8String("METER-12345678".to_string()));
    }

    #[test]
    fn test_real_world_example_manufacturer_code() {
        // Real-world example: Manufacturer code as unsigned integer
        let manufacturer = DataObject::new(
            ObisCode::new(0, 0, 96, 2, 0, 255), // Manufacturer code
            Data::LongUnsigned(0x1234),
        );

        assert_eq!(manufacturer.class_id(), 1);
        let attr = manufacturer.get_attribute(2).unwrap();
        assert_eq!(attr, Data::LongUnsigned(0x1234));
    }

    #[test]
    fn test_round_trip_attribute_access() {
        let mut obj =
            DataObject::new(ObisCode::new(0, 0, 96, 1, 0, 255), Data::DoubleLongUnsigned(0));

        // Get initial value
        let attr = obj.get_attribute(2).unwrap();
        assert_eq!(attr, Data::DoubleLongUnsigned(0));

        // Set new value
        let new_value = Data::DoubleLongUnsigned(999999);
        obj.set_attribute(2, new_value.clone()).unwrap();

        // Get updated value
        let attr = obj.get_attribute(2).unwrap();
        assert_eq!(attr, new_value);
        assert_eq!(obj.value, new_value);
    }
}
