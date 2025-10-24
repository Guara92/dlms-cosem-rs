use alloc::borrow::Cow;
use hdlcparse::type3::HdlcFrame;
use nom::{number::complete::u8, sequence::tuple};

use crate::{DlmsDataLinkLayer, Error};

enum Destination {
    Unicast,
    Broadcast,
}

enum MessageType {
    Command,
    Response,
}

#[allow(unused)]
struct LlcHeader {
    destination: Destination,
    message_type: MessageType,
    quality: u8,
}

fn parse_llc_header(input: &[u8]) -> Result<(&[u8], LlcHeader), Error> {
    let (input, (dest_lsap, src_lsap, quality)) =
        tuple::<_, _, (), _>((u8, u8, u8))(input).map_err(|_| Error::InvalidFormat)?;
    if quality != 0x00 {
        return Err(Error::InvalidFormat);
    }
    let destination = match dest_lsap {
        0xE6 => Destination::Unicast,
        0xFF => Destination::Broadcast,
        _ => return Err(Error::InvalidFormat),
    };
    let message_type = match src_lsap {
        0xE6 => MessageType::Command,
        0xE7 => MessageType::Response,
        _ => return Err(Error::InvalidFormat),
    };

    Ok((input, LlcHeader { destination, message_type, quality }))
}

#[derive(Debug)]
pub enum HdlcDataLinkLayer {}

impl<'i, 'f> DlmsDataLinkLayer<'i, &'f [HdlcFrame<'i>]> for HdlcDataLinkLayer {
    fn next_frame(
        frames: &'f [HdlcFrame<'i>],
    ) -> Result<(&'f [HdlcFrame<'i>], Cow<'i, [u8]>), Error> {
        if frames.is_empty() {
            Err(Error::Incomplete(None))
        } else if !frames[0].segmented {
            let information = frames[0].information;
            let (information, _) = parse_llc_header(information)?;
            Ok((&frames[1..], Cow::from(information)))
        } else {
            let mut done = false;
            let mut len = 0;
            let (information, _) = parse_llc_header(frames[0].information)?;
            let mut information = information.to_owned();
            for frame in &frames[1..] {
                information.extend(frame.information);
                len += 1;
                if !frame.segmented {
                    done = true;
                    break;
                }
            }
            if done {
                Ok((&frames[len + 1..], Cow::from(information)))
            } else {
                Err(Error::Incomplete(None))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hdlcparse::type3::HdlcFrame;

    #[test]
    fn test_parse_llc_header_unicast_command() {
        let data = [0xE6, 0xE6, 0x00, 0x01, 0x02, 0x03];
        let (remaining, header) = parse_llc_header(&data).unwrap();

        assert_eq!(remaining, &[0x01, 0x02, 0x03]);
        assert_eq!(header.quality, 0x00);
    }

    #[test]
    fn test_parse_llc_header_unicast_response() {
        let data = [0xE6, 0xE7, 0x00, 0xAA, 0xBB];
        let (remaining, header) = parse_llc_header(&data).unwrap();

        assert_eq!(remaining, &[0xAA, 0xBB]);
        assert_eq!(header.quality, 0x00);
    }

    #[test]
    fn test_parse_llc_header_broadcast_command() {
        let data = [0xFF, 0xE6, 0x00, 0xFF];
        let (remaining, header) = parse_llc_header(&data).unwrap();

        assert_eq!(remaining, &[0xFF]);
        assert_eq!(header.quality, 0x00);
    }

    #[test]
    fn test_parse_llc_header_invalid_quality() {
        let data = [0xE6, 0xE6, 0x01]; // quality != 0x00
        let result = parse_llc_header(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_llc_header_invalid_dest_lsap() {
        let data = [0xAA, 0xE6, 0x00]; // invalid dest_lsap
        let result = parse_llc_header(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_llc_header_invalid_src_lsap() {
        let data = [0xE6, 0xAA, 0x00]; // invalid src_lsap
        let result = parse_llc_header(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_llc_header_insufficient_data() {
        let data = [0xE6, 0xE6]; // only 2 bytes
        let result = parse_llc_header(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_next_frame_empty() {
        let frames: &[HdlcFrame] = &[];
        let result = HdlcDataLinkLayer::next_frame(frames);

        assert!(matches!(result, Err(Error::Incomplete(_))));
    }

    #[test]
    fn test_next_frame_single_unsegmented() {
        let information = vec![0xE6, 0xE6, 0x00, 0x01, 0x02, 0x03];
        let frame = HdlcFrame {
            src_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            dest_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            control: 0x10,
            information: &information,
            segmented: false,
        };
        let frames = vec![frame];

        let (remaining, payload) = HdlcDataLinkLayer::next_frame(&frames).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(&*payload, &[0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_next_frame_single_unsegmented_with_invalid_llc() {
        let information = vec![0xAA, 0xE6, 0x00]; // invalid LLC header
        let frame = HdlcFrame {
            src_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            dest_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            control: 0x10,
            information: &information,
            segmented: false,
        };
        let frames = vec![frame];

        let result = HdlcDataLinkLayer::next_frame(&frames);
        assert!(result.is_err());
    }

    #[test]
    fn test_next_frame_segmented_two_frames() {
        let info1 = vec![0xE6, 0xE6, 0x00, 0x01, 0x02];
        let info2 = vec![0x03, 0x04];

        let frame1 = HdlcFrame {
            src_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            dest_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            control: 0x10,
            information: &info1,
            segmented: true,
        };

        let frame2 = HdlcFrame {
            src_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            dest_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            control: 0x10,
            information: &info2,
            segmented: false,
        };

        let frames = vec![frame1, frame2];
        let (remaining, payload) = HdlcDataLinkLayer::next_frame(&frames).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(&*payload, &[0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_next_frame_segmented_three_frames() {
        let info1 = vec![0xE6, 0xE6, 0x00, 0xAA];
        let info2 = vec![0xBB];
        let info3 = vec![0xCC, 0xDD];

        let frame1 = HdlcFrame {
            src_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            dest_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            control: 0x10,
            information: &info1,
            segmented: true,
        };

        let frame2 = HdlcFrame {
            src_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            dest_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            control: 0x10,
            information: &info2,
            segmented: true,
        };

        let frame3 = HdlcFrame {
            src_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            dest_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            control: 0x10,
            information: &info3,
            segmented: false,
        };

        let frames = vec![frame1, frame2, frame3];
        let (remaining, payload) = HdlcDataLinkLayer::next_frame(&frames).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(&*payload, &[0xAA, 0xBB, 0xCC, 0xDD]);
    }

    #[test]
    fn test_next_frame_segmented_incomplete() {
        let info1 = vec![0xE6, 0xE6, 0x00, 0x01];
        let info2 = vec![0x02];

        let frame1 = HdlcFrame {
            src_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            dest_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            control: 0x10,
            information: &info1,
            segmented: true,
        };

        let frame2 = HdlcFrame {
            src_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            dest_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            control: 0x10,
            information: &info2,
            segmented: true, // still segmented, never ends
        };

        let frames = vec![frame1, frame2];
        let result = HdlcDataLinkLayer::next_frame(&frames);

        assert!(matches!(result, Err(Error::Incomplete(_))));
    }

    #[test]
    fn test_next_frame_multiple_messages() {
        let info1 = vec![0xE6, 0xE6, 0x00, 0x11];
        let info2 = vec![0xE6, 0xE7, 0x00, 0x22];

        let frame1 = HdlcFrame {
            src_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            dest_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            control: 0x10,
            information: &info1,
            segmented: false,
        };

        let frame2 = HdlcFrame {
            src_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            dest_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            control: 0x10,
            information: &info2,
            segmented: false,
        };

        let frames = vec![frame1, frame2];

        // First call should get frame1
        let (remaining, payload) = HdlcDataLinkLayer::next_frame(&frames).unwrap();
        assert_eq!(&*payload, &[0x11]);
        assert_eq!(remaining.len(), 1);

        // Second call should get frame2
        let (remaining, payload) = HdlcDataLinkLayer::next_frame(remaining).unwrap();
        assert_eq!(&*payload, &[0x22]);
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn test_next_frame_segmented_with_remaining_frames() {
        let info1 = vec![0xE6, 0xE6, 0x00, 0xAA];
        let info2 = vec![0xBB];
        let info3 = vec![0xE6, 0xE6, 0x00, 0xCC];

        let frame1 = HdlcFrame {
            src_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            dest_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            control: 0x10,
            information: &info1,
            segmented: true,
        };

        let frame2 = HdlcFrame {
            src_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            dest_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            control: 0x10,
            information: &info2,
            segmented: false,
        };

        let frame3 = HdlcFrame {
            src_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            dest_addr: hdlcparse::type3::HdlcAddress { upper: 0x01, lower: None },
            control: 0x10,
            information: &info3,
            segmented: false,
        };

        let frames = vec![frame1, frame2, frame3];

        // First call should combine frame1+frame2
        let (remaining, payload) = HdlcDataLinkLayer::next_frame(&frames).unwrap();
        assert_eq!(&*payload, &[0xAA, 0xBB]);
        assert_eq!(remaining.len(), 1);

        // Second call should get frame3
        let (remaining, payload) = HdlcDataLinkLayer::next_frame(remaining).unwrap();
        assert_eq!(&*payload, &[0xCC]);
        assert_eq!(remaining.len(), 0);
    }
}
