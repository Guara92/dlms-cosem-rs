//! ProfileGeneric (Class 7) COSEM Interface Class
//!
//! The ProfileGeneric interface class provides a simple and generic interface for
//! management of load profile buffers, event logs, and any other time-series data.
//!
//! ## Overview
//!
//! ProfileGeneric is used to capture and store periodic or event-driven data from
//! other COSEM objects. Common use cases include:
//! - Load profiles (15-minute/hourly energy data)
//! - Event logs and tamper records
//! - Demand registers snapshots
//! - Multi-column time-series data
//!
//! ## Phase 5.2 Implementation
//!
//! This implementation includes:
//! - Core buffer management (FIFO/LIFO)
//! - CaptureObjectDefinition (column definitions)
//! - Basic attribute access
//! - Methods: reset(), capture()
//!
//! ## Phase 5.3 Deferred Features
//!
//! The following features will be implemented in Phase 5.3:
//! - RangeDescriptor (filter by date range)
//! - EntryDescriptor (filter by row/column)
//! - Selective access integration
//! - Advanced sort methods (Largest, Smallest, etc.)
//!
//! ## Example Usage
//!
//! ```rust
//! extern crate alloc;
//! use dlms_cosem::cosem::{ProfileGeneric, CaptureObjectDefinition, SortMethod};
//! use dlms_cosem::{ObisCode, Data};
//! use alloc::collections::VecDeque;
//!
//! // Create a load profile for 15-minute energy data
//! let profile = ProfileGeneric {
//!     logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
//!     buffer: VecDeque::new(),
//!     capture_objects: vec![
//!         // Column 1: Clock (timestamp)
//!         CaptureObjectDefinition {
//!             class_id: 8,
//!             logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
//!             attribute_index: 2,
//!             data_index: 0,
//!         },
//!         // Column 2: Active Energy
//!         CaptureObjectDefinition {
//!             class_id: 3,
//!             logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
//!             attribute_index: 2,
//!             data_index: 0,
//!         },
//!     ],
//!     capture_period: 900, // 15 minutes
//!     sort_method: SortMethod::Fifo,
//!     sort_object: None,
//!     entries_in_use: 0,
//!     profile_entries: 96, // 24 hours × 4 intervals/hour
//!     executed_time: 0,
//!     use_compact_array_encoding: false,
//!     use_compact_array_encoding: false,
//! };
//! ```
//!
//! ## References
//!
//! - **Blue Book**: IEC 62056-6-2, Section 4.7.1 (Profile Generic IC)
//! - **Green Book**: DLMS UA 1000-2 Ed.12, Section 14.4 (Encoding examples)
//! - **Gurux**: `gxprofilegeneric.h` / `gxprofilegeneric.c`

use crate::action::ActionResult;
use crate::cosem::CosemObject;
use crate::get::DataAccessResult;
use crate::{Data, ObisCode};

#[cfg(not(feature = "std"))]
use alloc::collections::VecDeque;
#[cfg(feature = "std")]
use std::collections::VecDeque;

// Re-export CaptureObjectDefinition from selective_access for backward compatibility
pub use crate::selective_access::CaptureObjectDefinition;

/// Sort method for ProfileGeneric buffer
///
/// Defines how entries are organized in the buffer when new data is added.
///
/// ## DLMS/COSEM Specification
///
/// - **FIFO** (First In First Out): Most common for load profiles
/// - **LIFO** (Last In First Out): Used for stack-like behavior
/// - **Largest/Smallest**: Sort by value in sort_object column
/// - **NearestToZero/FarthestFromZero**: Sort by distance from zero
///
/// ## Phase 5.2 Status
///
/// - ✅ FIFO: Fully implemented
/// - ✅ LIFO: Fully implemented
/// - ⏳ Largest, Smallest, NearestToZero, FarthestFromZero: Deferred to Phase 5.3
///
/// ## Example
///
/// ```rust
/// use dlms_cosem::cosem::SortMethod;
///
/// let method = SortMethod::Fifo;
/// assert_eq!(method as u8, 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum SortMethod {
    /// First In First Out (ring buffer)
    ///
    /// Oldest entries are overwritten when buffer is full.
    /// This is the most common mode for load profiles.
    Fifo = 1,

    /// Last In First Out (stack)
    ///
    /// Newest entries are overwritten when buffer is full.
    /// Less common, used for specific applications.
    Lifo = 2,

    /// Sort by largest value in sort_object column
    ///
    /// **Phase 5.3**: Not yet implemented
    Largest = 3,

    /// Sort by smallest value in sort_object column
    ///
    /// **Phase 5.3**: Not yet implemented
    Smallest = 4,

    /// Sort by nearest to zero in sort_object column
    ///
    /// **Phase 5.3**: Not yet implemented
    NearestToZero = 5,

    /// Sort by farthest from zero in sort_object column
    ///
    /// **Phase 5.3**: Not yet implemented
    FarthestFromZero = 6,
}

impl SortMethod {
    /// Convert u8 to SortMethod
    ///
    /// Returns Err if value is not in range 1-6.
    pub fn from_u8(value: u8) -> Result<Self, DataAccessResult> {
        match value {
            1 => Ok(SortMethod::Fifo),
            2 => Ok(SortMethod::Lifo),
            3 => Ok(SortMethod::Largest),
            4 => Ok(SortMethod::Smallest),
            5 => Ok(SortMethod::NearestToZero),
            6 => Ok(SortMethod::FarthestFromZero),
            _ => Err(DataAccessResult::TypeUnmatched),
        }
    }
}

/// ProfileGeneric COSEM Interface Class (Class ID 7)
///
/// Provides a simple and generic interface for managing load profile buffers,
/// event logs, and any other time-series data.
///
/// ## Attributes
///
/// | ID | Name             | Type                  | Access | Description                    |
/// |----|------------------|-----------------------|--------|--------------------------------|
/// | 1  | logical_name     | OctetString(6)        | R      | OBIS code                      |
/// | 2  | buffer           | Array of Array        | R      | Captured data (read-only)      |
/// | 3  | capture_objects  | Array of Structure(4) | R/W    | Column definitions             |
/// | 4  | capture_period   | DoubleLongUnsigned    | R/W    | Seconds (0 = event-driven)     |
/// | 5  | sort_method      | Enum                  | R/W    | FIFO, LIFO, etc.               |
/// | 6  | sort_object      | Structure(4) or Null  | R/W    | Column to sort by              |
/// | 7  | entries_in_use   | DoubleLongUnsigned    | R      | Current buffer size            |
/// | 8  | profile_entries  | DoubleLongUnsigned    | R/W    | Maximum buffer size            |
///
/// ## Methods
///
/// | ID | Name    | Parameters | Description           |
/// |----|---------|------------|-----------------------|
/// | 1  | reset   | Integer    | Clear buffer          |
/// | 2  | capture | Integer    | Manually add entry    |
///
/// ## Example: 15-Minute Load Profile
/// ## Example
///
/// ```rust
/// extern crate alloc;
/// use dlms_cosem::cosem::{ProfileGeneric, CaptureObjectDefinition, SortMethod};
/// use dlms_cosem::{ObisCode, Data};
/// use alloc::collections::VecDeque;
///
/// let profile = ProfileGeneric {
///     logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
///     buffer: VecDeque::new(),
///     capture_objects: vec![
///         CaptureObjectDefinition {
///             class_id: 8,
///             logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
///             attribute_index: 2,
///             data_index: 0,
///         },
///     ],
///     capture_period: 900,
///     sort_method: SortMethod::Fifo,
///     sort_object: None,
///     entries_in_use: 0,
///     profile_entries: 96,
///     use_compact_array_encoding: false,
/// };
/// ```
///
/// ## References
///
/// - Blue Book IEC 62056-6-2, Section 4.7.1
/// - Green Book DLMS UA 1000-2 Ed.12, Section 14.4
/// - Gurux: gxProfileGeneric
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ProfileGeneric {
    /// Attribute 1: Logical name (OBIS code)
    pub logical_name: ObisCode,

    /// Attribute 2: Buffer - array of captured entries
    ///
    /// Each entry is an array of Data values matching the structure
    /// defined by capture_objects.
    ///
    /// **Read-only**: Modified via reset() and capture() methods only.
    ///
    /// **Implementation Note**: Uses `VecDeque` internally for O(1) FIFO/LIFO operations.
    /// Converted to `Vec` for DLMS encoding during `get_attribute()`.
    pub buffer: VecDeque<Vec<Data>>,

    /// Attribute 3: Capture objects - defines buffer structure
    ///
    /// Each CaptureObjectDefinition defines one column in the buffer.
    /// Typically the first column is a Clock (timestamp).
    pub capture_objects: Vec<CaptureObjectDefinition>,

    /// Attribute 4: Capture period in seconds
    ///
    /// - 0 = event-driven (manual capture)
    /// - >0 = periodic capture interval
    pub capture_period: u32,

    /// Attribute 5: Sort method
    ///
    /// Defines how buffer entries are organized.
    pub sort_method: SortMethod,

    /// Attribute 6: Sort object (which column to sort by)
    ///
    /// - None if sort_method is FIFO or LIFO
    /// - Some(def) for Largest, Smallest, etc. (Phase 5.3)
    pub sort_object: Option<CaptureObjectDefinition>,

    /// Attribute 7: Entries in use (current buffer size)
    ///
    /// **Read-only**: Automatically updated when buffer changes.
    /// Always ≤ profile_entries.
    pub entries_in_use: u32,

    /// Attribute 8: Profile entries (maximum buffer size)
    ///
    /// When buffer reaches this size, oldest/newest entries are removed
    /// according to sort_method.
    pub profile_entries: u32,

    /// Internal: Last execution timestamp (Unix epoch seconds)
    ///
    /// **Not a DLMS attribute** - Internal optimization for periodic capture scheduling.
    /// Tracks when the last automatic capture was executed to calculate the next
    /// trigger time based on `capture_period`.
    ///
    /// Matches Gurux `executedTime` field for compatibility.
    ///
    /// ## Usage
    ///
    /// - Updated when periodic capture is triggered
    /// - Used to calculate: `next_capture = executed_time + capture_period`
    /// - Not exposed via get_attribute() / set_attribute()
    /// - Reset to 0 on initialization or reset()
    #[cfg_attr(feature = "serde", serde(skip))]
    pub executed_time: u32,

    /// Configuration: Use Compact Array encoding for buffer (Attribute 2)
    ///
    /// If true, non-empty buffer will be encoded as CompactArray (tag 19).
    /// If false (default), buffer is encoded as standard Array (tag 1).
    ///
    /// **Note**: Compact Array encoding is an optimization introduced in DLMS Green Book 14.4.
    /// Ensure the client supports this encoding before enabling.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub use_compact_array_encoding: bool,
}

impl CosemObject for ProfileGeneric {
    fn class_id(&self) -> u16 {
        7
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
            2 => {
                // Encode buffer as Array of Arrays (convert VecDeque to Vec for encoding)
                let rows: Vec<Data> =
                    self.buffer.iter().map(|row| Data::Structure(row.clone())).collect();

                if self.use_compact_array_encoding && !rows.is_empty() {
                    Ok(Data::CompactArray(rows))
                } else {
                    // Standard encoding: Array of Structures
                    Ok(Data::Array(rows))
                }
            }
            3 => {
                // Encode capture_objects as Array of Structures
                let objects: Vec<Data> = self
                    .capture_objects
                    .iter()
                    .map(|obj| {
                        Data::Structure(vec![
                            Data::LongUnsigned(obj.class_id),
                            Data::OctetString(obj.logical_name.encode().to_vec()),
                            Data::Integer(obj.attribute_index),
                            Data::LongUnsigned(obj.data_index),
                        ])
                    })
                    .collect();
                Ok(Data::Structure(objects))
            }
            4 => Ok(Data::DoubleLongUnsigned(self.capture_period)),
            5 => Ok(Data::Enum(self.sort_method as u8)),
            6 => match &self.sort_object {
                Some(obj) => Ok(Data::Structure(vec![
                    Data::LongUnsigned(obj.class_id),
                    Data::OctetString(obj.logical_name.encode().to_vec()),
                    Data::Integer(obj.attribute_index),
                    Data::LongUnsigned(obj.data_index),
                ])),
                None => Ok(Data::Null),
            },
            7 => Ok(Data::DoubleLongUnsigned(self.entries_in_use)),
            8 => Ok(Data::DoubleLongUnsigned(self.profile_entries)),
            _ => Err(DataAccessResult::ObjectUndefined),
        }
    }

    fn set_attribute(&mut self, id: i8, value: Data) -> Result<(), DataAccessResult> {
        match id {
            1 | 2 | 7 => Err(DataAccessResult::ReadWriteDenied), // Read-only attributes
            3 => {
                // Parse capture_objects from Array of Structures
                if let Data::Structure(objects) = value {
                    let mut capture_objects = Vec::new();
                    for obj_data in objects {
                        if let Data::Structure(fields) = obj_data {
                            if fields.len() != 4 {
                                return Err(DataAccessResult::TypeUnmatched);
                            }
                            let class_id = match &fields[0] {
                                Data::LongUnsigned(v) => *v,
                                _ => return Err(DataAccessResult::TypeUnmatched),
                            };
                            let logical_name = match &fields[1] {
                                Data::OctetString(bytes) => {
                                    if bytes.len() != 6 {
                                        return Err(DataAccessResult::TypeUnmatched);
                                    }
                                    ObisCode::new(
                                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
                                    )
                                }
                                _ => return Err(DataAccessResult::TypeUnmatched),
                            };
                            let attribute_index = match &fields[2] {
                                Data::Integer(v) => *v,
                                _ => return Err(DataAccessResult::TypeUnmatched),
                            };
                            let data_index = match &fields[3] {
                                Data::LongUnsigned(v) => *v,
                                _ => return Err(DataAccessResult::TypeUnmatched),
                            };
                            capture_objects.push(CaptureObjectDefinition {
                                class_id,
                                logical_name,
                                attribute_index,
                                data_index,
                            });
                        } else {
                            return Err(DataAccessResult::TypeUnmatched);
                        }
                    }
                    self.capture_objects = capture_objects;
                    Ok(())
                } else {
                    Err(DataAccessResult::TypeUnmatched)
                }
            }
            4 => {
                if let Data::DoubleLongUnsigned(val) = value {
                    self.capture_period = val;
                    Ok(())
                } else {
                    Err(DataAccessResult::TypeUnmatched)
                }
            }
            5 => {
                if let Data::Enum(val) = value {
                    self.sort_method = SortMethod::from_u8(val)?;
                    Ok(())
                } else {
                    Err(DataAccessResult::TypeUnmatched)
                }
            }
            6 => {
                self.sort_object = if value == Data::Null {
                    None
                } else if let Data::Structure(fields) = value {
                    if fields.len() != 4 {
                        return Err(DataAccessResult::TypeUnmatched);
                    }
                    let class_id = match &fields[0] {
                        Data::LongUnsigned(v) => *v,
                        _ => return Err(DataAccessResult::TypeUnmatched),
                    };
                    let logical_name = match &fields[1] {
                        Data::OctetString(bytes) => {
                            if bytes.len() != 6 {
                                return Err(DataAccessResult::TypeUnmatched);
                            }
                            ObisCode::new(
                                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
                            )
                        }
                        _ => return Err(DataAccessResult::TypeUnmatched),
                    };
                    let attribute_index = match &fields[2] {
                        Data::Integer(v) => *v,
                        _ => return Err(DataAccessResult::TypeUnmatched),
                    };
                    let data_index = match &fields[3] {
                        Data::LongUnsigned(v) => *v,
                        _ => return Err(DataAccessResult::TypeUnmatched),
                    };
                    Some(CaptureObjectDefinition {
                        class_id,
                        logical_name,
                        attribute_index,
                        data_index,
                    })
                } else {
                    return Err(DataAccessResult::TypeUnmatched);
                };
                Ok(())
            }
            8 => {
                if let Data::DoubleLongUnsigned(val) = value {
                    self.profile_entries = val;
                    Ok(())
                } else {
                    Err(DataAccessResult::TypeUnmatched)
                }
            }
            _ => Err(DataAccessResult::ObjectUndefined),
        }
    }

    fn invoke_method(
        &mut self,
        id: i8,
        params: Option<Data>,
    ) -> Result<Option<Data>, ActionResult> {
        match id {
            1 => self.reset(params),
            2 => self.capture(params),
            _ => Err(ActionResult::ObjectUndefined),
        }
    }
}

impl ProfileGeneric {
    /// Create a new ProfileGeneric with default FIFO configuration
    ///
    /// ## Parameters
    ///
    /// - `logical_name`: OBIS code for this profile
    /// - `profile_entries`: Maximum number of entries in the buffer
    ///
    /// ## Example
    ///
    /// ```rust
    /// use dlms_cosem::cosem::ProfileGeneric;
    /// use dlms_cosem::ObisCode;
    ///
    /// let profile = ProfileGeneric::new(
    ///     ObisCode::new(1, 0, 99, 1, 0, 255),
    ///     96  // 24 hours × 4 intervals/hour
    /// );
    ///
    /// assert_eq!(profile.profile_entries, 96);
    /// assert_eq!(profile.entries_in_use, 0);
    /// ```
    pub fn new(logical_name: ObisCode, profile_entries: u32) -> Self {
        Self {
            logical_name,
            buffer: VecDeque::new(),
            capture_objects: Vec::new(),
            capture_period: 0,
            sort_method: SortMethod::Fifo,
            sort_object: None,
            entries_in_use: 0,
            profile_entries,
            executed_time: 0,
            use_compact_array_encoding: false,
        }
    }

    /// Create a new ProfileGeneric with FIFO ring buffer
    ///
    /// Convenience constructor for creating a FIFO load profile with
    /// predefined capture objects and period.
    ///
    /// ## Parameters
    ///
    /// - `logical_name`: OBIS code for this profile
    /// - `capture_objects`: Column definitions
    /// - `capture_period`: Capture interval in seconds (0 = event-driven)
    /// - `profile_entries`: Maximum buffer size
    ///
    /// ## Example
    ///
    /// ```rust
    /// extern crate alloc;
    /// use dlms_cosem::cosem::{ProfileGeneric, CaptureObjectDefinition};
    /// use dlms_cosem::ObisCode;
    ///
    /// let clock_def = CaptureObjectDefinition {
    ///     class_id: 8,
    ///     logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
    ///     attribute_index: 2,
    ///     data_index: 0,
    /// };
    ///
    /// let profile = ProfileGeneric::with_fifo(
    ///     ObisCode::new(1, 0, 99, 1, 0, 255),
    ///     vec![clock_def],
    ///     900,  // 15 minutes
    ///     96    // 24 hours
    /// );
    ///
    /// assert_eq!(profile.capture_period, 900);
    /// assert_eq!(profile.capture_objects.len(), 1);
    /// ```
    pub fn with_fifo(
        logical_name: ObisCode,
        capture_objects: Vec<CaptureObjectDefinition>,
        capture_period: u32,
        profile_entries: u32,
    ) -> Self {
        Self {
            logical_name,
            buffer: VecDeque::new(),
            capture_objects,
            capture_period,
            sort_method: SortMethod::Fifo,
            sort_object: None,
            entries_in_use: 0,
            profile_entries,
            executed_time: 0,
            use_compact_array_encoding: false,
        }
    }

    /// Create a new ProfileGeneric with LIFO stack buffer
    ///
    /// Convenience constructor for creating a LIFO (stack-based) profile.
    /// Less common than FIFO, used for specific applications.
    ///
    /// ## Parameters
    ///
    /// - `logical_name`: OBIS code for this profile
    /// - `capture_objects`: Column definitions
    /// - `capture_period`: Capture interval in seconds (0 = event-driven)
    /// - `profile_entries`: Maximum buffer size
    ///
    /// ## Example
    ///
    /// ```rust
    /// use dlms_cosem::cosem::{ProfileGeneric, CaptureObjectDefinition, SortMethod};
    /// use dlms_cosem::ObisCode;
    ///
    /// let clock_def = CaptureObjectDefinition {
    ///     class_id: 8,
    ///     logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
    ///     attribute_index: 2,
    ///     data_index: 0,
    /// };
    ///
    /// let profile = ProfileGeneric::with_lifo(
    ///     ObisCode::new(1, 0, 99, 2, 0, 255),
    ///     vec![clock_def],
    ///     0,   // Event-driven
    ///     100  // Max 100 entries
    /// );
    ///
    /// assert_eq!(profile.sort_method, SortMethod::Lifo);
    /// ```
    pub fn with_lifo(
        logical_name: ObisCode,
        capture_objects: Vec<CaptureObjectDefinition>,
        capture_period: u32,
        profile_entries: u32,
    ) -> Self {
        Self {
            logical_name,
            buffer: VecDeque::new(),
            capture_objects,
            capture_period,
            sort_method: SortMethod::Lifo,
            sort_object: None,
            entries_in_use: 0,
            profile_entries,
            executed_time: 0,
            use_compact_array_encoding: false,
        }
    }

    /// Method 1: Reset buffer
    ///
    /// Clears all entries from the buffer and sets entries_in_use to 0.
    /// The buffer configuration (capture_objects, etc.) is preserved.
    ///
    /// ## Parameters
    ///
    /// - `params`: Ignored (typically Integer(0))
    ///
    /// ## Returns
    ///
    /// - Ok(Some(Data::Integer(0))) on success
    ///
    /// ## Example
    ///
    /// ```rust
    /// extern crate alloc;
    /// use dlms_cosem::cosem::ProfileGeneric;
    /// use dlms_cosem::Data;
    /// # use dlms_cosem::cosem::{CosemObject, SortMethod};
    /// # use dlms_cosem::ObisCode;
    /// # use alloc::collections::VecDeque;
    /// # let mut profile = ProfileGeneric {
    /// #     logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
    /// #     buffer: VecDeque::from(vec![vec![Data::Integer(1)]]),
    /// #     capture_objects: vec![],
    /// #     capture_period: 0,
    /// #     sort_method: SortMethod::Fifo,
    /// #     sort_object: None,
    /// #     entries_in_use: 1,
    /// #     profile_entries: 10,
    /// #     executed_time: 0,
/// #     use_compact_array_encoding: false,
    /// # };
    ///
    /// let result = profile.invoke_method(1, Some(Data::Integer(0)));
    /// assert!(result.is_ok());
    /// assert_eq!(profile.buffer.len(), 0);
    /// assert_eq!(profile.entries_in_use, 0);
    /// assert_eq!(profile.executed_time, 0);
    /// ```
    fn reset(&mut self, _params: Option<Data>) -> Result<Option<Data>, ActionResult> {
        self.buffer.clear();
        self.entries_in_use = 0;
        self.executed_time = 0; // Reset execution timestamp
        Ok(Some(Data::Integer(0))) // Success
    }

    /// Method 2: Capture
    ///
    /// Manually triggers a capture. Reads all objects defined in capture_objects
    /// and adds a new entry to the buffer.
    ///
    /// ## Phase 5.2 Implementation
    ///
    /// In Phase 5.2, this method creates a mock entry with placeholder values.
    /// Real object attribute lookup will be implemented in the integration phase.
    ///
    /// ## Parameters
    ///
    /// - `params`: Ignored (typically Integer(0))
    ///
    /// ## Returns
    ///
    /// - Ok(Some(Data::Integer(0))) on success
    ///
    /// ## Example
    ///
    /// ```rust
    /// extern crate alloc;
    /// # use dlms_cosem::cosem::{ProfileGeneric, CosemObject, CaptureObjectDefinition, SortMethod};
    /// # use dlms_cosem::{Data, ObisCode};
    /// # use alloc::collections::VecDeque;
    ///
    /// let mut profile = ProfileGeneric {
    ///     logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
    ///     buffer: VecDeque::new(),
    ///     capture_objects: vec![
    ///         CaptureObjectDefinition {
    ///             class_id: 8,
    ///             logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
    ///             attribute_index: 2,
    ///             data_index: 0,
    ///         },
    ///     ],
    ///     capture_period: 0,
    ///     sort_method: SortMethod::Fifo,
    ///     sort_object: None,
    ///     entries_in_use: 0,
    ///     profile_entries: 10,
    ///     executed_time: 0,
/// #     use_compact_array_encoding: false,
    /// };
    ///
    /// let result = profile.invoke_method(2, Some(Data::Integer(0)));
    /// assert!(result.is_ok());
    /// assert_eq!(profile.entries_in_use, 1);
    /// ```
    fn capture(&mut self, _params: Option<Data>) -> Result<Option<Data>, ActionResult> {
        // Phase 5.2: Create mock entry matching capture_objects structure
        let mock_entry: Vec<Data> = self
            .capture_objects
            .iter()
            .map(|_obj| {
                // TODO Phase 5.3: Read actual object attributes
                // For now, use placeholder value
                Data::DoubleLongUnsigned(0)
            })
            .collect();

        self.add_entry(mock_entry);
        Ok(Some(Data::Integer(0))) // Success
    }

    /// Add entry to buffer according to sort_method
    ///
    /// Handles FIFO/LIFO ring buffer logic and updates entries_in_use.
    ///
    /// ## Phase 5.2 Implementation
    ///
    /// - FIFO: Fully implemented with O(1) operations using VecDeque
    /// - LIFO: Fully implemented with O(1) operations using VecDeque
    ///
    /// ## Phase 5.3.2 Implementation
    ///
    /// - Largest, Smallest, NearestToZero, FarthestFromZero: Fully implemented with O(n) operations
    ///
    /// ## Performance
    ///
    /// - VecDeque provides O(1) push_back/push_front and pop_front/pop_back for FIFO/LIFO
    /// - Sorted methods use O(n) linear search to find worst entry (acceptable for typical buffer sizes)
    fn add_entry(&mut self, entry: Vec<Data>) {
        match self.sort_method {
            SortMethod::Fifo => {
                // FIFO: Add to back, remove from front (O(1) operations)
                self.buffer.push_back(entry);
                while self.buffer.len() > self.profile_entries as usize {
                    self.buffer.pop_front(); // Remove oldest (first) - O(1)
                }
            }
            SortMethod::Lifo => {
                // LIFO: Add to front, remove from back (O(1) operations)
                self.buffer.push_front(entry);
                while self.buffer.len() > self.profile_entries as usize {
                    self.buffer.pop_back(); // Remove newest (last) - O(1)
                }
            }
            SortMethod::Largest
            | SortMethod::Smallest
            | SortMethod::NearestToZero
            | SortMethod::FarthestFromZero => {
                // Phase 5.3.2: Value-based sorting
                self.add_entry_sorted(entry);
            }
        }
        self.entries_in_use = self.buffer.len() as u32;
    }

    /// Add entry with value-based sorting (Phase 5.3.2)
    ///
    /// For Largest/Smallest/NearestToZero/FarthestFromZero sort methods,
    /// this adds the entry and removes the worst entry if buffer is full.
    ///
    /// ## Complexity
    ///
    /// O(n) where n = buffer size (linear search for worst entry)
    ///
    /// ## Fallback Behavior
    ///
    /// If sort_object is not configured or column not found, falls back to FIFO.
    fn add_entry_sorted(&mut self, entry: Vec<Data>) {
        // Add the new entry
        self.buffer.push_back(entry);

        // If under capacity, we're done
        if self.buffer.len() <= self.profile_entries as usize {
            return;
        }

        // Find and remove the worst entry based on sort_method
        if let Some(worst_idx) = self.find_worst_entry_index() {
            self.buffer.remove(worst_idx);
        } else {
            // Fallback to FIFO if sort_object not configured or not found
            self.buffer.pop_front();
        }
    }

    /// Find the index of the worst entry to remove
    ///
    /// Returns the index of the entry that should be removed when buffer is full,
    /// based on the current sort_method.
    ///
    /// ## Returns
    ///
    /// - Some(index) - Index of the worst entry
    /// - None - If no numeric values found or sort_object not configured
    fn find_worst_entry_index(&self) -> Option<usize> {
        if self.buffer.is_empty() {
            return None;
        }

        // Extract all values with their indices
        let indexed_values: Vec<(usize, f64)> = self
            .buffer
            .iter()
            .enumerate()
            .filter_map(|(idx, entry)| self.extract_sort_value(entry).map(|val| (idx, val)))
            .collect();

        if indexed_values.is_empty() {
            return None; // No numeric values found
        }

        // Find the worst entry based on sort method
        let worst = match self.sort_method {
            SortMethod::Largest => {
                // Worst = smallest value
                indexed_values
                    .iter()
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(core::cmp::Ordering::Equal))
            }
            SortMethod::Smallest => {
                // Worst = largest value
                indexed_values
                    .iter()
                    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(core::cmp::Ordering::Equal))
            }
            SortMethod::NearestToZero => {
                // Worst = farthest from zero
                indexed_values.iter().max_by(|a, b| {
                    a.1.abs().partial_cmp(&b.1.abs()).unwrap_or(core::cmp::Ordering::Equal)
                })
            }
            SortMethod::FarthestFromZero => {
                // Worst = nearest to zero
                indexed_values.iter().min_by(|a, b| {
                    a.1.abs().partial_cmp(&b.1.abs()).unwrap_or(core::cmp::Ordering::Equal)
                })
            }
            _ => return None,
        };

        worst.map(|(idx, _)| *idx)
    }

    /// Extract the numeric value from the sort_object column of an entry
    ///
    /// ## Returns
    ///
    /// - Some(value) - The numeric value from the sort column
    /// - None - If sort_object not configured, column not found, or value not numeric
    fn extract_sort_value(&self, entry: &[Data]) -> Option<f64> {
        let column_idx = self.find_sort_column_index()?;

        if column_idx >= entry.len() {
            return None; // Column index out of bounds
        }

        extract_numeric_value(&entry[column_idx])
    }

    /// Find the index of the sort_object column in capture_objects
    ///
    /// ## Returns
    ///
    /// - Some(index) - The column index
    /// - None - If sort_object is None or not found in capture_objects
    fn find_sort_column_index(&self) -> Option<usize> {
        let sort_obj = self.sort_object.as_ref()?;

        self.capture_objects.iter().position(|obj| obj == sort_obj)
    }
}

/// Extract numeric value from Data for comparison
///
/// Supports all DLMS numeric types and converts to f64 for uniform comparison.
///
/// ## Returns
///
/// - Some(value) - For all numeric types (Integer, LongUnsigned, Float64, etc.)
/// - None - For non-numeric types (OctetString, DateTime, etc.)
///
/// ## Example
///
/// ```rust
/// # use dlms_cosem::Data;
/// # use dlms_cosem::cosem::profile_generic::extract_numeric_value;
/// assert_eq!(extract_numeric_value(&Data::Integer(42)), Some(42.0));
/// assert_eq!(extract_numeric_value(&Data::Float64(3.14)), Some(3.14));
/// assert_eq!(extract_numeric_value(&Data::OctetString(vec![1, 2])), None);
/// ```
pub fn extract_numeric_value(data: &Data) -> Option<f64> {
    match data {
        Data::Integer(v) => Some(*v as f64),
        Data::Unsigned(v) => Some(*v as f64),
        Data::Long(v) => Some(*v as f64),
        Data::LongUnsigned(v) => Some(*v as f64),
        Data::DoubleLong(v) => Some(*v as f64),
        Data::DoubleLongUnsigned(v) => Some(*v as f64),
        Data::Long64(v) => Some(*v as f64),
        Data::Long64Unsigned(v) => Some(*v as f64),
        Data::Float32(v) => Some(*v as f64),
        Data::Float64(v) => Some(*v),
        // Non-numeric types return None
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== STEP 1: Structure Tests (8 tests) =====

    #[test]
    fn test_capture_object_definition_creation() {
        let def = CaptureObjectDefinition {
            class_id: 8,
            logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };
        assert_eq!(def.class_id, 8);
        assert_eq!(def.logical_name, ObisCode::new(0, 0, 1, 0, 0, 255));
        assert_eq!(def.attribute_index, 2);
        assert_eq!(def.data_index, 0);
    }

    #[test]
    fn test_capture_object_definition_clone() {
        let def1 = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };
        let def2 = def1.clone();
        assert_eq!(def1, def2);
    }

    #[test]
    fn test_capture_object_definition_partial_eq() {
        let def1 = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };
        let def2 = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };
        let def3 = CaptureObjectDefinition {
            class_id: 4,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };
        assert_eq!(def1, def2);
        assert_ne!(def1, def3);
    }

    #[test]
    fn test_sort_method_enum_values() {
        assert_eq!(SortMethod::Fifo as u8, 1);
        assert_eq!(SortMethod::Lifo as u8, 2);
        assert_eq!(SortMethod::Largest as u8, 3);
        assert_eq!(SortMethod::Smallest as u8, 4);
        assert_eq!(SortMethod::NearestToZero as u8, 5);
        assert_eq!(SortMethod::FarthestFromZero as u8, 6);
    }

    #[test]
    fn test_sort_method_from_u8_valid() {
        assert_eq!(SortMethod::from_u8(1).unwrap(), SortMethod::Fifo);
        assert_eq!(SortMethod::from_u8(2).unwrap(), SortMethod::Lifo);
        assert_eq!(SortMethod::from_u8(3).unwrap(), SortMethod::Largest);
        assert_eq!(SortMethod::from_u8(4).unwrap(), SortMethod::Smallest);
        assert_eq!(SortMethod::from_u8(5).unwrap(), SortMethod::NearestToZero);
        assert_eq!(SortMethod::from_u8(6).unwrap(), SortMethod::FarthestFromZero);
    }

    #[test]
    fn test_sort_method_from_u8_invalid() {
        assert!(SortMethod::from_u8(0).is_err());
        assert!(SortMethod::from_u8(7).is_err());
        assert!(SortMethod::from_u8(255).is_err());
    }

    #[test]
    fn test_profile_generic_new() {
        let profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 96);

        assert_eq!(profile.logical_name, ObisCode::new(1, 0, 99, 1, 0, 255));
        assert_eq!(profile.buffer.len(), 0);
        assert_eq!(profile.capture_period, 0);
        assert_eq!(profile.sort_method, SortMethod::Fifo);
        assert_eq!(profile.entries_in_use, 0);
        assert_eq!(profile.profile_entries, 96);
        assert_eq!(profile.executed_time, 0);
    }

    #[test]
    fn test_profile_generic_with_fifo() {
        let clock_def = CaptureObjectDefinition {
            class_id: 8,
            logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let profile = ProfileGeneric::with_fifo(
            ObisCode::new(1, 0, 99, 1, 0, 255),
            vec![clock_def.clone()],
            900,
            96,
        );

        assert_eq!(profile.logical_name, ObisCode::new(1, 0, 99, 1, 0, 255));
        assert_eq!(profile.capture_period, 900);
        assert_eq!(profile.sort_method, SortMethod::Fifo);
        assert_eq!(profile.profile_entries, 96);
        assert_eq!(profile.capture_objects.len(), 1);
        assert_eq!(profile.capture_objects[0], clock_def);
    }

    #[test]
    fn test_profile_generic_with_lifo() {
        let clock_def = CaptureObjectDefinition {
            class_id: 8,
            logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let profile = ProfileGeneric::with_lifo(
            ObisCode::new(1, 0, 99, 2, 0, 255),
            vec![clock_def.clone()],
            0,
            100,
        );

        assert_eq!(profile.logical_name, ObisCode::new(1, 0, 99, 2, 0, 255));
        assert_eq!(profile.capture_period, 0);
        assert_eq!(profile.sort_method, SortMethod::Lifo);
        assert_eq!(profile.profile_entries, 100);
        assert_eq!(profile.capture_objects.len(), 1);
        assert_eq!(profile.capture_objects[0], clock_def);
    }

    #[test]
    fn test_profile_generic_class_id_and_version() {
        let profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        assert_eq!(profile.class_id(), 7);
        assert_eq!(profile.version(), 0);
    }

    // ===== STEP 2: Attribute Access Tests (10 tests) =====

    #[test]
    fn test_get_attribute_1_logical_name() {
        let profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        let result = profile.get_attribute(1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Data::OctetString(vec![1, 0, 99, 1, 0, 255]));
    }

    #[test]
    fn test_get_attribute_2_buffer_empty() {
        let profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        let result = profile.get_attribute(2);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Data::Structure(vec![]));
    }

    #[test]
    fn test_get_attribute_2_buffer_with_entries() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        profile.buffer.push_back(vec![Data::DoubleLongUnsigned(100)]);
        profile.buffer.push_back(vec![Data::DoubleLongUnsigned(200)]);
        profile.entries_in_use = 2;
        let result = profile.get_attribute(2);
        assert!(result.is_ok());
        if let Data::Array(rows) = result.unwrap() {
            assert_eq!(rows.len(), 2);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_get_attribute_3_capture_objects() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        profile.capture_objects = vec![
            CaptureObjectDefinition {
                class_id: 8,
                logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
                attribute_index: 2,
                data_index: 0,
            },
            CaptureObjectDefinition {
                class_id: 3,
                logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
                attribute_index: 2,
                data_index: 0,
            },
        ];
        let result = profile.get_attribute(3);
        assert!(result.is_ok());
        if let Data::Structure(objects) = result.unwrap() {
            assert_eq!(objects.len(), 2);
            // First object
            if let Data::Structure(fields) = &objects[0] {
                assert_eq!(fields[0], Data::LongUnsigned(8));
                assert_eq!(fields[1], Data::OctetString(vec![0, 0, 1, 0, 0, 255]));
                assert_eq!(fields[2], Data::Integer(2));
                assert_eq!(fields[3], Data::LongUnsigned(0));
            } else {
                panic!("Expected Structure");
            }
        } else {
            panic!("Expected Structure");
        }
    }

    #[test]
    fn test_get_attribute_4_capture_period() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        profile.capture_period = 900;
        let result = profile.get_attribute(4);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Data::DoubleLongUnsigned(900));
    }

    #[test]
    fn test_get_attribute_5_sort_method() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        profile.sort_method = SortMethod::Lifo;
        let result = profile.get_attribute(5);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Data::Enum(2)); // LIFO = 2
    }

    #[test]
    fn test_get_attribute_6_sort_object_none() {
        let profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        let result = profile.get_attribute(6);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Data::Null);
    }

    #[test]
    fn test_get_attribute_6_sort_object_some() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        profile.sort_method = SortMethod::Largest;
        profile.sort_object = Some(CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        });
        let result = profile.get_attribute(6);
        assert!(result.is_ok());
        if let Data::Structure(fields) = result.unwrap() {
            assert_eq!(fields.len(), 4);
            assert_eq!(fields[0], Data::LongUnsigned(3));
            assert_eq!(fields[1], Data::OctetString(vec![1, 0, 1, 8, 0, 255]));
            assert_eq!(fields[2], Data::Integer(2));
            assert_eq!(fields[3], Data::LongUnsigned(0));
        } else {
            panic!("Expected Structure");
        }
    }

    #[test]
    fn test_get_attribute_7_entries_in_use() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        profile.buffer.push_back(vec![Data::Integer(1)]);
        profile.buffer.push_back(vec![Data::Integer(2)]);
        profile.entries_in_use = 2;
        let result = profile.get_attribute(7);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Data::DoubleLongUnsigned(2));
    }

    #[test]
    fn test_get_attribute_8_profile_entries() {
        let profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 96);
        let result = profile.get_attribute(8);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Data::DoubleLongUnsigned(96));
    }

    #[test]
    fn test_get_attribute_invalid_id() {
        let profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        assert_eq!(profile.get_attribute(0), Err(DataAccessResult::ObjectUndefined));
        assert_eq!(profile.get_attribute(9), Err(DataAccessResult::ObjectUndefined));
    }

    #[test]
    fn test_set_attribute_readonly_logical_name() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        let result = profile.set_attribute(1, Data::OctetString(vec![2, 0, 99, 1, 0, 255]));
        assert_eq!(result, Err(DataAccessResult::ReadWriteDenied));
    }

    #[test]
    fn test_set_attribute_readonly_buffer() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        let result = profile.set_attribute(2, Data::Structure(vec![]));
        assert_eq!(result, Err(DataAccessResult::ReadWriteDenied));
    }

    #[test]
    fn test_set_attribute_readonly_entries_in_use() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        let result = profile.set_attribute(7, Data::DoubleLongUnsigned(5));
        assert_eq!(result, Err(DataAccessResult::ReadWriteDenied));
    }

    #[test]
    fn test_set_attribute_3_capture_objects() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        let new_objects = Data::Structure(vec![Data::Structure(vec![
            Data::LongUnsigned(8),
            Data::OctetString(vec![0, 0, 1, 0, 0, 255]),
            Data::Integer(2),
            Data::LongUnsigned(0),
        ])]);
        let result = profile.set_attribute(3, new_objects);
        assert!(result.is_ok());
        assert_eq!(profile.capture_objects.len(), 1);
        assert_eq!(profile.capture_objects[0].class_id, 8);
        assert_eq!(profile.capture_objects[0].logical_name, ObisCode::new(0, 0, 1, 0, 0, 255));
        assert_eq!(profile.capture_objects[0].attribute_index, 2);
        assert_eq!(profile.capture_objects[0].data_index, 0);
    }

    #[test]
    fn test_set_attribute_3_type_mismatch() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        // Wrong type (not Structure)
        let result = profile.set_attribute(3, Data::Integer(123));
        assert_eq!(result, Err(DataAccessResult::TypeUnmatched));
    }

    #[test]
    fn test_set_attribute_4_capture_period() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        let result = profile.set_attribute(4, Data::DoubleLongUnsigned(900));
        assert!(result.is_ok());
        assert_eq!(profile.capture_period, 900);
    }

    #[test]
    fn test_set_attribute_5_sort_method() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        let result = profile.set_attribute(5, Data::Enum(2));
        assert!(result.is_ok());
        assert_eq!(profile.sort_method, SortMethod::Lifo);
    }

    #[test]
    fn test_set_attribute_5_invalid_enum() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        let result = profile.set_attribute(5, Data::Enum(99));
        assert_eq!(result, Err(DataAccessResult::TypeUnmatched));
    }

    #[test]
    fn test_set_attribute_6_sort_object_none() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        profile.sort_object = Some(CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        });
        let result = profile.set_attribute(6, Data::Null);
        assert!(result.is_ok());
        assert!(profile.sort_object.is_none());
    }

    #[test]
    fn test_set_attribute_6_sort_object_some() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        let new_sort_obj = Data::Structure(vec![
            Data::LongUnsigned(3),
            Data::OctetString(vec![1, 0, 1, 8, 0, 255]),
            Data::Integer(2),
            Data::LongUnsigned(0),
        ]);
        let result = profile.set_attribute(6, new_sort_obj);
        assert!(result.is_ok());
        assert!(profile.sort_object.is_some());
        let sort_obj = profile.sort_object.unwrap();
        assert_eq!(sort_obj.class_id, 3);
        assert_eq!(sort_obj.logical_name, ObisCode::new(1, 0, 1, 8, 0, 255));
        assert_eq!(sort_obj.attribute_index, 2);
        assert_eq!(sort_obj.data_index, 0);
    }

    #[test]
    fn test_set_attribute_8_profile_entries() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        let result = profile.set_attribute(8, Data::DoubleLongUnsigned(200));
        assert!(result.is_ok());
        assert_eq!(profile.profile_entries, 200);
    }

    #[test]
    fn test_set_attribute_invalid_id() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        assert_eq!(
            profile.set_attribute(0, Data::Integer(0)),
            Err(DataAccessResult::ObjectUndefined)
        );
        assert_eq!(
            profile.set_attribute(9, Data::Integer(0)),
            Err(DataAccessResult::ObjectUndefined)
        );
    }

    // ===== STEP 3: Buffer Management Tests (15 tests) =====

    #[test]
    fn test_add_entry_fifo_empty_buffer() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        profile.add_entry(vec![Data::DoubleLongUnsigned(100)]);
        assert_eq!(profile.buffer.len(), 1);
        assert_eq!(profile.entries_in_use, 1);
        // Test reset
        let _ = profile.invoke_method(1, None);
        assert_eq!(profile.buffer.len(), 0);
        assert_eq!(profile.entries_in_use, 0);
        assert_eq!(profile.executed_time, 0);
    }

    #[test]
    fn test_add_entry_fifo_multiple_entries() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        profile.add_entry(vec![Data::DoubleLongUnsigned(100)]);
        profile.add_entry(vec![Data::DoubleLongUnsigned(200)]);
        profile.add_entry(vec![Data::DoubleLongUnsigned(300)]);
        assert_eq!(profile.buffer.len(), 3);
        assert_eq!(profile.entries_in_use, 3);
        assert_eq!(profile.buffer[0], vec![Data::DoubleLongUnsigned(100)]);
        assert_eq!(profile.buffer[1], vec![Data::DoubleLongUnsigned(200)]);
        assert_eq!(profile.buffer[2], vec![Data::DoubleLongUnsigned(300)]);
    }

    #[test]
    fn test_add_entry_fifo_overflow_removes_oldest() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 3);
        profile.add_entry(vec![Data::DoubleLongUnsigned(100)]);
        profile.add_entry(vec![Data::DoubleLongUnsigned(200)]);
        profile.add_entry(vec![Data::DoubleLongUnsigned(300)]);
        // Buffer full: [100, 200, 300]
        profile.add_entry(vec![Data::DoubleLongUnsigned(400)]);
        // Should remove 100, add 400: [200, 300, 400]
        assert_eq!(profile.buffer.len(), 3);
        assert_eq!(profile.entries_in_use, 3);
        assert_eq!(profile.buffer[0], vec![Data::DoubleLongUnsigned(200)]);
        assert_eq!(profile.buffer[1], vec![Data::DoubleLongUnsigned(300)]);
        assert_eq!(profile.buffer[2], vec![Data::DoubleLongUnsigned(400)]);
    }

    #[test]
    fn test_add_entry_fifo_boundary_exactly_max() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 2);
        profile.add_entry(vec![Data::DoubleLongUnsigned(100)]);
        profile.add_entry(vec![Data::DoubleLongUnsigned(200)]);
        // Exactly at max capacity
        assert_eq!(profile.buffer.len(), 2);
        assert_eq!(profile.entries_in_use, 2);
    }

    #[test]
    fn test_add_entry_lifo_empty_buffer() {
        let mut profile =
            ProfileGeneric::with_lifo(ObisCode::new(1, 0, 99, 1, 0, 255), vec![], 0, 10);
        profile.add_entry(vec![Data::DoubleLongUnsigned(100)]);
        assert_eq!(profile.buffer.len(), 1);
        assert_eq!(profile.entries_in_use, 1);
        // Test reset
        let _ = profile.invoke_method(1, None);
        assert_eq!(profile.buffer.len(), 0);
        assert_eq!(profile.entries_in_use, 0);
        assert_eq!(profile.executed_time, 0);
    }

    #[test]
    fn test_add_entry_lifo_multiple_entries() {
        let mut profile =
            ProfileGeneric::with_lifo(ObisCode::new(1, 0, 99, 1, 0, 255), vec![], 0, 10);
        profile.add_entry(vec![Data::DoubleLongUnsigned(100)]);
        profile.add_entry(vec![Data::DoubleLongUnsigned(200)]);
        profile.add_entry(vec![Data::DoubleLongUnsigned(300)]);
        // LIFO: newest at index 0
        assert_eq!(profile.buffer.len(), 3);
        assert_eq!(profile.entries_in_use, 3);
        assert_eq!(profile.buffer[0], vec![Data::DoubleLongUnsigned(300)]);
        assert_eq!(profile.buffer[1], vec![Data::DoubleLongUnsigned(200)]);
        assert_eq!(profile.buffer[2], vec![Data::DoubleLongUnsigned(100)]);
    }

    #[test]
    fn test_add_entry_lifo_overflow_removes_newest() {
        let mut profile =
            ProfileGeneric::with_lifo(ObisCode::new(1, 0, 99, 1, 0, 255), vec![], 0, 3);
        profile.add_entry(vec![Data::DoubleLongUnsigned(100)]);
        profile.add_entry(vec![Data::DoubleLongUnsigned(200)]);
        profile.add_entry(vec![Data::DoubleLongUnsigned(300)]);
        // Buffer: [300, 200, 100]
        profile.add_entry(vec![Data::DoubleLongUnsigned(400)]);
        // Insert 400 at start, remove from end (oldest 100): [400, 300, 200]
        assert_eq!(profile.buffer.len(), 3);
        assert_eq!(profile.entries_in_use, 3);
        assert_eq!(profile.buffer[0], vec![Data::DoubleLongUnsigned(400)]);
        assert_eq!(profile.buffer[1], vec![Data::DoubleLongUnsigned(300)]);
        assert_eq!(profile.buffer[2], vec![Data::DoubleLongUnsigned(200)]);
    }

    #[test]
    fn test_add_entry_single_entry_buffer() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 1);
        profile.add_entry(vec![Data::DoubleLongUnsigned(100)]);
        assert_eq!(profile.buffer.len(), 1);
        profile.add_entry(vec![Data::DoubleLongUnsigned(200)]);
        // Should replace: [200]
        assert_eq!(profile.buffer.len(), 1);
        assert_eq!(profile.entries_in_use, 1);
        assert_eq!(profile.buffer[0], vec![Data::DoubleLongUnsigned(200)]);
    }

    #[test]
    fn test_add_entry_multi_column() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        let entry = vec![Data::OctetString(vec![1, 2, 3]), Data::DoubleLongUnsigned(12345)];
        profile.add_entry(entry.clone());
        assert_eq!(profile.buffer.len(), 1);
        assert_eq!(profile.buffer[0], entry);
    }

    #[test]
    fn test_buffer_clear_on_reset() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        profile.buffer.push_back(vec![Data::DoubleLongUnsigned(100)]);
        profile.buffer.push_back(vec![Data::DoubleLongUnsigned(200)]);
        profile.entries_in_use = 2;
        let result = profile.reset(Some(Data::Integer(0)));
        assert!(result.is_ok());
        assert_eq!(profile.buffer.len(), 0);
        assert_eq!(profile.entries_in_use, 0);
    }

    #[test]
    fn test_buffer_resize_profile_entries() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        profile.buffer.push_back(vec![Data::DoubleLongUnsigned(100)]);
        profile.buffer.push_back(vec![Data::DoubleLongUnsigned(200)]);
        profile.buffer.push_back(vec![Data::DoubleLongUnsigned(300)]);
        profile.entries_in_use = 3;
        // Reduce max size - buffer not trimmed until next add_entry()
        profile.profile_entries = 2;
        assert_eq!(profile.buffer.len(), 3); // Still 3 entries

        // Add new entry - should trigger overflow and trim
        profile.add_entry(vec![Data::DoubleLongUnsigned(400)]);
        // Buffer should have 2 entries (removed oldest)
        assert_eq!(profile.buffer.len(), 2);
        assert_eq!(profile.entries_in_use, 2);
    }

    #[test]
    fn test_fifo_preserves_order() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        for i in 1..=5 {
            profile.add_entry(vec![Data::DoubleLongUnsigned(i * 100)]);
        }
        // Buffer: [100, 200, 300, 400, 500]
        assert_eq!(profile.buffer.len(), 5);
        for i in 0..5 {
            assert_eq!(profile.buffer[i], vec![Data::DoubleLongUnsigned((i as u32 + 1) * 100)]);
        }
    }

    #[test]
    fn test_advanced_sort_methods_default_to_fifo() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 3);
        profile.sort_method = SortMethod::Largest;
        profile.add_entry(vec![Data::DoubleLongUnsigned(100)]);
        profile.add_entry(vec![Data::DoubleLongUnsigned(200)]);
        profile.add_entry(vec![Data::DoubleLongUnsigned(300)]);
        profile.add_entry(vec![Data::DoubleLongUnsigned(400)]);
        // Should behave like FIFO (oldest removed): [200, 300, 400]
        assert_eq!(profile.buffer.len(), 3);
        assert_eq!(profile.buffer[0], vec![Data::DoubleLongUnsigned(200)]);
        assert_eq!(profile.buffer[1], vec![Data::DoubleLongUnsigned(300)]);
        assert_eq!(profile.buffer[2], vec![Data::DoubleLongUnsigned(400)]);
    }

    // ===== STEP 4: Method Invocation Tests (10 tests) =====

    #[test]
    fn test_method_reset_clears_buffer() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        profile.buffer.push_back(vec![Data::DoubleLongUnsigned(100)]);
        profile.buffer.push_back(vec![Data::DoubleLongUnsigned(200)]);
        profile.buffer.push_back(vec![Data::DoubleLongUnsigned(300)]);
        profile.entries_in_use = 3;
        let result = profile.invoke_method(1, Some(Data::Integer(0)));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Data::Integer(0)));
        assert_eq!(profile.buffer.len(), 0);
        assert_eq!(profile.entries_in_use, 0);
    }

    #[test]
    fn test_method_reset_preserves_configuration() {
        let capture_def = CaptureObjectDefinition {
            class_id: 8,
            logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };
        let mut profile = ProfileGeneric::with_fifo(
            ObisCode::new(1, 0, 99, 1, 0, 255),
            vec![capture_def.clone()],
            900,
            96,
        );
        let _ = profile.invoke_method(1, None);
        // Configuration preserved
        assert_eq!(profile.capture_objects.len(), 1);
        assert_eq!(profile.capture_period, 900);
        assert_eq!(profile.sort_method, SortMethod::Fifo);
        assert_eq!(profile.profile_entries, 96);
    }

    #[test]
    fn test_method_reset_empty_buffer() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        let result = profile.invoke_method(1, Some(Data::Integer(0)));
        assert!(result.is_ok());
        assert_eq!(profile.buffer.len(), 0);
        assert_eq!(profile.entries_in_use, 0);
    }

    #[test]
    fn test_method_capture_adds_entry_fifo() {
        let mut profile = ProfileGeneric::with_fifo(
            ObisCode::new(1, 0, 99, 1, 0, 255),
            vec![
                CaptureObjectDefinition {
                    class_id: 8,
                    logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
                    attribute_index: 2,
                    data_index: 0,
                },
                CaptureObjectDefinition {
                    class_id: 3,
                    logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
                    attribute_index: 2,
                    data_index: 0,
                },
            ],
            0,
            10,
        );
        let result = profile.invoke_method(2, Some(Data::Integer(0)));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Data::Integer(0)));
        assert_eq!(profile.buffer.len(), 1);
        assert_eq!(profile.entries_in_use, 1);
        // Entry should have 2 columns (matching capture_objects)
        assert_eq!(profile.buffer[0].len(), 2);
    }

    #[test]
    fn test_method_capture_adds_entry_lifo() {
        let mut profile = ProfileGeneric::with_lifo(
            ObisCode::new(1, 0, 99, 1, 0, 255),
            vec![CaptureObjectDefinition {
                class_id: 8,
                logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
                attribute_index: 2,
                data_index: 0,
            }],
            0,
            10,
        );
        let result = profile.invoke_method(2, None);
        assert!(result.is_ok());
        assert_eq!(profile.buffer.len(), 1);
        assert_eq!(profile.entries_in_use, 1);
        assert_eq!(profile.buffer[0], vec![Data::DoubleLongUnsigned(0)]);
    }

    #[test]
    fn test_power_quality_event_log() {
        let mut profile = ProfileGeneric::with_fifo(
            ObisCode::new(0, 0, 99, 98, 4, 255),
            vec![
                CaptureObjectDefinition {
                    class_id: 8,
                    logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
                    attribute_index: 2,
                    data_index: 0,
                },
                CaptureObjectDefinition {
                    class_id: 1,
                    logical_name: ObisCode::new(0, 0, 96, 11, 3, 255),
                    attribute_index: 2,
                    data_index: 0,
                },
                CaptureObjectDefinition {
                    class_id: 3,
                    logical_name: ObisCode::new(1, 0, 32, 7, 0, 255),
                    attribute_index: 2,
                    data_index: 0,
                },
                CaptureObjectDefinition {
                    class_id: 3,
                    logical_name: ObisCode::new(1, 0, 52, 7, 0, 255),
                    attribute_index: 2,
                    data_index: 0,
                },
                CaptureObjectDefinition {
                    class_id: 3,
                    logical_name: ObisCode::new(1, 0, 72, 7, 0, 255),
                    attribute_index: 2,
                    data_index: 0,
                },
            ],
            0,
            200,
        );
        let _ = profile.invoke_method(2, Some(Data::Integer(0)));
        // Entry should have 5 columns (matching capture_objects)
        assert_eq!(profile.buffer[0].len(), 5);
    }

    #[test]
    fn test_method_capture_multiple_times() {
        let mut profile = ProfileGeneric::with_fifo(
            ObisCode::new(1, 0, 99, 1, 0, 255),
            vec![CaptureObjectDefinition {
                class_id: 8,
                logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
                attribute_index: 2,
                data_index: 0,
            }],
            0,
            10,
        );
        for _ in 0..5 {
            let result = profile.invoke_method(2, Some(Data::Integer(0)));
            assert!(result.is_ok());
        }
        assert_eq!(profile.buffer.len(), 5);
        assert_eq!(profile.entries_in_use, 5);
    }

    #[test]
    fn test_method_capture_with_overflow() {
        let mut profile = ProfileGeneric::with_fifo(
            ObisCode::new(1, 0, 99, 1, 0, 255),
            vec![CaptureObjectDefinition {
                class_id: 8,
                logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
                attribute_index: 2,
                data_index: 0,
            }],
            0,
            2,
        );
        // Add 5 entries to buffer with max 2
        for _ in 0..5 {
            let _ = profile.invoke_method(2, Some(Data::Integer(0)));
        }
        // Should have exactly 2 entries (FIFO)
        assert_eq!(profile.buffer.len(), 2);
        assert_eq!(profile.entries_in_use, 2);
    }

    #[test]
    fn test_method_capture_respects_capture_objects_structure() {
        let mut profile = ProfileGeneric::with_fifo(
            ObisCode::new(1, 0, 99, 1, 0, 255),
            vec![
                CaptureObjectDefinition {
                    class_id: 8,
                    logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
                    attribute_index: 2,
                    data_index: 0,
                },
                CaptureObjectDefinition {
                    class_id: 1,
                    logical_name: ObisCode::new(0, 0, 96, 7, 21, 255),
                    attribute_index: 2,
                    data_index: 0,
                },
            ],
            0,
            10,
        );
        let _ = profile.invoke_method(2, Some(Data::Integer(0)));
        // Entry should have 2 columns (matching capture_objects)
        assert_eq!(profile.buffer[0].len(), 2);
    }

    #[test]
    fn test_method_reset_then_capture() {
        let mut profile = ProfileGeneric::with_fifo(
            ObisCode::new(1, 0, 99, 1, 0, 255),
            vec![CaptureObjectDefinition {
                class_id: 8,
                logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
                attribute_index: 2,
                data_index: 0,
            }],
            0,
            10,
        );
        // Reset
        let _ = profile.invoke_method(1, Some(Data::Integer(0)));
        assert_eq!(profile.buffer.len(), 0);
        // Capture
        let _ = profile.invoke_method(2, Some(Data::Integer(0)));
        assert_eq!(profile.buffer.len(), 1);
        assert_eq!(profile.entries_in_use, 1);
    }

    // ===== STEP 5: Real-World Examples (8 tests) =====

    #[test]
    fn test_load_profile_15min_energy() {
        let mut profile = ProfileGeneric::with_fifo(
            ObisCode::new(1, 0, 99, 1, 0, 255),
            vec![
                CaptureObjectDefinition {
                    class_id: 8,
                    logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
                    attribute_index: 2,
                    data_index: 0,
                },
                CaptureObjectDefinition {
                    class_id: 3,
                    logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
                    attribute_index: 2,
                    data_index: 0,
                },
            ],
            900,
            96,
        );
        // Simulate one day of captures
        for _ in 0..96 {
            let _ = profile.invoke_method(2, Some(Data::Integer(0)));
        }
        assert_eq!(profile.buffer.len(), 96);
        assert_eq!(profile.entries_in_use, 96);
    }

    #[test]
    fn test_event_log_tamper_events() {
        let mut profile = ProfileGeneric::with_fifo(
            ObisCode::new(0, 0, 99, 98, 0, 255),
            vec![
                CaptureObjectDefinition {
                    class_id: 8,
                    logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
                    attribute_index: 2,
                    data_index: 0,
                },
                CaptureObjectDefinition {
                    class_id: 1,
                    logical_name: ObisCode::new(0, 0, 96, 11, 0, 255),
                    attribute_index: 2,
                    data_index: 0,
                },
            ],
            0,
            100,
        );
        // Simulate 10 tamper events
        for _ in 0..10 {
            let _ = profile.invoke_method(2, Some(Data::Integer(0)));
        }
        assert_eq!(profile.buffer.len(), 10);
        assert_eq!(profile.entries_in_use, 10);
        assert_eq!(profile.capture_period, 0); // Event-driven
    }

    #[test]
    fn test_multi_column_load_profile() {
        let mut profile = ProfileGeneric::with_fifo(
            ObisCode::new(1, 0, 99, 1, 0, 255),
            vec![
                CaptureObjectDefinition {
                    class_id: 8,
                    logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
                    attribute_index: 2,
                    data_index: 0,
                },
                CaptureObjectDefinition {
                    class_id: 3,
                    logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
                    attribute_index: 2,
                    data_index: 0,
                },
                CaptureObjectDefinition {
                    class_id: 3,
                    logical_name: ObisCode::new(1, 0, 2, 8, 0, 255),
                    attribute_index: 2,
                    data_index: 0,
                },
                CaptureObjectDefinition {
                    class_id: 3,
                    logical_name: ObisCode::new(1, 0, 3, 8, 0, 255),
                    attribute_index: 2,
                    data_index: 0,
                },
                CaptureObjectDefinition {
                    class_id: 3,
                    logical_name: ObisCode::new(1, 0, 4, 8, 0, 255),
                    attribute_index: 2,
                    data_index: 0,
                },
            ],
            900,
            96,
        );
        let _ = profile.invoke_method(2, Some(Data::Integer(0)));
        assert_eq!(profile.buffer.len(), 1);
        assert_eq!(profile.buffer[0].len(), 5); // 5 columns
    }

    #[test]
    fn test_hourly_demand_profile() {
        let mut profile = ProfileGeneric::with_fifo(
            ObisCode::new(1, 0, 99, 3, 0, 255),
            vec![
                CaptureObjectDefinition {
                    class_id: 8,
                    logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
                    attribute_index: 2,
                    data_index: 0,
                },
                CaptureObjectDefinition {
                    class_id: 5,
                    logical_name: ObisCode::new(1, 0, 1, 6, 0, 255),
                    attribute_index: 2,
                    data_index: 0,
                },
            ],
            3600,
            24,
        );
        for _ in 0..24 {
            let _ = profile.invoke_method(2, Some(Data::Integer(0)));
        }
        assert_eq!(profile.buffer.len(), 24);
        assert_eq!(profile.entries_in_use, 24);
    }

    #[test]
    fn test_single_column_timestamp_only() {
        let mut profile = ProfileGeneric::with_fifo(
            ObisCode::new(0, 0, 99, 98, 1, 255),
            vec![CaptureObjectDefinition {
                class_id: 8,
                logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
                attribute_index: 2,
                data_index: 0,
            }],
            0,
            50,
        );
        let _ = profile.invoke_method(2, Some(Data::Integer(0)));
        assert_eq!(profile.buffer.len(), 1);
        assert_eq!(profile.buffer[0].len(), 1); // Single column
    }

    #[test]
    fn test_billing_profile_monthly() {
        let mut profile = ProfileGeneric::with_fifo(
            ObisCode::new(1, 0, 98, 1, 0, 255),
            vec![
                CaptureObjectDefinition {
                    class_id: 8,
                    logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
                    attribute_index: 2,
                    data_index: 0,
                },
                CaptureObjectDefinition {
                    class_id: 5,
                    logical_name: ObisCode::new(1, 0, 1, 6, 0, 255),
                    attribute_index: 2,
                    data_index: 0,
                },
            ],
            2592000,
            12,
        );
        // Simulate 12 months
        for _ in 0..12 {
            let _ = profile.invoke_method(2, Some(Data::Integer(0)));
        }
        assert_eq!(profile.buffer.len(), 12);
        assert_eq!(profile.entries_in_use, 12);
    }

    #[test]
    fn test_load_profile_with_reset() {
        let mut profile = ProfileGeneric::with_fifo(
            ObisCode::new(1, 0, 99, 1, 0, 255),
            vec![CaptureObjectDefinition {
                class_id: 8,
                logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
                attribute_index: 2,
                data_index: 0,
            }],
            0,
            10,
        );
        // Capture some data
        for _ in 0..50 {
            let _ = profile.invoke_method(2, Some(Data::Integer(0)));
        }
        assert_eq!(profile.buffer.len(), 10); // FIFO: max is profile_entries
        // Reset
        let _ = profile.invoke_method(1, Some(Data::Integer(0)));
        assert_eq!(profile.buffer.len(), 0);
        assert_eq!(profile.entries_in_use, 0);
        // Continue capturing
        for _ in 0..20 {
            let _ = profile.invoke_method(2, Some(Data::Integer(0)));
        }
        assert_eq!(profile.buffer.len(), 10);
        assert_eq!(profile.entries_in_use, 10);
    }

    // ===== STEP 6: Edge Cases (5 tests) =====

    #[test]
    fn test_empty_capture_objects() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        // Capture should create empty entries (no columns)
        let _ = profile.invoke_method(2, Some(Data::Integer(0)));
        assert_eq!(profile.buffer.len(), 1);
        assert_eq!(profile.buffer[0].len(), 0); // No columns captured
    }

    #[test]
    fn test_profile_entries_zero() {
        let mut profile = ProfileGeneric::with_fifo(
            ObisCode::new(1, 0, 99, 1, 0, 255),
            vec![CaptureObjectDefinition {
                class_id: 8,
                logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
                attribute_index: 2,
                data_index: 0,
            }],
            0,
            0,
        );
        // Capture should work but buffer will be immediately cleared
        let _ = profile.invoke_method(2, Some(Data::Integer(0)));
        assert_eq!(profile.buffer.len(), 0); // No room for entries
        assert_eq!(profile.entries_in_use, 0);
    }

    #[test]
    fn test_large_buffer_serialization() {
        let mut profile = ProfileGeneric::with_fifo(
            ObisCode::new(1, 0, 99, 1, 0, 255),
            vec![CaptureObjectDefinition {
                class_id: 8,
                logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
                attribute_index: 2,
                data_index: 0,
            }],
            0,
            1000,
        );
        // Add 500 entries
        for _ in 0..500 {
            let _ = profile.invoke_method(2, Some(Data::Integer(0)));
        }
        assert_eq!(profile.buffer.len(), 500);
        // Get buffer attribute (should not panic)
        let result = profile.get_attribute(2);
        assert!(result.is_ok());
        if let Data::Array(rows) = result.unwrap() {
            assert_eq!(rows.len(), 500);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_sort_object_with_fifo_should_be_none() {
        // FIFO/LIFO don't use sort_object, but we allow it to be set
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        profile.sort_object = Some(CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        });
        // sort_object is set but ignored for FIFO
        assert!(profile.sort_object.is_some());
        // Can retrieve it via get_attribute
        let result = profile.get_attribute(6);
        assert!(result.is_ok());
        assert_ne!(result.unwrap(), Data::Null);
    }

    #[test]
    fn test_capture_objects_invalid_structure() {
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        // Wrong number of fields (should be 4)
        let invalid_objects = Data::Structure(vec![Data::Structure(vec![
            Data::LongUnsigned(8),
            Data::OctetString(vec![0, 0, 1, 0, 0, 255]),
            // Missing attribute_index and data_index
        ])]);
        let result = profile.set_attribute(3, invalid_objects);
        assert_eq!(result, Err(DataAccessResult::TypeUnmatched));
    }

    // ===== STEP 7: Trait Implementations (4 tests) =====

    #[test]
    fn test_profile_generic_debug() {
        let profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        let debug_str = format!("{:?}", profile);
        assert!(debug_str.contains("ProfileGeneric"));
        assert!(debug_str.contains("logical_name"));
    }

    #[test]
    fn test_profile_generic_clone() {
        let mut profile1 = ProfileGeneric::with_fifo(
            ObisCode::new(1, 0, 99, 1, 0, 255),
            vec![CaptureObjectDefinition {
                class_id: 8,
                logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
                attribute_index: 2,
                data_index: 0,
            }],
            900,
            96,
        );
        profile1.buffer.push_back(vec![Data::DoubleLongUnsigned(100)]);
        profile1.entries_in_use = 1;
        let profile2 = profile1.clone();
        assert_eq!(profile1, profile2);
        assert_eq!(profile1.buffer, profile2.buffer);
        assert_eq!(profile1.capture_objects, profile2.capture_objects);
    }

    #[test]
    fn test_profile_generic_partial_eq() {
        let profile1 = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        let profile2 = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        let mut profile3 = ProfileGeneric::new(ObisCode::new(1, 0, 99, 2, 0, 255), 96);
        profile3.capture_period = 900;
        assert_eq!(profile1, profile2);
        assert_ne!(profile1, profile3);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_profile_generic_serde_serialize() {
        // Test that Serialize derive works (compilation test)
        let mut _profile = ProfileGeneric::with_fifo(
            ObisCode::new(1, 0, 99, 1, 0, 255),
            vec![CaptureObjectDefinition {
                class_id: 8,
                logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
                attribute_index: 2,
                data_index: 0,
            }],
            900,
            96,
        );
        _profile.buffer.push_back(vec![Data::DoubleLongUnsigned(100)]);
        _profile.entries_in_use = 1;
        // If this compiles with serde feature, Serialize derive is working
        // No runtime assertions needed - this is a compile-time test
    }

    // ===== STEP 8: Placeholder Tests (5 tests) - Phase 5.3 Features =====

    #[test]
    fn test_selective_access_range_descriptor_placeholder() {
        // Phase 5.3: RangeDescriptor for filtering by date range
        let profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        // TODO Phase 5.3: Implement RangeDescriptor
        // Example: Get entries from 2024-01-01 to 2024-01-31
        // let range = RangeDescriptor { from: DateTime, to: DateTime };
        // let filtered = profile.get_with_range(range);
        assert_eq!(profile.class_id(), 7); // Placeholder assertion
    }

    #[test]
    fn test_selective_access_entry_descriptor_placeholder() {
        // Phase 5.3: EntryDescriptor for filtering by row/column indices
        let mut profile = ProfileGeneric::new(ObisCode::new(1, 0, 99, 1, 0, 255), 10);
        profile.buffer.push_back(vec![Data::DoubleLongUnsigned(100)]);
        profile.buffer.push_back(vec![Data::DoubleLongUnsigned(200)]);
        profile.buffer.push_back(vec![Data::DoubleLongUnsigned(300)]);
        profile.entries_in_use = 3;
        // TODO Phase 5.3: Implement EntryDescriptor
        // Example: Get rows 1-2, column 0
        // let descriptor = EntryDescriptor { from_row: 1, to_row: 2, from_col: 0, to_col: 0 };
        // let filtered = profile.get_with_entry_descriptor(descriptor);
        assert_eq!(profile.buffer.len(), 3); // Placeholder assertion
    }

    // ===== PHASE 5.3.2: Advanced Sort Methods Tests =====

    // ----- Helper Function Tests (8 tests) -----

    #[test]
    fn test_extract_numeric_value_integer() {
        assert_eq!(extract_numeric_value(&Data::Integer(42)), Some(42.0));
        assert_eq!(extract_numeric_value(&Data::Integer(-10)), Some(-10.0));
    }

    #[test]
    fn test_extract_numeric_value_unsigned() {
        assert_eq!(extract_numeric_value(&Data::Unsigned(255)), Some(255.0));
    }

    #[test]
    fn test_extract_numeric_value_long() {
        assert_eq!(extract_numeric_value(&Data::Long(1000)), Some(1000.0));
        assert_eq!(extract_numeric_value(&Data::Long(-500)), Some(-500.0));
    }

    #[test]
    fn test_extract_numeric_value_long_unsigned() {
        assert_eq!(extract_numeric_value(&Data::LongUnsigned(65000)), Some(65000.0));
    }

    #[test]
    fn test_extract_numeric_value_double_long() {
        assert_eq!(extract_numeric_value(&Data::DoubleLong(1_000_000)), Some(1_000_000.0));
        assert_eq!(extract_numeric_value(&Data::DoubleLong(-500_000)), Some(-500_000.0));
    }

    #[test]
    fn test_extract_numeric_value_double_long_unsigned() {
        assert_eq!(
            extract_numeric_value(&Data::DoubleLongUnsigned(4_000_000_000)),
            Some(4_000_000_000.0)
        );
    }

    #[test]
    fn test_extract_numeric_value_float() {
        // Float32 has precision loss when converting to f64
        let result = extract_numeric_value(&Data::Float32(3.5));
        assert!(result.is_some());
        let value = result.unwrap();
        assert!((value - 3.5).abs() < 0.01); // Use tolerance for Float32

        assert_eq!(extract_numeric_value(&Data::Float64(2.75)), Some(2.75));
    }

    #[test]
    fn test_extract_numeric_value_non_numeric() {
        assert_eq!(extract_numeric_value(&Data::OctetString(vec![1, 2, 3])), None);
        assert_eq!(extract_numeric_value(&Data::Null), None);
        assert_eq!(extract_numeric_value(&Data::Utf8String("123".to_string())), None);
    }

    // ----- Largest Sort Method Tests (5 tests) -----

    #[test]
    fn test_largest_sort_basic() {
        let clock_def = CaptureObjectDefinition {
            class_id: 8,
            logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };
        let energy_def = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let mut profile = ProfileGeneric {
            logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
            buffer: VecDeque::new(),
            capture_objects: vec![clock_def.clone(), energy_def.clone()],
            capture_period: 0,
            sort_method: SortMethod::Largest,
            sort_object: Some(energy_def),
            entries_in_use: 0,
            profile_entries: 3,
            executed_time: 0,
            use_compact_array_encoding: false,
        };

        // Add entries - buffer size = 3, keep 3 largest
        profile.add_entry(vec![Data::Integer(1), Data::DoubleLongUnsigned(100)]);
        profile.add_entry(vec![Data::Integer(2), Data::DoubleLongUnsigned(300)]);
        profile.add_entry(vec![Data::Integer(3), Data::DoubleLongUnsigned(200)]);

        assert_eq!(profile.buffer.len(), 3);

        // Add 4th entry (500) - should remove smallest (100)
        profile.add_entry(vec![Data::Integer(4), Data::DoubleLongUnsigned(500)]);

        assert_eq!(profile.buffer.len(), 3);
        // Buffer should contain: 500, 300, 200 (not necessarily in this order)
        let values: Vec<u32> = profile
            .buffer
            .iter()
            .map(|entry| if let Data::DoubleLongUnsigned(v) = entry[1] { v } else { 0 })
            .collect();

        assert!(values.contains(&500));
        assert!(values.contains(&300));
        assert!(values.contains(&200));
        assert!(!values.contains(&100)); // Smallest removed
    }

    #[test]
    fn test_largest_sort_negative_values() {
        let energy_def = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let mut profile = ProfileGeneric {
            logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
            buffer: VecDeque::new(),
            capture_objects: vec![energy_def.clone()],
            capture_period: 0,
            sort_method: SortMethod::Largest,
            sort_object: Some(energy_def),
            entries_in_use: 0,
            profile_entries: 3,
            executed_time: 0,
            use_compact_array_encoding: false,
        };

        // Test with negative values: -10, -5, -20, -3
        profile.add_entry(vec![Data::DoubleLong(-10)]);
        profile.add_entry(vec![Data::DoubleLong(-5)]);
        profile.add_entry(vec![Data::DoubleLong(-20)]);
        profile.add_entry(vec![Data::DoubleLong(-3)]);

        assert_eq!(profile.buffer.len(), 3);
        // Keep largest (least negative): -3, -5, -10
        let values: Vec<i32> = profile
            .buffer
            .iter()
            .map(|entry| if let Data::DoubleLong(v) = entry[0] { v } else { 0 })
            .collect();

        assert!(values.contains(&-3));
        assert!(values.contains(&-5));
        assert!(values.contains(&-10));
        assert!(!values.contains(&-20)); // Smallest removed
    }

    #[test]
    fn test_largest_sort_mixed_types() {
        let value_def = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let mut profile = ProfileGeneric {
            logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
            buffer: VecDeque::new(),
            capture_objects: vec![value_def.clone()],
            capture_period: 0,
            sort_method: SortMethod::Largest,
            sort_object: Some(value_def),
            entries_in_use: 0,
            profile_entries: 3,
            executed_time: 0,
            use_compact_array_encoding: false,
        };

        // Mix different numeric types
        profile.add_entry(vec![Data::Integer(50)]);
        profile.add_entry(vec![Data::LongUnsigned(1000)]);
        profile.add_entry(vec![Data::Float64(250.5)]);
        profile.add_entry(vec![Data::DoubleLongUnsigned(100)]);

        assert_eq!(profile.buffer.len(), 3);
        // Keep largest: 1000, 250.5, 100
        // Remove: 50
    }

    #[test]
    fn test_largest_sort_single_entry() {
        let value_def = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let mut profile = ProfileGeneric {
            logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
            buffer: VecDeque::new(),
            capture_objects: vec![value_def.clone()],
            capture_period: 0,
            sort_method: SortMethod::Largest,
            sort_object: Some(value_def),
            entries_in_use: 0,
            profile_entries: 1,
            executed_time: 0,
            use_compact_array_encoding: false,
        };

        profile.add_entry(vec![Data::Integer(10)]);
        assert_eq!(profile.buffer.len(), 1);

        profile.add_entry(vec![Data::Integer(20)]);
        assert_eq!(profile.buffer.len(), 1);
        assert_eq!(profile.buffer[0][0], Data::Integer(20)); // Keep largest
    }

    #[test]
    fn test_largest_sort_overflow() {
        let value_def = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let mut profile = ProfileGeneric {
            logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
            buffer: VecDeque::new(),
            capture_objects: vec![value_def.clone()],
            capture_period: 0,
            sort_method: SortMethod::Largest,
            sort_object: Some(value_def),
            entries_in_use: 0,
            profile_entries: 5,
            executed_time: 0,
            use_compact_array_encoding: false,
        };

        // Add 10 entries to buffer of size 5
        for i in 1..=10 {
            profile.add_entry(vec![Data::Integer(i * 10)]);
        }

        assert_eq!(profile.buffer.len(), 5);
        // Should keep: 100, 90, 80, 70, 60
        let values: Vec<i8> = profile
            .buffer
            .iter()
            .map(|entry| if let Data::Integer(v) = entry[0] { v } else { 0 })
            .collect();

        assert!(values.contains(&100));
        assert!(values.contains(&90));
        assert!(values.contains(&80));
        assert!(values.contains(&70));
        assert!(values.contains(&60));
    }

    // ----- Smallest Sort Method Tests (5 tests) -----

    #[test]
    fn test_smallest_sort_basic() {
        let value_def = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let mut profile = ProfileGeneric {
            logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
            buffer: VecDeque::new(),
            capture_objects: vec![value_def.clone()],
            capture_period: 0,
            sort_method: SortMethod::Smallest,
            sort_object: Some(value_def),
            entries_in_use: 0,
            profile_entries: 3,
            executed_time: 0,
            use_compact_array_encoding: false,
        };

        profile.add_entry(vec![Data::DoubleLongUnsigned(100)]);
        profile.add_entry(vec![Data::DoubleLongUnsigned(300)]);
        profile.add_entry(vec![Data::DoubleLongUnsigned(200)]);
        profile.add_entry(vec![Data::DoubleLongUnsigned(50)]);

        assert_eq!(profile.buffer.len(), 3);
        // Keep smallest: 50, 100, 200
        let values: Vec<u32> = profile
            .buffer
            .iter()
            .map(|entry| if let Data::DoubleLongUnsigned(v) = entry[0] { v } else { 0 })
            .collect();

        assert!(values.contains(&50));
        assert!(values.contains(&100));
        assert!(values.contains(&200));
        assert!(!values.contains(&300)); // Largest removed
    }

    #[test]
    fn test_smallest_sort_negative_values() {
        let value_def = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let mut profile = ProfileGeneric {
            logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
            buffer: VecDeque::new(),
            capture_objects: vec![value_def.clone()],
            capture_period: 0,
            sort_method: SortMethod::Smallest,
            sort_object: Some(value_def),
            entries_in_use: 0,
            profile_entries: 3,
            executed_time: 0,
            use_compact_array_encoding: false,
        };

        // Test with negative values
        profile.add_entry(vec![Data::DoubleLong(-10)]);
        profile.add_entry(vec![Data::DoubleLong(-5)]);
        profile.add_entry(vec![Data::DoubleLong(-20)]);
        profile.add_entry(vec![Data::DoubleLong(-3)]);

        assert_eq!(profile.buffer.len(), 3);
        // Keep smallest (most negative): -20, -10, -5
        let values: Vec<i32> = profile
            .buffer
            .iter()
            .map(|entry| if let Data::DoubleLong(v) = entry[0] { v } else { 0 })
            .collect();

        assert!(values.contains(&-20));
        assert!(values.contains(&-10));
        assert!(values.contains(&-5));
        assert!(!values.contains(&-3)); // Largest removed
    }

    #[test]
    fn test_smallest_sort_mixed_types() {
        let value_def = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let mut profile = ProfileGeneric {
            logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
            buffer: VecDeque::new(),
            capture_objects: vec![value_def.clone()],
            capture_period: 0,
            sort_method: SortMethod::Smallest,
            sort_object: Some(value_def),
            entries_in_use: 0,
            profile_entries: 3,
            executed_time: 0,
            use_compact_array_encoding: false,
        };

        profile.add_entry(vec![Data::Integer(50)]);
        profile.add_entry(vec![Data::LongUnsigned(1000)]);
        profile.add_entry(vec![Data::Float64(25.5)]);
        profile.add_entry(vec![Data::DoubleLongUnsigned(100)]);

        assert_eq!(profile.buffer.len(), 3);
        // Keep smallest: 25.5, 50, 100
    }

    #[test]
    fn test_smallest_sort_single_entry() {
        let value_def = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let mut profile = ProfileGeneric {
            logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
            buffer: VecDeque::new(),
            capture_objects: vec![value_def.clone()],
            capture_period: 0,
            sort_method: SortMethod::Smallest,
            sort_object: Some(value_def),
            entries_in_use: 0,
            profile_entries: 1,
            executed_time: 0,
            use_compact_array_encoding: false,
        };

        profile.add_entry(vec![Data::Integer(20)]);
        assert_eq!(profile.buffer.len(), 1);

        profile.add_entry(vec![Data::Integer(10)]);
        assert_eq!(profile.buffer.len(), 1);
        assert_eq!(profile.buffer[0][0], Data::Integer(10)); // Keep smallest
    }

    #[test]
    fn test_smallest_sort_overflow() {
        let value_def = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let mut profile = ProfileGeneric {
            logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
            buffer: VecDeque::new(),
            capture_objects: vec![value_def.clone()],
            capture_period: 0,
            sort_method: SortMethod::Smallest,
            sort_object: Some(value_def),
            entries_in_use: 0,
            profile_entries: 5,
            executed_time: 0,
            use_compact_array_encoding: false,
        };

        // Add 10 entries to buffer of size 5
        for i in 1..=10 {
            profile.add_entry(vec![Data::Integer(i * 10)]);
        }

        assert_eq!(profile.buffer.len(), 5);
        // Should keep: 10, 20, 30, 40, 50
        let values: Vec<i8> = profile
            .buffer
            .iter()
            .map(|entry| if let Data::Integer(v) = entry[0] { v } else { 0 })
            .collect();

        assert!(values.contains(&10));
        assert!(values.contains(&20));
        assert!(values.contains(&30));
        assert!(values.contains(&40));
        assert!(values.contains(&50));
    }

    // ----- NearestToZero Sort Method Tests (4 tests) -----

    #[test]
    fn test_nearest_to_zero_basic() {
        let value_def = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let mut profile = ProfileGeneric {
            logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
            buffer: VecDeque::new(),
            capture_objects: vec![value_def.clone()],
            capture_period: 0,
            sort_method: SortMethod::NearestToZero,
            sort_object: Some(value_def),
            entries_in_use: 0,
            profile_entries: 3,
            executed_time: 0,
            use_compact_array_encoding: false,
        };

        // Values: -10, -5, 3, 8, 1
        profile.add_entry(vec![Data::DoubleLong(-10)]);
        profile.add_entry(vec![Data::DoubleLong(-5)]);
        profile.add_entry(vec![Data::DoubleLong(3)]);
        profile.add_entry(vec![Data::DoubleLong(8)]);
        profile.add_entry(vec![Data::DoubleLong(1)]);

        assert_eq!(profile.buffer.len(), 3);
        // Keep nearest to zero: 1, 3, -5
        let values: Vec<i32> = profile
            .buffer
            .iter()
            .map(|entry| if let Data::DoubleLong(v) = entry[0] { v } else { 0 })
            .collect();

        assert!(values.contains(&1));
        assert!(values.contains(&3));
        assert!(values.contains(&-5));
        // -10 and 8 removed (farthest from zero)
    }

    #[test]
    fn test_nearest_to_zero_positive_only() {
        let value_def = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let mut profile = ProfileGeneric {
            logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
            buffer: VecDeque::new(),
            capture_objects: vec![value_def.clone()],
            capture_period: 0,
            sort_method: SortMethod::NearestToZero,
            sort_object: Some(value_def),
            entries_in_use: 0,
            profile_entries: 3,
            executed_time: 0,
            use_compact_array_encoding: false,
        };

        profile.add_entry(vec![Data::DoubleLongUnsigned(100)]);
        profile.add_entry(vec![Data::DoubleLongUnsigned(10)]);
        profile.add_entry(vec![Data::DoubleLongUnsigned(50)]);
        profile.add_entry(vec![Data::DoubleLongUnsigned(5)]);

        assert_eq!(profile.buffer.len(), 3);
        // Keep: 5, 10, 50
        let values: Vec<u32> = profile
            .buffer
            .iter()
            .map(|entry| if let Data::DoubleLongUnsigned(v) = entry[0] { v } else { 0 })
            .collect();

        assert!(values.contains(&5));
        assert!(values.contains(&10));
        assert!(values.contains(&50));
    }

    #[test]
    fn test_nearest_to_zero_negative_only() {
        let value_def = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let mut profile = ProfileGeneric {
            logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
            buffer: VecDeque::new(),
            capture_objects: vec![value_def.clone()],
            capture_period: 0,
            sort_method: SortMethod::NearestToZero,
            sort_object: Some(value_def),
            entries_in_use: 0,
            profile_entries: 3,
            executed_time: 0,
            use_compact_array_encoding: false,
        };

        profile.add_entry(vec![Data::DoubleLong(-100)]);
        profile.add_entry(vec![Data::DoubleLong(-10)]);
        profile.add_entry(vec![Data::DoubleLong(-50)]);
        profile.add_entry(vec![Data::DoubleLong(-5)]);

        assert_eq!(profile.buffer.len(), 3);
        // Keep: -5, -10, -50
        let values: Vec<i32> = profile
            .buffer
            .iter()
            .map(|entry| if let Data::DoubleLong(v) = entry[0] { v } else { 0 })
            .collect();

        assert!(values.contains(&-5));
        assert!(values.contains(&-10));
        assert!(values.contains(&-50));
    }

    #[test]
    fn test_nearest_to_zero_mixed_signs() {
        let value_def = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let mut profile = ProfileGeneric {
            logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
            buffer: VecDeque::new(),
            capture_objects: vec![value_def.clone()],
            capture_period: 0,
            sort_method: SortMethod::NearestToZero,
            sort_object: Some(value_def),
            entries_in_use: 0,
            profile_entries: 4,
            executed_time: 0,
            use_compact_array_encoding: false,
        };

        profile.add_entry(vec![Data::DoubleLong(50)]);
        profile.add_entry(vec![Data::DoubleLong(-30)]);
        profile.add_entry(vec![Data::DoubleLong(20)]);
        profile.add_entry(vec![Data::DoubleLong(-10)]);
        profile.add_entry(vec![Data::DoubleLong(5)]);

        assert_eq!(profile.buffer.len(), 4);
        // Keep: 5, -10, 20, -30 (abs values: 5, 10, 20, 30)
        let values: Vec<i32> = profile
            .buffer
            .iter()
            .map(|entry| if let Data::DoubleLong(v) = entry[0] { v } else { 0 })
            .collect();

        assert!(values.contains(&5));
        assert!(values.contains(&-10));
        assert!(values.contains(&20));
        assert!(values.contains(&-30));
        assert!(!values.contains(&50)); // Farthest removed
    }

    // ----- FarthestFromZero Sort Method Tests (4 tests) -----

    #[test]
    fn test_farthest_from_zero_basic() {
        let value_def = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let mut profile = ProfileGeneric {
            logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
            buffer: VecDeque::new(),
            capture_objects: vec![value_def.clone()],
            capture_period: 0,
            sort_method: SortMethod::FarthestFromZero,
            sort_object: Some(value_def),
            entries_in_use: 0,
            profile_entries: 3,
            executed_time: 0,
            use_compact_array_encoding: false,
        };

        // Values: -10, -5, 3, 8, 1
        profile.add_entry(vec![Data::DoubleLong(-10)]);
        profile.add_entry(vec![Data::DoubleLong(-5)]);
        profile.add_entry(vec![Data::DoubleLong(3)]);
        profile.add_entry(vec![Data::DoubleLong(8)]);
        profile.add_entry(vec![Data::DoubleLong(1)]);

        assert_eq!(profile.buffer.len(), 3);
        // Keep farthest from zero: -10, 8, -5
        let values: Vec<i32> = profile
            .buffer
            .iter()
            .map(|entry| if let Data::DoubleLong(v) = entry[0] { v } else { 0 })
            .collect();

        assert!(values.contains(&-10));
        assert!(values.contains(&8));
        assert!(values.contains(&-5));
        // 1 and 3 removed (nearest to zero)
    }

    #[test]
    fn test_farthest_from_zero_positive_only() {
        let value_def = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let mut profile = ProfileGeneric {
            logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
            buffer: VecDeque::new(),
            capture_objects: vec![value_def.clone()],
            capture_period: 0,
            sort_method: SortMethod::FarthestFromZero,
            sort_object: Some(value_def),
            entries_in_use: 0,
            profile_entries: 3,
            executed_time: 0,
            use_compact_array_encoding: false,
        };

        profile.add_entry(vec![Data::DoubleLongUnsigned(100)]);
        profile.add_entry(vec![Data::DoubleLongUnsigned(10)]);
        profile.add_entry(vec![Data::DoubleLongUnsigned(50)]);
        profile.add_entry(vec![Data::DoubleLongUnsigned(5)]);

        assert_eq!(profile.buffer.len(), 3);
        // Keep: 100, 50, 10
        let values: Vec<u32> = profile
            .buffer
            .iter()
            .map(|entry| if let Data::DoubleLongUnsigned(v) = entry[0] { v } else { 0 })
            .collect();

        assert!(values.contains(&100));
        assert!(values.contains(&50));
        assert!(values.contains(&10));
    }

    #[test]
    fn test_farthest_from_zero_negative_only() {
        let value_def = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let mut profile = ProfileGeneric {
            logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
            buffer: VecDeque::new(),
            capture_objects: vec![value_def.clone()],
            capture_period: 0,
            sort_method: SortMethod::FarthestFromZero,
            sort_object: Some(value_def),
            entries_in_use: 0,
            profile_entries: 3,
            executed_time: 0,
            use_compact_array_encoding: false,
        };

        profile.add_entry(vec![Data::DoubleLong(-100)]);
        profile.add_entry(vec![Data::DoubleLong(-10)]);
        profile.add_entry(vec![Data::DoubleLong(-50)]);
        profile.add_entry(vec![Data::DoubleLong(-5)]);

        assert_eq!(profile.buffer.len(), 3);
        // Keep: -100, -50, -10
        let values: Vec<i32> = profile
            .buffer
            .iter()
            .map(|entry| if let Data::DoubleLong(v) = entry[0] { v } else { 0 })
            .collect();

        assert!(values.contains(&-100));
        assert!(values.contains(&-50));
        assert!(values.contains(&-10));
    }

    #[test]
    fn test_farthest_from_zero_mixed_signs() {
        let value_def = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let mut profile = ProfileGeneric {
            logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
            buffer: VecDeque::new(),
            capture_objects: vec![value_def.clone()],
            capture_period: 0,
            sort_method: SortMethod::FarthestFromZero,
            sort_object: Some(value_def),
            entries_in_use: 0,
            profile_entries: 4,
            executed_time: 0,
            use_compact_array_encoding: false,
        };

        profile.add_entry(vec![Data::DoubleLong(50)]);
        profile.add_entry(vec![Data::DoubleLong(-30)]);
        profile.add_entry(vec![Data::DoubleLong(20)]);
        profile.add_entry(vec![Data::DoubleLong(-10)]);
        profile.add_entry(vec![Data::DoubleLong(5)]);

        assert_eq!(profile.buffer.len(), 4);
        // Keep: 50, -30, 20, -10 (abs values: 50, 30, 20, 10)
        let values: Vec<i32> = profile
            .buffer
            .iter()
            .map(|entry| if let Data::DoubleLong(v) = entry[0] { v } else { 0 })
            .collect();

        assert!(values.contains(&50));
        assert!(values.contains(&-30));
        assert!(values.contains(&20));
        assert!(values.contains(&-10));
        assert!(!values.contains(&5)); // Nearest removed
    }

    // ----- Edge Cases Tests (4 tests) -----

    #[test]
    fn test_sorted_method_no_sort_object() {
        let value_def = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let mut profile = ProfileGeneric {
            logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
            buffer: VecDeque::new(),
            capture_objects: vec![value_def],
            capture_period: 0,
            sort_method: SortMethod::Largest,
            sort_object: None, // No sort_object configured
            entries_in_use: 0,
            profile_entries: 3,
            executed_time: 0,
            use_compact_array_encoding: false,
        };

        // Should fallback to FIFO
        profile.add_entry(vec![Data::Integer(1)]);
        profile.add_entry(vec![Data::Integer(2)]);
        profile.add_entry(vec![Data::Integer(3)]);
        profile.add_entry(vec![Data::Integer(4)]);

        assert_eq!(profile.buffer.len(), 3);
        // FIFO: keep 2, 3, 4 (remove 1)
        assert_eq!(profile.buffer[0][0], Data::Integer(2));
        assert_eq!(profile.buffer[1][0], Data::Integer(3));
        assert_eq!(profile.buffer[2][0], Data::Integer(4));
    }

    #[test]
    fn test_sorted_method_column_not_found() {
        let value_def = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let different_def = CaptureObjectDefinition {
            class_id: 4,
            logical_name: ObisCode::new(1, 0, 2, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let mut profile = ProfileGeneric {
            logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
            buffer: VecDeque::new(),
            capture_objects: vec![value_def],
            capture_period: 0,
            sort_method: SortMethod::Largest,
            sort_object: Some(different_def), // Not in capture_objects
            entries_in_use: 0,
            profile_entries: 3,
            executed_time: 0,
            use_compact_array_encoding: false,
        };

        // Should fallback to FIFO
        profile.add_entry(vec![Data::Integer(1)]);
        profile.add_entry(vec![Data::Integer(2)]);
        profile.add_entry(vec![Data::Integer(3)]);
        profile.add_entry(vec![Data::Integer(4)]);

        assert_eq!(profile.buffer.len(), 3);
    }

    #[test]
    fn test_sorted_method_non_numeric_values() {
        let value_def = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let mut profile = ProfileGeneric {
            logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
            buffer: VecDeque::new(),
            capture_objects: vec![value_def.clone()],
            capture_period: 0,
            sort_method: SortMethod::Largest,
            sort_object: Some(value_def),
            entries_in_use: 0,
            profile_entries: 3,
            executed_time: 0,
            use_compact_array_encoding: false,
        };

        // Add non-numeric values - should fallback to FIFO
        profile.add_entry(vec![Data::OctetString(vec![1, 2, 3])]);
        profile.add_entry(vec![Data::OctetString(vec![4, 5, 6])]);
        profile.add_entry(vec![Data::OctetString(vec![7, 8, 9])]);
        profile.add_entry(vec![Data::OctetString(vec![10, 11, 12])]);

        assert_eq!(profile.buffer.len(), 3);
    }

    #[test]
    fn test_sorted_method_empty_buffer() {
        let value_def = CaptureObjectDefinition {
            class_id: 3,
            logical_name: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_index: 2,
            data_index: 0,
        };

        let profile = ProfileGeneric {
            logical_name: ObisCode::new(1, 0, 99, 1, 0, 255),
            buffer: VecDeque::new(),
            capture_objects: vec![value_def.clone()],
            capture_period: 0,
            sort_method: SortMethod::Largest,
            sort_object: Some(value_def),
            entries_in_use: 0,
            profile_entries: 3,
            executed_time: 0,
            use_compact_array_encoding: false,
        };

        // Empty buffer - should handle gracefully
        assert_eq!(profile.buffer.len(), 0);
        assert_eq!(profile.find_worst_entry_index(), None);
    }

    #[test]
    fn test_compact_array_encoding_opt_in() {
        // Phase 5.3: Compact array encoding for large buffers (Green Book 14.4)
        let mut profile = ProfileGeneric::with_fifo(
            ObisCode::new(1, 0, 99, 1, 0, 255),
            vec![CaptureObjectDefinition {
                class_id: 8,
                logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
                attribute_index: 2,
                data_index: 0,
            }],
            0,
            100,
        );

        // Enable Compact Array encoding
        profile.use_compact_array_encoding = true;

        // Add many entries
        for _ in 0..50 {
            profile.add_entry(vec![Data::DoubleLongUnsigned(0)]);
        }

        let result = profile.get_attribute(2);
        assert!(result.is_ok());

        // Should be CompactArray now
        if let Data::CompactArray(rows) = result.unwrap() {
            assert_eq!(rows.len(), 50);
        } else {
            panic!("Expected CompactArray when enabled");
        }
    }
}
