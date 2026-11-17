use crate::dimension::Dimension;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct UnitDefinition {
    pub symbol: String,
    pub name: String,
    pub dimension: Dimension,
    pub scale: f64,
    pub offset: f64,
}

#[derive(Debug, Clone)]
pub struct UnitRegistry {
    units: HashMap<String, UnitDefinition>,
}

impl UnitRegistry {
    pub fn new() -> Self {
        let mut reg = Self {
            units: HashMap::new(),
        };
        reg.populate_standard_units();
        reg
    }

    fn add_unit(&mut self, symbol: &str, name: &str, dimension: Dimension, scale: f64, offset: f64) {
        let def = UnitDefinition {
            symbol: symbol.to_string(),
            name: name.to_string(),
            dimension,
            scale,
            offset,
        };
        self.units.insert(symbol.to_string(), def.clone());
        self.units.insert(name.to_string(), def);
    }

    fn populate_standard_units(&mut self) {
        // Length (L)
        self.add_unit("m", "meter", Dimension::LENGTH, 1.0, 0.0);
        self.add_unit("km", "kilometer", Dimension::LENGTH, 1000.0, 0.0);
        self.add_unit("cm", "centimeter", Dimension::LENGTH, 0.01, 0.0);
        self.add_unit("mm", "millimeter", Dimension::LENGTH, 0.001, 0.0);
        self.add_unit("um", "micrometer", Dimension::LENGTH, 1e-6, 0.0);
        self.add_unit("nm", "nanometer", Dimension::LENGTH, 1e-9, 0.0);
        self.add_unit("mi", "mile", Dimension::LENGTH, 1609.344, 0.0);
        self.add_unit("yd", "yard", Dimension::LENGTH, 0.9144, 0.0);
        self.add_unit("ft", "foot", Dimension::LENGTH, 0.3048, 0.0);
        self.add_unit("in", "inch", Dimension::LENGTH, 0.0254, 0.0);
        self.add_unit("ly", "lightyear", Dimension::LENGTH, 9.4607304725808e15, 0.0);
        self.add_unit("au", "astronomical_unit", Dimension::LENGTH, 1.495978707e11, 0.0);

        // Mass (M)
        self.add_unit("kg", "kilogram", Dimension::MASS, 1.0, 0.0);
        self.add_unit("g", "gram", Dimension::MASS, 0.001, 0.0);
        self.add_unit("mg", "milligram", Dimension::MASS, 1e-6, 0.0);
        self.add_unit("ug", "microgram", Dimension::MASS, 1e-9, 0.0);
        self.add_unit("lb", "pound", Dimension::MASS, 0.45359237, 0.0);
        self.add_unit("oz", "ounce", Dimension::MASS, 0.028349523125, 0.0);
        self.add_unit("ton", "tonne", Dimension::MASS, 1000.0, 0.0);

        // Time (T)
        self.add_unit("s", "second", Dimension::TIME, 1.0, 0.0);
        self.add_unit("sec", "seconds", Dimension::TIME, 1.0, 0.0);
        self.add_unit("ms", "millisecond", Dimension::TIME, 0.001, 0.0);
        self.add_unit("us", "microsecond", Dimension::TIME, 1e-6, 0.0);
        self.add_unit("ns", "nanosecond", Dimension::TIME, 1e-9, 0.0);
        self.add_unit("min", "minute", Dimension::TIME, 60.0, 0.0);
        self.add_unit("h", "hour", Dimension::TIME, 3600.0, 0.0);
        self.add_unit("hr", "hours", Dimension::TIME, 3600.0, 0.0);
        self.add_unit("day", "days", Dimension::TIME, 86400.0, 0.0);
        self.add_unit("week", "weeks", Dimension::TIME, 604800.0, 0.0);
        self.add_unit("yr", "year", Dimension::TIME, 31557600.0, 0.0);

        // Current (I)
        self.add_unit("A", "ampere", Dimension::CURRENT, 1.0, 0.0);
        self.add_unit("mA", "milliampere", Dimension::CURRENT, 0.001, 0.0);
        self.add_unit("uA", "microampere", Dimension::CURRENT, 1e-6, 0.0);

        // Temperature (Θ)
        self.add_unit("K", "kelvin", Dimension::TEMPERATURE, 1.0, 0.0);
        self.add_unit("degC", "celsius", Dimension::TEMPERATURE, 1.0, 273.15);
        self.add_unit("degF", "fahrenheit", Dimension::TEMPERATURE, 5.0 / 9.0, 459.67);

        // Frequency
        self.add_unit("Hz", "hertz", Dimension::FREQUENCY, 1.0, 0.0);
        self.add_unit("kHz", "kilohertz", Dimension::FREQUENCY, 1e3, 0.0);
        self.add_unit("MHz", "megahertz", Dimension::FREQUENCY, 1e6, 0.0);
        self.add_unit("GHz", "gigahertz", Dimension::FREQUENCY, 1e9, 0.0);
        self.add_unit("rpm", "rev_per_min", Dimension::FREQUENCY, 1.0 / 60.0, 0.0);

        // Velocity
        self.add_unit("kph", "kmh", Dimension::VELOCITY, 1000.0 / 3600.0, 0.0);
        self.add_unit("mph", "miles_per_hour", Dimension::VELOCITY, 1609.344 / 3600.0, 0.0);
        self.add_unit("knot", "knots", Dimension::VELOCITY, 1852.0 / 3600.0, 0.0);

        // Force
        self.add_unit("N", "newton", Dimension::FORCE, 1.0, 0.0);
        self.add_unit("kN", "kilonewton", Dimension::FORCE, 1000.0, 0.0);
        self.add_unit("lbf", "pound_force", Dimension::FORCE, 4.4482216152605, 0.0);

        // Energy
        self.add_unit("J", "joule", Dimension::ENERGY, 1.0, 0.0);
        self.add_unit("kJ", "kilojoule", Dimension::ENERGY, 1000.0, 0.0);
        self.add_unit("MJ", "megajoule", Dimension::ENERGY, 1e6, 0.0);
        self.add_unit("cal", "calorie", Dimension::ENERGY, 4.184, 0.0);
        self.add_unit("kcal", "kilocalorie", Dimension::ENERGY, 4184.0, 0.0);
        self.add_unit("Wh", "watt_hour", Dimension::ENERGY, 3600.0, 0.0);
        self.add_unit("kWh", "kilowatt_hour", Dimension::ENERGY, 3.6e6, 0.0);
        self.add_unit("eV", "electronvolt", Dimension::ENERGY, 1.602176634e-19, 0.0);

        // Power
        self.add_unit("W", "watt", Dimension::POWER, 1.0, 0.0);
        self.add_unit("kW", "kilowatt", Dimension::POWER, 1000.0, 0.0);
        self.add_unit("MW", "megawatt", Dimension::POWER, 1e6, 0.0);
        self.add_unit("hp", "horsepower", Dimension::POWER, 745.69987158227022, 0.0);

        // Pressure
        self.add_unit("Pa", "pascal", Dimension::PRESSURE, 1.0, 0.0);
        self.add_unit("kPa", "kilopascal", Dimension::PRESSURE, 1000.0, 0.0);
        self.add_unit("MPa", "megapascal", Dimension::PRESSURE, 1e6, 0.0);
        self.add_unit("bar", "bars", Dimension::PRESSURE, 100000.0, 0.0);
        self.add_unit("mbar", "millibar", Dimension::PRESSURE, 100.0, 0.0);
        self.add_unit("atm", "atmosphere", Dimension::PRESSURE, 101325.0, 0.0);
        self.add_unit("psi", "pounds_per_sq_in", Dimension::PRESSURE, 6894.757293168, 0.0);
        self.add_unit("torr", "mmHg", Dimension::PRESSURE, 133.322368421, 0.0);

        // Electromagnetism
        self.add_unit("V", "volt", Dimension::VOLTAGE, 1.0, 0.0);
        self.add_unit("mV", "millivolt", Dimension::VOLTAGE, 0.001, 0.0);
        self.add_unit("kV", "kilovolt", Dimension::VOLTAGE, 1000.0, 0.0);
        self.add_unit("ohm", "ohms", Dimension::RESISTANCE, 1.0, 0.0);
        self.add_unit("kohm", "kiloohm", Dimension::RESISTANCE, 1000.0, 0.0);
        self.add_unit("Mohm", "megaohm", Dimension::RESISTANCE, 1e6, 0.0);
        self.add_unit("C", "coulomb", Dimension::CHARGE, 1.0, 0.0);

        // Digital Storage (D)
        self.add_unit("bit", "bits", Dimension::DIGITAL, 1.0, 0.0);
        self.add_unit("kbit", "kilobit", Dimension::DIGITAL, 1e3, 0.0);
        self.add_unit("Mbit", "megabit", Dimension::DIGITAL, 1e6, 0.0);
        self.add_unit("Gbit", "gigabit", Dimension::DIGITAL, 1e9, 0.0);
        self.add_unit("Tbit", "terabit", Dimension::DIGITAL, 1e12, 0.0);
        self.add_unit("kilobits", "kilobit", Dimension::DIGITAL, 1e3, 0.0);
        self.add_unit("megabits", "megabit", Dimension::DIGITAL, 1e6, 0.0);
        self.add_unit("gigabits", "gigabit", Dimension::DIGITAL, 1e9, 0.0);
        self.add_unit("terabits", "terabit", Dimension::DIGITAL, 1e12, 0.0);
        self.add_unit("kibibit", "kibibit", Dimension::DIGITAL, 1024.0, 0.0);
        self.add_unit("mebibit", "mebibit", Dimension::DIGITAL, 1024.0 * 1024.0, 0.0);
        self.add_unit("gibibit", "gibibit", Dimension::DIGITAL, 1024.0 * 1024.0 * 1024.0, 0.0);

        self.add_unit("B", "byte", Dimension::DIGITAL, 8.0, 0.0);
        self.add_unit("bytes", "Bytes", Dimension::DIGITAL, 8.0, 0.0);
        self.add_unit("kB", "kilobyte", Dimension::DIGITAL, 8.0 * 1e3, 0.0);
        self.add_unit("MB", "megabyte", Dimension::DIGITAL, 8.0 * 1e6, 0.0);
        self.add_unit("GB", "gigabyte", Dimension::DIGITAL, 8.0 * 1e9, 0.0);
        self.add_unit("TB", "terabyte", Dimension::DIGITAL, 8.0 * 1e12, 0.0);
        self.add_unit("kilobytes", "kilobyte", Dimension::DIGITAL, 8.0 * 1e3, 0.0);
        self.add_unit("megabytes", "megabyte", Dimension::DIGITAL, 8.0 * 1e6, 0.0);
        self.add_unit("gigabytes", "gigabyte", Dimension::DIGITAL, 8.0 * 1e9, 0.0);
        self.add_unit("terabytes", "terabyte", Dimension::DIGITAL, 8.0 * 1e12, 0.0);
        self.add_unit("KiB", "kibibyte", Dimension::DIGITAL, 8.0 * 1024.0, 0.0);
        self.add_unit("MiB", "mebibyte", Dimension::DIGITAL, 8.0 * 1024.0 * 1024.0, 0.0);
        self.add_unit("GiB", "gibibyte", Dimension::DIGITAL, 8.0 * 1024.0 * 1024.0 * 1024.0, 0.0);
        self.add_unit("TiB", "tebibyte", Dimension::DIGITAL, 8.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0, 0.0);
        self.add_unit("kibibytes", "kibibyte", Dimension::DIGITAL, 8.0 * 1024.0, 0.0);
        self.add_unit("mebibytes", "mebibyte", Dimension::DIGITAL, 8.0 * 1024.0 * 1024.0, 0.0);
        self.add_unit("gibibytes", "gibibyte", Dimension::DIGITAL, 8.0 * 1024.0 * 1024.0 * 1024.0, 0.0);
        self.add_unit("tebibytes", "tebibyte", Dimension::DIGITAL, 8.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0, 0.0);

        // Data Rates (D / T)
        self.add_unit("bps", "bits_per_second", Dimension::DATA_RATE, 1.0, 0.0);
        self.add_unit("kbps", "kilobits_per_second", Dimension::DATA_RATE, 1e3, 0.0);
        self.add_unit("Mbps", "megabits_per_second", Dimension::DATA_RATE, 1e6, 0.0);
        self.add_unit("Gbps", "gigabits_per_second", Dimension::DATA_RATE, 1e9, 0.0);
        self.add_unit("Bps", "bytes_per_second", Dimension::DATA_RATE, 8.0, 0.0);
        self.add_unit("MBps", "megabytes_per_second", Dimension::DATA_RATE, 8.0 * 1e6, 0.0);

        // Currencies (C) - baseline 2026 exchange rates to USD
        self.add_unit("USD", "dollar", Dimension::CURRENCY, 1.0, 0.0);
        self.add_unit("EUR", "euro", Dimension::CURRENCY, 1.038, 0.0);
        self.add_unit("GBP", "pound_sterling", Dimension::CURRENCY, 1.250, 0.0);
        self.add_unit("JPY", "yen", Dimension::CURRENCY, 1.0 / 155.0, 0.0);
        self.add_unit("INR", "rupee", Dimension::CURRENCY, 1.0 / 85.5, 0.0);
        self.add_unit("CAD", "canadian_dollar", Dimension::CURRENCY, 0.710, 0.0);
        self.add_unit("AUD", "australian_dollar", Dimension::CURRENCY, 0.650, 0.0);
        self.add_unit("CHF", "swiss_franc", Dimension::CURRENCY, 1.120, 0.0);
        self.add_unit("CNY", "chinese_yuan", Dimension::CURRENCY, 1.0 / 7.25, 0.0);
    }

    pub fn lookup(&self, name: &str) -> Option<UnitDefinition> {
        let clean = name.trim();
        if let Some(unit) = self.units.get(clean) {
            return Some(unit.clone());
        }

        // Try standard metric prefixes on base units
        let prefixes: &[(&str, f64)] = &[
            ("k", 1e3),
            ("M", 1e6),
            ("G", 1e9),
            ("T", 1e12),
            ("m", 1e-3),
            ("u", 1e-6),
            ("n", 1e-9),
            ("p", 1e-12),
            ("c", 1e-2),
            ("d", 1e-1),
        ];

        for (pfx, mult) in prefixes {
            if clean.starts_with(pfx) && clean.len() > pfx.len() {
                let remainder = &clean[pfx.len()..];
                if let Some(base_def) = self.units.get(remainder) {
                    if base_def.offset == 0.0 {
                        return Some(UnitDefinition {
                            symbol: clean.to_string(),
                            name: format!("{}{}", pfx, base_def.name),
                            dimension: base_def.dimension,
                            scale: base_def.scale * mult,
                            offset: 0.0,
                        });
                    }
                }
            }
        }

        None
    }

    /// Converts a value from one unit to another.
    /// Checks dimensional consistency strictly (Numbat principle).
    pub fn convert(&self, value: f64, from_unit_name: &str, to_unit_name: &str) -> Result<f64, String> {
        let from = self.lookup(from_unit_name)
            .ok_or_else(|| format!("Unknown unit: '{}'", from_unit_name))?;
        let to = self.lookup(to_unit_name)
            .ok_or_else(|| format!("Unknown unit: '{}'", to_unit_name))?;

        if from.dimension != to.dimension {
            return Err(format!(
                "Dimensional mismatch: cannot convert '{}' ({}) to '{}' ({})",
                from_unit_name, from.dimension, to_unit_name, to.dimension
            ));
        }

        // Base SI value = (val + from.offset) * from.scale
        let base_value = (value + from.offset) * from.scale;
        // Target value = (base_value / to.scale) - to.offset
        let target_value = (base_value / to.scale) - to.offset;
        Ok(target_value)
    }
}
