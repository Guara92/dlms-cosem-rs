use core::fmt;

use derive_try_from_primitive::TryFromPrimitive;
#[cfg(feature = "serde")]
use serde::{Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
#[non_exhaustive]
#[rustfmt::skip]
pub enum Unit {
  Year                       =   1,
  Month                      =   2,
  Week                       =   3,
  Day                        =   4,
  Hour                       =   5,
  Minute                     =   6,
  Second                     =   7,
  Degree                     =   8,
  DegreeCelsius              =   9,
  Currency                   =  10,
  Meter                      =  11,
  MeterPerSecond             =  12,
  CubicMeter                 =  13,
  CubicMeterCorrected        =  14,
  CubicMeterPerHour          =  15,
  CubicMeterPerHourCorrected =  16,
  CubicMeterPerDay           =  17,
  CubicMeterPerDayCorrected  =  18,
  Liter                      =  19,
  Kilogramm                  =  20,
  Newton                     =  21,
  Newtonmeter                =  22,
  Pascal                     =  23,
  Bar                        =  24,
  Joule                      =  25,
  JoulePerHour               =  26,
  Watt                       =  27,
  VoltAmpere                 =  28,
  Var                        =  29,
  WattHour                   =  30,
  VoltAmpereHour             =  31,
  VarHour                    =  32,
  Ampere                     =  33,
  Coulomb                    =  34,
  Volt                       =  35,
  VoltPerMeter               =  36,
  Farad                      =  37,
  Ohm                        =  38,
  OhmMeter                   =  39,
  Weber                      =  40,
  Tesla                      =  41,
  AmperePerMeter             =  42,
  Henry                      =  43,
  Hertz                      =  44,
  InverseWattHour            =  45,
  InverseVarHour             =  46,
  InverseVoltAmpereHour      =  47,
  VoltSquaredHour            =  48,
  AmpereSquaredHour          =  49,
  KilogrammPerSecond         =  50,
  Siemens                    =  51,
  Kelvin                     =  52,
  InverseVoltSquaredHour     =  53,
  InverseAmpereSquaredHour   =  54,
  InverseCubicMeter          =  55,
  Percent                    =  56,
  AmpereHour                 =  57,
  // 58-59 reserved
  WattHourPerCubicMeter      =  60,
  JoulePerCubicMeter         =  61,
  MolePercent                =  62,
  GrammPerCubicMeter         =  63,
  PascalSecond               =  64,
  JoulePerKilogramm          =  65,
  GramPerSquareCentimeter    =  66,
  Atmosphere                 =  67,
  // 68-69 reserved
  DezibelMilliwatt           =  70,
  DezibelMicrovolt           =  71,
  Dezibel                    =  72,
  // 73-127 reserved
  // 128-174 non-SI-units
  // 175-252 reserved
  // 253 extended table of units
  Other                      = 254,
  Count                      = 255,
}

impl Unit {
    #[rustfmt::skip]
    pub fn as_str(&self) -> Option<&'static str> {
    Some(match self {
      Self::Year                       => "a",
      Self::Month                      => "mo",
      Self::Week                       => "wk",
      Self::Day                        => "d",
      Self::Hour                       => "h",
      Self::Minute                     => "min",
      Self::Second                     => "s",
      Self::Degree                     => "°",
      Self::DegreeCelsius              => "°C",
      Self::Currency                   => "currency",
      Self::Meter                      => "m",
      Self::MeterPerSecond             => "m/s",
      Self::CubicMeter                 => "m³",
      Self::CubicMeterCorrected        => "m³",
      Self::CubicMeterPerHour          => "m³/h",
      Self::CubicMeterPerHourCorrected => "m³/h",
      Self::CubicMeterPerDay           => "m³/d",
      Self::CubicMeterPerDayCorrected  => "m³/d",
      Self::Liter                      => "l",
      Self::Kilogramm                  => "kg",
      Self::Newton                     => "N",
      Self::Newtonmeter                => "Nm",
      Self::Pascal                     => "Pa",
      Self::Bar                        => "bar",
      Self::Joule                      => "J",
      Self::JoulePerHour               => "J/h",
      Self::Watt                       => "W",
      Self::VoltAmpere                 => "VA",
      Self::Var                        => "var",
      Self::WattHour                   => "Wh",
      Self::VoltAmpereHour             => "VAh",
      Self::VarHour                    => "varh",
      Self::Ampere                     => "A",
      Self::Coulomb                    => "C",
      Self::Volt                       => "V",
      Self::VoltPerMeter               => "V/m",
      Self::Farad                      => "F",
      Self::Ohm                        => "Ω",
      Self::OhmMeter                   => "Ωm",
      Self::Weber                      => "Wb",
      Self::Tesla                      => "T",
      Self::AmperePerMeter             => "A/m",
      Self::Henry                      => "H",
      Self::Hertz                      => "Hz",
      Self::InverseWattHour            => "1/(Wh)",
      Self::InverseVarHour             => "1/(varh)",
      Self::InverseVoltAmpereHour      => "1/(VAh)",
      Self::VoltSquaredHour            => "V²h",
      Self::AmpereSquaredHour          => "A²h",
      Self::KilogrammPerSecond         => "kg/s",
      Self::Siemens                    => "S",
      Self::Kelvin                     => "K",
      Self::InverseVoltSquaredHour     => "1/(V²h)",
      Self::InverseAmpereSquaredHour   => "1/(A²h)",
      Self::InverseCubicMeter          => "1/m³",
      Self::Percent                    => "%",
      Self::AmpereHour                 => "Ah",
      Self::WattHourPerCubicMeter      => "Wh/m³",
      Self::JoulePerCubicMeter         => "J/m³",
      Self::MolePercent                => "Mol %",
      Self::GrammPerCubicMeter         => "g/m³",
      Self::PascalSecond               => "Pa s",
      Self::JoulePerKilogramm          => "J/kg",
      Self::GramPerSquareCentimeter    => "g/cm²",
      Self::Atmosphere                 => "atm",
      Self::DezibelMilliwatt           => "dBm",
      Self::DezibelMicrovolt           => "dBµV",
      Self::Dezibel                    => "dB",
      Self::Other | Self::Count        => return None,
    })
  }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(s) = self.as_str() { s.fmt(f) } else { Ok(()) }
    }
}

#[cfg(feature = "serde")]
impl Serialize for Unit {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(s) = self.as_str() {
            serializer.serialize_str(s)
        } else {
            serializer.serialize_none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::convert::TryFrom;

    #[test]
    fn test_try_from_valid_units() {
        // Test common units
        assert_eq!(Unit::try_from(1).unwrap(), Unit::Year);
        assert_eq!(Unit::try_from(5).unwrap(), Unit::Hour);
        assert_eq!(Unit::try_from(7).unwrap(), Unit::Second);
        assert_eq!(Unit::try_from(27).unwrap(), Unit::Watt);
        assert_eq!(Unit::try_from(30).unwrap(), Unit::WattHour);
        assert_eq!(Unit::try_from(33).unwrap(), Unit::Ampere);
        assert_eq!(Unit::try_from(35).unwrap(), Unit::Volt);
        assert_eq!(Unit::try_from(44).unwrap(), Unit::Hertz);
        assert_eq!(Unit::try_from(255).unwrap(), Unit::Count);
        assert_eq!(Unit::try_from(254).unwrap(), Unit::Other);
    }

    #[test]
    fn test_try_from_invalid_units() {
        // Reserved values should fail
        assert!(Unit::try_from(0).is_err());
        assert!(Unit::try_from(58).is_err());
        assert!(Unit::try_from(59).is_err());
        assert!(Unit::try_from(68).is_err());
        assert!(Unit::try_from(69).is_err());
        assert!(Unit::try_from(73).is_err());
        assert!(Unit::try_from(127).is_err());
        assert!(Unit::try_from(253).is_err());
    }

    #[test]
    fn test_as_str_time_units() {
        assert_eq!(Unit::Year.as_str(), Some("a"));
        assert_eq!(Unit::Month.as_str(), Some("mo"));
        assert_eq!(Unit::Week.as_str(), Some("wk"));
        assert_eq!(Unit::Day.as_str(), Some("d"));
        assert_eq!(Unit::Hour.as_str(), Some("h"));
        assert_eq!(Unit::Minute.as_str(), Some("min"));
        assert_eq!(Unit::Second.as_str(), Some("s"));
    }

    #[test]
    fn test_as_str_temperature_units() {
        assert_eq!(Unit::Degree.as_str(), Some("°"));
        assert_eq!(Unit::DegreeCelsius.as_str(), Some("°C"));
        assert_eq!(Unit::Kelvin.as_str(), Some("K"));
    }

    #[test]
    fn test_as_str_electrical_units() {
        assert_eq!(Unit::Ampere.as_str(), Some("A"));
        assert_eq!(Unit::Volt.as_str(), Some("V"));
        assert_eq!(Unit::Watt.as_str(), Some("W"));
        assert_eq!(Unit::WattHour.as_str(), Some("Wh"));
        assert_eq!(Unit::VoltAmpere.as_str(), Some("VA"));
        assert_eq!(Unit::VoltAmpereHour.as_str(), Some("VAh"));
        assert_eq!(Unit::Var.as_str(), Some("var"));
        assert_eq!(Unit::VarHour.as_str(), Some("varh"));
        assert_eq!(Unit::AmpereHour.as_str(), Some("Ah"));
        assert_eq!(Unit::Hertz.as_str(), Some("Hz"));
        assert_eq!(Unit::Ohm.as_str(), Some("Ω"));
        assert_eq!(Unit::Farad.as_str(), Some("F"));
        assert_eq!(Unit::Henry.as_str(), Some("H"));
        assert_eq!(Unit::Siemens.as_str(), Some("S"));
        assert_eq!(Unit::Coulomb.as_str(), Some("C"));
    }

    #[test]
    fn test_as_str_volume_units() {
        assert_eq!(Unit::CubicMeter.as_str(), Some("m³"));
        assert_eq!(Unit::CubicMeterCorrected.as_str(), Some("m³"));
        assert_eq!(Unit::CubicMeterPerHour.as_str(), Some("m³/h"));
        assert_eq!(Unit::CubicMeterPerHourCorrected.as_str(), Some("m³/h"));
        assert_eq!(Unit::CubicMeterPerDay.as_str(), Some("m³/d"));
        assert_eq!(Unit::CubicMeterPerDayCorrected.as_str(), Some("m³/d"));
        assert_eq!(Unit::Liter.as_str(), Some("l"));
    }

    #[test]
    fn test_as_str_pressure_units() {
        assert_eq!(Unit::Pascal.as_str(), Some("Pa"));
        assert_eq!(Unit::Bar.as_str(), Some("bar"));
        assert_eq!(Unit::Atmosphere.as_str(), Some("atm"));
    }

    #[test]
    fn test_as_str_other_units() {
        assert_eq!(Unit::Meter.as_str(), Some("m"));
        assert_eq!(Unit::Kilogramm.as_str(), Some("kg"));
        assert_eq!(Unit::Newton.as_str(), Some("N"));
        assert_eq!(Unit::Joule.as_str(), Some("J"));
        assert_eq!(Unit::Percent.as_str(), Some("%"));
    }

    #[test]
    fn test_as_str_special_units() {
        // Other and Count return None
        assert_eq!(Unit::Other.as_str(), None);
        assert_eq!(Unit::Count.as_str(), None);
    }

    #[test]
    fn test_display_format() {
        // Test Display trait
        assert_eq!(format!("{}", Unit::WattHour), "Wh");
        assert_eq!(format!("{}", Unit::Ampere), "A");
        assert_eq!(format!("{}", Unit::Volt), "V");
        assert_eq!(format!("{}", Unit::Hertz), "Hz");
        assert_eq!(format!("{}", Unit::DegreeCelsius), "°C");
        assert_eq!(format!("{}", Unit::Percent), "%");

        // Special units display as empty string
        assert_eq!(format!("{}", Unit::Other), "");
        assert_eq!(format!("{}", Unit::Count), "");
    }

    #[test]
    fn test_equality() {
        assert_eq!(Unit::WattHour, Unit::WattHour);
        assert_ne!(Unit::WattHour, Unit::VoltAmpereHour);
        assert_ne!(Unit::Ampere, Unit::Volt);
    }

    #[test]
    fn test_clone() {
        let unit1 = Unit::WattHour;
        let unit2 = unit1.clone();
        assert_eq!(unit1, unit2);
    }

    #[test]
    fn test_copy() {
        let unit1 = Unit::Ampere;
        let unit2 = unit1; // Copy, not move
        assert_eq!(unit1, unit2);
    }

    #[test]
    fn test_debug_format() {
        let debug_str = format!("{:?}", Unit::WattHour);
        assert!(debug_str.contains("WattHour"));
    }

    #[test]
    fn test_energy_units() {
        // Common energy-related units for metering
        assert_eq!(Unit::try_from(30).unwrap(), Unit::WattHour);
        assert_eq!(Unit::WattHour.as_str(), Some("Wh"));

        assert_eq!(Unit::try_from(31).unwrap(), Unit::VoltAmpereHour);
        assert_eq!(Unit::VoltAmpereHour.as_str(), Some("VAh"));

        assert_eq!(Unit::try_from(32).unwrap(), Unit::VarHour);
        assert_eq!(Unit::VarHour.as_str(), Some("varh"));
    }

    #[test]
    fn test_power_units() {
        // Common power units
        assert_eq!(Unit::try_from(27).unwrap(), Unit::Watt);
        assert_eq!(Unit::Watt.as_str(), Some("W"));

        assert_eq!(Unit::try_from(28).unwrap(), Unit::VoltAmpere);
        assert_eq!(Unit::VoltAmpere.as_str(), Some("VA"));

        assert_eq!(Unit::try_from(29).unwrap(), Unit::Var);
        assert_eq!(Unit::Var.as_str(), Some("var"));
    }

    #[test]
    fn test_inverse_units() {
        assert_eq!(Unit::InverseWattHour.as_str(), Some("1/(Wh)"));
        assert_eq!(Unit::InverseVarHour.as_str(), Some("1/(varh)"));
        assert_eq!(Unit::InverseVoltAmpereHour.as_str(), Some("1/(VAh)"));
        assert_eq!(Unit::InverseVoltSquaredHour.as_str(), Some("1/(V²h)"));
        assert_eq!(Unit::InverseAmpereSquaredHour.as_str(), Some("1/(A²h)"));
        assert_eq!(Unit::InverseCubicMeter.as_str(), Some("1/m³"));
    }

    #[test]
    fn test_squared_units() {
        assert_eq!(Unit::VoltSquaredHour.as_str(), Some("V²h"));
        assert_eq!(Unit::AmpereSquaredHour.as_str(), Some("A²h"));
        assert_eq!(Unit::GramPerSquareCentimeter.as_str(), Some("g/cm²"));
    }

    #[test]
    fn test_decibel_units() {
        assert_eq!(Unit::DezibelMilliwatt.as_str(), Some("dBm"));
        assert_eq!(Unit::DezibelMicrovolt.as_str(), Some("dBµV"));
        assert_eq!(Unit::Dezibel.as_str(), Some("dB"));
    }

    #[test]
    fn test_composite_units() {
        assert_eq!(Unit::MeterPerSecond.as_str(), Some("m/s"));
        assert_eq!(Unit::KilogrammPerSecond.as_str(), Some("kg/s"));
        assert_eq!(Unit::VoltPerMeter.as_str(), Some("V/m"));
        assert_eq!(Unit::AmperePerMeter.as_str(), Some("A/m"));
        assert_eq!(Unit::JoulePerHour.as_str(), Some("J/h"));
        assert_eq!(Unit::WattHourPerCubicMeter.as_str(), Some("Wh/m³"));
        assert_eq!(Unit::JoulePerCubicMeter.as_str(), Some("J/m³"));
        assert_eq!(Unit::GrammPerCubicMeter.as_str(), Some("g/m³"));
        assert_eq!(Unit::JoulePerKilogramm.as_str(), Some("J/kg"));
    }

    #[test]
    fn test_rare_scientific_units() {
        // Test uncommon scientific units
        assert_eq!(Unit::try_from(40).unwrap(), Unit::Weber);
        assert_eq!(Unit::Weber.as_str(), Some("Wb"));

        assert_eq!(Unit::try_from(41).unwrap(), Unit::Tesla);
        assert_eq!(Unit::Tesla.as_str(), Some("T"));

        assert_eq!(Unit::try_from(43).unwrap(), Unit::Henry);
        assert_eq!(Unit::Henry.as_str(), Some("H"));

        assert_eq!(Unit::try_from(37).unwrap(), Unit::Farad);
        assert_eq!(Unit::Farad.as_str(), Some("F"));

        assert_eq!(Unit::try_from(51).unwrap(), Unit::Siemens);
        assert_eq!(Unit::Siemens.as_str(), Some("S"));
    }

    #[test]
    fn test_rare_gas_units() {
        // Test uncommon gas/fluid units
        assert_eq!(Unit::try_from(62).unwrap(), Unit::MolePercent);
        assert_eq!(Unit::MolePercent.as_str(), Some("Mol %"));

        assert_eq!(Unit::try_from(64).unwrap(), Unit::PascalSecond);
        assert_eq!(Unit::PascalSecond.as_str(), Some("Pa s"));

        assert_eq!(Unit::try_from(67).unwrap(), Unit::Atmosphere);
        assert_eq!(Unit::Atmosphere.as_str(), Some("atm"));
    }

    #[test]
    fn test_rare_mechanical_units() {
        // Test uncommon mechanical units
        assert_eq!(Unit::try_from(21).unwrap(), Unit::Newton);
        assert_eq!(Unit::Newton.as_str(), Some("N"));

        assert_eq!(Unit::try_from(22).unwrap(), Unit::Newtonmeter);
        assert_eq!(Unit::Newtonmeter.as_str(), Some("Nm"));

        assert_eq!(Unit::try_from(34).unwrap(), Unit::Coulomb);
        assert_eq!(Unit::Coulomb.as_str(), Some("C"));

        assert_eq!(Unit::try_from(52).unwrap(), Unit::Kelvin);
        assert_eq!(Unit::Kelvin.as_str(), Some("K"));
    }

    #[test]
    fn test_rare_energy_density_units() {
        // Test uncommon energy/mass units
        assert_eq!(Unit::try_from(65).unwrap(), Unit::JoulePerKilogramm);
        assert_eq!(Unit::JoulePerKilogramm.as_str(), Some("J/kg"));

        assert_eq!(Unit::try_from(63).unwrap(), Unit::GrammPerCubicMeter);
        assert_eq!(Unit::GrammPerCubicMeter.as_str(), Some("g/m³"));

        assert_eq!(Unit::try_from(66).unwrap(), Unit::GramPerSquareCentimeter);
        assert_eq!(Unit::GramPerSquareCentimeter.as_str(), Some("g/cm²"));
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialize() {
        use serde::Serialize;

        // Units with string representation
        let unit1 = Unit::WattHour;
        assert_eq!(unit1.as_str(), Some("Wh"));

        let unit2 = Unit::Ampere;
        assert_eq!(unit2.as_str(), Some("A"));

        // Special units return None for as_str (will serialize as null)
        let unit3 = Unit::Other;
        assert_eq!(unit3.as_str(), None);

        let unit4 = Unit::Count;
        assert_eq!(unit4.as_str(), None);

        // Verify the trait is implemented (compile-time check)
        fn assert_serialize<T: Serialize>(_: &T) {}
        assert_serialize(&unit1);
        assert_serialize(&unit2);
        assert_serialize(&unit3);
        assert_serialize(&unit4);
    }

    #[test]
    fn test_all_time_units_roundtrip() {
        let units = [
            (1, Unit::Year, "a"),
            (2, Unit::Month, "mo"),
            (3, Unit::Week, "wk"),
            (4, Unit::Day, "d"),
            (5, Unit::Hour, "h"),
            (6, Unit::Minute, "min"),
            (7, Unit::Second, "s"),
        ];

        for (value, unit, expected_str) in units {
            assert_eq!(Unit::try_from(value).unwrap(), unit);
            assert_eq!(unit.as_str(), Some(expected_str));
            assert_eq!(format!("{}", unit), expected_str);
        }
    }
}
