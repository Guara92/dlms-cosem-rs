#[cfg(feature = "serde")]
use alloc::string::ToString;
use alloc::{string::String, vec::Vec};
use core::convert::TryFrom;
use core::fmt;

use nom::{
    IResult,
    combinator::fail,
    multi::length_count,
    number::streaming::{be_f32, be_f64, be_i16, be_i32, be_i64, be_u16, be_u32, be_u64, i8, u8},
    sequence::tuple,
};
#[cfg(feature = "serde")]
use serde::{Serialize, Serializer};

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
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (hour, minute, second, hundredth)) = tuple((u8, u8, u8, u8))(input)?;

        let hour = match hour {
            0xff => None,
            0..=23 => Some(hour),
            _ => return fail(input),
        };
        let minute = match minute {
            0xff => None,
            0..=59 => Some(minute),
            _ => return fail(input),
        };
        let second = match second {
            0xff => None,
            0..=59 => Some(second),
            _ => return fail(input),
        };
        let hundredth = match hundredth {
            0xff => None,
            0..=99 => Some(hundredth),
            _ => return fail(input),
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
                let (input, structure) = length_count(u8, Self::parse)(input)?;
                (input, Data::Structure(structure))
            }
            DataType::OctetString => {
                let (input, bytes) = length_count(u8, u8)(input)?;
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
            dt => unimplemented!("decoding data type {:?}", dt),
        })
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
    #[should_panic(expected = "not implemented")]
    fn test_data_parse_unsigned() {
        // Unsigned is not yet implemented
        let input = [0x11, 0x2A];
        let _ = Data::parse(&input).unwrap();
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
}
