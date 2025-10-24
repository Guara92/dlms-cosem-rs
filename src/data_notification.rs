use nom::{
    IResult,
    multi::length_value,
    number::streaming::{be_u32, u8},
};

use crate::{Data, DateTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LongInvokeIdAndPriority(pub(crate) u32);

impl LongInvokeIdAndPriority {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, id) = be_u32(input)?;
        Ok((input, Self(id)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Normal,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceClass {
    Confirmed,
    Unconfirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessingOption {
    ContinueOnError,
    BreakOnError,
}

impl LongInvokeIdAndPriority {
    pub fn priority(&self) -> Priority {
        if (self.0 & (1 << 31)) == 0 { Priority::Normal } else { Priority::High }
    }

    pub fn processing_option(&self) -> ProcessingOption {
        if (self.0 & (1 << 29)) == 0 {
            ProcessingOption::ContinueOnError
        } else {
            ProcessingOption::BreakOnError
        }
    }

    pub fn self_descriptive(&self) -> bool {
        (self.0 & (1 << 28)) != 0
    }

    pub fn service_class(&self) -> ServiceClass {
        if (self.0 & (1 << 30)) == 0 { ServiceClass::Unconfirmed } else { ServiceClass::Confirmed }
    }

    pub fn invoke_id(&self) -> u32 {
        self.0 & 0x00ffffff
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataNotification {
    pub(crate) long_invoke_id_and_priority: LongInvokeIdAndPriority,
    pub(crate) date_time: DateTime,
    pub(crate) notification_body: Data,
}

impl DataNotification {
    pub fn priority(&self) -> Priority {
        self.long_invoke_id_and_priority.priority()
    }

    pub fn self_descriptive(&self) -> bool {
        self.long_invoke_id_and_priority.self_descriptive()
    }

    pub fn processing_option(&self) -> ProcessingOption {
        self.long_invoke_id_and_priority.processing_option()
    }

    pub fn service_class(&self) -> ServiceClass {
        self.long_invoke_id_and_priority.service_class()
    }

    pub fn invoke_id(&self) -> u32 {
        self.long_invoke_id_and_priority.invoke_id()
    }

    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, long_invoke_id_and_priority) = LongInvokeIdAndPriority::parse(input)?;
        let (input, date_time) = length_value(u8, DateTime::parse)(input)?;
        let (input, notification_body) = Data::parse(input)?;
        Ok((input, Self { long_invoke_id_and_priority, date_time, notification_body }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_long_invoke_id_parse() {
        // Test parsing a basic 4-byte big-endian u32
        let input = [0x00, 0x00, 0x00, 0x01, 0xFF];
        let (remaining, id) = LongInvokeIdAndPriority::parse(&input).unwrap();

        assert_eq!(remaining, &[0xFF]);
        assert_eq!(id.0, 1);
    }

    #[test]
    fn test_invoke_id_extraction() {
        // Invoke ID is the lower 24 bits (0x00ffffff)
        let id = LongInvokeIdAndPriority(0x12345678);
        assert_eq!(id.invoke_id(), 0x00345678);

        let id = LongInvokeIdAndPriority(0xFFFFFFFF);
        assert_eq!(id.invoke_id(), 0x00FFFFFF);

        let id = LongInvokeIdAndPriority(0x00000042);
        assert_eq!(id.invoke_id(), 0x00000042);
    }

    #[test]
    fn test_priority_bit() {
        // Bit 31 (0x80000000) = Priority
        let id = LongInvokeIdAndPriority(0x00000000);
        assert_eq!(id.priority(), Priority::Normal);

        let id = LongInvokeIdAndPriority(0x80000000);
        assert_eq!(id.priority(), Priority::High);

        let id = LongInvokeIdAndPriority(0x80FFFFFF);
        assert_eq!(id.priority(), Priority::High);
    }

    #[test]
    fn test_service_class_bit() {
        // Bit 30 (0x40000000) = Service Class
        let id = LongInvokeIdAndPriority(0x00000000);
        assert_eq!(id.service_class(), ServiceClass::Unconfirmed);

        let id = LongInvokeIdAndPriority(0x40000000);
        assert_eq!(id.service_class(), ServiceClass::Confirmed);

        let id = LongInvokeIdAndPriority(0x40FFFFFF);
        assert_eq!(id.service_class(), ServiceClass::Confirmed);
    }

    #[test]
    fn test_processing_option_bit() {
        // Bit 29 (0x20000000) = Processing Option
        let id = LongInvokeIdAndPriority(0x00000000);
        assert_eq!(id.processing_option(), ProcessingOption::ContinueOnError);

        let id = LongInvokeIdAndPriority(0x20000000);
        assert_eq!(id.processing_option(), ProcessingOption::BreakOnError);

        let id = LongInvokeIdAndPriority(0x20FFFFFF);
        assert_eq!(id.processing_option(), ProcessingOption::BreakOnError);
    }

    #[test]
    fn test_self_descriptive_bit() {
        // Bit 28 (0x10000000) = Self Descriptive
        let id = LongInvokeIdAndPriority(0x00000000);
        assert_eq!(id.self_descriptive(), false);

        let id = LongInvokeIdAndPriority(0x10000000);
        assert_eq!(id.self_descriptive(), true);

        let id = LongInvokeIdAndPriority(0x10FFFFFF);
        assert_eq!(id.self_descriptive(), true);
    }

    #[test]
    fn test_all_flags_combined() {
        // Test with all flags set
        let id = LongInvokeIdAndPriority(0xF0FFFFFF);
        assert_eq!(id.priority(), Priority::High);
        assert_eq!(id.service_class(), ServiceClass::Confirmed);
        assert_eq!(id.processing_option(), ProcessingOption::BreakOnError);
        assert_eq!(id.self_descriptive(), true);
        assert_eq!(id.invoke_id(), 0x00FFFFFF);

        // Test with no flags set
        let id = LongInvokeIdAndPriority(0x00000042);
        assert_eq!(id.priority(), Priority::Normal);
        assert_eq!(id.service_class(), ServiceClass::Unconfirmed);
        assert_eq!(id.processing_option(), ProcessingOption::ContinueOnError);
        assert_eq!(id.self_descriptive(), false);
        assert_eq!(id.invoke_id(), 0x00000042);
    }

    #[test]
    fn test_common_combinations() {
        // Common case: Confirmed, Normal priority, invoke_id=1
        let id = LongInvokeIdAndPriority(0x40000001);
        assert_eq!(id.priority(), Priority::Normal);
        assert_eq!(id.service_class(), ServiceClass::Confirmed);
        assert_eq!(id.invoke_id(), 1);

        // High priority unconfirmed
        let id = LongInvokeIdAndPriority(0x80000001);
        assert_eq!(id.priority(), Priority::High);
        assert_eq!(id.service_class(), ServiceClass::Unconfirmed);
        assert_eq!(id.invoke_id(), 1);
    }

    #[test]
    fn test_long_invoke_id_equality() {
        let id1 = LongInvokeIdAndPriority(0x12345678);
        let id2 = LongInvokeIdAndPriority(0x12345678);
        let id3 = LongInvokeIdAndPriority(0x87654321);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_long_invoke_id_clone() {
        let id1 = LongInvokeIdAndPriority(0x12345678);
        let id2 = id1.clone();

        assert_eq!(id1, id2);
        assert_eq!(id1.0, id2.0);
    }

    #[test]
    fn test_long_invoke_id_debug() {
        let id = LongInvokeIdAndPriority(0x12345678);
        let debug_str = format!("{:?}", id);

        assert!(debug_str.contains("LongInvokeIdAndPriority"));
    }

    #[test]
    fn test_priority_enum_equality() {
        assert_eq!(Priority::Normal, Priority::Normal);
        assert_eq!(Priority::High, Priority::High);
        assert_ne!(Priority::Normal, Priority::High);
    }

    #[test]
    fn test_service_class_enum_equality() {
        assert_eq!(ServiceClass::Confirmed, ServiceClass::Confirmed);
        assert_eq!(ServiceClass::Unconfirmed, ServiceClass::Unconfirmed);
        assert_ne!(ServiceClass::Confirmed, ServiceClass::Unconfirmed);
    }

    #[test]
    fn test_processing_option_enum_equality() {
        assert_eq!(ProcessingOption::ContinueOnError, ProcessingOption::ContinueOnError);
        assert_eq!(ProcessingOption::BreakOnError, ProcessingOption::BreakOnError);
        assert_ne!(ProcessingOption::ContinueOnError, ProcessingOption::BreakOnError);
    }

    #[test]
    fn test_data_notification_delegates() {
        // Create a simple DataNotification to test delegation methods
        use crate::data::{Date, Time};

        let long_invoke_id = LongInvokeIdAndPriority(0xC0000001); // Confirmed + High priority

        // Create minimal DateTime
        let date = Date { year: 2024, month: 1, day_of_month: 15, day_of_week: 0xff };
        let time = Time { hour: Some(12), minute: Some(30), second: Some(0), hundredth: Some(0) };
        let date_time = DateTime { date, time, offset_minutes: None, clock_status: None };

        // Create minimal Data::Null
        let notification_body = Data::Null;

        let notification = DataNotification {
            long_invoke_id_and_priority: long_invoke_id,
            date_time,
            notification_body,
        };

        // Test that methods delegate correctly
        assert_eq!(notification.priority(), Priority::High);
        assert_eq!(notification.service_class(), ServiceClass::Confirmed);
        assert_eq!(notification.invoke_id(), 1);
        assert_eq!(notification.self_descriptive(), false);
        assert_eq!(notification.processing_option(), ProcessingOption::ContinueOnError);
    }

    #[test]
    fn test_data_notification_clone() {
        use crate::data::{Date, Time};

        let long_invoke_id = LongInvokeIdAndPriority(0x40000001);
        let date = Date { year: 2024, month: 1, day_of_month: 15, day_of_week: 0xff };
        let time = Time { hour: Some(12), minute: Some(30), second: Some(0), hundredth: Some(0) };
        let date_time = DateTime { date, time, offset_minutes: None, clock_status: None };
        let notification_body = Data::Null;

        let notification1 = DataNotification {
            long_invoke_id_and_priority: long_invoke_id,
            date_time,
            notification_body,
        };

        let notification2 = notification1.clone();

        assert_eq!(notification1, notification2);
        assert_eq!(notification1.invoke_id(), notification2.invoke_id());
    }

    #[test]
    fn test_data_notification_debug() {
        use crate::data::{Date, Time};

        let long_invoke_id = LongInvokeIdAndPriority(0x40000001);
        let date = Date { year: 2024, month: 1, day_of_month: 15, day_of_week: 0xff };
        let time = Time { hour: Some(12), minute: Some(30), second: Some(0), hundredth: Some(0) };
        let date_time = DateTime { date, time, offset_minutes: None, clock_status: None };
        let notification_body = Data::Null;

        let notification = DataNotification {
            long_invoke_id_and_priority: long_invoke_id,
            date_time,
            notification_body,
        };

        let debug_str = format!("{:?}", notification);

        assert!(debug_str.contains("DataNotification"));
        assert!(debug_str.contains("long_invoke_id_and_priority"));
        assert!(debug_str.contains("date_time"));
        assert!(debug_str.contains("notification_body"));
    }
}
