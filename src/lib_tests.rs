#[cfg(test)]
mod register_tests {
    use super::super::*;
    use alloc::vec;

    #[test]
    fn test_register_getters() {
        let obis = ObisCode::new(1, 0, 1, 8, 0, 255);
        let value = Data::DoubleLongUnsigned(12345);
        let unit = Some(Unit::try_from(0x1e).unwrap());

        let register = Register { obis_code: obis.clone(), value: value.clone(), unit };

        assert_eq!(register.obis_code(), &obis);
        assert_eq!(register.value(), &value);
        assert_eq!(register.unit(), Some(&Unit::try_from(0x1e).unwrap()));
    }

    #[test]
    fn test_register_parse_simple() {
        let obis_bytes = vec![1, 0, 1, 8, 0, 255];
        let input = vec![Data::OctetString(obis_bytes), Data::DoubleLongUnsigned(12345)];

        let (remaining, register) = Register::parse(&input).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(register.obis_code(), &ObisCode::new(1, 0, 1, 8, 0, 255));
        assert_eq!(register.value(), &Data::DoubleLongUnsigned(12345));
        assert_eq!(register.unit(), None);
    }

    #[test]
    fn test_register_parse_with_scaler_unit() {
        let obis_bytes = vec![1, 0, 1, 7, 0, 255];
        let input = vec![
            Data::OctetString(obis_bytes),
            Data::DoubleLongUnsigned(3934),
            Data::Structure(vec![
                Data::Integer(-2), // scaler = 10^-2
                Data::Enum(0x1b),  // Unit::try_from(0x1b).unwrap()
            ]),
        ];

        let (remaining, register) = Register::parse(&input).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(register.obis_code(), &ObisCode::new(1, 0, 1, 7, 0, 255));
        assert_eq!(register.value(), &Data::Float64(39.34)); // 3934 / 100
        assert_eq!(register.unit(), Some(&Unit::try_from(0x1b).unwrap()));
    }

    #[test]
    fn test_register_parse_scaler_positive() {
        let obis_bytes = vec![1, 0, 1, 8, 0, 255];
        let input = vec![
            Data::OctetString(obis_bytes),
            Data::LongUnsigned(123),
            Data::Structure(vec![
                Data::Integer(2), // scaler = 10^2
                Data::Enum(0x1e), // Unit::try_from(0x1e).unwrap()
            ]),
        ];

        let (remaining, register) = Register::parse(&input).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(register.value(), &Data::Float32(12300.0)); // 123 * 100
        assert_eq!(register.unit(), Some(&Unit::try_from(0x1e).unwrap()));
    }

    #[test]
    fn test_register_parse_nested_structure() {
        let obis_bytes = vec![1, 0, 1, 8, 0, 255];
        let input = vec![Data::Structure(vec![
            Data::OctetString(obis_bytes),
            Data::DoubleLongUnsigned(5000),
            Data::Structure(vec![
                Data::Integer(-1),
                Data::Enum(0x1e), // Unit::try_from(0x1e).unwrap()
            ]),
        ])];

        let (remaining, register) = Register::parse(&input).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(register.obis_code(), &ObisCode::new(1, 0, 1, 8, 0, 255));
        assert_eq!(register.value(), &Data::Float64(500.0)); // 5000 / 10
        assert_eq!(register.unit(), Some(&Unit::try_from(0x1e).unwrap()));
    }

    #[test]
    fn test_register_parse_no_scaling_when_zero() {
        let obis_bytes = vec![1, 0, 1, 8, 0, 255];
        let input = vec![
            Data::OctetString(obis_bytes),
            Data::DoubleLongUnsigned(1000),
            Data::Structure(vec![
                Data::Integer(0), // scaler = 0
                Data::Enum(0xff), // unit = 0xff
            ]),
        ];

        let (remaining, register) = Register::parse(&input).unwrap();
        // When scaler=0 and unit=0xff, parse succeeds but scaler/unit is ignored
        assert_eq!(remaining.len(), 1); // The invalid scaler/unit structure remains
        assert_eq!(register.value(), &Data::DoubleLongUnsigned(1000));
        assert_eq!(register.unit(), None);
    }

    #[test]
    fn test_register_parse_invalid_obis_code() {
        let input = vec![
            Data::OctetString(vec![1, 2, 3]), // Too short for OBIS code
            Data::DoubleLongUnsigned(100),
        ];

        let result = Register::parse(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_parse_with_voltage() {
        let obis_bytes = vec![1, 0, 32, 7, 0, 255];
        let input = vec![
            Data::OctetString(obis_bytes),
            Data::LongUnsigned(2337), // Voltage in decivolts
            Data::Structure(vec![
                Data::Integer(-1),
                Data::Enum(0x23), // Unit::try_from(0x23).unwrap()
            ]),
        ];

        let (remaining, register) = Register::parse(&input).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(register.value(), &Data::Float32(233.7)); // 2337 / 10
        assert_eq!(register.unit(), Some(&Unit::try_from(0x23).unwrap()));
    }

    #[test]
    fn test_register_parse_incomplete_input() {
        let input = vec![
            Data::OctetString(vec![1, 0, 1, 8, 0, 255]),
            // Missing value
        ];

        let result = Register::parse(&input);
        // Implementation returns Incomplete correctly
        assert!(result.is_err());
    }

    #[test]
    fn test_register_parse_wrong_obis_data_type() {
        let input = vec![
            Data::DoubleLongUnsigned(12345), // Should be OctetString
            Data::DoubleLongUnsigned(100),
        ];

        let result = Register::parse(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_parse_invalid_scaler_unit_structure() {
        let obis_bytes = vec![1, 0, 1, 8, 0, 255];
        let input = vec![
            Data::OctetString(obis_bytes),
            Data::DoubleLongUnsigned(1000),
            Data::Structure(vec![
                Data::Integer(1), // Only one element, needs two
            ]),
        ];

        let (remaining, register) = Register::parse(&input).unwrap();
        // Should parse but ignore the invalid scaler/unit structure
        assert_eq!(remaining.len(), 1); // The invalid structure remains
        assert_eq!(register.unit(), None);
    }
}

#[cfg(test)]
mod obis_map_tests {
    use super::super::*;
    use alloc::vec;

    #[test]
    fn test_obis_map_parse_empty() {
        let apdu = Apdu::DataNotification(DataNotification {
            long_invoke_id_and_priority: LongInvokeIdAndPriority(0),
            date_time: DateTime {
                date: Date { year: 2024, month: 1, day_of_month: 1, day_of_week: 1 },
                time: Time { hour: None, minute: None, second: None, hundredth: None },
                offset_minutes: None,
                clock_status: None,
            },
            notification_body: Data::Structure(vec![]),
        });

        let (_, map) = ObisMap::parse(&apdu).unwrap();
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_obis_map_parse_single_register() {
        let obis_bytes = vec![1, 0, 1, 8, 0, 255];
        let apdu = Apdu::DataNotification(DataNotification {
            long_invoke_id_and_priority: LongInvokeIdAndPriority(0),
            date_time: DateTime {
                date: Date { year: 2024, month: 1, day_of_month: 1, day_of_week: 1 },
                time: Time { hour: None, minute: None, second: None, hundredth: None },
                offset_minutes: None,
                clock_status: None,
            },
            notification_body: Data::Structure(vec![
                Data::OctetString(obis_bytes),
                Data::DoubleLongUnsigned(12345),
            ]),
        });

        let (_, map) = ObisMap::parse(&apdu).unwrap();
        assert_eq!(map.len(), 1);

        let obis = ObisCode::new(1, 0, 1, 8, 0, 255);
        assert!(map.contains_key(&obis));
        assert_eq!(map.get(&obis).unwrap().value(), &Data::DoubleLongUnsigned(12345));
    }

    #[test]
    fn test_obis_map_parse_multiple_registers() {
        let apdu = Apdu::DataNotification(DataNotification {
            long_invoke_id_and_priority: LongInvokeIdAndPriority(0),
            date_time: DateTime {
                date: Date { year: 2024, month: 1, day_of_month: 1, day_of_week: 1 },
                time: Time { hour: None, minute: None, second: None, hundredth: None },
                offset_minutes: None,
                clock_status: None,
            },
            notification_body: Data::Structure(vec![
                Data::OctetString(vec![1, 0, 1, 8, 0, 255]),
                Data::DoubleLongUnsigned(12345),
                Data::Structure(vec![Data::Integer(0), Data::Enum(0x1e)]),
                Data::OctetString(vec![1, 0, 2, 8, 0, 255]),
                Data::DoubleLongUnsigned(6789),
                Data::OctetString(vec![1, 0, 1, 7, 0, 255]),
                Data::LongUnsigned(2500),
                Data::Structure(vec![Data::Integer(-2), Data::Enum(0x1b)]),
            ]),
        });

        let (_, map) = ObisMap::parse(&apdu).unwrap();
        assert_eq!(map.len(), 3);

        let obis1 = ObisCode::new(1, 0, 1, 8, 0, 255);
        let obis2 = ObisCode::new(1, 0, 2, 8, 0, 255);
        let obis3 = ObisCode::new(1, 0, 1, 7, 0, 255);

        assert!(map.contains_key(&obis1));
        assert!(map.contains_key(&obis2));
        assert!(map.contains_key(&obis3));

        assert_eq!(map.get(&obis2).unwrap().value(), &Data::DoubleLongUnsigned(6789));
        assert_eq!(map.get(&obis3).unwrap().value(), &Data::Float32(25.0));
    }

    #[test]
    fn test_obis_map_deref() {
        let obis_bytes = vec![1, 0, 1, 8, 0, 255];
        let apdu = Apdu::DataNotification(DataNotification {
            long_invoke_id_and_priority: LongInvokeIdAndPriority(0),
            date_time: DateTime {
                date: Date { year: 2024, month: 1, day_of_month: 1, day_of_week: 1 },
                time: Time { hour: None, minute: None, second: None, hundredth: None },
                offset_minutes: None,
                clock_status: None,
            },
            notification_body: Data::Structure(vec![
                Data::OctetString(obis_bytes),
                Data::DoubleLongUnsigned(100),
            ]),
        });

        let (_, map) = ObisMap::parse(&apdu).unwrap();

        // Test Deref - can use BTreeMap methods directly
        assert_eq!(map.len(), 1);
        assert!(map.contains_key(&ObisCode::new(1, 0, 1, 8, 0, 255)));
    }

    #[test]
    fn test_obis_map_deref_mut() {
        let apdu = Apdu::DataNotification(DataNotification {
            long_invoke_id_and_priority: LongInvokeIdAndPriority(0),
            date_time: DateTime {
                date: Date { year: 2024, month: 1, day_of_month: 1, day_of_week: 1 },
                time: Time { hour: None, minute: None, second: None, hundredth: None },
                offset_minutes: None,
                clock_status: None,
            },
            notification_body: Data::Structure(vec![
                Data::OctetString(vec![1, 0, 1, 8, 0, 255]),
                Data::DoubleLongUnsigned(100),
            ]),
        });

        let (_, mut map) = ObisMap::parse(&apdu).unwrap();

        // Test DerefMut - can modify using BTreeMap methods
        let obis = ObisCode::new(1, 0, 2, 8, 0, 255);
        let new_reg =
            Register { obis_code: obis.clone(), value: Data::DoubleLongUnsigned(200), unit: None };
        map.insert(obis, new_reg);

        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_obis_map_convert() {
        let apdu = Apdu::DataNotification(DataNotification {
            long_invoke_id_and_priority: LongInvokeIdAndPriority(0),
            date_time: DateTime {
                date: Date { year: 2024, month: 1, day_of_month: 1, day_of_week: 1 },
                time: Time { hour: None, minute: None, second: None, hundredth: None },
                offset_minutes: None,
                clock_status: None,
            },
            notification_body: Data::Structure(vec![
                Data::OctetString(vec![1, 0, 1, 8, 0, 255]),
                Data::DoubleLongUnsigned(100),
            ]),
        });

        let (_, mut map) = ObisMap::parse(&apdu).unwrap();
        let obis = ObisCode::new(1, 0, 1, 8, 0, 255);

        // Convert the data using the convert method
        map.convert(&obis, |data| {
            if let Data::DoubleLongUnsigned(val) = data {
                Data::DoubleLongUnsigned(val * 2)
            } else {
                data
            }
        });

        assert_eq!(map.get(&obis).unwrap().value(), &Data::DoubleLongUnsigned(200));
    }

    #[test]
    fn test_obis_map_convert_nonexistent() {
        let apdu = Apdu::DataNotification(DataNotification {
            long_invoke_id_and_priority: LongInvokeIdAndPriority(0),
            date_time: DateTime {
                date: Date { year: 2024, month: 1, day_of_month: 1, day_of_week: 1 },
                time: Time { hour: None, minute: None, second: None, hundredth: None },
                offset_minutes: None,
                clock_status: None,
            },
            notification_body: Data::Structure(vec![
                Data::OctetString(vec![1, 0, 1, 8, 0, 255]),
                Data::DoubleLongUnsigned(100),
            ]),
        });

        let (_, mut map) = ObisMap::parse(&apdu).unwrap();
        let nonexistent = ObisCode::new(9, 9, 9, 9, 9, 9);

        // Converting a non-existent key should do nothing
        map.convert(&nonexistent, |_| Data::Null);

        assert_eq!(map.len(), 1);
        assert!(!map.contains_key(&nonexistent));
    }

    #[test]
    fn test_obis_map_parse_invalid_notification_body() {
        // notification_body should be Structure, not a scalar value
        let apdu = Apdu::DataNotification(DataNotification {
            long_invoke_id_and_priority: LongInvokeIdAndPriority(0),
            date_time: DateTime {
                date: Date { year: 2024, month: 1, day_of_month: 1, day_of_week: 1 },
                time: Time { hour: None, minute: None, second: None, hundredth: None },
                offset_minutes: None,
                clock_status: None,
            },
            notification_body: Data::Null, // Invalid: should be Structure
        });

        let result = ObisMap::parse(&apdu);
        assert!(result.is_err());
    }

    #[test]
    fn test_obis_map_parse_nested_registers() {
        let apdu = Apdu::DataNotification(DataNotification {
            long_invoke_id_and_priority: LongInvokeIdAndPriority(0),
            date_time: DateTime {
                date: Date { year: 2024, month: 1, day_of_month: 1, day_of_week: 1 },
                time: Time { hour: None, minute: None, second: None, hundredth: None },
                offset_minutes: None,
                clock_status: None,
            },
            notification_body: Data::Structure(vec![Data::Structure(vec![
                Data::OctetString(vec![1, 0, 1, 8, 0, 255]),
                Data::DoubleLongUnsigned(5000),
                Data::Structure(vec![
                    Data::Integer(-1),
                    Data::Enum(0x1e), // Unit::try_from(0x1e).unwrap()
                ]),
            ])]),
        });

        let (_, map) = ObisMap::parse(&apdu).unwrap();
        assert_eq!(map.len(), 1);

        let obis = ObisCode::new(1, 0, 1, 8, 0, 255);
        assert_eq!(map.get(&obis).unwrap().value(), &Data::Float64(500.0));
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_obis_map_serialize() {
        use serde::Serialize;

        let apdu = Apdu::DataNotification(DataNotification {
            long_invoke_id_and_priority: LongInvokeIdAndPriority(0),
            date_time: DateTime {
                date: Date { year: 2024, month: 1, day_of_month: 1, day_of_week: 1 },
                time: Time { hour: None, minute: None, second: None, hundredth: None },
                offset_minutes: None,
                clock_status: None,
            },
            notification_body: Data::Structure(vec![
                Data::Structure(vec![
                    Data::OctetString(vec![1, 0, 1, 8, 0, 255]),
                    Data::DoubleLongUnsigned(1234),
                    Data::Structure(vec![Data::Integer(0), Data::Enum(0x1e)]), // Wh
                ]),
                Data::Structure(vec![
                    Data::OctetString(vec![1, 0, 32, 7, 0, 255]),
                    Data::DoubleLongUnsigned(230),
                    Data::Structure(vec![Data::Integer(0), Data::Enum(0x23)]), // V
                ]),
            ]),
        });

        let (_, map) = ObisMap::parse(&apdu).unwrap();

        // Verify the map was parsed correctly
        assert_eq!(map.len(), 2);
        assert!(map.contains_key(&ObisCode::new(1, 0, 1, 8, 0, 255)));
        assert!(map.contains_key(&ObisCode::new(1, 0, 32, 7, 0, 255)));

        // Verify the trait is implemented (compile-time check)
        fn assert_serialize<T: Serialize>(_: &T) {}
        assert_serialize(&map);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_obis_map_serialize_no_unit() {
        use serde::Serialize;

        let apdu = Apdu::DataNotification(DataNotification {
            long_invoke_id_and_priority: LongInvokeIdAndPriority(0),
            date_time: DateTime {
                date: Date { year: 2024, month: 1, day_of_month: 1, day_of_week: 1 },
                time: Time { hour: None, minute: None, second: None, hundredth: None },
                offset_minutes: None,
                clock_status: None,
            },
            notification_body: Data::Structure(vec![Data::Structure(vec![
                Data::OctetString(vec![1, 0, 1, 8, 0, 255]),
                Data::DoubleLongUnsigned(999),
            ])]),
        });

        let (_, map) = ObisMap::parse(&apdu).unwrap();

        // Verify the map was parsed correctly with no unit
        assert_eq!(map.len(), 1);
        let obis = ObisCode::new(1, 0, 1, 8, 0, 255);
        assert!(map.contains_key(&obis));
        assert!(map.get(&obis).unwrap().unit().is_none());

        // Verify the trait is implemented (compile-time check)
        fn assert_serialize<T: Serialize>(_: &T) {}
        assert_serialize(&map);
    }
}
