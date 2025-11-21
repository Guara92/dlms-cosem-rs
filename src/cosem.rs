//! COSEM Object Model
//!
//! This module provides the foundational types and traits for implementing COSEM interface classes
//! as defined in the DLMS/COSEM Blue Book.
//!
//! # Overview
//!
//! COSEM (Companion Specification for Energy Metering) defines a comprehensive object model
//! for smart metering devices. Each COSEM object has:
//! - A class ID identifying the interface class type
//! - A version number for the class implementation
//! - A logical name (OBIS code) for unique identification
//! - Attributes that can be read and/or written
//! - Methods that can be invoked
//!
//! # Example
//!
//! ```
//! # #[cfg(feature = "encode")]
//! # {
//! use dlms_cosem::cosem::{CosemObject, CosemAttribute, AttributeAccess};
//! use dlms_cosem::{Data, ObisCode};
//! use dlms_cosem::get::DataAccessResult;
//! use dlms_cosem::action::ActionResult;
//!
//! // Example: Simple Data object (Class ID 1)
//! struct DataObject {
//!     logical_name: ObisCode,
//!     value: Data,
//! }
//!
//! impl CosemObject for DataObject {
//!     fn class_id(&self) -> u16 {
//!         1  // Data class
//!     }
//!
//!     fn version(&self) -> u8 {
//!         0  // Version 0
//!     }
//!
//!     fn logical_name(&self) -> &ObisCode {
//!         &self.logical_name
//!     }
//!
//!     fn get_attribute(&self, attribute_id: i8) -> Result<Data, DataAccessResult> {
//!         match attribute_id {
//!             1 => Ok(Data::OctetString(self.logical_name.encode_with_type())),
//!             2 => Ok(self.value.clone()),
//!             _ => Err(DataAccessResult::ObjectUndefined),
//!         }
//!     }
//!
//!     fn set_attribute(&mut self, attribute_id: i8, value: Data) -> Result<(), DataAccessResult> {
//!         match attribute_id {
//!             2 => {
//!                 self.value = value;
//!                 Ok(())
//!             }
//!             _ => Err(DataAccessResult::ReadWriteDenied),
//!         }
//!     }
//!
//!     fn invoke_method(&mut self, _method_id: i8, _params: Option<Data>) -> Result<Option<Data>, ActionResult> {
//!         Err(ActionResult::ObjectUndefined)  // Data class has no methods
//!     }
//! }
//! # }
//! ```


extern crate alloc;

use crate::action::ActionResult;
use crate::get::DataAccessResult;
use crate::{Data, ObisCode};

pub mod clock;
pub mod data;
pub mod demand_register;
pub mod extended_register;
pub mod profile_generic;
pub mod register;

// Re-export commonly used types
pub use crate::selective_access::CaptureObjectDefinition;
pub use profile_generic::{ProfileGeneric, SortMethod};

/// Core trait for all COSEM interface class objects.
///
/// Every COSEM object must implement this trait to provide standardized access
/// to its attributes and methods. The trait defines the common interface for:
/// - Identification (class ID, version, logical name)
/// - Attribute access (read/write)
/// - Method invocation
///
/// # Attribute IDs
///
/// - Attribute 1 is always the logical_name (OBIS code) - inherited from base class
/// - Attributes 2+ are class-specific and defined in the Blue Book
/// - Negative attribute IDs are reserved for internal use
///
/// # Method IDs
///
/// - Method IDs start from 1
/// - Each class defines its own methods in the Blue Book
/// - Not all classes have methods
pub trait CosemObject {
    /// Returns the COSEM interface class ID.
    ///
    /// Common class IDs:
    /// - 1: Data
    /// - 3: Register
    /// - 4: Extended Register
    /// - 5: Demand Register
    /// - 7: Profile Generic
    /// - 8: Clock
    /// - 15: Association LN
    ///
    /// See DLMS Blue Book for complete list.
    fn class_id(&self) -> u16;

    /// Returns the version of the interface class implementation.
    ///
    /// Most classes use version 0, but some have evolved over time.
    fn version(&self) -> u8;

    /// Returns a reference to the logical name (OBIS code) of this object.
    ///
    /// The logical name uniquely identifies this object instance.
    /// It corresponds to attribute 1, which is inherited from the base COSEM class.
    fn logical_name(&self) -> &ObisCode;

    /// Reads an attribute value.
    ///
    /// # Arguments
    ///
    /// * `attribute_id` - The attribute ID to read (1 = logical_name, 2+ = class-specific)
    ///
    /// # Returns
    ///
    /// - `Ok(Data)` - The attribute value
    /// - `Err(DataAccessResult)` - Access error (e.g., ObjectUndefined, ReadWriteDenied)
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "encode")]
    /// # {
    /// # use dlms_cosem::cosem::CosemObject;
    /// # use dlms_cosem::{Data, ObisCode};
    /// # use dlms_cosem::get::DataAccessResult;
    /// # use dlms_cosem::action::ActionResult;
    /// # struct MyObject { logical_name: ObisCode, value: Data }
    /// # impl CosemObject for MyObject {
    /// #     fn class_id(&self) -> u16 { 1 }
    /// #     fn version(&self) -> u8 { 0 }
    /// #     fn logical_name(&self) -> &ObisCode { &self.logical_name }
    /// #     fn set_attribute(&mut self, _: i8, _: Data) -> Result<(), DataAccessResult> { Ok(()) }
    /// #     fn invoke_method(&mut self, _: i8, _: Option<Data>) -> Result<Option<Data>, ActionResult> { Ok(None) }
    /// fn get_attribute(&self, attribute_id: i8) -> Result<Data, DataAccessResult> {
    ///     match attribute_id {
    ///         1 => Ok(Data::OctetString(self.logical_name.encode_with_type())),
    ///         2 => Ok(self.value.clone()),
    ///         _ => Err(DataAccessResult::ObjectUndefined),
    ///     }
    /// }
    /// # }
    /// # }
    /// ```
    fn get_attribute(&self, attribute_id: i8) -> Result<Data, DataAccessResult>;

    /// Writes an attribute value.
    ///
    /// # Arguments
    ///
    /// * `attribute_id` - The attribute ID to write (typically 2+, attribute 1 is read-only)
    /// * `value` - The new value to write
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Write successful
    /// - `Err(DataAccessResult)` - Access error (e.g., ReadWriteDenied, TypeUnmatched)
    ///
    /// # Example
    ///
    /// ```
    /// # use dlms_cosem::cosem::CosemObject;
    /// # use dlms_cosem::{Data, ObisCode};
    /// # use dlms_cosem::get::DataAccessResult;
    /// # use dlms_cosem::action::ActionResult;
    /// # struct MyObject { logical_name: ObisCode, value: Data }
    /// # impl CosemObject for MyObject {
    /// #     fn class_id(&self) -> u16 { 1 }
    /// #     fn version(&self) -> u8 { 0 }
    /// #     fn logical_name(&self) -> &ObisCode { &self.logical_name }
    /// #     fn get_attribute(&self, _: i8) -> Result<Data, DataAccessResult> { Ok(Data::Null) }
    /// #     fn invoke_method(&mut self, _: i8, _: Option<Data>) -> Result<Option<Data>, ActionResult> { Ok(None) }
    /// fn set_attribute(&mut self, attribute_id: i8, value: Data) -> Result<(), DataAccessResult> {
    ///     match attribute_id {
    ///         1 => Err(DataAccessResult::ReadWriteDenied),  // Logical name is read-only
    ///         2 => {
    ///             self.value = value;
    ///             Ok(())
    ///         }
    ///         _ => Err(DataAccessResult::ObjectUndefined),
    ///     }
    /// }
    /// # }
    /// ```
    fn set_attribute(&mut self, attribute_id: i8, value: Data) -> Result<(), DataAccessResult>;

    /// Invokes a method on this object.
    ///
    /// # Arguments
    ///
    /// * `method_id` - The method ID to invoke (1+)
    /// * `params` - Optional method parameters as Data
    ///
    /// # Returns
    ///
    /// - `Ok(Some(Data))` - Method executed successfully with return value
    /// - `Ok(None)` - Method executed successfully without return value
    /// - `Err(ActionResult)` - Execution error
    ///
    /// # Example
    ///
    /// ```
    /// # use dlms_cosem::cosem::CosemObject;
    /// # use dlms_cosem::{Data, ObisCode};
    /// # use dlms_cosem::get::DataAccessResult;
    /// # use dlms_cosem::action::ActionResult;
    /// # struct Register { logical_name: ObisCode, value: Data }
    /// # impl CosemObject for Register {
    /// #     fn class_id(&self) -> u16 { 3 }
    /// #     fn version(&self) -> u8 { 0 }
    /// #     fn logical_name(&self) -> &ObisCode { &self.logical_name }
    /// #     fn get_attribute(&self, _: i8) -> Result<Data, DataAccessResult> { Ok(Data::Null) }
    /// #     fn set_attribute(&mut self, _: i8, _: Data) -> Result<(), DataAccessResult> { Ok(()) }
    /// fn invoke_method(&mut self, method_id: i8, params: Option<Data>) -> Result<Option<Data>, ActionResult> {
    ///     match method_id {
    ///         1 => {  // Method 1: reset(data)
    ///             self.value = params.unwrap_or(Data::Null);
    ///             Ok(None)
    ///         }
    ///         _ => Err(ActionResult::ObjectUndefined),
    ///     }
    /// }
    /// # }
    /// ```
    fn invoke_method(
        &mut self,
        method_id: i8,
        params: Option<Data>,
    ) -> Result<Option<Data>, ActionResult>;
}

/// Represents a COSEM attribute with its ID, access rights, and current value.
///
/// Each COSEM object has multiple attributes that define its state.
/// This structure encapsulates an attribute's metadata and value.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CosemAttribute {
    /// The attribute identifier (1 = logical_name, 2+ = class-specific)
    pub id: i8,

    /// Access rights for this attribute
    pub access: AttributeAccess,

    /// The current value of the attribute
    pub value: Data,
}

impl CosemAttribute {
    /// Creates a new COSEM attribute.
    ///
    /// # Arguments
    ///
    /// * `id` - The attribute ID
    /// * `access` - The access rights
    /// * `value` - The initial value
    ///
    /// # Example
    ///
    /// ```
    /// use dlms_cosem::cosem::{CosemAttribute, AttributeAccess};
    /// use dlms_cosem::Data;
    ///
    /// let attr = CosemAttribute::new(2, AttributeAccess::READ_WRITE, Data::Integer(42));
    /// assert_eq!(attr.id, 2);
    /// assert!(attr.access.contains(AttributeAccess::READ_ONLY));
    /// assert!(attr.access.contains(AttributeAccess::WRITE_ONLY));
    /// ```
    pub fn new(id: i8, access: AttributeAccess, value: Data) -> Self {
        Self { id, access, value }
    }

    /// Checks if this attribute is readable.
    ///
    /// # Example
    ///
    /// ```
    /// use dlms_cosem::cosem::{CosemAttribute, AttributeAccess};
    /// use dlms_cosem::Data;
    ///
    /// let attr = CosemAttribute::new(2, AttributeAccess::READ_ONLY, Data::Null);
    /// assert!(attr.is_readable());
    ///
    /// let attr2 = CosemAttribute::new(3, AttributeAccess::WRITE_ONLY, Data::Null);
    /// assert!(!attr2.is_readable());
    /// ```
    pub fn is_readable(&self) -> bool {
        self.access.intersects(AttributeAccess::READ_ONLY)
    }

    /// Checks if this attribute is writable.
    ///
    /// # Example
    ///
    /// ```
    /// use dlms_cosem::cosem::{CosemAttribute, AttributeAccess};
    /// use dlms_cosem::Data;
    ///
    /// let attr = CosemAttribute::new(2, AttributeAccess::WRITE_ONLY, Data::Null);
    /// assert!(attr.is_writable());
    ///
    /// let attr2 = CosemAttribute::new(1, AttributeAccess::READ_ONLY, Data::Null);
    /// assert!(!attr2.is_writable());
    /// ```
    pub fn is_writable(&self) -> bool {
        self.access.intersects(AttributeAccess::WRITE_ONLY)
    }

    /// Checks if this attribute requires authentication for reading.
    pub fn requires_authenticated_read(&self) -> bool {
        self.access.contains(AttributeAccess::AUTHENTICATED_READ)
    }

    /// Checks if this attribute requires authentication for writing.
    pub fn requires_authenticated_write(&self) -> bool {
        self.access.contains(AttributeAccess::AUTHENTICATED_WRITE)
    }
}

/// Access rights for COSEM attributes.
///
/// These flags define what operations are allowed on an attribute.
/// Multiple flags can be combined using bitwise OR.
///
/// # Example
///
/// ```
/// use dlms_cosem::cosem::AttributeAccess;
///
/// // Read-write attribute
/// let rw = AttributeAccess::READ_WRITE;
/// assert!(rw.contains(AttributeAccess::READ_ONLY));
/// assert!(rw.contains(AttributeAccess::WRITE_ONLY));
///
/// // Authenticated read-write
/// let auth_rw = AttributeAccess::READ_ONLY |
///               AttributeAccess::WRITE_ONLY |
///               AttributeAccess::AUTHENTICATED_READ |
///               AttributeAccess::AUTHENTICATED_WRITE;
/// assert!(auth_rw.contains(AttributeAccess::AUTHENTICATED_READ));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AttributeAccess(u8);

impl AttributeAccess {
    /// No access allowed
    pub const NO_ACCESS: AttributeAccess = AttributeAccess(0x00);

    /// Read access allowed
    pub const READ_ONLY: AttributeAccess = AttributeAccess(0x01);

    /// Write access allowed
    pub const WRITE_ONLY: AttributeAccess = AttributeAccess(0x02);

    /// Both read and write access allowed
    pub const READ_WRITE: AttributeAccess = AttributeAccess(0x03);

    /// Authenticated read required
    pub const AUTHENTICATED_READ: AttributeAccess = AttributeAccess(0x04);

    /// Authenticated write required
    pub const AUTHENTICATED_WRITE: AttributeAccess = AttributeAccess(0x08);

    /// Creates a new AttributeAccess from raw bits.
    ///
    /// # Example
    ///
    /// ```
    /// use dlms_cosem::cosem::AttributeAccess;
    ///
    /// let access = AttributeAccess::from_bits(0x03);
    /// assert_eq!(access, AttributeAccess::READ_WRITE);
    /// ```
    pub const fn from_bits(bits: u8) -> Self {
        AttributeAccess(bits)
    }

    /// Returns the raw bits of this access rights value.
    pub const fn bits(&self) -> u8 {
        self.0
    }

    /// Returns `true` if this access rights contains the specified flags.
    pub const fn contains(&self, other: AttributeAccess) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Returns `true` if this access rights intersects with the specified flags.
    pub const fn intersects(&self, other: AttributeAccess) -> bool {
        (self.0 & other.0) != 0
    }

    /// Returns `true` if no access is allowed.
    pub const fn is_no_access(&self) -> bool {
        self.0 == 0
    }
}

impl core::ops::BitOr for AttributeAccess {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        AttributeAccess(self.0 | rhs.0)
    }
}

impl core::ops::BitAnd for AttributeAccess {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        AttributeAccess(self.0 & rhs.0)
    }
}

/// Represents a COSEM method with its ID and access rights.
///
/// Each COSEM interface class may define methods that can be invoked
/// to perform specific operations on the object.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CosemMethod {
    /// The method identifier (starts from 1)
    pub id: i8,

    /// Access rights for this method
    pub access: MethodAccess,
}

impl CosemMethod {
    /// Creates a new COSEM method.
    ///
    /// # Arguments
    ///
    /// * `id` - The method ID
    /// * `access` - The access rights
    ///
    /// # Example
    ///
    /// ```
    /// use dlms_cosem::cosem::{CosemMethod, MethodAccess};
    ///
    /// let method = CosemMethod::new(1, MethodAccess::ACCESS);
    /// assert_eq!(method.id, 1);
    /// assert!(method.is_accessible());
    /// ```
    pub fn new(id: i8, access: MethodAccess) -> Self {
        Self { id, access }
    }

    /// Checks if this method is accessible (without authentication).
    pub fn is_accessible(&self) -> bool {
        self.access.contains(MethodAccess::ACCESS)
    }

    /// Checks if this method requires authentication.
    pub fn requires_authentication(&self) -> bool {
        self.access.contains(MethodAccess::AUTHENTICATED_ACCESS)
    }
}

/// Access rights for COSEM methods.
///
/// These flags define what level of access is required to invoke a method.
///
/// # Example
///
/// ```
/// use dlms_cosem::cosem::MethodAccess;
///
/// // Public method
/// let public = MethodAccess::ACCESS;
/// assert!(!public.requires_authentication());
///
/// // Authenticated method
/// let auth = MethodAccess::AUTHENTICATED_ACCESS;
/// assert!(auth.requires_authentication());
///
/// // Both access levels allowed
/// let both = MethodAccess::ACCESS | MethodAccess::AUTHENTICATED_ACCESS;
/// assert!(both.contains(MethodAccess::ACCESS));
/// assert!(both.contains(MethodAccess::AUTHENTICATED_ACCESS));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MethodAccess(u8);

impl MethodAccess {
    /// No access allowed
    pub const NO_ACCESS: MethodAccess = MethodAccess(0x00);

    /// Method can be invoked
    pub const ACCESS: MethodAccess = MethodAccess(0x01);

    /// Method requires authenticated access
    pub const AUTHENTICATED_ACCESS: MethodAccess = MethodAccess(0x02);

    /// Creates a new MethodAccess from raw bits.
    ///
    /// # Example
    ///
    /// ```
    /// use dlms_cosem::cosem::MethodAccess;
    ///
    /// let access = MethodAccess::from_bits(0x01);
    /// assert_eq!(access, MethodAccess::ACCESS);
    /// ```
    pub const fn from_bits(bits: u8) -> Self {
        MethodAccess(bits)
    }

    /// Returns the raw bits of this access rights value.
    pub const fn bits(&self) -> u8 {
        self.0
    }

    /// Returns `true` if this access rights contains the specified flags.
    pub const fn contains(&self, other: MethodAccess) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Returns `true` if this access rights intersects with the specified flags.
    pub const fn intersects(&self, other: MethodAccess) -> bool {
        (self.0 & other.0) != 0
    }

    /// Returns `true` if method requires authentication.
    pub const fn requires_authentication(&self) -> bool {
        (self.0 & MethodAccess::AUTHENTICATED_ACCESS.0) != 0
    }

    /// Returns `true` if no access is allowed.
    pub const fn is_no_access(&self) -> bool {
        self.0 == 0
    }
}

impl core::ops::BitOr for MethodAccess {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        MethodAccess(self.0 | rhs.0)
    }
}

impl core::ops::BitAnd for MethodAccess {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        MethodAccess(self.0 & rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock object for testing CosemObject trait
    struct TestObject {
        logical_name: ObisCode,
        value: Data,
        read_only_value: Data,
    }

    impl CosemObject for TestObject {
        fn class_id(&self) -> u16 {
            99 // Test class
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
                    #[cfg(feature = "encode")]
                    {
                        Ok(Data::OctetString(self.logical_name.encode_with_type()))
                    }
                    #[cfg(not(feature = "encode"))]
                    {
                        Ok(Data::Null)
                    }
                }
                2 => Ok(self.value.clone()),
                3 => Ok(self.read_only_value.clone()),
                _ => Err(DataAccessResult::ObjectUndefined),
            }
        }

        fn set_attribute(&mut self, attribute_id: i8, value: Data) -> Result<(), DataAccessResult> {
            match attribute_id {
                1 => Err(DataAccessResult::ReadWriteDenied), // Logical name is read-only
                2 => {
                    self.value = value;
                    Ok(())
                }
                3 => Err(DataAccessResult::ReadWriteDenied), // Read-only
                _ => Err(DataAccessResult::ObjectUndefined),
            }
        }

        fn invoke_method(
            &mut self,
            method_id: i8,
            params: Option<Data>,
        ) -> Result<Option<Data>, ActionResult> {
            match method_id {
                1 => {
                    // Reset method
                    self.value = params.unwrap_or(Data::Null);
                    Ok(None)
                }
                2 => {
                    // Echo method - returns parameter
                    Ok(params)
                }
                _ => Err(ActionResult::ObjectUndefined),
            }
        }
    }

    #[test]
    fn test_cosem_object_class_id() {
        let obj = TestObject {
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            value: Data::Integer(42),
            read_only_value: Data::Unsigned(100),
        };
        assert_eq!(obj.class_id(), 99);
    }

    #[test]
    fn test_cosem_object_version() {
        let obj = TestObject {
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            value: Data::Integer(42),
            read_only_value: Data::Unsigned(100),
        };
        assert_eq!(obj.version(), 0);
    }

    #[test]
    fn test_cosem_object_logical_name() {
        let obis = ObisCode::new(1, 0, 1, 8, 0, 255);
        let obj = TestObject {
            logical_name: obis,
            value: Data::Integer(42),
            read_only_value: Data::Unsigned(100),
        };
        assert_eq!(obj.logical_name(), &obis);
    }

    #[test]
    fn test_cosem_object_get_attribute() {
        let obj = TestObject {
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            value: Data::Integer(42),
            read_only_value: Data::Unsigned(100),
        };

        // Attribute 1: logical name
        let result = obj.get_attribute(1);
        assert!(result.is_ok());

        // Attribute 2: writable value
        let result = obj.get_attribute(2);
        assert_eq!(result, Ok(Data::Integer(42)));

        // Attribute 3: read-only value
        let result = obj.get_attribute(3);
        assert_eq!(result, Ok(Data::Unsigned(100)));

        // Invalid attribute
        let result = obj.get_attribute(99);
        assert_eq!(result, Err(DataAccessResult::ObjectUndefined));
    }

    #[test]
    fn test_cosem_object_set_attribute() {
        let mut obj = TestObject {
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            value: Data::Integer(42),
            read_only_value: Data::Unsigned(100),
        };

        // Attribute 1: logical name is read-only
        let result = obj.set_attribute(1, Data::Null);
        assert_eq!(result, Err(DataAccessResult::ReadWriteDenied));

        // Attribute 2: writable
        let result = obj.set_attribute(2, Data::Integer(123));
        assert_eq!(result, Ok(()));
        assert_eq!(obj.value, Data::Integer(123));

        // Attribute 3: read-only
        let result = obj.set_attribute(3, Data::Unsigned(200));
        assert_eq!(result, Err(DataAccessResult::ReadWriteDenied));

        // Invalid attribute
        let result = obj.set_attribute(99, Data::Null);
        assert_eq!(result, Err(DataAccessResult::ObjectUndefined));
    }

    #[test]
    fn test_cosem_object_invoke_method() {
        let mut obj = TestObject {
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            value: Data::Integer(42),
            read_only_value: Data::Unsigned(100),
        };

        // Method 1: reset with parameter
        let result = obj.invoke_method(1, Some(Data::Integer(0)));
        assert_eq!(result, Ok(None));
        assert_eq!(obj.value, Data::Integer(0));

        // Method 1: reset without parameter (uses Null)
        obj.value = Data::Integer(42);
        let result = obj.invoke_method(1, None);
        assert_eq!(result, Ok(None));
        assert_eq!(obj.value, Data::Null);

        // Method 2: echo
        let result = obj.invoke_method(2, Some(Data::Integer(99)));
        assert_eq!(result, Ok(Some(Data::Integer(99))));

        // Invalid method
        let result = obj.invoke_method(99, None);
        assert_eq!(result, Err(ActionResult::ObjectUndefined));
    }

    #[test]
    fn test_attribute_access_constants() {
        assert_eq!(AttributeAccess::NO_ACCESS.bits(), 0x00);
        assert_eq!(AttributeAccess::READ_ONLY.bits(), 0x01);
        assert_eq!(AttributeAccess::WRITE_ONLY.bits(), 0x02);
        assert_eq!(AttributeAccess::READ_WRITE.bits(), 0x03);
        assert_eq!(AttributeAccess::AUTHENTICATED_READ.bits(), 0x04);
        assert_eq!(AttributeAccess::AUTHENTICATED_WRITE.bits(), 0x08);
    }

    #[test]
    fn test_attribute_access_contains() {
        let rw = AttributeAccess::READ_WRITE;
        assert!(rw.contains(AttributeAccess::READ_ONLY));
        assert!(rw.contains(AttributeAccess::WRITE_ONLY));
        assert!(!rw.contains(AttributeAccess::AUTHENTICATED_READ));

        let auth_read = AttributeAccess::READ_ONLY | AttributeAccess::AUTHENTICATED_READ;
        assert!(auth_read.contains(AttributeAccess::READ_ONLY));
        assert!(auth_read.contains(AttributeAccess::AUTHENTICATED_READ));
        assert!(!auth_read.contains(AttributeAccess::WRITE_ONLY));
    }

    #[test]
    fn test_attribute_access_intersects() {
        let rw = AttributeAccess::READ_WRITE;
        assert!(rw.intersects(AttributeAccess::READ_ONLY));
        assert!(rw.intersects(AttributeAccess::WRITE_ONLY));
        assert!(rw.intersects(AttributeAccess::READ_WRITE));
        assert!(!rw.intersects(AttributeAccess::AUTHENTICATED_READ));
    }

    #[test]
    fn test_attribute_access_is_no_access() {
        assert!(AttributeAccess::NO_ACCESS.is_no_access());
        assert!(!AttributeAccess::READ_ONLY.is_no_access());
        assert!(!AttributeAccess::READ_WRITE.is_no_access());
    }

    #[test]
    fn test_attribute_access_bitwise_operations() {
        let read = AttributeAccess::READ_ONLY;
        let write = AttributeAccess::WRITE_ONLY;
        let rw = read | write;
        assert_eq!(rw, AttributeAccess::READ_WRITE);

        let masked = rw & AttributeAccess::READ_ONLY;
        assert_eq!(masked, AttributeAccess::READ_ONLY);
    }

    #[test]
    fn test_cosem_attribute_new() {
        let attr = CosemAttribute::new(2, AttributeAccess::READ_WRITE, Data::Integer(42));
        assert_eq!(attr.id, 2);
        assert_eq!(attr.access, AttributeAccess::READ_WRITE);
        assert_eq!(attr.value, Data::Integer(42));
    }

    #[test]
    fn test_cosem_attribute_is_readable() {
        let read_only = CosemAttribute::new(2, AttributeAccess::READ_ONLY, Data::Null);
        assert!(read_only.is_readable());

        let write_only = CosemAttribute::new(3, AttributeAccess::WRITE_ONLY, Data::Null);
        assert!(!write_only.is_readable());

        let read_write = CosemAttribute::new(4, AttributeAccess::READ_WRITE, Data::Null);
        assert!(read_write.is_readable());

        let no_access = CosemAttribute::new(5, AttributeAccess::NO_ACCESS, Data::Null);
        assert!(!no_access.is_readable());
    }

    #[test]
    fn test_cosem_attribute_is_writable() {
        let write_only = CosemAttribute::new(2, AttributeAccess::WRITE_ONLY, Data::Null);
        assert!(write_only.is_writable());

        let read_only = CosemAttribute::new(3, AttributeAccess::READ_ONLY, Data::Null);
        assert!(!read_only.is_writable());

        let read_write = CosemAttribute::new(4, AttributeAccess::READ_WRITE, Data::Null);
        assert!(read_write.is_writable());

        let no_access = CosemAttribute::new(5, AttributeAccess::NO_ACCESS, Data::Null);
        assert!(!no_access.is_writable());
    }

    #[test]
    fn test_cosem_attribute_authentication() {
        let auth_read = CosemAttribute::new(
            2,
            AttributeAccess::READ_ONLY | AttributeAccess::AUTHENTICATED_READ,
            Data::Null,
        );
        assert!(auth_read.requires_authenticated_read());
        assert!(!auth_read.requires_authenticated_write());

        let auth_write = CosemAttribute::new(
            3,
            AttributeAccess::WRITE_ONLY | AttributeAccess::AUTHENTICATED_WRITE,
            Data::Null,
        );
        assert!(!auth_write.requires_authenticated_read());
        assert!(auth_write.requires_authenticated_write());

        let auth_both = CosemAttribute::new(
            4,
            AttributeAccess::READ_WRITE
                | AttributeAccess::AUTHENTICATED_READ
                | AttributeAccess::AUTHENTICATED_WRITE,
            Data::Null,
        );
        assert!(auth_both.requires_authenticated_read());
        assert!(auth_both.requires_authenticated_write());
    }

    #[test]
    fn test_method_access_constants() {
        assert_eq!(MethodAccess::NO_ACCESS.bits(), 0x00);
        assert_eq!(MethodAccess::ACCESS.bits(), 0x01);
        assert_eq!(MethodAccess::AUTHENTICATED_ACCESS.bits(), 0x02);
    }

    #[test]
    fn test_method_access_contains() {
        let access = MethodAccess::ACCESS;
        assert!(access.contains(MethodAccess::ACCESS));
        assert!(!access.contains(MethodAccess::AUTHENTICATED_ACCESS));

        let both = MethodAccess::ACCESS | MethodAccess::AUTHENTICATED_ACCESS;
        assert!(both.contains(MethodAccess::ACCESS));
        assert!(both.contains(MethodAccess::AUTHENTICATED_ACCESS));
    }

    #[test]
    fn test_method_access_requires_authentication() {
        assert!(!MethodAccess::NO_ACCESS.requires_authentication());
        assert!(!MethodAccess::ACCESS.requires_authentication());
        assert!(MethodAccess::AUTHENTICATED_ACCESS.requires_authentication());

        let both = MethodAccess::ACCESS | MethodAccess::AUTHENTICATED_ACCESS;
        assert!(both.requires_authentication());
    }

    #[test]
    fn test_method_access_is_no_access() {
        assert!(MethodAccess::NO_ACCESS.is_no_access());
        assert!(!MethodAccess::ACCESS.is_no_access());
        assert!(!MethodAccess::AUTHENTICATED_ACCESS.is_no_access());
    }

    #[test]
    fn test_cosem_method_new() {
        let method = CosemMethod::new(1, MethodAccess::ACCESS);
        assert_eq!(method.id, 1);
        assert_eq!(method.access, MethodAccess::ACCESS);
    }

    #[test]
    fn test_cosem_method_is_accessible() {
        let public = CosemMethod::new(1, MethodAccess::ACCESS);
        assert!(public.is_accessible());

        let auth = CosemMethod::new(2, MethodAccess::AUTHENTICATED_ACCESS);
        assert!(!auth.is_accessible());

        let both = CosemMethod::new(3, MethodAccess::ACCESS | MethodAccess::AUTHENTICATED_ACCESS);
        assert!(both.is_accessible());

        let none = CosemMethod::new(4, MethodAccess::NO_ACCESS);
        assert!(!none.is_accessible());
    }

    #[test]
    fn test_cosem_method_requires_authentication() {
        let public = CosemMethod::new(1, MethodAccess::ACCESS);
        assert!(!public.requires_authentication());

        let auth = CosemMethod::new(2, MethodAccess::AUTHENTICATED_ACCESS);
        assert!(auth.requires_authentication());

        let both = CosemMethod::new(3, MethodAccess::ACCESS | MethodAccess::AUTHENTICATED_ACCESS);
        assert!(both.requires_authentication());

        let none = CosemMethod::new(4, MethodAccess::NO_ACCESS);
        assert!(!none.requires_authentication());
    }

    #[test]
    fn test_attribute_clone() {
        let attr1 = CosemAttribute::new(2, AttributeAccess::READ_WRITE, Data::Integer(42));
        let attr2 = attr1.clone();
        assert_eq!(attr1, attr2);
    }

    #[test]
    fn test_method_clone() {
        let method1 = CosemMethod::new(1, MethodAccess::ACCESS);
        let method2 = method1.clone();
        assert_eq!(method1, method2);
    }
}
