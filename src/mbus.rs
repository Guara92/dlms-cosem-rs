use core::convert::TryFrom;

use crate::{
    DlmsDataLinkLayer, Error,
    control_information::{ControlInformation, HeaderType},
    map_nom_error,
};

use alloc::{borrow::Cow, vec::Vec};
use mbusparse::Telegram;
use nom::{IResult, Parser};

#[derive(Debug)]
pub enum MBusDataLinkLayer {}

fn parse_mbus<'i, 'f>(
    input: &'f [Telegram<'i>],
) -> IResult<&'f [Telegram<'i>], Cow<'i, [u8]>, Error> {
    let mut payload = Vec::new();
    let mut current_segment = 0;
    let mut len = 0;

    for telegram in input {
        match telegram {
            Telegram::LongFrame { control_information, user_data, .. } => {
                use nom::number::complete::u8;

                let user_data: &[u8] = user_data;

                let control_information = ControlInformation::try_from(*control_information)
                    .map_err(|_| nom::Err::Failure(Error::InvalidFormat))?;

                let (user_data, last_segment) = match control_information {
                    ControlInformation::Segmented { segment, last_segment } => {
                        if current_segment != segment {
                            return Err(nom::Err::Failure(Error::ChecksumMismatch));
                        }
                        current_segment = (current_segment + 1) & 0b1111;

                        (user_data, last_segment)
                    }
                    ControlInformation::Unsegmented { header, .. } => {
                        let (user_data, _ala) = if header == HeaderType::Long {
                            let (user_data, (m_id, ver, dt)) = (u8, u8, u8).parse(user_data)?;
                            (user_data, Some((m_id, ver, dt)))
                        } else {
                            (user_data, None)
                        };

                        let (user_data, (_acc, _sts, _cfg)) = (u8, u8, u8).parse(user_data)?;

                        len += 1;
                        return Ok((&input[len..], Cow::from(user_data)));
                    }
                };

                let (user_data, (_stsap, _dtsap)) = (u8, u8).parse(user_data)?;

                payload.extend(user_data);
                len += 1;

                if last_segment {
                    return Ok((&input[len..], Cow::from(payload)));
                }
            }
            _ => return Err(nom::Err::Failure(Error::InvalidFormat)),
        }
    }

    Err(nom::Err::Incomplete(nom::Needed::Unknown))
}

impl<'i, 'f> DlmsDataLinkLayer<'i, &'f [Telegram<'i>]> for MBusDataLinkLayer {
    fn next_frame(input: &'f [Telegram<'i>]) -> Result<(&'f [Telegram<'i>], Cow<'i, [u8]>), Error> {
        map_nom_error(parse_mbus(input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mbusparse::Telegram;

    #[test]
    fn test_next_frame_empty() {
        let telegrams: &[Telegram] = &[];
        let result = MBusDataLinkLayer::next_frame(telegrams);
        assert!(result.is_err());
    }

    #[test]
    fn test_unsegmented_short_header() {
        // Unsegmented telegram with short header (0x61)
        let user_data = vec![0x00, 0x00, 0x00, 0x01, 0x02, 0x03]; // acc, sts, cfg + payload
        let telegram = Telegram::LongFrame {
            control: mbusparse::Control::SndUd { fcb: false },
            address: mbusparse::Address::from(0x01),
            control_information: 0x61, // Unsegmented, Short, MasterSlave
            user_data: &user_data,
        };
        let telegrams = vec![telegram];

        let (remaining, payload) = MBusDataLinkLayer::next_frame(&telegrams).unwrap();
        assert_eq!(remaining.len(), 0);
        assert_eq!(&*payload, &[0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_unsegmented_long_header() {
        // Unsegmented telegram with long header (0x60)
        let user_data = vec![
            0xAA, 0xBB, 0xCC, // m_id, ver, dt
            0x00, 0x00, 0x00, // acc, sts, cfg
            0x11, 0x22, // payload
        ];
        let telegram = Telegram::LongFrame {
            control: mbusparse::Control::SndUd { fcb: false },
            address: mbusparse::Address::from(0x01),
            control_information: 0x60, // Unsegmented, Long, MasterSlave
            user_data: &user_data,
        };
        let telegrams = vec![telegram];

        let (remaining, payload) = MBusDataLinkLayer::next_frame(&telegrams).unwrap();
        assert_eq!(remaining.len(), 0);
        assert_eq!(&*payload, &[0x11, 0x22]);
    }

    #[test]
    fn test_segmented_two_telegrams() {
        // First segment (segment 0, not last)
        let user_data1 = vec![0x55, 0x66, 0x01, 0x02]; // stsap, dtsap + payload
        let telegram1 = Telegram::LongFrame {
            control: mbusparse::Control::SndUd { fcb: false },
            address: mbusparse::Address::from(0x01),
            control_information: 0x00, // Segmented, segment 0, not last
            user_data: &user_data1,
        };

        // Last segment (segment 1, last)
        let user_data2 = vec![0x55, 0x66, 0x03, 0x04]; // stsap, dtsap + payload
        let telegram2 = Telegram::LongFrame {
            control: mbusparse::Control::SndUd { fcb: false },
            address: mbusparse::Address::from(0x01),
            control_information: 0x11, // Segmented, segment 1, last (0x10 | 0x01)
            user_data: &user_data2,
        };

        let telegrams = vec![telegram1, telegram2];
        let (remaining, payload) = MBusDataLinkLayer::next_frame(&telegrams).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(&*payload, &[0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_segmented_three_telegrams() {
        let user_data1 = vec![0x55, 0x66, 0xAA];
        let telegram1 = Telegram::LongFrame {
            control: mbusparse::Control::SndUd { fcb: false },
            address: mbusparse::Address::from(0x01),
            control_information: 0x00, // segment 0, not last
            user_data: &user_data1,
        };

        let user_data2 = vec![0x55, 0x66, 0xBB];
        let telegram2 = Telegram::LongFrame {
            control: mbusparse::Control::SndUd { fcb: false },
            address: mbusparse::Address::from(0x01),
            control_information: 0x01, // segment 1, not last
            user_data: &user_data2,
        };

        let user_data3 = vec![0x55, 0x66, 0xCC, 0xDD];
        let telegram3 = Telegram::LongFrame {
            control: mbusparse::Control::SndUd { fcb: false },
            address: mbusparse::Address::from(0x01),
            control_information: 0x12, // segment 2, last (0x10 | 0x02)
            user_data: &user_data3,
        };

        let telegrams = vec![telegram1, telegram2, telegram3];
        let (remaining, payload) = MBusDataLinkLayer::next_frame(&telegrams).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(&*payload, &[0xAA, 0xBB, 0xCC, 0xDD]);
    }

    #[test]
    fn test_segmented_incomplete() {
        // Only first segment, not marked as last
        let user_data = vec![0x55, 0x66, 0x01];
        let telegram = Telegram::LongFrame {
            control: mbusparse::Control::SndUd { fcb: false },
            address: mbusparse::Address::from(0x01),
            control_information: 0x00, // segment 0, not last
            user_data: &user_data,
        };

        let telegrams = vec![telegram];
        let result = MBusDataLinkLayer::next_frame(&telegrams);

        assert!(matches!(result, Err(Error::Incomplete(_))));
    }

    #[test]
    fn test_segmented_wrong_segment_order() {
        // First telegram is segment 0
        let user_data1 = vec![0x55, 0x66, 0x01];
        let telegram1 = Telegram::LongFrame {
            control: mbusparse::Control::SndUd { fcb: false },
            address: mbusparse::Address::from(0x01),
            control_information: 0x00, // segment 0
            user_data: &user_data1,
        };

        // Second telegram is segment 2 (skipping segment 1)
        let user_data2 = vec![0x55, 0x66, 0x02];
        let telegram2 = Telegram::LongFrame {
            control: mbusparse::Control::SndUd { fcb: false },
            address: mbusparse::Address::from(0x01),
            control_information: 0x12, // segment 2, last
            user_data: &user_data2,
        };

        let telegrams = vec![telegram1, telegram2];
        let result = MBusDataLinkLayer::next_frame(&telegrams);

        assert!(matches!(result, Err(Error::ChecksumMismatch)));
    }

    #[test]
    fn test_invalid_telegram_type() {
        // ShortFrame is not supported
        let telegram = Telegram::ShortFrame {
            control: mbusparse::Control::SndUd { fcb: false },
            address: mbusparse::Address::from(0x01),
        };

        let telegrams = vec![telegram];
        let result = MBusDataLinkLayer::next_frame(&telegrams);

        assert!(matches!(result, Err(Error::InvalidFormat)));
    }

    #[test]
    fn test_invalid_control_information() {
        // Control information 0xFF is not valid
        let user_data = vec![0x00, 0x00, 0x00];
        let telegram = Telegram::LongFrame {
            control: mbusparse::Control::SndUd { fcb: false },
            address: mbusparse::Address::from(0x01),
            control_information: 0xFF,
            user_data: &user_data,
        };

        let telegrams = vec![telegram];
        let result = MBusDataLinkLayer::next_frame(&telegrams);

        assert!(matches!(result, Err(Error::InvalidFormat)));
    }

    #[test]
    fn test_multiple_messages() {
        // First unsegmented message
        let user_data1 = vec![0x00, 0x00, 0x00, 0x11];
        let telegram1 = Telegram::LongFrame {
            control: mbusparse::Control::SndUd { fcb: false },
            address: mbusparse::Address::from(0x01),
            control_information: 0x61, // Unsegmented, Short
            user_data: &user_data1,
        };

        // Second unsegmented message
        let user_data2 = vec![0x00, 0x00, 0x00, 0x22];
        let telegram2 = Telegram::LongFrame {
            control: mbusparse::Control::SndUd { fcb: false },
            address: mbusparse::Address::from(0x01),
            control_information: 0x61,
            user_data: &user_data2,
        };

        let telegrams = vec![telegram1, telegram2];

        // First call gets first message
        let (remaining, payload) = MBusDataLinkLayer::next_frame(&telegrams).unwrap();
        assert_eq!(&*payload, &[0x11]);
        assert_eq!(remaining.len(), 1);

        // Second call gets second message
        let (remaining, payload) = MBusDataLinkLayer::next_frame(remaining).unwrap();
        assert_eq!(&*payload, &[0x22]);
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn test_unsegmented_slave_master_short() {
        // Unsegmented, Short, SlaveMaster (0x7d)
        let user_data = vec![0x00, 0x00, 0x00, 0xAB, 0xCD];
        let telegram = Telegram::LongFrame {
            control: mbusparse::Control::SndUd { fcb: false },
            address: mbusparse::Address::from(0x01),
            control_information: 0x7d,
            user_data: &user_data,
        };

        let telegrams = vec![telegram];
        let (remaining, payload) = MBusDataLinkLayer::next_frame(&telegrams).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(&*payload, &[0xAB, 0xCD]);
    }

    #[test]
    fn test_unsegmented_slave_master_long() {
        // Unsegmented, Long, SlaveMaster (0x7c)
        let user_data = vec![
            0x11, 0x22, 0x33, // m_id, ver, dt
            0x00, 0x00, 0x00, // acc, sts, cfg
            0xFF, 0xEE, // payload
        ];
        let telegram = Telegram::LongFrame {
            control: mbusparse::Control::SndUd { fcb: false },
            address: mbusparse::Address::from(0x01),
            control_information: 0x7c,
            user_data: &user_data,
        };

        let telegrams = vec![telegram];
        let (remaining, payload) = MBusDataLinkLayer::next_frame(&telegrams).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(&*payload, &[0xFF, 0xEE]);
    }

    #[test]
    fn test_segmented_wrapping_from_15_to_0() {
        // Test 4-bit segment counter wrapping: ...13->14->15->0
        // We need to start from 0 and go through the full sequence to test wrapping
        let user_data_0 = vec![0x55, 0x66, 0x00];
        let user_data_1 = vec![0x55, 0x66, 0x01];
        let user_data_2 = vec![0x55, 0x66, 0x02];
        let user_data_3 = vec![0x55, 0x66, 0x03];
        let user_data_4 = vec![0x55, 0x66, 0x04];
        let user_data_5 = vec![0x55, 0x66, 0x05];
        let user_data_6 = vec![0x55, 0x66, 0x06];
        let user_data_7 = vec![0x55, 0x66, 0x07];
        let user_data_8 = vec![0x55, 0x66, 0x08];
        let user_data_9 = vec![0x55, 0x66, 0x09];
        let user_data_10 = vec![0x55, 0x66, 0x0A];
        let user_data_11 = vec![0x55, 0x66, 0x0B];
        let user_data_12 = vec![0x55, 0x66, 0x0C];
        let user_data_13 = vec![0x55, 0x66, 0x0D];
        let user_data_14 = vec![0x55, 0x66, 0x0E];
        let user_data_15 = vec![0x55, 0x66, 0x0F];
        let user_data_16 = vec![0x55, 0x66, 0xFF]; // wraps back to 0

        let telegrams = vec![
            Telegram::LongFrame {
                control: mbusparse::Control::SndUd { fcb: false },
                address: mbusparse::Address::from(0x01),
                control_information: 0x00,
                user_data: &user_data_0,
            },
            Telegram::LongFrame {
                control: mbusparse::Control::SndUd { fcb: false },
                address: mbusparse::Address::from(0x01),
                control_information: 0x01,
                user_data: &user_data_1,
            },
            Telegram::LongFrame {
                control: mbusparse::Control::SndUd { fcb: false },
                address: mbusparse::Address::from(0x01),
                control_information: 0x02,
                user_data: &user_data_2,
            },
            Telegram::LongFrame {
                control: mbusparse::Control::SndUd { fcb: false },
                address: mbusparse::Address::from(0x01),
                control_information: 0x03,
                user_data: &user_data_3,
            },
            Telegram::LongFrame {
                control: mbusparse::Control::SndUd { fcb: false },
                address: mbusparse::Address::from(0x01),
                control_information: 0x04,
                user_data: &user_data_4,
            },
            Telegram::LongFrame {
                control: mbusparse::Control::SndUd { fcb: false },
                address: mbusparse::Address::from(0x01),
                control_information: 0x05,
                user_data: &user_data_5,
            },
            Telegram::LongFrame {
                control: mbusparse::Control::SndUd { fcb: false },
                address: mbusparse::Address::from(0x01),
                control_information: 0x06,
                user_data: &user_data_6,
            },
            Telegram::LongFrame {
                control: mbusparse::Control::SndUd { fcb: false },
                address: mbusparse::Address::from(0x01),
                control_information: 0x07,
                user_data: &user_data_7,
            },
            Telegram::LongFrame {
                control: mbusparse::Control::SndUd { fcb: false },
                address: mbusparse::Address::from(0x01),
                control_information: 0x08,
                user_data: &user_data_8,
            },
            Telegram::LongFrame {
                control: mbusparse::Control::SndUd { fcb: false },
                address: mbusparse::Address::from(0x01),
                control_information: 0x09,
                user_data: &user_data_9,
            },
            Telegram::LongFrame {
                control: mbusparse::Control::SndUd { fcb: false },
                address: mbusparse::Address::from(0x01),
                control_information: 0x0A,
                user_data: &user_data_10,
            },
            Telegram::LongFrame {
                control: mbusparse::Control::SndUd { fcb: false },
                address: mbusparse::Address::from(0x01),
                control_information: 0x0B,
                user_data: &user_data_11,
            },
            Telegram::LongFrame {
                control: mbusparse::Control::SndUd { fcb: false },
                address: mbusparse::Address::from(0x01),
                control_information: 0x0C,
                user_data: &user_data_12,
            },
            Telegram::LongFrame {
                control: mbusparse::Control::SndUd { fcb: false },
                address: mbusparse::Address::from(0x01),
                control_information: 0x0D,
                user_data: &user_data_13,
            },
            Telegram::LongFrame {
                control: mbusparse::Control::SndUd { fcb: false },
                address: mbusparse::Address::from(0x01),
                control_information: 0x0E,
                user_data: &user_data_14,
            },
            Telegram::LongFrame {
                control: mbusparse::Control::SndUd { fcb: false },
                address: mbusparse::Address::from(0x01),
                control_information: 0x0F,
                user_data: &user_data_15,
            },
            // Segment wraps from 15 to 0, marked as last
            Telegram::LongFrame {
                control: mbusparse::Control::SndUd { fcb: false },
                address: mbusparse::Address::from(0x01),
                control_information: 0x10, // segment 0, last (0x10 = bit 4 set)
                user_data: &user_data_16,
            },
        ];

        let (remaining, payload) = MBusDataLinkLayer::next_frame(&telegrams).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(
            &*payload,
            &[
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
                0x0E, 0x0F, 0xFF
            ]
        );
    }
}
