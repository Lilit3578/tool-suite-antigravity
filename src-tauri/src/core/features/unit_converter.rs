use crate::shared::types::{ActionType, CommandItem, ConvertUnitsRequest, ConvertUnitsResponse, ExecuteActionResponse};

// Error constants - inline for now (can be moved to shared::errors later)
const ERR_MISSING_TEXT_PARAM: &str = "Missing 'text' parameter";
const ERR_NEGATIVE_LENGTH: &str = "Length cannot be negative. Please provide a positive value.";
const ERR_NEGATIVE_MASS: &str = "Mass cannot be negative. Please provide a positive value.";
const ERR_NEGATIVE_VOLUME: &str = "Volume cannot be negative. Please provide a positive value.";
const ERR_UNSUPPORTED_ACTION: &str = "Unsupported action type";
const ERR_CANNOT_PARSE_UNIT: &str = "Could not parse unit from text";
use serde_json::json;
use super::Feature;
use regex::Regex;
use once_cell::sync::Lazy;

pub struct UnitConverter;

impl Feature for UnitConverter {
    fn id(&self) -> &'static str {
        "unit_converter"
    }
    
    fn widget_commands(&self) -> Vec<CommandItem> {
        vec![CommandItem {
            id: "widget_unit_converter".to_string(),
            label: "Unit Converter".to_string(),
            description: Some("Open unit converter widget".to_string()),
            action_type: None,
            widget_type: Some("unit_converter".to_string()),
            category: None, // Will be assigned in get_all_command_items()
        }]
    }
    
    fn action_commands(&self) -> Vec<CommandItem> {
        let conversions = vec![
            // Length
            ("convert_to_mm", "Convert to Millimeters", ActionType::ConvertToMM),
            ("convert_to_cm", "Convert to Centimeters", ActionType::ConvertToCM),
            ("convert_to_m", "Convert to Meters", ActionType::ConvertToM),
            ("convert_to_km", "Convert to Kilometers", ActionType::ConvertToKM),
            ("convert_to_in", "Convert to Inches", ActionType::ConvertToIN),
            ("convert_to_ft", "Convert to Feet", ActionType::ConvertToFT),
            ("convert_to_yd", "Convert to Yards", ActionType::ConvertToYD),
            ("convert_to_mi", "Convert to Miles", ActionType::ConvertToMI),
            // Mass
            ("convert_to_mg", "Convert to Milligrams", ActionType::ConvertToMG),
            ("convert_to_g", "Convert to Grams", ActionType::ConvertToG),
            ("convert_to_kg", "Convert to Kilograms", ActionType::ConvertToKG),
            ("convert_to_oz", "Convert to Ounces", ActionType::ConvertToOZ),
            ("convert_to_lb", "Convert to Pounds", ActionType::ConvertToLB),
            // Volume
            ("convert_to_ml", "Convert to Milliliters", ActionType::ConvertToML),
            ("convert_to_l", "Convert to Liters", ActionType::ConvertToL),
            ("convert_to_fl_oz", "Convert to Fluid Ounces", ActionType::ConvertToFlOz),
            ("convert_to_cup", "Convert to Cups", ActionType::ConvertToCup),
            ("convert_to_pint", "Convert to Pints", ActionType::ConvertToPint),
            ("convert_to_quart", "Convert to Quarts", ActionType::ConvertToQuart),
            ("convert_to_gal", "Convert to Gallons", ActionType::ConvertToGal),
            // Temperature
            ("convert_to_c", "Convert to Celsius", ActionType::ConvertToC),
            ("convert_to_f", "Convert to Fahrenheit", ActionType::ConvertToF),
            // Speed
            ("convert_to_kmh", "Convert to Kilometers/Hour", ActionType::ConvertToKMH),
            ("convert_to_mph", "Convert to Miles/Hour", ActionType::ConvertToMPH),
            // Note: Cross-category actions (ConvertVolTo*, ConvertMassTo*) are deprecated
            // Generic actions (ConvertTo*) now handle cross-category conversions automatically
        ];
        
        conversions
            .into_iter()
            .map(|(id, label, action_type)| CommandItem {
                id: id.to_string(),
                label: label.to_string(),
                description: None,
                action_type: Some(action_type),
                widget_type: None,
                category: None, // Will be assigned in get_all_command_items()
            })
            .collect()
    }

    fn execute_action(
        &self,
        action_type: &ActionType,
        params: &serde_json::Value,
    ) -> Result<ExecuteActionResponse, String> {
        // Extract text from params
        let text = params.get("text")
            .and_then(|v| v.as_str())
            .ok_or(ERR_MISSING_TEXT_PARAM)?;

        // Parse amount and source unit from text
        let (amount, source_unit) = parse_unit_from_text(text)?;
        
        // Validate negative numbers for physical quantities
        match action_type {
            ActionType::ConvertToMM | ActionType::ConvertToCM | ActionType::ConvertToM 
            | ActionType::ConvertToKM | ActionType::ConvertToIN | ActionType::ConvertToFT 
            | ActionType::ConvertToYD | ActionType::ConvertToMI => {
                // Length cannot be negative
                if amount < 0.0 {
                    return Err(ERR_NEGATIVE_LENGTH.to_string());
                }
            }
            ActionType::ConvertToMG | ActionType::ConvertToG | ActionType::ConvertToKG 
            | ActionType::ConvertToOZ | ActionType::ConvertToLB => {
                // Mass cannot be negative
                if amount < 0.0 {
                    return Err(ERR_NEGATIVE_MASS.to_string());
                }
            }
            ActionType::ConvertToML | ActionType::ConvertToL | ActionType::ConvertToFlOz 
            | ActionType::ConvertToCup | ActionType::ConvertToPint | ActionType::ConvertToQuart 
            | ActionType::ConvertToGal => {
                // Volume cannot be negative
                if amount < 0.0 {
                    return Err(ERR_NEGATIVE_VOLUME.to_string());
                }
            }
            // Temperature and Speed can be negative (valid use cases)
            _ => {}
        }

        // Determine target unit and handle cross-category conversions
        let (target_unit, converted_value) = match action_type {
            // Length conversions
            ActionType::ConvertToMM => ("mm", convert_unit(amount, &source_unit, "mm")?),
            ActionType::ConvertToCM => ("cm", convert_unit(amount, &source_unit, "cm")?),
            ActionType::ConvertToM => ("m", convert_unit(amount, &source_unit, "m")?),
            ActionType::ConvertToKM => ("km", convert_unit(amount, &source_unit, "km")?),
            ActionType::ConvertToIN => ("in", convert_unit(amount, &source_unit, "in")?),
            ActionType::ConvertToFT => ("ft", convert_unit(amount, &source_unit, "ft")?),
            ActionType::ConvertToYD => ("yd", convert_unit(amount, &source_unit, "yd")?),
            ActionType::ConvertToMI => ("mi", convert_unit(amount, &source_unit, "mi")?),
            
            // Mass conversions (smart: handles Volume → Mass cross-category)
            ActionType::ConvertToMG => {
                if is_volume_unit(&source_unit) {
                    // Cross-category: Volume → Mass
                    let (val, _) = convert_volume_to_mass(amount, &source_unit, "mg", params)?;
                    ("mg", val)
                } else {
                    ("mg", convert_unit(amount, &source_unit, "mg")?)
                }
            },
            ActionType::ConvertToG => {
                if is_volume_unit(&source_unit) {
                    let (val, _) = convert_volume_to_mass(amount, &source_unit, "g", params)?;
                    ("g", val)
                } else {
                    ("g", convert_unit(amount, &source_unit, "g")?)
                }
            },
            ActionType::ConvertToKG => {
                if is_volume_unit(&source_unit) {
                    let (val, _) = convert_volume_to_mass(amount, &source_unit, "kg", params)?;
                    ("kg", val)
                } else {
                    ("kg", convert_unit(amount, &source_unit, "kg")?)
                }
            },
            ActionType::ConvertToOZ => {
                if is_volume_unit(&source_unit) {
                    let (val, _) = convert_volume_to_mass(amount, &source_unit, "oz", params)?;
                    ("oz", val)
                } else {
                    ("oz", convert_unit(amount, &source_unit, "oz")?)
                }
            },
            ActionType::ConvertToLB => {
                if is_volume_unit(&source_unit) {
                    let (val, _) = convert_volume_to_mass(amount, &source_unit, "lb", params)?;
                    ("lb", val)
                } else {
                    ("lb", convert_unit(amount, &source_unit, "lb")?)
                }
            },
            
            // Volume conversions (smart: handles Mass → Volume cross-category)
            ActionType::ConvertToML => {
                if is_mass_unit(&source_unit) {
                    // Cross-category: Mass → Volume
                    let (val, _) = convert_mass_to_volume(amount, &source_unit, "ml", params)?;
                    ("ml", val)
                } else {
                    ("ml", convert_unit(amount, &source_unit, "ml")?)
                }
            },
            ActionType::ConvertToL => {
                if is_mass_unit(&source_unit) {
                    let (val, _) = convert_mass_to_volume(amount, &source_unit, "L", params)?;
                    ("L", val)
                } else {
                    ("L", convert_unit(amount, &source_unit, "L")?)
                }
            },
            ActionType::ConvertToFlOz => {
                if is_mass_unit(&source_unit) {
                    let (val, _) = convert_mass_to_volume(amount, &source_unit, "fl-oz", params)?;
                    ("fl-oz", val)
                } else {
                    ("fl-oz", convert_unit(amount, &source_unit, "fl-oz")?)
                }
            },
            ActionType::ConvertToCup => {
                if is_mass_unit(&source_unit) {
                    let (val, _) = convert_mass_to_volume(amount, &source_unit, "cup", params)?;
                    ("cup", val)
                } else {
                    ("cup", convert_unit(amount, &source_unit, "cup")?)
                }
            },
            ActionType::ConvertToPint => {
                if is_mass_unit(&source_unit) {
                    let (val, _) = convert_mass_to_volume(amount, &source_unit, "pint", params)?;
                    ("pint", val)
                } else {
                    ("pint", convert_unit(amount, &source_unit, "pint")?)
                }
            },
            ActionType::ConvertToQuart => {
                if is_mass_unit(&source_unit) {
                    let (val, _) = convert_mass_to_volume(amount, &source_unit, "quart", params)?;
                    ("quart", val)
                } else {
                    ("quart", convert_unit(amount, &source_unit, "quart")?)
                }
            },
            ActionType::ConvertToGal => {
                if is_mass_unit(&source_unit) {
                    let (val, _) = convert_mass_to_volume(amount, &source_unit, "gal", params)?;
                    ("gal", val)
                } else {
                    ("gal", convert_unit(amount, &source_unit, "gal")?)
                }
            },
            
            // Temperature conversions
            ActionType::ConvertToC => ("C", convert_unit(amount, &source_unit, "C")?),
            ActionType::ConvertToF => ("F", convert_unit(amount, &source_unit, "F")?),
            
            // Speed conversions
            ActionType::ConvertToKMH => ("km/h", convert_unit(amount, &source_unit, "km/h")?),
            ActionType::ConvertToMPH => ("m/h", convert_unit(amount, &source_unit, "m/h")?),
            
            // Legacy cross-category actions (deprecated, but still supported for backward compatibility)
            ActionType::ConvertVolToG | ActionType::ConvertVolToKG | ActionType::ConvertVolToOZ | ActionType::ConvertVolToLB => {
                let target_unit = match action_type {
                    ActionType::ConvertVolToG => "g",
                    ActionType::ConvertVolToKG => "kg",
                    ActionType::ConvertVolToOZ => "oz",
                    ActionType::ConvertVolToLB => "lb",
                    _ => unreachable!(),
                };
                let (val, _) = convert_volume_to_mass(amount, &source_unit, target_unit, params)?;
                (target_unit, val)
            },
            ActionType::ConvertMassToML | ActionType::ConvertMassToL | ActionType::ConvertMassToFlOz | ActionType::ConvertMassToCup | ActionType::ConvertMassToPint | ActionType::ConvertMassToQuart | ActionType::ConvertMassToGal => {
                let target_unit = match action_type {
                    ActionType::ConvertMassToML => "ml",
                    ActionType::ConvertMassToL => "L",
                    ActionType::ConvertMassToFlOz => "fl-oz",
                    ActionType::ConvertMassToCup => "cup",
                    ActionType::ConvertMassToPint => "pint",
                    ActionType::ConvertMassToQuart => "quart",
                    ActionType::ConvertMassToGal => "gal",
                    _ => unreachable!(),
                };
                let (val, _) = convert_mass_to_volume(amount, &source_unit, target_unit, params)?;
                (target_unit, val)
            },
            
            _ => return Err(ERR_UNSUPPORTED_ACTION.to_string()),
        };

        // Format result with beautiful number formatting
        let formatted_value = format_number(converted_value);
        let result = format!("{} {}", formatted_value, target_unit);

        Ok(ExecuteActionResponse {
            result,
            metadata: Some(json!({
                "from_unit": source_unit,
                "target_unit": target_unit,
                "original_amount": amount,
                "converted_amount": converted_value,
                "widget": "unit_converter"
            })),
        })
    }
}

// Tauri commands
#[tauri::command]
pub async fn convert_units(request: ConvertUnitsRequest) -> Result<ConvertUnitsResponse, String> {
    // Note: The actual conversion logic will be handled on the frontend using convert-units library
    // This command is here for potential future server-side conversions or validation
    Ok(ConvertUnitsResponse {
        result: 0.0, // Frontend will handle the actual conversion
        from_unit: request.from_unit,
        to_unit: request.to_unit,
    })
}

#[tauri::command]
pub async fn get_units_for_category(category: String) -> Result<Vec<String>, String> {
    let units = match category.as_str() {
        "length" => vec!["mm", "cm", "m", "km", "in", "ft", "yd", "mi"],
        "mass" => vec!["mg", "g", "kg", "oz", "lb"],
        "volume" => vec!["ml", "cl", "dl", "L", "kl", "m3", "km3", "tsp", "Tbs", "in3", "fl-oz", "cup", "gal", "ft3", "yd3"],
        "temperature" => vec!["C", "F"],
        "speed" => vec!["km/h", "m/h", "ft/s"],
        _ => return Err(format!("Unknown category: {}", category)),
    };
    
    Ok(units.iter().map(|s| s.to_string()).collect())
}

#[tauri::command]
pub async fn get_unit_settings() -> Result<serde_json::Value, String> {
    Ok(json!({
        "default_from_unit": "m",
        "default_to_unit": "ft",
        "default_category": "length"
    }))
}

// Unit aliases mapping (similar to frontend)
fn normalize_unit(unit: &str) -> Option<&'static str> {
    let unit_lower = unit.to_lowercase();
    match unit_lower.as_str() {
        // Length
        "mm" | "millimeter" | "millimeters" | "millimetre" | "millimetres" => Some("mm"),
        "cm" | "centimeter" | "centimeters" | "centimetre" | "centimetres" => Some("cm"),
        "m" | "meter" | "meters" | "metre" | "metres" => Some("m"),
        "km" | "kilometer" | "kilometers" | "kilometre" | "kilometres" => Some("km"),
        "in" | "inch" | "inches" | "\"" => Some("in"),
        "ft" | "foot" | "feet" | "'" => Some("ft"),
        "yd" | "yard" | "yards" => Some("yd"),
        "mi" | "mile" | "miles" => Some("mi"),
        // Mass
        "mg" | "milligram" | "milligrams" => Some("mg"),
        "g" | "gram" | "grams" => Some("g"),
        "kg" | "kilogram" | "kilograms" => Some("kg"),
        "oz" | "ounce" | "ounces" => Some("oz"),
        "lb" | "lbs" | "pound" | "pounds" => Some("lb"),
        // Volume
        "ml" | "milliliter" | "milliliters" | "millilitre" | "millilitres" => Some("ml"),
        "l" | "L" | "liter" | "liters" | "litre" | "litres" => Some("L"),
        "fl-oz" | "floz" | "fluid ounce" | "fluid ounces" => Some("fl-oz"),
        "cup" | "cups" => Some("cup"),
        "gal" | "gallon" | "gallons" => Some("gal"),
        // Temperature
        "c" | "C" | "celsius" | "°c" | "°C" => Some("C"),
        "f" | "F" | "fahrenheit" | "°f" | "°F" => Some("F"),
        // Speed
        "km/h" | "kmh" | "kph" | "kilometers/hour" | "kilometers per hour" => Some("km/h"),
        "m/h" | "mph" | "miles/hour" | "miles per hour" => Some("m/h"),
        _ => None,
    }
}

// Compile regex patterns once at module level (compile-time constants)
// Using expect is safe here since these are compile-time constant patterns
static RE_PATTERN_1: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([+-]?\d+(?:\.\d+)?)\s*([a-zA-Z°'/″³]+(?:\/[a-zA-Z]+)?)$")
        .expect("Failed to compile regex pattern 1")
});

static RE_PATTERN_2: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([a-zA-Z°'/″³]+(?:\/[a-zA-Z]+)?)\s*([+-]?\d+(?:\.\d+)?)$")
        .expect("Failed to compile regex pattern 2")
});

static RE_PATTERN_3: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"([+-]?\d+(?:\.\d+)?)")
        .expect("Failed to compile regex pattern 3")
});

// Parse amount and unit from text (e.g., "100m", "12 km", "3.5 meters")
fn parse_unit_from_text(text: &str) -> Result<(f64, String), String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("Empty text".to_string());
    }

    // Normalize comma decimal separators to dots
    let normalized_text = text.replace(',', ".");

    // Pattern 1: Number followed by unit (e.g., "12km", "12 km", "12 kilometers")
    if let Some(caps) = RE_PATTERN_1.captures(&normalized_text) {
        if let (Ok(amount), Some(unit_str)) = (caps[1].parse::<f64>(), caps.get(2)) {
            if let Some(canonical_unit) = normalize_unit(unit_str.as_str()) {
                return Ok((amount, canonical_unit.to_string()));
            }
        }
    }

    // Pattern 2: Unit followed by number (e.g., "km12", "m 100")
    if let Some(caps) = RE_PATTERN_2.captures(&normalized_text) {
        if let (Some(unit_str), Ok(amount)) = (caps.get(1), caps[2].parse::<f64>()) {
            if let Some(canonical_unit) = normalize_unit(unit_str.as_str()) {
                return Ok((amount, canonical_unit.to_string()));
            }
        }
    }

    // Pattern 3: Try to extract any number and any known unit from the text
    if let Some(caps) = RE_PATTERN_3.captures(&normalized_text) {
        if let Ok(amount) = caps[1].parse::<f64>() {
            let text_lower = normalized_text.to_lowercase();
            // Check common unit patterns
            for (alias, canonical) in [
                ("kilometers", "km"), ("kilometer", "km"), ("kilometres", "km"), ("kilometre", "km"),
                ("meters", "m"), ("meter", "m"), ("metres", "m"), ("metre", "m"),
                ("centimeters", "cm"), ("centimeter", "cm"), ("centimetres", "cm"), ("centimetre", "cm"),
                ("millimeters", "mm"), ("millimeter", "mm"), ("millimetres", "mm"), ("millimetre", "mm"),
                ("inches", "in"), ("inch", "in"),
                ("feet", "ft"), ("foot", "ft"),
                ("yards", "yd"), ("yard", "yd"),
                ("miles", "mi"), ("mile", "mi"),
                ("grams", "g"), ("gram", "g"),
                ("kilograms", "kg"), ("kilogram", "kg"),
                ("milligrams", "mg"), ("milligram", "mg"),
                ("ounces", "oz"), ("ounce", "oz"),
                ("pounds", "lb"), ("pound", "lb"),
                ("liters", "L"), ("liter", "L"), ("litres", "L"), ("litre", "L"),
                ("milliliters", "ml"), ("milliliter", "ml"), ("millilitres", "ml"), ("millilitre", "ml"),
                ("gallons", "gal"), ("gallon", "gal"),
                ("cups", "cup"), ("cup", "cup"),
                ("fluid ounces", "fl-oz"), ("fluid ounce", "fl-oz"),
                ("celsius", "C"), ("fahrenheit", "F"),
                ("kilometers per hour", "km/h"), ("miles per hour", "m/h"),
            ] {
                if text_lower.contains(alias) {
                    return Ok((amount, canonical.to_string()));
                }
            }
        }
    }

    Err(format!("{}: {}", ERR_CANNOT_PARSE_UNIT, text))
}

// Convert between units
fn convert_unit(amount: f64, from_unit: &str, to_unit: &str) -> Result<f64, String> {
    if from_unit == to_unit {
        return Ok(amount);
    }

    // Convert to base unit, then to target unit
    // Base units: meters (length), grams (mass), liters (volume), celsius (temperature), m/s (speed)
    
    // Length conversions (base: meters)
    if is_length_unit(from_unit) && is_length_unit(to_unit) {
        let base_value = to_meters(amount, from_unit)?;
        return from_meters(base_value, to_unit);
    }
    
    // Mass conversions (base: grams)
    if is_mass_unit(from_unit) && is_mass_unit(to_unit) {
        let base_value = to_grams(amount, from_unit)?;
        return from_grams(base_value, to_unit);
    }
    
    // Volume conversions (base: liters)
    if is_volume_unit(from_unit) && is_volume_unit(to_unit) {
        let base_value = to_liters(amount, from_unit)?;
        return from_liters(base_value, to_unit);
    }
    
    // Temperature conversions
    if is_temperature_unit(from_unit) && is_temperature_unit(to_unit) {
        return convert_temperature(amount, from_unit, to_unit);
    }
    
    // Speed conversions (base: m/s)
    if is_speed_unit(from_unit) && is_speed_unit(to_unit) {
        let base_value = to_meters_per_second(amount, from_unit)?;
        return from_meters_per_second(base_value, to_unit);
    }
    
    Err(format!("Cannot convert from {} to {} (different categories)", from_unit, to_unit))
}

fn is_length_unit(unit: &str) -> bool {
    matches!(unit, "mm" | "cm" | "m" | "km" | "in" | "ft" | "yd" | "mi")
}

fn is_mass_unit(unit: &str) -> bool {
    matches!(unit, "mg" | "g" | "kg" | "oz" | "lb")
}

fn is_volume_unit(unit: &str) -> bool {
    matches!(unit, "ml" | "L" | "fl-oz" | "cup" | "pint" | "quart" | "gal")
}

fn is_temperature_unit(unit: &str) -> bool {
    matches!(unit, "C" | "F")
}

fn is_speed_unit(unit: &str) -> bool {
    matches!(unit, "km/h" | "m/h")
}

// Length conversion helpers
fn to_meters(value: f64, unit: &str) -> Result<f64, String> {
    Ok(match unit {
        "mm" => value / 1000.0,
        "cm" => value / 100.0,
        "m" => value,
        "km" => value * 1000.0,
        "in" => value * 0.0254,
        "ft" => value * 0.3048,
        "yd" => value * 0.9144,
        "mi" => value * 1609.344,
        _ => return Err(format!("Unknown length unit: {}", unit)),
    })
}

fn from_meters(value: f64, unit: &str) -> Result<f64, String> {
    Ok(match unit {
        "mm" => value * 1000.0,
        "cm" => value * 100.0,
        "m" => value,
        "km" => value / 1000.0,
        "in" => value / 0.0254,
        "ft" => value / 0.3048,
        "yd" => value / 0.9144,
        "mi" => value / 1609.344,
        _ => return Err(format!("Unknown length unit: {}", unit)),
    })
}

// Mass conversion helpers
// Precise constants:
// 1 kg = 1000 g
// 1 oz = 28.3495 g (exact)
// 1 lb = 453.592 g (exact)
// 1 kg = 2.20462 lb (derived: 1000 / 453.592)
fn to_grams(value: f64, unit: &str) -> Result<f64, String> {
    Ok(match unit {
        "mg" => value / 1000.0,
        "g" => value,
        "kg" => value * 1000.0,
        "oz" => value * 28.3495,  // Exact: 1 oz = 28.3495 g
        "lb" => value * 453.592,  // Exact: 1 lb = 453.592 g
        _ => return Err(format!("Unknown mass unit: {}", unit)),
    })
}

fn from_grams(value: f64, unit: &str) -> Result<f64, String> {
    Ok(match unit {
        "mg" => value * 1000.0,
        "g" => value,
        "kg" => value / 1000.0,
        "oz" => value / 28.3495,  // Exact: 1 oz = 28.3495 g
        "lb" => value / 453.592,  // Exact: 1 lb = 453.592 g (ensures 1 kg = 2.20462 lb)
        _ => return Err(format!("Unknown mass unit: {}", unit)),
    })
}

// Volume conversion helpers
fn to_liters(value: f64, unit: &str) -> Result<f64, String> {
    Ok(match unit {
        "ml" => value / 1000.0,
        "L" => value,
        "fl-oz" => value * 0.0295735,
        "cup" => value * 0.236588,
        "pint" => value * 0.473176,
        "quart" => value * 0.946353,
        "gal" => value * 3.78541,
        _ => return Err(format!("Unknown volume unit: {}", unit)),
    })
}

fn from_liters(value: f64, unit: &str) -> Result<f64, String> {
    Ok(match unit {
        "ml" => value * 1000.0,
        "L" => value,
        "fl-oz" => value / 0.0295735,
        "cup" => value / 0.236588,
        "pint" => value / 0.473176,
        "quart" => value / 0.946353,
        "gal" => value / 3.78541,
        _ => return Err(format!("Unknown volume unit: {}", unit)),
    })
}

// Cross-category conversion helpers
fn convert_volume_to_mass(
    volume_amount: f64,
    volume_unit: &str,
    mass_unit: &str,
    params: &serde_json::Value,
) -> Result<(f64, String), String> {
    // Get material density (default to water: 1 g/mL)
    let density_g_per_ml = params.get("material")
        .and_then(|v| v.as_str())
        .and_then(|m| get_material_density(m))
        .unwrap_or(1.0); // Default: water density (1 g/mL)
    
    // Convert volume to liters first
    let volume_liters = to_liters(volume_amount, volume_unit)?;
    
    // Convert to milliliters
    let volume_ml = volume_liters * 1000.0;
    
    // Calculate mass in grams: mass = volume × density
    let mass_grams = volume_ml * density_g_per_ml;
    
    // Convert to target mass unit
    let converted_value = from_grams(mass_grams, mass_unit)?;
    Ok((converted_value, mass_unit.to_string()))
}

fn convert_mass_to_volume(
    mass_amount: f64,
    mass_unit: &str,
    volume_unit: &str,
    params: &serde_json::Value,
) -> Result<(f64, String), String> {
    // Get material density (default to water: 1 g/mL)
    let density_g_per_ml = params.get("material")
        .and_then(|v| v.as_str())
        .and_then(|m| get_material_density(m))
        .unwrap_or(1.0); // Default: water density (1 g/mL)
    
    // Convert mass to grams first
    let mass_grams = to_grams(mass_amount, mass_unit)?;
    
    // Calculate volume in milliliters: volume = mass / density
    let volume_ml = mass_grams / density_g_per_ml;
    
    // Convert to liters
    let volume_liters = volume_ml / 1000.0;
    
    // Convert to target volume unit
    let converted_value = from_liters(volume_liters, volume_unit)?;
    Ok((converted_value, volume_unit.to_string()))
}

// Temperature conversion
// Formula: F = (C × 9/5) + 32
// Example: 5°C = (5 × 9/5) + 32 = 9 + 32 = 41°F
fn convert_temperature(value: f64, from_unit: &str, to_unit: &str) -> Result<f64, String> {
    // Convert to Celsius first
    let celsius = match from_unit {
        "C" => value,
        "F" => (value - 32.0) * 5.0 / 9.0,
        _ => return Err(format!("Unknown temperature unit: {}", from_unit)),
    };
    
    // Convert from Celsius to target
    // CRITICAL: Use explicit parentheses to ensure correct order: (c * 9/5) + 32
    Ok(match to_unit {
        "C" => celsius,
        "F" => (celsius * 9.0 / 5.0) + 32.0,
        _ => return Err(format!("Unknown temperature unit: {}", to_unit)),
    })
}

// Speed conversion helpers
fn to_meters_per_second(value: f64, unit: &str) -> Result<f64, String> {
    Ok(match unit {
        "km/h" => value / 3.6,
        "m/h" => value * 0.44704,
        _ => return Err(format!("Unknown speed unit: {}", unit)),
    })
}

fn from_meters_per_second(value: f64, unit: &str) -> Result<f64, String> {
    Ok(match unit {
        "km/h" => value * 3.6,
        "m/h" => value / 0.44704,
        _ => return Err(format!("Unknown speed unit: {}", unit)),
    })
}

// Helper to add thousands separators to a string
fn add_thousands_separators(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().rev().collect();
    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(*ch);
    }
    result.chars().rev().collect()
}

// Format number with thousands separators, max 2 decimals, strip trailing zeros
// Examples: 130000.000 -> "130,000", 12.500 -> "12.5", 12.567 -> "12.57"
fn format_number(value: f64) -> String {
    // Round to 2 decimal places
    let rounded = (value * 100.0).round() / 100.0;
    
    // Handle special cases
    if rounded.is_nan() {
        return "NaN".to_string();
    }
    if rounded.is_infinite() {
        return if rounded.is_sign_positive() { "∞".to_string() } else { "-∞".to_string() };
    }
    
    // Handle overflow for very large numbers
    let integer_part = if rounded.abs() > i64::MAX as f64 {
        // For numbers beyond i64::MAX, use string formatting directly
        let formatted = format!("{:.2}", rounded)
            .replace(".00", "")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string();
        // Add thousands separators manually for large numbers
        return if formatted.contains('.') {
            let parts: Vec<&str> = formatted.split('.').collect();
            format!("{}.{}", add_thousands_separators(parts[0]), parts[1])
        } else {
            add_thousands_separators(&formatted)
        };
    } else {
        rounded.trunc() as i64
    };
    let decimal_part = (rounded.fract() * 100.0).round() as i64;
    
    // Format integer part with thousands separators
    let integer_str = format!("{}", integer_part.abs());
    let formatted_integer = add_thousands_separators(&integer_str);
    
    // Handle negative sign
    let formatted_integer = if integer_part < 0 {
        format!("-{}", formatted_integer)
    } else {
        formatted_integer
    };
    
    // Format decimal part (strip trailing zeros)
    if decimal_part == 0 {
        // No decimal part needed
        formatted_integer
    } else {
        // Format with up to 2 decimal places, strip trailing zeros
        let decimal_str = format!("{:02}", decimal_part);
        let trimmed_decimal = decimal_str.trim_end_matches('0');
        
        if trimmed_decimal.is_empty() {
            formatted_integer
        } else {
            format!("{}.{}", formatted_integer, trimmed_decimal)
        }
    }
}

// Material density lookup (g/mL)
// Default is water (1.0 g/mL)
fn get_material_density(material: &str) -> Option<f64> {
    let material_lower = material.to_lowercase();
    match material_lower.as_str() {
        "water" | "h2o" => Some(1.0),
        "milk" => Some(1.03),
        "oil" | "vegetable oil" => Some(0.92),
        "gasoline" | "petrol" => Some(0.74),
        "alcohol" | "ethanol" => Some(0.79),
        "mercury" => Some(13.6),
        "iron" => Some(7.87),
        "aluminum" | "aluminium" => Some(2.70),
        "gold" => Some(19.3),
        "lead" => Some(11.3),
        _ => None, // Unknown material, will use default (water)
    }
}
