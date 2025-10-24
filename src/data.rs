#[cfg(feature = "serde")]
use alloc::string::ToString;
use alloc::{string::String, vec, vec::Vec};
use core::convert::TryFrom;
use core::fmt;

use nom::{
    IResult, Parser,
    combinator::fail,
    multi::length_count,
    number::streaming::{be_f32, be_f64, be_i16, be_i32, be_i64, be_u16, be_u32, be_u64, i8, u8},
};
#[cfg(feature = "serde")]
use serde::{Serialize, Serializer};

#[cfg(feature = "chrono-conversions")]
use chrono::{Datelike, Timelike};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[rustfmt::skip]
pub enum DataType {
  Null               =  0,
  Array              =  1,
  Structure          =  2,
  Bool               =  3,
  BitString          =  4,
  DoubleLong         =  5,
  DoubleLongUnsigned =  6,
  OctetString        =  9,
  VisibleString      = 10,
  Utf8String         = 12,
  BinaryCodedDecimal = 13,
  Integer            = 15,
  Long               = 16,
  Unsigned           = 17,
  LongUnsigned       = 18,
  CompactArray       = 19,
  Long64             = 20,
  Long64Unsigned     = 21,
  Enum               = 22,
  Float32            = 23,
  Float64            = 24,
  DateTime           = 25,
  Date               = 26,
  Time               = 27,
}

impl TryFrom<u8> for DataType {
    type Error = u8;

    fn try_from(dt: u8) -> Result<Self, Self::Error> {
        Ok(match dt {
            0x00 => Self::Null,
            0x01 => Self::Array,
            0x02 => Self::Structure,
            0x03 => Self::Bool,
            0x04 => Self::BitString,
            0x05 => Self::DoubleLong,
            0x06 => Self::DoubleLongUnsigned,
            0x09 => Self::OctetString,
            0x0a => Self::VisibleString,
            0x0c => Self::Utf8String,
            0x0d => Self::BinaryCodedDecimal,
            0x0f => Self::Integer,
            0x10 => Self::Long,
            0x11 => Self::Unsigned,
            0x12 => Self::LongUnsigned,
            0x13 => Self::CompactArray,
            0x14 => Self::Long64,
            0x15 => Self::Long64Unsigned,
            0x16 => Self::Enum,
            0x17 => Self::Float32,
            0x18 => Self::Float64,
            0x19 => Self::DateTime,
            0x1a => Self::Date,
            0x1b => Self::Time,
            dt => return Err(dt),
        })
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Date {
    pub(crate) year: u16,
    pub(crate) month: u8,
    pub(crate) day_of_month: u8,
    pub(crate) day_of_week: u8,
}

impl Date {
    #[cfg(feature = "encode")]
    /// Encode Date to 5 bytes: year_high, year_low, month, day_of_month, day_of_week
    /// Reference: Green Book Ed. 12, Section 4.1.6.1
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(5);
        buffer.push_u16(self.year);
        buffer.push_u8(self.month);
        buffer.push_u8(self.day_of_month);
        buffer.push_u8(self.day_of_week);
        buffer
    }

    #[cfg(feature = "chrono-conversions")]
    /// Create a Date from a chrono NaiveDate
    ///
    /// The day_of_week is automatically calculated from the date.
    /// Both chrono and DLMS use ISO 8601 weekday numbering (Monday=1, Sunday=7).
    ///
    /// This method works in both `std` and `no_std` environments when the
    /// `chrono-conversions` feature is enabled.
    ///
    /// # Example
    /// ```
    /// # #[cfg(feature = "chrono-conversions")]
    /// # {
    /// use dlms_cosem::Date;
    /// use chrono::NaiveDate;
    ///
    /// let naive_date = NaiveDate::from_ymd_opt(2024, 12, 25).unwrap();
    /// let date = Date::from_chrono(&naive_date);
    /// // Date is created with the correct values from chrono
    /// # }
    /// ```
    pub fn from_chrono(date: &chrono::NaiveDate) -> Self {
        Self {
            year: date.year() as u16,
            month: date.month() as u8,
            day_of_month: date.day() as u8,
            day_of_week: date.weekday().number_from_monday() as u8,
        }
    }

    #[cfg(feature = "jiff-conversions")]
    /// Create a Date from a jiff civil::Date
    ///
    /// The day_of_week is automatically calculated from the date.
    /// Both jiff and DLMS use ISO 8601 weekday numbering (Monday=1, Sunday=7).
    ///
    /// This method works in both `std` and `no_std` environments when the
    /// `jiff-conversions` feature is enabled.
    ///
    /// # Example
    /// ```
    /// # #[cfg(feature = "jiff-conversions")]
    /// # {
    /// use dlms_cosem::Date;
    /// use jiff::civil::Date as JiffDate;
    ///
    /// let jiff_date = JiffDate::new(2024, 12, 25).unwrap();
    /// let date = Date::from_jiff(&jiff_date);
    /// // Date is created with the correct values from jiff
    /// # }
    /// ```
    pub fn from_jiff(date: &jiff::civil::Date) -> Self {
        Self {
            year: date.year() as u16,
            month: date.month() as u8,
            day_of_month: date.day() as u8,
            day_of_week: date.weekday().to_monday_one_offset() as u8,
        }
    }

    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, year) = be_u16(input)?;
        let (input, month) = u8(input)?;
        let (input, day_of_month) = u8(input)?;
        let (input, day_of_week) = u8(input)?;

        Ok((input, Self { year, month, day_of_month, day_of_week }))
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day_of_month)
    }
}

impl fmt::Debug for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Date(\"{}\")", self)
    }
}

#[cfg(feature = "serde")]
impl Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Time {
    pub(crate) hour: Option<u8>,
    pub(crate) minute: Option<u8>,
    pub(crate) second: Option<u8>,
    pub(crate) hundredth: Option<u8>,
}

impl Time {
    #[cfg(feature = "encode")]
    /// Encode Time to 4 bytes: hour, minute, second, hundredth
    /// None values are encoded as 0xFF (wildcard per DLMS spec)
    /// Reference: Green Book Ed. 12, Section 4.1.6.1
    pub fn encode(&self) -> Vec<u8> {
        vec![
            self.hour.unwrap_or(0xFF),
            self.minute.unwrap_or(0xFF),
            self.second.unwrap_or(0xFF),
            self.hundredth.unwrap_or(0xFF),
        ]
    }

    #[cfg(feature = "chrono-conversions")]
    /// Create a Time from a chrono NaiveTime
    ///
    /// Milliseconds are converted to hundredths of a second (truncated).
    /// All fields are set to Some values (no wildcards).
    ///
    /// This method works in both `std` and `no_std` environments when the
    /// `chrono-conversions` feature is enabled.
    ///
    /// # Example
    /// ```
    /// # #[cfg(feature = "chrono-conversions")]
    /// # {
    /// use dlms_cosem::Time;
    /// use chrono::NaiveTime;
    ///
    /// let naive_time = NaiveTime::from_hms_milli_opt(14, 30, 45, 500).unwrap();
    /// let time = Time::from_chrono(&naive_time);
    /// // Time is created with hour=14, minute=30, second=45, hundredth=50 (500ms)
    /// # }
    /// ```
    pub fn from_chrono(time: &chrono::NaiveTime) -> Self {
        Self {
            hour: Some(time.hour() as u8),
            minute: Some(time.minute() as u8),
            second: Some(time.second() as u8),
            hundredth: Some((time.nanosecond() / 10_000_000) as u8), // ns to hundredths
        }
    }

    #[cfg(feature = "jiff-conversions")]
    /// Create a Time from a jiff civil::Time
    ///
    /// Nanoseconds are converted to hundredths of a second (truncated).
    /// All fields are set to Some values (no wildcards).
    ///
    /// This method works in both `std` and `no_std` environments when the
    /// `jiff-conversions` feature is enabled.
    ///
    /// # Example
    /// ```
    /// # #[cfg(feature = "jiff-conversions")]
    /// # {
    /// use dlms_cosem::Time;
    /// use jiff::civil::Time as JiffTime;
    ///
    /// let jiff_time = JiffTime::new(14, 30, 45, 500_000_000).unwrap();
    /// let time = Time::from_jiff(&jiff_time);
    /// // Time is created with hour=14, minute=30, second=45, hundredth=50 (500ms)
    /// # }
    /// ```
    pub fn from_jiff(time: &jiff::civil::Time) -> Self {
        Self {
            hour: Some(time.hour() as u8),
            minute: Some(time.minute() as u8),
            second: Some(time.second() as u8),
            hundredth: Some((time.subsec_nanosecond() / 10_000_000) as u8), // ns to hundredths
        }
    }

    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (hour, minute, second, hundredth)) = (u8, u8, u8, u8).parse(input)?;

        let hour = match hour {
            0xff => None,
            0..=23 => Some(hour),
            _ => return fail().parse(input),
        };
        let minute = match minute {
            0xff => None,
            0..=59 => Some(minute),
            _ => return fail().parse(input),
        };
        let second = match second {
            0xff => None,
            0..=59 => Some(second),
            _ => return fail().parse(input),
        };
        let hundredth = match hundredth {
            0xff => None,
            0..=99 => Some(hundredth),
            _ => return fail().parse(input),
        };

        Ok((input, Self { hour, minute, second, hundredth }))
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}.{:02}",
            self.hour.unwrap_or(0),
            self.minute.unwrap_or(0),
            self.second.unwrap_or(0),
            self.hundredth.unwrap_or(0),
        )
    }
}

impl fmt::Debug for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Time(\"{}\")", self)
    }
}

#[cfg(feature = "serde")]
impl Serialize for Time {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ClockStatus(pub(crate) u8);

impl ClockStatus {
    #[rustfmt::skip]
    const INVALID_VALUE_BIT:   u8 = 0b00000001;
  #[rustfmt::skip]
    const DOUBTFUL_VALUE_BIT:  u8 = 0b00000010;
  #[rustfmt::skip]
    const DIFFERENT_BASE_BIT:  u8 = 0b00000100;
  #[rustfmt::skip]
    const INVALID_STATUS_BIT:  u8 = 0b00001000;
  #[rustfmt::skip]
    const DAYLIGHT_SAVING_BIT: u8 = 0b10000000;

    pub fn invalid_value(&self) -> bool {
        (self.0 & Self::INVALID_VALUE_BIT) != 0
    }

    pub fn doubtful_value(&self) -> bool {
        (self.0 & Self::DOUBTFUL_VALUE_BIT) != 0
    }

    pub fn different_base(&self) -> bool {
        (self.0 & Self::DIFFERENT_BASE_BIT) != 0
    }

    pub fn invalid_status(&self) -> bool {
        (self.0 & Self::INVALID_STATUS_BIT) != 0
    }

    pub fn daylight_saving(&self) -> bool {
        (self.0 & Self::DAYLIGHT_SAVING_BIT) != 0
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct DateTime {
    pub(crate) date: Date,
    pub(crate) time: Time,
    pub(crate) offset_minutes: Option<i16>,
    pub(crate) clock_status: Option<ClockStatus>,
}

impl DateTime {
    #[cfg(feature = "encode")]
    /// Encode DateTime to 12 bytes: date (5) + time (4) + offset (2) + clock_status (1)
    /// None offset is encoded as 0x8000 (wildcard per DLMS spec)
    /// None clock_status is encoded as 0xFF (wildcard per DLMS spec)
    /// Reference: Green Book Ed. 12, Section 4.1.6.1
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(12);
        buffer.push_bytes(&self.date.encode());
        buffer.push_bytes(&self.time.encode());
        buffer.push_i16(self.offset_minutes.unwrap_or(-0x8000));
        buffer.push_u8(self.clock_status.as_ref().map(|cs| cs.0).unwrap_or(0xFF));
        buffer
    }

    #[cfg(feature = "chrono-conversions")]
    /// Create a DateTime from a chrono NaiveDateTime with timezone offset and clock status
    ///
    /// This method works in both `std` and `no_std` environments when the
    /// `chrono-conversions` feature is enabled.
    ///
    /// # Arguments
    /// * `dt` - The NaiveDateTime to convert
    /// * `offset_minutes` - Timezone offset in minutes from UTC (positive = ahead of UTC)
    /// * `clock_status` - Clock status byte (see ClockStatus for bit definitions)
    ///
    /// # Example
    /// ```
    /// # #[cfg(feature = "chrono-conversions")]
    /// # {
    /// use dlms_cosem::DateTime;
    /// use chrono::NaiveDateTime;
    ///
    /// let naive_dt = NaiveDateTime::parse_from_str(
    ///     "2024-06-15 14:30:45",
    ///     "%Y-%m-%d %H:%M:%S"
    /// ).unwrap();
    /// let datetime = DateTime::from_chrono(&naive_dt, 120, 0x00); // UTC+2
    /// # }
    /// ```
    pub fn from_chrono(dt: &chrono::NaiveDateTime, offset_minutes: i16, clock_status: u8) -> Self {
        Self {
            date: Date::from_chrono(&dt.date()),
            time: Time::from_chrono(&dt.time()),
            offset_minutes: Some(offset_minutes),
            clock_status: Some(ClockStatus(clock_status)),
        }
    }

    #[cfg(all(feature = "std", feature = "chrono-conversions"))]
    /// Create a DateTime representing the current local time
    ///
    /// This uses the system clock to get the current time and timezone offset.
    /// The clock_status is set to 0x00 (no special status).
    ///
    /// **Note**: This method requires both `std` and `chrono-conversions` features
    /// because it uses `chrono::Local` which depends on the system clock.
    /// The `from_chrono()` methods work in `no_std` environments.
    ///
    /// # Example
    /// ```
    /// # #[cfg(all(feature = "std", feature = "chrono-conversions"))]
    /// # {
    /// use dlms_cosem::DateTime;
    ///
    /// let now = DateTime::now();
    /// // Use the current DateTime in your application
    /// # }
    /// ```
    pub fn now() -> Self {
        use chrono::Local;
        let local_time = Local::now();
        let naive = local_time.naive_local();
        let offset_seconds = local_time.offset().local_minus_utc();
        let offset_minutes = (offset_seconds / 60) as i16;

        Self::from_chrono(&naive, offset_minutes, 0x00)
    }

    #[cfg(feature = "jiff-conversions")]
    /// Create a DateTime from a jiff civil::DateTime with timezone offset and clock status
    ///
    /// This method works in both `std` and `no_std` environments when the
    /// `jiff-conversions` feature is enabled.
    ///
    /// # Arguments
    /// * `dt` - The jiff civil::DateTime to convert
    /// * `offset_minutes` - Timezone offset in minutes from UTC (positive = ahead of UTC)
    /// * `clock_status` - Clock status byte (see ClockStatus for bit definitions)
    ///
    /// # Example
    /// ```
    /// # #[cfg(feature = "jiff-conversions")]
    /// # {
    /// use dlms_cosem::DateTime;
    /// use jiff::civil::DateTime as JiffDateTime;
    ///
    /// let jiff_dt = JiffDateTime::new(2024, 6, 15, 14, 30, 45, 0).unwrap();
    /// let datetime = DateTime::from_jiff(&jiff_dt, 120, 0x00); // UTC+2
    /// # }
    /// ```
    pub fn from_jiff(dt: &jiff::civil::DateTime, offset_minutes: i16, clock_status: u8) -> Self {
        Self {
            date: Date::from_jiff(&dt.date()),
            time: Time::from_jiff(&dt.time()),
            offset_minutes: Some(offset_minutes),
            clock_status: Some(ClockStatus(clock_status)),
        }
    }

    #[cfg(all(feature = "std", feature = "jiff-conversions"))]
    /// Create a DateTime representing the current local time using jiff
    ///
    /// This uses the system clock to get the current time and timezone offset.
    /// The clock_status is set to 0x00 (no special status).
    ///
    /// **Note**: This method requires both `std` and `jiff-conversions` features
    /// because it uses `jiff::Zoned` which depends on the system clock.
    /// The `from_jiff()` methods work in `no_std` environments.
    ///
    /// # Example
    /// ```
    /// # #[cfg(all(feature = "std", feature = "jiff-conversions"))]
    /// # {
    /// use dlms_cosem::DateTime;
    ///
    /// let now = DateTime::now_jiff();
    /// // Use the current DateTime in your application
    /// # }
    /// ```
    pub fn now_jiff() -> Self {
        let zoned = jiff::Zoned::now();
        let offset_seconds = zoned.offset().seconds();
        let offset_minutes = (offset_seconds / 60) as i16;

        Self::from_jiff(&zoned.datetime(), offset_minutes, 0x00)
    }

    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, date) = Date::parse(input)?;
        let (input, time) = Time::parse(input)?;
        let (input, offset_minutes) = be_i16(input)?;
        let offset_minutes = Some(offset_minutes).filter(|&b| b != 0x8000u16 as i16);
        let (input, clock_status) = u8(input)?;
        let clock_status = Some(clock_status).filter(|&b| b != 0xff).map(ClockStatus);

        Ok((input, Self { date, time, offset_minutes, clock_status }))
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}T{}", self.date, self.time)?;

        if let Some(offset_minutes) = self.offset_minutes {
            if offset_minutes >= 0 {
                '-'.fmt(f)?;
            } else {
                '+'.fmt(f)?;
            };
            let offset_minutes = offset_minutes.abs();
            write!(f, "{:02}:{:02}", offset_minutes / 60, offset_minutes % 60)?;
        }

        Ok(())
    }
}

impl fmt::Debug for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DateTime(\"{}\")", self)
    }
}

#[cfg(feature = "serde")]
impl Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Data {
    Null,
    OctetString(Vec<u8>),
    Utf8String(String),
    Integer(i8),
    Unsigned(u8),
    Long(i16),
    LongUnsigned(u16),
    DoubleLong(i32),
    DoubleLongUnsigned(u32),
    Long64(i64),
    Long64Unsigned(u64),
    Float32(f32),
    Float64(f64),
    DateTime(DateTime),
    Date(Date),
    Time(Time),
    Structure(Vec<Data>),
    Enum(u8),
}

// ====================
// ENCODING SUPPORT
// ====================
// Phase 1.1: Data Type Encoding (IMPLEMENTATION_ROADMAP.md)
// Reference: Green Book Ed. 12, Section 4.1.6 - A-XDR encoding rules

#[cfg(feature = "encode")]
/// Helper trait for building encoded buffers with big-endian byte order
/// All multi-byte integers MUST be big-endian per DLMS specification
pub trait ByteBuffer {
    fn push_u8(&mut self, value: u8);
    fn push_u16(&mut self, value: u16); // Big-endian
    fn push_u32(&mut self, value: u32); // Big-endian
    fn push_u64(&mut self, value: u64); // Big-endian
    fn push_i8(&mut self, value: i8);
    fn push_i16(&mut self, value: i16); // Big-endian
    fn push_i32(&mut self, value: i32); // Big-endian
    fn push_i64(&mut self, value: i64); // Big-endian
    fn push_bytes(&mut self, bytes: &[u8]);
}

#[cfg(feature = "encode")]
impl ByteBuffer for Vec<u8> {
    fn push_u8(&mut self, value: u8) {
        self.push(value);
    }

    fn push_u16(&mut self, value: u16) {
        self.extend_from_slice(&value.to_be_bytes());
    }

    fn push_u32(&mut self, value: u32) {
        self.extend_from_slice(&value.to_be_bytes());
    }

    fn push_u64(&mut self, value: u64) {
        self.extend_from_slice(&value.to_be_bytes());
    }

    fn push_i8(&mut self, value: i8) {
        self.push(value as u8);
    }

    fn push_i16(&mut self, value: i16) {
        self.extend_from_slice(&value.to_be_bytes());
    }

    fn push_i32(&mut self, value: i32) {
        self.extend_from_slice(&value.to_be_bytes());
    }

    fn push_i64(&mut self, value: i64) {
        self.extend_from_slice(&value.to_be_bytes());
    }

    fn push_bytes(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
}

impl Data {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, data_type) = u8(input)?;
        let data_type = DataType::try_from(data_type).map_err(|_| {
            nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Fail))
        })?;
        Ok(match data_type {
            DataType::DateTime => {
                let (input, date_time) = DateTime::parse(input)?;
                (input, Data::DateTime(date_time))
            }
            DataType::Date => {
                let (input, date) = Date::parse(input)?;
                (input, Data::Date(date))
            }
            DataType::Time => {
                let (input, time) = Time::parse(input)?;
                (input, Data::Time(time))
            }
            DataType::Null => (input, Data::Null),
            DataType::Structure => {
                let (input, structure) = length_count(u8, Self::parse).parse(input)?;
                (input, Data::Structure(structure))
            }
            DataType::OctetString => {
                let (input, bytes) = length_count(u8, u8).parse(input)?;
                (input, Data::OctetString(bytes))
            }
            DataType::Float32 => {
                let (input, n) = be_f32(input)?;
                (input, Data::Float32(n))
            }
            DataType::Float64 => {
                let (input, n) = be_f64(input)?;
                (input, Data::Float64(n))
            }
            DataType::Integer => {
                let (input, n) = i8(input)?;
                (input, Data::Integer(n))
            }
            DataType::Long => {
                let (input, n) = be_i16(input)?;
                (input, Data::Long(n))
            }
            DataType::DoubleLong => {
                let (input, n) = be_i32(input)?;
                (input, Data::DoubleLong(n))
            }
            DataType::Long64 => {
                let (input, n) = be_i64(input)?;
                (input, Data::Long64(n))
            }
            DataType::Enum => {
                let (input, n) = u8(input)?;
                (input, Data::Enum(n))
            }
            DataType::LongUnsigned => {
                let (input, n) = be_u16(input)?;
                (input, Data::LongUnsigned(n))
            }
            DataType::DoubleLongUnsigned => {
                let (input, n) = be_u32(input)?;
                (input, Data::DoubleLongUnsigned(n))
            }
            DataType::Long64Unsigned => {
                let (input, n) = be_u64(input)?;
                (input, Data::Long64Unsigned(n))
            }
            DataType::Unsigned => {
                let (input, n) = u8(input)?;
                (input, Data::Unsigned(n))
            }
            DataType::Utf8String => {
                let (input, bytes) = length_count(u8, u8).parse(input)?;
                let string = String::from_utf8(bytes).map_err(|_| {
                    nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Fail))
                })?;
                (input, Data::Utf8String(string))
            }
            dt => unimplemented!("decoding data type {:?}", dt),
        })
    }

    #[cfg(feature = "encode")]
    /// Encode the Data value to A-XDR format (tag + data)
    /// Returns a Vec<u8> containing the encoded bytes
    /// Reference: Green Book Ed. 12, Section 4.1.6
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(self.encoded_len());

        match self {
            // Null type: only the tag, no data
            Data::Null => {
                buffer.push_u8(0x00);
            }

            // Integer: tag + i8 value
            Data::Integer(value) => {
                buffer.push_u8(0x0F);
                buffer.push_i8(*value);
            }

            // Unsigned: tag + u8 value
            Data::Unsigned(value) => {
                buffer.push_u8(0x11);
                buffer.push_u8(*value);
            }

            // Long: tag + i16 value (big-endian)
            Data::Long(value) => {
                buffer.push_u8(0x10);
                buffer.push_i16(*value);
            }

            // LongUnsigned: tag + u16 value (big-endian)
            Data::LongUnsigned(value) => {
                buffer.push_u8(0x12);
                buffer.push_u16(*value);
            }

            // DoubleLong: tag + i32 value (big-endian)
            Data::DoubleLong(value) => {
                buffer.push_u8(0x05);
                buffer.push_i32(*value);
            }

            // DoubleLongUnsigned: tag + u32 value (big-endian)
            Data::DoubleLongUnsigned(value) => {
                buffer.push_u8(0x06);
                buffer.push_u32(*value);
            }

            // Long64: tag + i64 value (big-endian)
            Data::Long64(value) => {
                buffer.push_u8(0x14);
                buffer.push_i64(*value);
            }

            // Long64Unsigned: tag + u64 value (big-endian)
            Data::Long64Unsigned(value) => {
                buffer.push_u8(0x15);
                buffer.push_u64(*value);
            }

            // Enum: tag + u8 value
            Data::Enum(value) => {
                buffer.push_u8(0x16);
                buffer.push_u8(*value);
            }

            // Float32: tag + IEEE 754 single precision (big-endian)
            Data::Float32(value) => {
                buffer.push_u8(0x17);
                buffer.push_u32(value.to_bits());
            }

            // Float64: tag + IEEE 754 double precision (big-endian)
            Data::Float64(value) => {
                buffer.push_u8(0x18);
                buffer.push_u64(value.to_bits());
            }

            // OctetString: tag + length + bytes
            Data::OctetString(bytes) => {
                buffer.push_u8(0x09);
                buffer.push_u8(bytes.len() as u8);
                buffer.push_bytes(bytes);
            }

            // Utf8String: tag + length + UTF-8 bytes
            Data::Utf8String(string) => {
                buffer.push_u8(0x0C);
                let bytes = string.as_bytes();
                buffer.push_u8(bytes.len() as u8);
                buffer.push_bytes(bytes);
            }

            // DateTime: tag + encoded DateTime
            Data::DateTime(dt) => {
                buffer.push_u8(0x19);
                buffer.push_bytes(&dt.encode());
            }

            // Date: tag + encoded Date
            Data::Date(date) => {
                buffer.push_u8(0x1A);
                buffer.push_bytes(&date.encode());
            }

            // Time: tag + encoded Time
            Data::Time(time) => {
                buffer.push_u8(0x1B);
                buffer.push_bytes(&time.encode());
            }

            // Structure: tag + count + encoded elements
            Data::Structure(elements) => {
                buffer.push_u8(0x02);
                buffer.push_u8(elements.len() as u8);
                for element in elements {
                    buffer.push_bytes(&element.encode());
                }
            }
        }

        buffer
    }

    #[cfg(feature = "encode")]
    /// Calculate the encoded length without allocating
    /// Useful for pre-allocating buffers
    pub fn encoded_len(&self) -> usize {
        match self {
            Data::Null => 1,                                  // Just the tag
            Data::Integer(_) => 2,                            // Tag + i8
            Data::Unsigned(_) => 2,                           // Tag + u8
            Data::Long(_) => 3,                               // Tag + i16
            Data::LongUnsigned(_) => 3,                       // Tag + u16
            Data::DoubleLong(_) => 5,                         // Tag + i32
            Data::DoubleLongUnsigned(_) => 5,                 // Tag + u32
            Data::Long64(_) => 9,                             // Tag + i64
            Data::Long64Unsigned(_) => 9,                     // Tag + u64
            Data::Enum(_) => 2,                               // Tag + u8
            Data::Float32(_) => 5,                            // Tag + f32
            Data::Float64(_) => 9,                            // Tag + f64
            Data::OctetString(bytes) => 1 + 1 + bytes.len(),  // Tag + length + data
            Data::Utf8String(string) => 1 + 1 + string.len(), // Tag + length + UTF-8 bytes
            Data::DateTime(_) => 1 + 12,                      // Tag + 12 bytes for DateTime
            Data::Date(_) => 1 + 5,                           // Tag + 5 bytes for Date
            Data::Time(_) => 1 + 4,                           // Tag + 4 bytes for Time
            Data::Structure(elements) => {
                1 + 1 + elements.iter().map(|e| e.encoded_len()).sum::<usize>()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // DataType tests
    #[test]
    fn test_data_type_try_from_valid() {
        assert_eq!(DataType::try_from(0x00).unwrap(), DataType::Null);
        assert_eq!(DataType::try_from(0x01).unwrap(), DataType::Array);
        assert_eq!(DataType::try_from(0x02).unwrap(), DataType::Structure);
        assert_eq!(DataType::try_from(0x03).unwrap(), DataType::Bool);
        assert_eq!(DataType::try_from(0x09).unwrap(), DataType::OctetString);
        assert_eq!(DataType::try_from(0x0f).unwrap(), DataType::Integer);
        assert_eq!(DataType::try_from(0x10).unwrap(), DataType::Long);
        assert_eq!(DataType::try_from(0x11).unwrap(), DataType::Unsigned);
        assert_eq!(DataType::try_from(0x12).unwrap(), DataType::LongUnsigned);
        assert_eq!(DataType::try_from(0x16).unwrap(), DataType::Enum);
        assert_eq!(DataType::try_from(0x17).unwrap(), DataType::Float32);
        assert_eq!(DataType::try_from(0x18).unwrap(), DataType::Float64);
        assert_eq!(DataType::try_from(0x19).unwrap(), DataType::DateTime);
        assert_eq!(DataType::try_from(0x1a).unwrap(), DataType::Date);
        assert_eq!(DataType::try_from(0x1b).unwrap(), DataType::Time);
    }

    #[test]
    fn test_data_type_try_from_invalid() {
        assert!(DataType::try_from(0x07).is_err());
        assert!(DataType::try_from(0x08).is_err());
        assert!(DataType::try_from(0x0b).is_err());
        assert!(DataType::try_from(0x0e).is_err());
        assert!(DataType::try_from(0xFF).is_err());
    }

    // Date tests
    #[test]
    fn test_date_parse() {
        let input = [0x07, 0xE9, 0x01, 0x0F, 0x01, 0xFF];
        let (remaining, date) = Date::parse(&input).unwrap();

        assert_eq!(remaining, &[0xFF]);
        assert_eq!(date.year, 2025);
        assert_eq!(date.month, 1);
        assert_eq!(date.day_of_month, 15);
        assert_eq!(date.day_of_week, 1);
    }

    #[test]
    fn test_date_parse_wildcard() {
        // 0xFFFF for year, 0xFF for others = wildcard/not specified
        let input = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        let (remaining, date) = Date::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(date.year, 0xFFFF);
        assert_eq!(date.month, 0xFF);
        assert_eq!(date.day_of_month, 0xFF);
        assert_eq!(date.day_of_week, 0xFF);
    }

    #[test]
    fn test_date_display() {
        let date = Date { year: 2025, month: 1, day_of_month: 15, day_of_week: 1 };
        assert_eq!(format!("{}", date), "2025-01-15");

        // With wildcards (displayed as numbers, not wildcards)
        let date = Date { year: 0xFFFF, month: 0xFF, day_of_month: 0xFF, day_of_week: 0xFF };
        assert_eq!(format!("{}", date), "65535-255-255");
    }

    #[test]
    fn test_date_clone_and_equality() {
        let date1 = Date { year: 2025, month: 1, day_of_month: 15, day_of_week: 1 };
        let date2 = date1.clone();

        assert_eq!(date1, date2);
    }

    // Time tests
    #[test]
    fn test_time_parse() {
        let input = [0x0C, 0x1E, 0x00, 0x00, 0xFF];
        let (remaining, time) = Time::parse(&input).unwrap();

        assert_eq!(remaining, &[0xFF]);
        assert_eq!(time.hour, Some(12));
        assert_eq!(time.minute, Some(30));
        assert_eq!(time.second, Some(0));
        assert_eq!(time.hundredth, Some(0));
    }

    #[test]
    fn test_time_parse_wildcard() {
        let input = [0xFF, 0xFF, 0xFF, 0xFF];
        let (remaining, time) = Time::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(time.hour, None);
        assert_eq!(time.minute, None);
        assert_eq!(time.second, None);
        assert_eq!(time.hundredth, None);
    }

    #[test]
    fn test_time_parse_invalid_hour() {
        let input = [0x18, 0x00, 0x00, 0x00]; // 24 is invalid
        let result = Time::parse(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_time_parse_invalid_minute() {
        let input = [0x0C, 0x3C, 0x00, 0x00]; // 60 is invalid
        let result = Time::parse(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_time_parse_invalid_second() {
        let input = [0x0C, 0x1E, 0x3C, 0x00]; // 60 is invalid
        let result = Time::parse(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_time_parse_invalid_hundredth() {
        let input = [0x0C, 0x1E, 0x00, 0x64]; // 100 is invalid
        let result = Time::parse(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_time_display() {
        let time = Time { hour: Some(12), minute: Some(30), second: Some(45), hundredth: Some(50) };
        assert_eq!(format!("{}", time), "12:30:45.50");

        // With None values (displayed as empty/dashes)
        let time = Time { hour: None, minute: None, second: None, hundredth: None };
        let display = format!("{}", time);
        // Just verify it doesn't panic, format may vary
        assert!(!display.is_empty());
    }

    // Data tests - Null
    #[test]
    fn test_data_parse_null() {
        let input = [0x00, 0xFF];
        let (remaining, data) = Data::parse(&input).unwrap();

        assert_eq!(remaining, &[0xFF]);
        assert_eq!(data, Data::Null);
    }

    // Data tests - Integer types
    #[test]
    fn test_data_parse_integer() {
        let input = [0x0f, 0x2A, 0xFF]; // Integer = 42
        let (remaining, data) = Data::parse(&input).unwrap();

        assert_eq!(remaining, &[0xFF]);
        assert_eq!(data, Data::Integer(42));
    }

    #[test]
    fn test_data_parse_integer_negative() {
        let input = [0x0f, 0xD6]; // Integer = -42
        let (remaining, data) = Data::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(data, Data::Integer(-42));
    }

    #[test]
    fn test_data_parse_unsigned() {
        // Unsigned type parsing
        let input = [0x11, 0x2A];
        let (remaining, data) = Data::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(data, Data::Unsigned(0x2A));
    }

    #[test]
    fn test_data_parse_long() {
        let input = [0x10, 0x01, 0x00]; // Long = 256
        let (remaining, data) = Data::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(data, Data::Long(256));
    }

    #[test]
    fn test_data_parse_long_unsigned() {
        let input = [0x12, 0x01, 0x00]; // LongUnsigned = 256
        let (remaining, data) = Data::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(data, Data::LongUnsigned(256));
    }

    #[test]
    fn test_data_parse_double_long() {
        let input = [0x05, 0x00, 0x00, 0x01, 0x00]; // DoubleLong = 256
        let (remaining, data) = Data::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(data, Data::DoubleLong(256));
    }

    #[test]
    fn test_data_parse_double_long_unsigned() {
        let input = [0x06, 0x00, 0x00, 0x01, 0x00]; // DoubleLongUnsigned = 256
        let (remaining, data) = Data::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(data, Data::DoubleLongUnsigned(256));
    }

    #[test]
    fn test_data_parse_long64() {
        let input = [0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00];
        let (remaining, data) = Data::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(data, Data::Long64(256));
    }

    #[test]
    fn test_data_parse_long64_unsigned() {
        let input = [0x15, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00];
        let (remaining, data) = Data::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(data, Data::Long64Unsigned(256));
    }

    #[test]
    fn test_data_parse_enum() {
        let input = [0x16, 0x05];
        let (remaining, data) = Data::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(data, Data::Enum(5));
    }

    // Data tests - Float types
    #[test]
    fn test_data_parse_float32() {
        let input = [0x17, 0x42, 0x28, 0x00, 0x00]; // Float32 = 42.0
        let (remaining, data) = Data::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(data, Data::Float32(42.0));
    }

    #[test]
    fn test_data_parse_float64() {
        let input = [0x18, 0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // Float64 = 42.0
        let (remaining, data) = Data::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(data, Data::Float64(42.0));
    }

    // Data tests - OctetString
    #[test]
    fn test_data_parse_octet_string() {
        let input = [0x09, 0x04, 0xAA, 0xBB, 0xCC, 0xDD, 0xFF];
        let (remaining, data) = Data::parse(&input).unwrap();

        assert_eq!(remaining, &[0xFF]);
        assert_eq!(data, Data::OctetString(vec![0xAA, 0xBB, 0xCC, 0xDD]));
    }

    #[test]
    fn test_data_parse_octet_string_empty() {
        let input = [0x09, 0x00];
        let (remaining, data) = Data::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(data, Data::OctetString(vec![]));
    }

    // Data tests - Structure
    #[test]
    fn test_data_parse_structure_empty() {
        let input = [0x02, 0x00];
        let (remaining, data) = Data::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(data, Data::Structure(vec![]));
    }

    #[test]
    fn test_data_parse_structure_simple() {
        // Structure with Integer values (Unsigned not implemented yet)
        let input = [0x02, 0x02, 0x0f, 0x2A, 0x0f, 0x0D];
        let (remaining, data) = Data::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(data, Data::Structure(vec![Data::Integer(42), Data::Integer(13),]));
    }

    #[test]
    fn test_data_parse_structure_nested() {
        // Nested structure with Integer values
        let input = [0x02, 0x02, 0x02, 0x01, 0x0f, 0x01, 0x0f, 0x02];
        let (remaining, data) = Data::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(
            data,
            Data::Structure(vec![Data::Structure(vec![Data::Integer(1)]), Data::Integer(2),])
        );
    }

    // Data tests - Date, Time, DateTime
    #[test]
    fn test_data_parse_date() {
        let input = [0x1a, 0x07, 0xE9, 0x01, 0x0F, 0x01];
        let (remaining, data) = Data::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        match data {
            Data::Date(date) => {
                assert_eq!(date.year, 2025);
                assert_eq!(date.month, 1);
                assert_eq!(date.day_of_month, 15);
            }
            _ => panic!("Expected Data::Date"),
        }
    }

    #[test]
    fn test_data_parse_time() {
        let input = [0x1b, 0x0C, 0x1E, 0x00, 0x00];
        let (remaining, data) = Data::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        match data {
            Data::Time(time) => {
                assert_eq!(time.hour, Some(12));
                assert_eq!(time.minute, Some(30));
            }
            _ => panic!("Expected Data::Time"),
        }
    }

    #[test]
    fn test_data_parse_datetime() {
        let input = [
            0x19, // DateTime tag
            0x07, 0xE9, 0x01, 0x0F, 0x01, // Date: 2025-01-15
            0x0C, 0x1E, 0x00, 0x00, // Time: 12:30:00.00
            0x80, 0x00, // Offset: not specified
            0xFF, // Clock status: not specified
        ];
        let (remaining, data) = Data::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        match data {
            Data::DateTime(dt) => {
                assert_eq!(dt.date.year, 2025);
                assert_eq!(dt.time.hour, Some(12));
            }
            _ => panic!("Expected Data::DateTime"),
        }
    }

    // Data tests - Invalid type
    #[test]
    fn test_data_parse_invalid_type() {
        let input = [0x07]; // Invalid data type
        let result = Data::parse(&input);
        assert!(result.is_err());
    }

    // Data tests - Clone and equality
    #[test]
    fn test_data_clone() {
        let data1 = Data::Integer(42);
        let data2 = data1.clone();
        assert_eq!(data1, data2);

        let data1 = Data::OctetString(vec![1, 2, 3]);
        let data2 = data1.clone();
        assert_eq!(data1, data2);
    }

    #[test]
    fn test_data_equality() {
        assert_eq!(Data::Null, Data::Null);
        assert_eq!(Data::Integer(42), Data::Integer(42));
        assert_ne!(Data::Integer(42), Data::Integer(43));
        assert_ne!(Data::Integer(42), Data::Unsigned(42));
    }

    #[test]
    fn test_data_debug() {
        let data = Data::Integer(42);
        let debug_str = format!("{:?}", data);
        assert!(debug_str.contains("Integer"));
        assert!(debug_str.contains("42"));
    }

    // Serialization tests - verify Serialize trait is implemented
    #[test]
    #[cfg(feature = "serde")]
    fn test_date_serialize() {
        use serde::Serialize;
        let input = [0x07, 0xE5, 12, 25, 6]; // 2021-12-25, Saturday
        let (_remaining, date) = Date::parse(&input[..]).unwrap();

        // Verify Display output which is used by Serialize
        let display = format!("{}", date);
        assert_eq!(display, "2021-12-25");

        // Verify the trait is implemented (compile-time check)
        fn assert_serialize<T: Serialize>(_: &T) {}
        assert_serialize(&date);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_time_serialize() {
        use serde::Serialize;
        let input = [14, 30, 45, 50]; // 14:30:45.50
        let (_remaining, time) = Time::parse(&input[..]).unwrap();

        // Verify Display output which is used by Serialize
        let display = format!("{}", time);
        assert_eq!(display, "14:30:45.50");

        // Verify the trait is implemented (compile-time check)
        fn assert_serialize<T: Serialize>(_: &T) {}
        assert_serialize(&time);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_datetime_serialize() {
        use serde::Serialize;
        let input = [
            0x07, 0xE5, // year 2021
            12,   // month
            25,   // day
            6,    // day of week
            14,   // hour
            30,   // minute
            0,    // second
            0,    // hundredth
            0xFF, 0x80, // deviation not specified
            0x00, // clock status
        ];
        let (_remaining, datetime) = DateTime::parse(&input[..]).unwrap();

        // Verify Display output which is used by Serialize
        let display = format!("{}", datetime);
        assert!(display.contains("2021-12-25"));
        assert!(display.contains("14:30:00"));

        // Verify the trait is implemented (compile-time check)
        fn assert_serialize<T: Serialize>(_: &T) {}
        assert_serialize(&datetime);
    }

    // ClockStatus bit flag tests
    #[test]
    fn test_clock_status_invalid_value() {
        let status = ClockStatus(0b00000001);
        assert!(status.invalid_value());
        assert!(!status.doubtful_value());
        assert!(!status.different_base());
        assert!(!status.invalid_status());
        assert!(!status.daylight_saving());
    }

    #[test]
    fn test_clock_status_doubtful_value() {
        let status = ClockStatus(0b00000010);
        assert!(!status.invalid_value());
        assert!(status.doubtful_value());
        assert!(!status.different_base());
        assert!(!status.invalid_status());
        assert!(!status.daylight_saving());
    }

    #[test]
    fn test_clock_status_different_base() {
        let status = ClockStatus(0b00000100);
        assert!(!status.invalid_value());
        assert!(!status.doubtful_value());
        assert!(status.different_base());
        assert!(!status.invalid_status());
        assert!(!status.daylight_saving());
    }

    #[test]
    fn test_clock_status_invalid_status() {
        let status = ClockStatus(0b00001000);
        assert!(!status.invalid_value());
        assert!(!status.doubtful_value());
        assert!(!status.different_base());
        assert!(status.invalid_status());
        assert!(!status.daylight_saving());
    }

    #[test]
    fn test_clock_status_daylight_saving() {
        let status = ClockStatus(0b10000000);
        assert!(!status.invalid_value());
        assert!(!status.doubtful_value());
        assert!(!status.different_base());
        assert!(!status.invalid_status());
        assert!(status.daylight_saving());
    }

    #[test]
    fn test_clock_status_multiple_flags() {
        // Multiple flags set at once
        let status = ClockStatus(0b10000011); // invalid_value + doubtful_value + daylight_saving
        assert!(status.invalid_value());
        assert!(status.doubtful_value());
        assert!(!status.different_base());
        assert!(!status.invalid_status());
        assert!(status.daylight_saving());
    }

    #[test]
    fn test_clock_status_no_flags() {
        let status = ClockStatus(0b00000000);
        assert!(!status.invalid_value());
        assert!(!status.doubtful_value());
        assert!(!status.different_base());
        assert!(!status.invalid_status());
        assert!(!status.daylight_saving());
    }

    // Uncommon DataType enum variants tests
    #[test]
    fn test_datatype_bitstring() {
        let data_type = DataType::BitString;
        assert_eq!(data_type as u8, 4);
    }

    #[test]
    fn test_datatype_double_long() {
        let data_type = DataType::DoubleLong;
        assert_eq!(data_type as u8, 5);
    }

    #[test]
    fn test_datatype_double_long_unsigned() {
        let data_type = DataType::DoubleLongUnsigned;
        assert_eq!(data_type as u8, 6);
    }

    #[test]
    fn test_datatype_binary_coded_decimal() {
        let data_type = DataType::BinaryCodedDecimal;
        assert_eq!(data_type as u8, 13);
    }

    #[test]
    fn test_datatype_float32() {
        let data_type = DataType::Float32;
        assert_eq!(data_type as u8, 23);
    }

    #[test]
    fn test_datatype_float64() {
        let data_type = DataType::Float64;
        assert_eq!(data_type as u8, 24);
    }

    #[test]
    fn test_datatype_from_u8_bitstring() {
        let result = DataType::try_from(0x04u8);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DataType::BitString);
    }

    #[test]
    fn test_datatype_from_u8_double_long() {
        let result = DataType::try_from(0x05u8);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DataType::DoubleLong);
    }

    #[test]
    fn test_datatype_from_u8_double_long_unsigned() {
        let result = DataType::try_from(0x06u8);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DataType::DoubleLongUnsigned);
    }

    #[test]
    fn test_datatype_from_u8_binary_coded_decimal() {
        let result = DataType::try_from(0x0du8);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DataType::BinaryCodedDecimal);
    }

    #[test]
    fn test_datatype_from_u8_float32() {
        let result = DataType::try_from(0x17u8);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DataType::Float32);
    }

    #[test]
    fn test_datatype_from_u8_float64() {
        let result = DataType::try_from(0x18u8);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DataType::Float64);
    }

    #[test]
    fn test_datatype_from_u8_invalid() {
        let result = DataType::try_from(0x99u8);
        assert!(result.is_err());
    }

    // ====================
    // ENCODING TESTS (TDD)
    // ====================
    // Following Phase 1.1 of IMPLEMENTATION_ROADMAP.md
    // Green Book Ed. 12: Section 4.1.5 - Encoding of data types

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_null() {
        // Green Book: Data type 0x00 = Null, no content
        let data = Data::Null;
        let encoded = data.encode();

        // Expected: [0x00] - just the type tag
        assert_eq!(encoded, vec![0x00]);
        assert_eq!(data.encoded_len(), 1);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_null_roundtrip() {
        let original = Data::Null;
        let encoded = original.encode();
        let (remaining, parsed) = Data::parse(&encoded).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(parsed, original);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_integer() {
        // Green Book: Data type 0x0F = Integer (i8)
        let data = Data::Integer(42);
        let encoded = data.encode();

        // Expected: [0x0F, 0x2A] - type tag + value
        assert_eq!(encoded, vec![0x0F, 0x2A]);
        assert_eq!(data.encoded_len(), 2);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_integer_negative() {
        let data = Data::Integer(-42);
        let encoded = data.encode();

        // Expected: [0x0F, 0xD6] - type tag + two's complement
        assert_eq!(encoded, vec![0x0F, 0xD6]);
        assert_eq!(data.encoded_len(), 2);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_integer_roundtrip() {
        let original = Data::Integer(-123);
        let encoded = original.encode();
        let (remaining, parsed) = Data::parse(&encoded).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(parsed, original);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_unsigned() {
        // Green Book: Data type 0x11 = Unsigned (u8)
        let data = Data::Unsigned(255);
        let encoded = data.encode();

        // Expected: [0x11, 0xFF]
        assert_eq!(encoded, vec![0x11, 0xFF]);
        assert_eq!(data.encoded_len(), 2);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_unsigned_roundtrip() {
        let original = Data::Unsigned(200);
        let encoded = original.encode();
        let (remaining, parsed) = Data::parse(&encoded).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(parsed, original);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_long() {
        // Green Book: Data type 0x10 = Long (i16), big-endian
        let data = Data::Long(1234);
        let encoded = data.encode();

        // Expected: [0x10, 0x04, 0xD2] - type tag + big-endian i16
        assert_eq!(encoded, vec![0x10, 0x04, 0xD2]);
        assert_eq!(data.encoded_len(), 3);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_long_negative() {
        let data = Data::Long(-1234);
        let encoded = data.encode();

        // Expected: [0x10, 0xFB, 0x2E]
        assert_eq!(encoded, vec![0x10, 0xFB, 0x2E]);
        assert_eq!(data.encoded_len(), 3);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_long_roundtrip() {
        let original = Data::Long(-5678);
        let encoded = original.encode();
        let (remaining, parsed) = Data::parse(&encoded).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(parsed, original);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_long_unsigned() {
        // Green Book: Data type 0x12 = LongUnsigned (u16), big-endian
        let data = Data::LongUnsigned(65535);
        let encoded = data.encode();

        // Expected: [0x12, 0xFF, 0xFF]
        assert_eq!(encoded, vec![0x12, 0xFF, 0xFF]);
        assert_eq!(data.encoded_len(), 3);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_long_unsigned_roundtrip() {
        let original = Data::LongUnsigned(12345);
        let encoded = original.encode();
        let (remaining, parsed) = Data::parse(&encoded).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(parsed, original);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_double_long() {
        // Green Book: Data type 0x05 = DoubleLong (i32), big-endian
        let data = Data::DoubleLong(123456789);
        let encoded = data.encode();

        // Expected: [0x05, 0x07, 0x5B, 0xCD, 0x15]
        assert_eq!(encoded, vec![0x05, 0x07, 0x5B, 0xCD, 0x15]);
        assert_eq!(data.encoded_len(), 5);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_double_long_roundtrip() {
        let original = Data::DoubleLong(-987654321);
        let encoded = original.encode();
        let (remaining, parsed) = Data::parse(&encoded).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(parsed, original);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_double_long_unsigned() {
        // Green Book: Data type 0x06 = DoubleLongUnsigned (u32)
        let data = Data::DoubleLongUnsigned(4294967295);
        let encoded = data.encode();

        // Expected: [0x06, 0xFF, 0xFF, 0xFF, 0xFF]
        assert_eq!(encoded, vec![0x06, 0xFF, 0xFF, 0xFF, 0xFF]);
        assert_eq!(data.encoded_len(), 5);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_double_long_unsigned_roundtrip() {
        let original = Data::DoubleLongUnsigned(123456789);
        let encoded = original.encode();
        let (remaining, parsed) = Data::parse(&encoded).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(parsed, original);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_long64() {
        // Green Book: Data type 0x14 = Long64 (i64), big-endian
        let data = Data::Long64(9223372036854775807);
        let encoded = data.encode();

        // Expected: [0x14, 0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
        assert_eq!(encoded, vec![0x14, 0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        assert_eq!(data.encoded_len(), 9);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_long64_roundtrip() {
        let original = Data::Long64(-1234567890123456789);
        let encoded = original.encode();
        let (remaining, parsed) = Data::parse(&encoded).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(parsed, original);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_long64_unsigned() {
        // Green Book: Data type 0x15 = Long64Unsigned (u64)
        let data = Data::Long64Unsigned(18446744073709551615);
        let encoded = data.encode();

        // Expected: [0x15, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
        assert_eq!(encoded, vec![0x15, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        assert_eq!(data.encoded_len(), 9);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_long64_unsigned_roundtrip() {
        let original = Data::Long64Unsigned(9876543210123456789);
        let encoded = original.encode();
        let (remaining, parsed) = Data::parse(&encoded).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(parsed, original);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_enum() {
        // Green Book: Data type 0x16 = Enum (u8)
        let data = Data::Enum(42);
        let encoded = data.encode();

        // Expected: [0x16, 0x2A]
        assert_eq!(encoded, vec![0x16, 0x2A]);
        assert_eq!(data.encoded_len(), 2);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_enum_roundtrip() {
        let original = Data::Enum(255);
        let encoded = original.encode();
        let (remaining, parsed) = Data::parse(&encoded).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(parsed, original);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_float32() {
        // Green Book: Data type 0x17 = Float32 (IEEE 754 single precision, big-endian)
        let data = Data::Float32(42.0);
        let encoded = data.encode();

        // Expected: [0x17, 0x42, 0x28, 0x00, 0x00]
        assert_eq!(encoded, vec![0x17, 0x42, 0x28, 0x00, 0x00]);
        assert_eq!(data.encoded_len(), 5);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_float32_negative() {
        let data = Data::Float32(-3.14);
        let encoded = data.encode();

        // Expected: [0x17] + big-endian IEEE 754 representation
        assert_eq!(encoded[0], 0x17);
        assert_eq!(data.encoded_len(), 5);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_float32_roundtrip() {
        let original = Data::Float32(123.456);
        let encoded = original.encode();
        let (remaining, parsed) = Data::parse(&encoded).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(parsed, original);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_float64() {
        // Green Book: Data type 0x18 = Float64 (IEEE 754 double precision, big-endian)
        let data = Data::Float64(42.0);
        let encoded = data.encode();

        // Expected: [0x18, 0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        assert_eq!(encoded, vec![0x18, 0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(data.encoded_len(), 9);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_float64_roundtrip() {
        let original = Data::Float64(-987.654321);
        let encoded = original.encode();
        let (remaining, parsed) = Data::parse(&encoded).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(parsed, original);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_octet_string() {
        // Green Book: Data type 0x09 = OctetString, length-prefixed
        let data = Data::OctetString(vec![0xAA, 0xBB, 0xCC, 0xDD]);
        let encoded = data.encode();

        // Expected: [0x09, 0x04, 0xAA, 0xBB, 0xCC, 0xDD]
        assert_eq!(encoded, vec![0x09, 0x04, 0xAA, 0xBB, 0xCC, 0xDD]);
        assert_eq!(data.encoded_len(), 6);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_octet_string_empty() {
        let data = Data::OctetString(vec![]);
        let encoded = data.encode();

        // Expected: [0x09, 0x00]
        assert_eq!(encoded, vec![0x09, 0x00]);
        assert_eq!(data.encoded_len(), 2);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_octet_string_roundtrip() {
        let original = Data::OctetString(vec![0x01, 0x02, 0x03, 0x04, 0x05]);
        let encoded = original.encode();
        let (remaining, parsed) = Data::parse(&encoded).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(parsed, original);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_utf8_string() {
        // Green Book: Data type 0x0C = Utf8String, length-prefixed
        let data = Data::Utf8String("Hello".to_string());
        let encoded = data.encode();

        // Expected: [0x0C, 0x05, 'H', 'e', 'l', 'l', 'o']
        assert_eq!(encoded, vec![0x0C, 0x05, b'H', b'e', b'l', b'l', b'o']);
        assert_eq!(data.encoded_len(), 7);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_utf8_string_empty() {
        let data = Data::Utf8String(String::new());
        let encoded = data.encode();

        // Expected: [0x0C, 0x00]
        assert_eq!(encoded, vec![0x0C, 0x00]);
        assert_eq!(data.encoded_len(), 2);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_date() {
        // Green Book: Data type 0x1A = Date (5 bytes)
        let date = Date { year: 2024, month: 12, day_of_month: 25, day_of_week: 3 };
        let data = Data::Date(date);
        let encoded = data.encode();

        // Expected: [0x1A, year_high, year_low, month, day_of_month, day_of_week]
        // 2024 = 0x07E8
        assert_eq!(encoded, vec![0x1A, 0x07, 0xE8, 0x0C, 0x19, 0x03]);
        assert_eq!(data.encoded_len(), 6);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_date_roundtrip() {
        let date = Date { year: 2025, month: 1, day_of_month: 24, day_of_week: 5 };
        let original = Data::Date(date);
        let encoded = original.encode();
        let (remaining, parsed) = Data::parse(&encoded).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(parsed, original);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_time() {
        // Green Book: Data type 0x1B = Time (4 bytes)
        let time = Time { hour: Some(14), minute: Some(30), second: Some(45), hundredth: Some(99) };
        let data = Data::Time(time);
        let encoded = data.encode();

        // Expected: [0x1B, 0x0E, 0x1E, 0x2D, 0x63]
        assert_eq!(encoded, vec![0x1B, 0x0E, 0x1E, 0x2D, 0x63]);
        assert_eq!(data.encoded_len(), 5);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_time_with_wildcards() {
        // Test encoding with None values (wildcards = 0xFF)
        let time = Time { hour: Some(12), minute: None, second: Some(30), hundredth: None };
        let data = Data::Time(time);
        let encoded = data.encode();

        // Expected: [0x1B, 0x0C, 0xFF, 0x1E, 0xFF]
        assert_eq!(encoded, vec![0x1B, 0x0C, 0xFF, 0x1E, 0xFF]);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_time_roundtrip() {
        let time = Time { hour: Some(23), minute: Some(59), second: Some(59), hundredth: Some(0) };
        let original = Data::Time(time);
        let encoded = original.encode();
        let (remaining, parsed) = Data::parse(&encoded).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(parsed, original);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_datetime() {
        // Green Book: Data type 0x19 = DateTime (12 bytes)
        let date = Date { year: 2024, month: 1, day_of_month: 15, day_of_week: 1 };
        let time = Time { hour: Some(10), minute: Some(30), second: Some(0), hundredth: Some(0) };
        let datetime = DateTime {
            date,
            time,
            offset_minutes: Some(60), // UTC+1
            clock_status: Some(ClockStatus(0x00)),
        };
        let data = Data::DateTime(datetime);
        let encoded = data.encode();

        // Expected: [0x19] + date(5) + time(4) + offset(2) + status(1)
        // 2024 = 0x07E8, offset 60 = 0x003C
        assert_eq!(
            encoded,
            vec![0x19, 0x07, 0xE8, 0x01, 0x0F, 0x01, 0x0A, 0x1E, 0x00, 0x00, 0x00, 0x3C, 0x00]
        );
        assert_eq!(data.encoded_len(), 13);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_datetime_roundtrip() {
        let date = Date { year: 2025, month: 12, day_of_month: 31, day_of_week: 2 };
        let time = Time { hour: Some(23), minute: Some(59), second: Some(59), hundredth: Some(99) };
        let datetime = DateTime {
            date,
            time,
            offset_minutes: Some(-120),            // UTC-2
            clock_status: Some(ClockStatus(0x80)), // Daylight saving
        };
        let original = Data::DateTime(datetime);
        let encoded = original.encode();
        let (remaining, parsed) = Data::parse(&encoded).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(parsed, original);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_date_wildcard_year() {
        // Test wildcard year (0xFFFF)
        let date = Date { year: 0xFFFF, month: 0xFF, day_of_month: 0xFF, day_of_week: 0xFF };
        let data = Data::Date(date);
        let encoded = data.encode();

        // Expected: [0x1A, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
        assert_eq!(encoded, vec![0x1A, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);

        // Round-trip test
        let (remaining, parsed) = Data::parse(&encoded).unwrap();
        assert_eq!(remaining.len(), 0);
        assert_eq!(parsed, data);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_datetime_with_wildcards() {
        // Test DateTime with all wildcards (None values)
        let date = Date { year: 0xFFFF, month: 0xFF, day_of_month: 0xFF, day_of_week: 0xFF };
        let time = Time { hour: None, minute: None, second: None, hundredth: None };
        let datetime = DateTime {
            date,
            time,
            offset_minutes: None, // Wildcard = 0x8000
            clock_status: None,   // Wildcard = 0xFF
        };
        let data = Data::DateTime(datetime);
        let encoded = data.encode();

        // Expected: [0x19] + date(5xFF) + time(4xFF) + offset(0x8000) + status(0xFF)
        assert_eq!(
            encoded,
            vec![0x19, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x80, 0x00, 0xFF]
        );
        assert_eq!(data.encoded_len(), 13);

        // Round-trip test
        let (remaining, parsed) = Data::parse(&encoded).unwrap();
        assert_eq!(remaining.len(), 0);
        assert_eq!(parsed, data);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_datetime_timezone_extremes() {
        // Test maximum positive timezone offset (+12 hours = +720 minutes)
        let date = Date { year: 2024, month: 6, day_of_month: 15, day_of_week: 6 };
        let time = Time { hour: Some(12), minute: Some(0), second: Some(0), hundredth: Some(0) };
        let datetime_plus = DateTime {
            date: date.clone(),
            time: time.clone(),
            offset_minutes: Some(720), // UTC+12
            clock_status: Some(ClockStatus(0x00)),
        };
        let data_plus = Data::DateTime(datetime_plus);
        let encoded_plus = data_plus.encode();

        // 720 = 0x02D0
        assert_eq!(
            encoded_plus,
            vec![0x19, 0x07, 0xE8, 0x06, 0x0F, 0x06, 0x0C, 0x00, 0x00, 0x00, 0x02, 0xD0, 0x00]
        );

        // Test maximum negative timezone offset (-12 hours = -720 minutes)
        let datetime_minus = DateTime {
            date,
            time,
            offset_minutes: Some(-720), // UTC-12
            clock_status: Some(ClockStatus(0x00)),
        };
        let data_minus = Data::DateTime(datetime_minus);
        let encoded_minus = data_minus.encode();

        // -720 = 0xFD30 (two's complement)
        assert_eq!(
            encoded_minus,
            vec![0x19, 0x07, 0xE8, 0x06, 0x0F, 0x06, 0x0C, 0x00, 0x00, 0x00, 0xFD, 0x30, 0x00]
        );

        // Round-trip tests
        let (_, parsed_plus) = Data::parse(&encoded_plus).unwrap();
        assert_eq!(parsed_plus, data_plus);

        let (_, parsed_minus) = Data::parse(&encoded_minus).unwrap();
        assert_eq!(parsed_minus, data_minus);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_datetime_clock_status_bits() {
        // Test all ClockStatus bits
        let date = Date { year: 2024, month: 1, day_of_month: 1, day_of_week: 1 };
        let time = Time { hour: Some(0), minute: Some(0), second: Some(0), hundredth: Some(0) };

        // Test invalid value bit (0x01)
        let datetime = DateTime {
            date: date.clone(),
            time: time.clone(),
            offset_minutes: Some(0),
            clock_status: Some(ClockStatus(0x01)),
        };
        let data = Data::DateTime(datetime);
        let encoded = data.encode();
        assert_eq!(encoded[12], 0x01);

        // Test doubtful value bit (0x02)
        let datetime = DateTime {
            date: date.clone(),
            time: time.clone(),
            offset_minutes: Some(0),
            clock_status: Some(ClockStatus(0x02)),
        };
        let data = Data::DateTime(datetime);
        let encoded = data.encode();
        assert_eq!(encoded[12], 0x02);

        // Test daylight saving bit (0x80)
        let datetime = DateTime {
            date: date.clone(),
            time: time.clone(),
            offset_minutes: Some(0),
            clock_status: Some(ClockStatus(0x80)),
        };
        let data = Data::DateTime(datetime);
        let encoded = data.encode();
        assert_eq!(encoded[12], 0x80);

        // Test multiple bits combined (0x83 = invalid + doubtful + daylight)
        let datetime =
            DateTime { date, time, offset_minutes: Some(0), clock_status: Some(ClockStatus(0x83)) };
        let data = Data::DateTime(datetime);
        let encoded = data.encode();
        assert_eq!(encoded[12], 0x83);

        // Verify round-trip
        let (_, parsed) = Data::parse(&encoded).unwrap();
        assert_eq!(parsed, data);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_date_edge_values() {
        // Test minimum valid date values
        let date_min = Date { year: 0, month: 1, day_of_month: 1, day_of_week: 0 };
        let data_min = Data::Date(date_min);
        let encoded_min = data_min.encode();
        assert_eq!(encoded_min, vec![0x1A, 0x00, 0x00, 0x01, 0x01, 0x00]);

        // Test maximum valid date values
        let date_max = Date { year: 0xFFFE, month: 12, day_of_month: 31, day_of_week: 7 };
        let data_max = Data::Date(date_max);
        let encoded_max = data_max.encode();
        assert_eq!(encoded_max, vec![0x1A, 0xFF, 0xFE, 0x0C, 0x1F, 0x07]);

        // Round-trip tests
        let (_, parsed_min) = Data::parse(&encoded_min).unwrap();
        assert_eq!(parsed_min, data_min);

        let (_, parsed_max) = Data::parse(&encoded_max).unwrap();
        assert_eq!(parsed_max, data_max);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_time_edge_values() {
        // Test minimum valid time values
        let time_min = Time { hour: Some(0), minute: Some(0), second: Some(0), hundredth: Some(0) };
        let data_min = Data::Time(time_min);
        let encoded_min = data_min.encode();
        assert_eq!(encoded_min, vec![0x1B, 0x00, 0x00, 0x00, 0x00]);

        // Test maximum valid time values
        let time_max =
            Time { hour: Some(23), minute: Some(59), second: Some(59), hundredth: Some(99) };
        let data_max = Data::Time(time_max);
        let encoded_max = data_max.encode();
        assert_eq!(encoded_max, vec![0x1B, 0x17, 0x3B, 0x3B, 0x63]);

        // Round-trip tests
        let (_, parsed_min) = Data::parse(&encoded_min).unwrap();
        assert_eq!(parsed_min, data_min);

        let (_, parsed_max) = Data::parse(&encoded_max).unwrap();
        assert_eq!(parsed_max, data_max);
    }

    #[test]
    #[cfg(all(feature = "encode", feature = "chrono-conversions"))]
    fn test_date_from_chrono() {
        use chrono::NaiveDate;

        // Test typical date
        let naive_date = NaiveDate::from_ymd_opt(2024, 12, 25).unwrap();
        let date = Date::from_chrono(&naive_date);

        assert_eq!(date.year, 2024);
        assert_eq!(date.month, 12);
        assert_eq!(date.day_of_month, 25);
        // chrono's weekday: Mon=1, ..., Sun=7 (ISO 8601)
        // DLMS weekday: Mon=1, ..., Sun=7 (same)
        assert_eq!(date.day_of_week, 3); // Wednesday (2024-12-25)

        // Test encoding round-trip
        let data = Data::Date(date);
        let encoded = data.encode();
        let (_, parsed) = Data::parse(&encoded).unwrap();
        assert_eq!(parsed, data);
    }

    #[test]
    #[cfg(all(feature = "encode", feature = "chrono-conversions"))]
    fn test_time_from_chrono() {
        use chrono::NaiveTime;

        // Test typical time
        let naive_time = NaiveTime::from_hms_milli_opt(14, 30, 45, 500).unwrap();
        let time = Time::from_chrono(&naive_time);

        assert_eq!(time.hour, Some(14));
        assert_eq!(time.minute, Some(30));
        assert_eq!(time.second, Some(45));
        assert_eq!(time.hundredth, Some(50)); // 500ms = 50 hundredths

        // Test encoding round-trip
        let data = Data::Time(time);
        let encoded = data.encode();
        let (_, parsed) = Data::parse(&encoded).unwrap();
        assert_eq!(parsed, data);
    }

    #[test]
    #[cfg(all(feature = "encode", feature = "chrono-conversions"))]
    fn test_time_from_chrono_millisecond_rounding() {
        use chrono::NaiveTime;

        // Test millisecond rounding: 505ms -> 50 hundredths (truncated)
        let naive_time = NaiveTime::from_hms_milli_opt(12, 0, 0, 505).unwrap();
        let time = Time::from_chrono(&naive_time);
        assert_eq!(time.hundredth, Some(50));

        // Test 999ms -> 99 hundredths
        let naive_time = NaiveTime::from_hms_milli_opt(12, 0, 0, 999).unwrap();
        let time = Time::from_chrono(&naive_time);
        assert_eq!(time.hundredth, Some(99));
    }

    #[test]
    #[cfg(all(feature = "encode", feature = "chrono-conversions"))]
    fn test_datetime_from_chrono() {
        use chrono::NaiveDateTime;

        let naive_dt =
            NaiveDateTime::parse_from_str("2024-06-15 14:30:45.500", "%Y-%m-%d %H:%M:%S%.3f")
                .unwrap();
        let datetime = DateTime::from_chrono(&naive_dt, 120, 0x00); // UTC+2, no special status

        assert_eq!(datetime.date.year, 2024);
        assert_eq!(datetime.date.month, 6);
        assert_eq!(datetime.date.day_of_month, 15);
        assert_eq!(datetime.time.hour, Some(14));
        assert_eq!(datetime.time.minute, Some(30));
        assert_eq!(datetime.time.second, Some(45));
        assert_eq!(datetime.time.hundredth, Some(50));
        assert_eq!(datetime.offset_minutes, Some(120));
        assert_eq!(datetime.clock_status, Some(ClockStatus(0x00)));

        // Test encoding round-trip
        let data = Data::DateTime(datetime);
        let encoded = data.encode();
        let (_, parsed) = Data::parse(&encoded).unwrap();
        assert_eq!(parsed, data);
    }

    #[test]
    #[cfg(all(feature = "encode", feature = "chrono-conversions"))]
    fn test_datetime_from_chrono_with_timezone() {
        use chrono::NaiveDateTime;

        // Test negative timezone offset (UTC-5)
        let naive_dt =
            NaiveDateTime::parse_from_str("2024-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let datetime = DateTime::from_chrono(&naive_dt, -300, 0x80); // UTC-5, daylight saving

        assert_eq!(datetime.offset_minutes, Some(-300));
        assert_eq!(datetime.clock_status, Some(ClockStatus(0x80)));

        let data = Data::DateTime(datetime);
        let encoded = data.encode();
        let (_, parsed) = Data::parse(&encoded).unwrap();
        assert_eq!(parsed, data);
    }

    #[test]
    #[cfg(all(feature = "std", feature = "chrono-conversions"))]
    fn test_datetime_now() {
        // Test that now() creates a valid DateTime
        let datetime = DateTime::now();

        // Basic sanity checks
        assert!(datetime.date.year >= 2024); // Assuming test runs in 2024 or later
        assert!(datetime.date.month >= 1 && datetime.date.month <= 12);
        assert!(datetime.date.day_of_month >= 1 && datetime.date.day_of_month <= 31);
        assert!(datetime.time.hour.is_some());
        assert!(datetime.time.minute.is_some());
        assert!(datetime.time.second.is_some());

        // Test that it encodes correctly
        let data = Data::DateTime(datetime);
        let encoded = data.encode();
        assert_eq!(encoded.len(), 13);
        assert_eq!(encoded[0], 0x19); // DateTime type tag

        // Test round-trip
        let (_, parsed) = Data::parse(&encoded).unwrap();
        assert_eq!(parsed, data);
    }

    // Jiff conversion tests
    #[test]
    #[cfg(all(feature = "encode", feature = "jiff-conversions"))]
    fn test_date_from_jiff() {
        use jiff::civil::Date as JiffDate;

        // Test typical date
        let jiff_date = JiffDate::new(2024, 12, 25).unwrap();
        let date = Date::from_jiff(&jiff_date);

        assert_eq!(date.year, 2024);
        assert_eq!(date.month, 12);
        assert_eq!(date.day_of_month, 25);
        // jiff's weekday: Monday=1, ..., Sunday=7 (ISO 8601, same as DLMS)
        assert_eq!(date.day_of_week, 3); // Wednesday (2024-12-25)

        // Test encoding round-trip
        let data = Data::Date(date);
        let encoded = data.encode();
        let (_, parsed) = Data::parse(&encoded).unwrap();
        assert_eq!(parsed, data);
    }

    #[test]
    #[cfg(all(feature = "encode", feature = "jiff-conversions"))]
    fn test_time_from_jiff() {
        use jiff::civil::Time as JiffTime;

        // Test typical time
        let jiff_time = JiffTime::new(14, 30, 45, 500_000_000).unwrap(); // 500ms in nanoseconds
        let time = Time::from_jiff(&jiff_time);

        assert_eq!(time.hour, Some(14));
        assert_eq!(time.minute, Some(30));
        assert_eq!(time.second, Some(45));
        assert_eq!(time.hundredth, Some(50)); // 500ms = 50 hundredths

        // Test encoding round-trip
        let data = Data::Time(time);
        let encoded = data.encode();
        let (_, parsed) = Data::parse(&encoded).unwrap();
        assert_eq!(parsed, data);
    }

    #[test]
    #[cfg(all(feature = "encode", feature = "jiff-conversions"))]
    fn test_time_from_jiff_nanosecond_rounding() {
        use jiff::civil::Time as JiffTime;

        // Test nanosecond rounding: 505ms -> 50 hundredths (truncated)
        let jiff_time = JiffTime::new(12, 0, 0, 505_000_000).unwrap();
        let time = Time::from_jiff(&jiff_time);
        assert_eq!(time.hundredth, Some(50));

        // Test 999ms -> 99 hundredths
        let jiff_time = JiffTime::new(12, 0, 0, 999_000_000).unwrap();
        let time = Time::from_jiff(&jiff_time);
        assert_eq!(time.hundredth, Some(99));
    }

    #[test]
    #[cfg(all(feature = "encode", feature = "jiff-conversions"))]
    fn test_datetime_from_jiff() {
        use jiff::civil::DateTime as JiffDateTime;

        let jiff_dt = JiffDateTime::new(2024, 6, 15, 14, 30, 45, 500_000_000).unwrap();
        let datetime = DateTime::from_jiff(&jiff_dt, 120, 0x00); // UTC+2, no special status

        assert_eq!(datetime.date.year, 2024);
        assert_eq!(datetime.date.month, 6);
        assert_eq!(datetime.date.day_of_month, 15);
        assert_eq!(datetime.time.hour, Some(14));
        assert_eq!(datetime.time.minute, Some(30));
        assert_eq!(datetime.time.second, Some(45));
        assert_eq!(datetime.time.hundredth, Some(50));
        assert_eq!(datetime.offset_minutes, Some(120));
        assert_eq!(datetime.clock_status, Some(ClockStatus(0x00)));

        // Test encoding round-trip
        let data = Data::DateTime(datetime);
        let encoded = data.encode();
        let (_, parsed) = Data::parse(&encoded).unwrap();
        assert_eq!(parsed, data);
    }

    #[test]
    #[cfg(all(feature = "encode", feature = "jiff-conversions"))]
    fn test_datetime_from_jiff_with_timezone() {
        use jiff::civil::DateTime as JiffDateTime;

        // Test negative timezone offset (UTC-5)
        let jiff_dt = JiffDateTime::new(2024, 1, 1, 0, 0, 0, 0).unwrap();
        let datetime = DateTime::from_jiff(&jiff_dt, -300, 0x80); // UTC-5, daylight saving

        assert_eq!(datetime.offset_minutes, Some(-300));
        assert_eq!(datetime.clock_status, Some(ClockStatus(0x80)));

        let data = Data::DateTime(datetime);
        let encoded = data.encode();
        let (_, parsed) = Data::parse(&encoded).unwrap();
        assert_eq!(parsed, data);
    }

    #[test]
    #[cfg(all(feature = "std", feature = "jiff-conversions"))]
    fn test_datetime_now_jiff() {
        // Test that now_jiff() creates a valid DateTime
        let datetime = DateTime::now_jiff();

        // Basic sanity checks
        assert!(datetime.date.year >= 2024); // Assuming test runs in 2024 or later
        assert!(datetime.date.month >= 1 && datetime.date.month <= 12);
        assert!(datetime.date.day_of_month >= 1 && datetime.date.day_of_month <= 31);
        assert!(datetime.time.hour.is_some());
        assert!(datetime.time.minute.is_some());
        assert!(datetime.time.second.is_some());

        // Test that it encodes correctly
        let data = Data::DateTime(datetime);
        let encoded = data.encode();
        assert_eq!(encoded.len(), 13);
        assert_eq!(encoded[0], 0x19); // DateTime type tag

        // Test round-trip
        let (_, parsed) = Data::parse(&encoded).unwrap();
        assert_eq!(parsed, data);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_structure_empty() {
        // Green Book: Data type 0x02 = Structure, count-prefixed
        let data = Data::Structure(vec![]);
        let encoded = data.encode();

        // Expected: [0x02, 0x00]
        assert_eq!(encoded, vec![0x02, 0x00]);
        assert_eq!(data.encoded_len(), 2);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_structure_simple() {
        let data = Data::Structure(vec![Data::Integer(42), Data::LongUnsigned(1000)]);
        let encoded = data.encode();

        // Expected: [0x02, 0x02, 0x0F, 0x2A, 0x12, 0x03, 0xE8]
        assert_eq!(encoded, vec![0x02, 0x02, 0x0F, 0x2A, 0x12, 0x03, 0xE8]);
        assert_eq!(data.encoded_len(), 7);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_structure_nested() {
        let inner = Data::Structure(vec![Data::Integer(1), Data::Integer(2)]);
        let data = Data::Structure(vec![Data::Null, inner, Data::Enum(5)]);
        let encoded = data.encode();

        // Expected: [0x02, 0x03, 0x00, 0x02, 0x02, 0x0F, 0x01, 0x0F, 0x02, 0x16, 0x05]
        assert_eq!(encoded, vec![0x02, 0x03, 0x00, 0x02, 0x02, 0x0F, 0x01, 0x0F, 0x02, 0x16, 0x05]);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_structure_roundtrip() {
        let original = Data::Structure(vec![
            Data::Integer(42),
            Data::OctetString(vec![0xAA, 0xBB]),
            Data::LongUnsigned(12345),
        ]);
        let encoded = original.encode();
        let (remaining, parsed) = Data::parse(&encoded).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(parsed, original);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_complex_structure() {
        // Test a more realistic structure like scaler-unit
        let scaler_unit = Data::Structure(vec![
            Data::Integer(-3), // scaler
            Data::Enum(30),    // unit (Wh)
        ]);
        let encoded = scaler_unit.encode();

        // Expected: [0x02, 0x02, 0x0F, 0xFD, 0x16, 0x1E]
        assert_eq!(encoded, vec![0x02, 0x02, 0x0F, 0xFD, 0x16, 0x1E]);
        assert_eq!(scaler_unit.encoded_len(), 6);

        // Verify roundtrip
        let (remaining, parsed) = Data::parse(&encoded).unwrap();
        assert_eq!(remaining.len(), 0);
        assert_eq!(parsed, scaler_unit);
    }
}
