use core::convert::TryFrom;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderType {
    Short,
    Long,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    MasterSlave,
    SlaveMaster,
}

#[allow(dead_code)]
/// M-Bus control information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlInformation {
    Segmented { segment: u8, last_segment: bool },
    Unsegmented { header: HeaderType, direction: Direction },
}

impl TryFrom<u8> for ControlInformation {
    type Error = u8;

    fn try_from(control_information: u8) -> Result<Self, Self::Error> {
        use {Direction::*, HeaderType::*};

        Ok(match control_information {
            0x00..=0x1f => {
                let segment = control_information & 0b1111;
                let last_segment = (control_information & 0b10000) != 0;

                Self::Segmented { segment, last_segment }
            }
            0x60 => Self::Unsegmented { header: Long, direction: MasterSlave },
            0x61 => Self::Unsegmented { header: Short, direction: MasterSlave },
            0x7c => Self::Unsegmented { header: Long, direction: SlaveMaster },
            0x7d => Self::Unsegmented { header: Short, direction: SlaveMaster },
            _ => return Err(control_information),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segmented_first_segment() {
        // Segment 0, not last
        let ci = ControlInformation::try_from(0x00).unwrap();
        match ci {
            ControlInformation::Segmented { segment, last_segment } => {
                assert_eq!(segment, 0);
                assert!(!last_segment);
            }
            _ => panic!("Expected Segmented"),
        }
    }

    #[test]
    fn test_segmented_middle_segment() {
        // Segment 5, not last
        let ci = ControlInformation::try_from(0x05).unwrap();
        match ci {
            ControlInformation::Segmented { segment, last_segment } => {
                assert_eq!(segment, 5);
                assert!(!last_segment);
            }
            _ => panic!("Expected Segmented"),
        }
    }

    #[test]
    fn test_segmented_last_segment() {
        // Segment 0, last (bit 4 set)
        let ci = ControlInformation::try_from(0x10).unwrap();
        match ci {
            ControlInformation::Segmented { segment, last_segment } => {
                assert_eq!(segment, 0);
                assert!(last_segment);
            }
            _ => panic!("Expected Segmented"),
        }
    }

    #[test]
    fn test_segmented_last_segment_with_number() {
        // Segment 7, last
        let ci = ControlInformation::try_from(0x17).unwrap();
        match ci {
            ControlInformation::Segmented { segment, last_segment } => {
                assert_eq!(segment, 7);
                assert!(last_segment);
            }
            _ => panic!("Expected Segmented"),
        }
    }

    #[test]
    fn test_segmented_max_segment() {
        // Segment 15 (max), not last
        let ci = ControlInformation::try_from(0x0f).unwrap();
        match ci {
            ControlInformation::Segmented { segment, last_segment } => {
                assert_eq!(segment, 15);
                assert!(!last_segment);
            }
            _ => panic!("Expected Segmented"),
        }
    }

    #[test]
    fn test_segmented_max_segment_last() {
        // Segment 15 (max), last
        let ci = ControlInformation::try_from(0x1f).unwrap();
        match ci {
            ControlInformation::Segmented { segment, last_segment } => {
                assert_eq!(segment, 15);
                assert!(last_segment);
            }
            _ => panic!("Expected Segmented"),
        }
    }

    #[test]
    fn test_unsegmented_long_master_slave() {
        let ci = ControlInformation::try_from(0x60).unwrap();
        match ci {
            ControlInformation::Unsegmented { header, direction } => {
                assert_eq!(header, HeaderType::Long);
                assert_eq!(direction, Direction::MasterSlave);
            }
            _ => panic!("Expected Unsegmented"),
        }
    }

    #[test]
    fn test_unsegmented_short_master_slave() {
        let ci = ControlInformation::try_from(0x61).unwrap();
        match ci {
            ControlInformation::Unsegmented { header, direction } => {
                assert_eq!(header, HeaderType::Short);
                assert_eq!(direction, Direction::MasterSlave);
            }
            _ => panic!("Expected Unsegmented"),
        }
    }

    #[test]
    fn test_unsegmented_long_slave_master() {
        let ci = ControlInformation::try_from(0x7c).unwrap();
        match ci {
            ControlInformation::Unsegmented { header, direction } => {
                assert_eq!(header, HeaderType::Long);
                assert_eq!(direction, Direction::SlaveMaster);
            }
            _ => panic!("Expected Unsegmented"),
        }
    }

    #[test]
    fn test_unsegmented_short_slave_master() {
        let ci = ControlInformation::try_from(0x7d).unwrap();
        match ci {
            ControlInformation::Unsegmented { header, direction } => {
                assert_eq!(header, HeaderType::Short);
                assert_eq!(direction, Direction::SlaveMaster);
            }
            _ => panic!("Expected Unsegmented"),
        }
    }

    #[test]
    fn test_invalid_control_information() {
        // Values outside valid ranges should return Err
        assert_eq!(ControlInformation::try_from(0x20), Err(0x20));
        assert_eq!(ControlInformation::try_from(0x50), Err(0x50));
        assert_eq!(ControlInformation::try_from(0x62), Err(0x62));
        assert_eq!(ControlInformation::try_from(0x7b), Err(0x7b));
        assert_eq!(ControlInformation::try_from(0x7e), Err(0x7e));
        assert_eq!(ControlInformation::try_from(0xff), Err(0xff));
    }

    #[test]
    fn test_control_information_clone() {
        let ci1 = ControlInformation::try_from(0x60).unwrap();
        let ci2 = ci1.clone();
        assert_eq!(ci1, ci2);
    }

    #[test]
    fn test_control_information_debug() {
        let ci = ControlInformation::try_from(0x60).unwrap();
        let debug_str = format!("{:?}", ci);
        assert!(debug_str.contains("Unsegmented"));
    }

    #[test]
    fn test_header_type_equality() {
        assert_eq!(HeaderType::Long, HeaderType::Long);
        assert_eq!(HeaderType::Short, HeaderType::Short);
        assert_ne!(HeaderType::Long, HeaderType::Short);
    }

    #[test]
    fn test_direction_equality() {
        assert_eq!(Direction::MasterSlave, Direction::MasterSlave);
        assert_eq!(Direction::SlaveMaster, Direction::SlaveMaster);
        assert_ne!(Direction::MasterSlave, Direction::SlaveMaster);
    }
}
