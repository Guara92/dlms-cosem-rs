use core::fmt;

use derive_try_from_primitive::TryFromPrimitive;
#[cfg(feature = "serde")]
use serde::{Serialize, Serializer};

#[cfg(feature = "encode")]
use alloc::vec::Vec;

#[cfg(any(feature = "parse", feature = "encode"))]
use crate::data::Data;

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
    #[cfg(feature = "encode")]
    /// Encode unit as single byte enum value (0-255)
    /// Reference: Green Book Ed. 12, Section 4.1.6.1 - enum encoding
    pub fn encode(&self) -> u8 {
        *self as u8
    }

    #[cfg(feature = "encode")]
    /// Convert unit enum to signed i8 for DLMS Structure encoding
    ///
    /// This method is used when encoding scaler_unit as Structure(Integer, Enum).
    /// The DLMS specification requires the unit to be encoded as an Enum type,
    /// which is represented as a signed i8 in the Structure.
    ///
    /// Values 0-127 map to positive i8 (0 to 127).
    /// Values 128-255 map to negative i8 (-128 to -1).
    ///
    /// # Example
    /// ```
    /// use dlms_cosem::Unit;
    ///
    /// // WattHour = 30 (0x1E) - positive value
    /// assert_eq!(Unit::WattHour.as_i8(), 30);
    ///
    /// // Volt = 35 (0x23) - positive value
    /// assert_eq!(Unit::Volt.as_i8(), 35);
    ///
    /// // Values >= 128 become negative
    /// // (none in current DLMS unit table)
    /// ```
    ///
    /// Reference: Green Book Ed. 12, Section 4.1.3.8 (scal_unit_type)
    #[cfg(feature = "encode")]
    pub fn as_i8(&self) -> i8 {
        *self as u8 as i8
    }
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

/// ScalerUnit type - common COSEM attribute for registers
/// Encoded as: Structure(2) with Integer (scaler) and Enum (unit)
/// Example: scaler=-2, unit=30 (Wh) → 02 02 0F FE 16 1E
/// Reference: Green Book Ed. 12, Section 4.1.3.8 (scal_unit_type)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ScalerUnit {
    /// Power of 10 multiplier: actual_value = raw_value * 10^scaler
    /// Example: scaler=-2 means divide by 100 (e.g., 12345 → 123.45)
    pub scaler: i8,
    /// DLMS unit enum (1-255)
    pub unit: Unit,
}

impl Default for ScalerUnit {
    /// Returns the default ScalerUnit: scaler=0, unit=Count
    ///
    /// This represents a dimensionless value with no scaling (multiplier of 1).
    ///
    /// # Example
    /// ```
    /// use dlms_cosem::ScalerUnit;
    /// use dlms_cosem::Unit;
    ///
    /// let default_su = ScalerUnit::default();
    /// assert_eq!(default_su.scaler, 0);
    /// assert_eq!(default_su.unit, Unit::Count);
    /// ```
    fn default() -> Self {
        Self { scaler: 0, unit: Unit::Count }
    }
}

impl ScalerUnit {
    #[cfg(feature = "encode")]
    /// Encode as A-XDR Structure(2): [Integer(scaler), Enum(unit)]
    /// Reference: Green Book Ed. 12, Section 4.1.3.8
    pub fn encode(&self) -> Vec<u8> {
        let structure = Data::Structure(alloc::vec![
            Data::Integer(self.scaler),
            Data::Enum(self.unit.encode()),
        ]);
        structure.encode()
    }

    /// Parse ScalerUnit from A-XDR Structure(2)
    /// Expected format: Structure with 2 elements [Integer, Enum]
    #[cfg(feature = "parse")]
    pub fn parse(input: &[u8]) -> nom::IResult<&[u8], Self> {
        let (input, data) = Data::parse(input)?;

        match data {
            Data::Structure(elements) if elements.len() == 2 => {
                let scaler = match &elements[0] {
                    Data::Integer(s) => *s,
                    _ => {
                        return Err(nom::Err::Error(nom::error::Error::new(
                            input,
                            nom::error::ErrorKind::Tag,
                        )));
                    }
                };

                let unit_value = match &elements[1] {
                    Data::Enum(u) => *u,
                    _ => {
                        return Err(nom::Err::Error(nom::error::Error::new(
                            input,
                            nom::error::ErrorKind::Tag,
                        )));
                    }
                };

                let unit = Unit::try_from(unit_value).map_err(|_| {
                    nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify))
                })?;

                Ok((input, ScalerUnit { scaler, unit }))
            }
            _ => Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))),
        }
    }
}

#[cfg(all(test, feature = "encode", feature = "parse"))]
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
        let unit2 = unit1;
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

    // ========================================================================
    // Phase 1.4: Unit and Scaler Encoding Tests (TDD Approach)
    // ========================================================================

    #[test]
    #[cfg(feature = "encode")]
    fn test_unit_encode_common_units() {
        // Green Book: Common units used in energy meters
        assert_eq!(Unit::Year.encode(), 1);
        assert_eq!(Unit::Hour.encode(), 5);
        assert_eq!(Unit::Second.encode(), 7);
        assert_eq!(Unit::Watt.encode(), 27);
        assert_eq!(Unit::WattHour.encode(), 30);
        assert_eq!(Unit::Ampere.encode(), 33);
        assert_eq!(Unit::Volt.encode(), 35);
        assert_eq!(Unit::Hertz.encode(), 44);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_unit_encode_edge_values() {
        // Test boundary values
        assert_eq!(Unit::Year.encode(), 1); // First valid unit
        assert_eq!(Unit::Count.encode(), 255); // Last unit value
        assert_eq!(Unit::Other.encode(), 254); // Second-to-last
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_unit_encode_all_defined_units() {
        // Comprehensive test: ensure all defined units have correct values
        let units = [
            (Unit::Year, 1),
            (Unit::Month, 2),
            (Unit::Week, 3),
            (Unit::Day, 4),
            (Unit::Hour, 5),
            (Unit::Minute, 6),
            (Unit::Second, 7),
            (Unit::Degree, 8),
            (Unit::DegreeCelsius, 9),
            (Unit::Currency, 10),
            (Unit::Meter, 11),
            (Unit::MeterPerSecond, 12),
            (Unit::CubicMeter, 13),
            (Unit::CubicMeterCorrected, 14),
            (Unit::CubicMeterPerHour, 15),
            (Unit::CubicMeterPerHourCorrected, 16),
            (Unit::CubicMeterPerDay, 17),
            (Unit::CubicMeterPerDayCorrected, 18),
            (Unit::Liter, 19),
            (Unit::Kilogramm, 20),
            (Unit::Newton, 21),
            (Unit::Newtonmeter, 22),
            (Unit::Pascal, 23),
            (Unit::Bar, 24),
            (Unit::Joule, 25),
            (Unit::JoulePerHour, 26),
            (Unit::Watt, 27),
            (Unit::VoltAmpere, 28),
            (Unit::Var, 29),
            (Unit::WattHour, 30),
            (Unit::VoltAmpereHour, 31),
            (Unit::VarHour, 32),
            (Unit::Ampere, 33),
            (Unit::Coulomb, 34),
            (Unit::Volt, 35),
            (Unit::VoltPerMeter, 36),
            (Unit::Farad, 37),
            (Unit::Ohm, 38),
            (Unit::OhmMeter, 39),
            (Unit::Weber, 40),
            (Unit::Tesla, 41),
            (Unit::AmperePerMeter, 42),
            (Unit::Henry, 43),
            (Unit::Hertz, 44),
            (Unit::InverseWattHour, 45),
            (Unit::InverseVarHour, 46),
            (Unit::InverseVoltAmpereHour, 47),
            (Unit::VoltSquaredHour, 48),
            (Unit::AmpereSquaredHour, 49),
            (Unit::KilogrammPerSecond, 50),
            (Unit::Siemens, 51),
            (Unit::Kelvin, 52),
            (Unit::InverseVoltSquaredHour, 53),
            (Unit::InverseAmpereSquaredHour, 54),
            (Unit::InverseCubicMeter, 55),
            (Unit::Percent, 56),
            (Unit::AmpereHour, 57),
            (Unit::WattHourPerCubicMeter, 60),
            (Unit::JoulePerCubicMeter, 61),
            (Unit::MolePercent, 62),
            (Unit::GrammPerCubicMeter, 63),
            (Unit::PascalSecond, 64),
            (Unit::JoulePerKilogramm, 65),
            (Unit::GramPerSquareCentimeter, 66),
            (Unit::Atmosphere, 67),
            (Unit::DezibelMilliwatt, 70),
            (Unit::DezibelMicrovolt, 71),
            (Unit::Dezibel, 72),
            (Unit::Other, 254),
            (Unit::Count, 255),
        ];

        for (unit, expected) in units {
            assert_eq!(unit.encode(), expected, "Unit {:?} should encode to {}", unit, expected);
        }
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_unit_as_i8_positive_range() {
        // Values 1-127 remain positive
        assert_eq!(Unit::Year.as_i8(), 1);
        assert_eq!(Unit::WattHour.as_i8(), 30);
        assert_eq!(Unit::Ampere.as_i8(), 33);
        assert_eq!(Unit::Hertz.as_i8(), 44);
        assert_eq!(Unit::AmpereHour.as_i8(), 57);
        assert_eq!(Unit::Dezibel.as_i8(), 72);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_unit_as_i8_negative_range() {
        // Values 128-255 become negative i8 (-128 to -1)
        assert_eq!(Unit::Other.as_i8(), -2); // 254 as i8 = -2
        assert_eq!(Unit::Count.as_i8(), -1); // 255 as i8 = -1
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_scaler_unit_encode_basic() {
        // Green Book example: scaler=-2, unit=30 (Wh)
        // Expected: 02 02 0F FE 16 1E
        let su = ScalerUnit { scaler: -2, unit: Unit::WattHour };
        let encoded = su.encode();

        assert_eq!(encoded[0], 0x02); // Structure tag
        assert_eq!(encoded[1], 0x02); // 2 elements
        assert_eq!(encoded[2], 0x0F); // Integer tag
        assert_eq!(encoded[3], 0xFE); // -2 as i8
        assert_eq!(encoded[4], 0x16); // Enum tag
        assert_eq!(encoded[5], 0x1E); // 30 (WattHour)
        assert_eq!(encoded.len(), 6);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_scaler_unit_encode_positive_scaler() {
        // Positive scaler: scaler=3, unit=33 (Ampere)
        let su = ScalerUnit { scaler: 3, unit: Unit::Ampere };
        let encoded = su.encode();

        assert_eq!(encoded[0], 0x02); // Structure tag
        assert_eq!(encoded[1], 0x02); // 2 elements
        assert_eq!(encoded[2], 0x0F); // Integer tag
        assert_eq!(encoded[3], 0x03); // 3 as i8
        assert_eq!(encoded[4], 0x16); // Enum tag
        assert_eq!(encoded[5], 0x21); // 33 (Ampere)
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_scaler_unit_encode_zero_scaler() {
        // Zero scaler: scaler=0, unit=11 (Meter)
        let su = ScalerUnit { scaler: 0, unit: Unit::Meter };
        let encoded = su.encode();

        assert_eq!(encoded[0], 0x02); // Structure tag
        assert_eq!(encoded[1], 0x02); // 2 elements
        assert_eq!(encoded[2], 0x0F); // Integer tag
        assert_eq!(encoded[3], 0x00); // 0 as i8
        assert_eq!(encoded[4], 0x16); // Enum tag
        assert_eq!(encoded[5], 0x0B); // 11 (Meter)
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_scaler_unit_encode_extremes() {
        // Maximum positive scaler
        let su_max = ScalerUnit { scaler: 127, unit: Unit::Volt };
        let encoded_max = su_max.encode();
        assert_eq!(encoded_max[3], 0x7F); // 127 as i8

        // Minimum negative scaler
        let su_min = ScalerUnit { scaler: -128, unit: Unit::Watt };
        let encoded_min = su_min.encode();
        assert_eq!(encoded_min[3], 0x80); // -128 as i8
    }

    #[test]
    fn test_scaler_unit_parse_basic() {
        // Parse: 02 02 0F FE 16 1E (scaler=-2, unit=30/Wh)
        let input = [0x02, 0x02, 0x0F, 0xFE, 0x16, 0x1E];
        let (remaining, su) = ScalerUnit::parse(&input).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(su.scaler, -2);
        assert_eq!(su.unit, Unit::WattHour);
    }

    #[test]
    fn test_scaler_unit_parse_positive_scaler() {
        // Parse: 02 02 0F 03 16 21 (scaler=3, unit=33/Ampere)
        let input = [0x02, 0x02, 0x0F, 0x03, 0x16, 0x21];
        let (remaining, su) = ScalerUnit::parse(&input).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(su.scaler, 3);
        assert_eq!(su.unit, Unit::Ampere);
    }

    #[test]
    fn test_scaler_unit_parse_zero_scaler() {
        // Parse: 02 02 0F 00 16 0B (scaler=0, unit=11/Meter)
        let input = [0x02, 0x02, 0x0F, 0x00, 0x16, 0x0B];
        let (remaining, su) = ScalerUnit::parse(&input).unwrap();

        assert_eq!(remaining.len(), 0);
        assert_eq!(su.scaler, 0);
        assert_eq!(su.unit, Unit::Meter);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_scaler_unit_roundtrip() {
        // Test round-trip: encode → parse → verify
        let test_cases = [
            ScalerUnit { scaler: -2, unit: Unit::WattHour },
            ScalerUnit { scaler: 3, unit: Unit::Ampere },
            ScalerUnit { scaler: 0, unit: Unit::Meter },
            ScalerUnit { scaler: -127, unit: Unit::Volt },
            ScalerUnit { scaler: 127, unit: Unit::Watt },
            ScalerUnit { scaler: -3, unit: Unit::VoltAmpere },
            ScalerUnit { scaler: 1, unit: Unit::Hertz },
        ];

        for original in &test_cases {
            let encoded = original.encode();
            let (remaining, parsed) = ScalerUnit::parse(&encoded).unwrap();

            assert_eq!(remaining.len(), 0, "Should consume all bytes");
            assert_eq!(parsed, *original, "Round-trip failed for {:?}", original);
        }
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_scaler_unit_real_world_examples() {
        // Common real-world register configurations from DLMS meters
        let examples = [
            // Active energy (Wh) with scaler -2 (0.01 Wh precision)
            (ScalerUnit { scaler: -2, unit: Unit::WattHour }, "Active energy register"),
            // Reactive energy (varh) with scaler -3 (0.001 varh precision)
            (ScalerUnit { scaler: -3, unit: Unit::VarHour }, "Reactive energy register"),
            // Voltage (V) with scaler -1 (0.1 V precision)
            (ScalerUnit { scaler: -1, unit: Unit::Volt }, "Voltage register"),
            // Current (A) with scaler -2 (0.01 A precision)
            (ScalerUnit { scaler: -2, unit: Unit::Ampere }, "Current register"),
            // Power (W) with scaler 0 (1 W precision)
            (ScalerUnit { scaler: 0, unit: Unit::Watt }, "Power register"),
            // Frequency (Hz) with scaler -2 (0.01 Hz precision)
            (ScalerUnit { scaler: -2, unit: Unit::Hertz }, "Frequency register"),
        ];

        for (su, description) in &examples {
            let encoded = su.encode();
            let (remaining, parsed) = ScalerUnit::parse(&encoded).unwrap();

            assert_eq!(remaining.len(), 0, "{}: Should consume all bytes", description);
            assert_eq!(parsed, *su, "{}: Round-trip failed", description);
        }
    }

    #[test]
    fn test_scaler_unit_parse_invalid_structure() {
        // Not a structure (just an integer)
        let input = [0x0F, 0x01];
        assert!(ScalerUnit::parse(&input).is_err());

        // Structure with wrong element count (3 instead of 2)
        let input = [0x02, 0x03, 0x0F, 0x01, 0x16, 0x1E, 0x00];
        assert!(ScalerUnit::parse(&input).is_err());
    }

    #[test]
    fn test_scaler_unit_parse_invalid_element_types() {
        // First element is not Integer (using Enum instead)
        let input = [0x02, 0x02, 0x16, 0xFE, 0x16, 0x1E];
        assert!(ScalerUnit::parse(&input).is_err());

        // Second element is not Enum (using Integer instead)
        let input = [0x02, 0x02, 0x0F, 0xFE, 0x0F, 0x1E];
        assert!(ScalerUnit::parse(&input).is_err());
    }

    #[test]
    fn test_scaler_unit_parse_invalid_unit_value() {
        // Valid structure but invalid unit enum value (58 is reserved)
        let input = [0x02, 0x02, 0x0F, 0xFE, 0x16, 0x3A]; // 58 is reserved
        assert!(ScalerUnit::parse(&input).is_err());
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_scaler_unit_encode_all_common_units() {
        // Test encoding with all commonly used units
        let common_units = [
            Unit::WattHour,
            Unit::VarHour,
            Unit::VoltAmpereHour,
            Unit::Watt,
            Unit::Var,
            Unit::VoltAmpere,
            Unit::Volt,
            Unit::Ampere,
            Unit::Hertz,
            Unit::CubicMeter,
            Unit::Liter,
        ];

        for unit in &common_units {
            let su = ScalerUnit { scaler: -2, unit: *unit };
            let encoded = su.encode();

            // Verify structure format
            assert_eq!(encoded[0], 0x02, "Unit {:?}: Structure tag", unit);
            assert_eq!(encoded[1], 0x02, "Unit {:?}: Element count", unit);
            assert_eq!(encoded[2], 0x0F, "Unit {:?}: Integer tag", unit);
            assert_eq!(encoded[3], 0xFE, "Unit {:?}: Scaler value", unit);
            assert_eq!(encoded[4], 0x16, "Unit {:?}: Enum tag", unit);
            assert_eq!(encoded[5], unit.encode(), "Unit {:?}: Unit value", unit);
        }
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_scaler_unit_compatibility_with_data_structure() {
        // Verify ScalerUnit encoding is compatible with Data::Structure encoding
        let su = ScalerUnit { scaler: -2, unit: Unit::WattHour };

        let su_encoded = su.encode();

        // Create equivalent Data::Structure manually
        let data_structure = Data::Structure(alloc::vec![Data::Integer(-2), Data::Enum(30),]);
        let data_encoded = data_structure.encode();

        // Both should produce identical bytes
        assert_eq!(
            su_encoded, data_encoded,
            "ScalerUnit encoding should match Data::Structure encoding"
        );
    }

    #[test]
    fn test_scaler_unit_debug_format() {
        let su = ScalerUnit { scaler: -2, unit: Unit::WattHour };
        let debug_str = format!("{:?}", su);
        assert!(debug_str.contains("scaler"));
        assert!(debug_str.contains("-2"));
        assert!(debug_str.contains("WattHour"));
    }

    #[test]
    fn test_scaler_unit_equality() {
        let su1 = ScalerUnit { scaler: -2, unit: Unit::WattHour };
        let su2 = ScalerUnit { scaler: -2, unit: Unit::WattHour };
        let su3 = ScalerUnit { scaler: -3, unit: Unit::WattHour };
        let su4 = ScalerUnit { scaler: -2, unit: Unit::Ampere };

        assert_eq!(su1, su2);
        assert_ne!(su1, su3);
        assert_ne!(su1, su4);
    }

    #[test]
    fn test_scaler_unit_clone() {
        let su1 = ScalerUnit { scaler: -2, unit: Unit::WattHour };
        let su2 = su1;
        assert_eq!(su1, su2);
    }

    #[test]
    fn test_scaler_unit_copy() {
        let su1 = ScalerUnit { scaler: -2, unit: Unit::WattHour };
        let su2 = su1; // Copy, not move
        assert_eq!(su1, su2);
        assert_eq!(su1.scaler, -2); // Original still accessible (proves Copy)
    }
}
