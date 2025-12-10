use crate::shared::types::{ActionType, CommandItem, ConvertUnitsRequest, ConvertUnitsResponse, ExecuteActionResponse, ParseUnitResponse, GetUnitsResponse, UnitDTO};

// Error constants - inline for now (can be moved to shared::errors later)
const ERR_MISSING_TEXT_PARAM: &str = "Missing 'text' parameter";
const ERR_NEGATIVE_LENGTH: &str = "Length cannot be negative. Please provide a positive value.";
const ERR_NEGATIVE_MASS: &str = "Mass cannot be negative. Please provide a positive value.";
const ERR_NEGATIVE_VOLUME: &str = "Volume cannot be negative. Please provide a positive value.";
const ERR_UNSUPPORTED_ACTION: &str = "Unsupported action type";
const ERR_CANNOT_PARSE_UNIT: &str = "Could not parse unit from text";
use super::{FeatureSync, FeatureAsync};
use async_trait::async_trait;
use serde_json::json;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

// ============================================================================
// Unit Registry - Backend-First Pattern
// ============================================================================

/// Unit categories for type-safe conversions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitCategory {
    Length,
    Mass,
    Volume,
    Temperature,
    Speed,
}

/// Unit definition with conversion factors
#[derive(Debug, Clone)]
pub struct UnitDefinition {
    pub symbol: &'static str,
    pub name: &'static str,
    pub category: UnitCategory,
    pub base_factor: f64,  // Multiplier to convert to base unit
    pub offset: f64,       // Offset for affine conversions (e.g., temperature)
}

/// Thread-safe unit registry initialized once at startup
static UNIT_REGISTRY: Lazy<HashMap<&'static str, UnitDefinition>> = Lazy::new(|| {
    let mut registry = HashMap::new();
    
    // Length (base: meters)
    registry.insert("mm", UnitDefinition { 
        symbol: "mm", name: "Millimeters", category: UnitCategory::Length, 
        base_factor: 0.001, offset: 0.0 
    });
    registry.insert("cm", UnitDefinition { 
        symbol: "cm", name: "Centimeters", category: UnitCategory::Length, 
        base_factor: 0.01, offset: 0.0 
    });
    registry.insert("m", UnitDefinition { 
        symbol: "m", name: "Meters", category: UnitCategory::Length, 
        base_factor: 1.0, offset: 0.0 
    });
    registry.insert("km", UnitDefinition { 
        symbol: "km", name: "Kilometers", category: UnitCategory::Length, 
        base_factor: 1000.0, offset: 0.0 
    });
    registry.insert("in", UnitDefinition { 
        symbol: "in", name: "Inches", category: UnitCategory::Length, 
        base_factor: 0.0254, offset: 0.0 
    });
    registry.insert("ft", UnitDefinition { 
        symbol: "ft", name: "Feet", category: UnitCategory::Length, 
        base_factor: 0.3048, offset: 0.0 
    });
    registry.insert("yd", UnitDefinition { 
        symbol: "yd", name: "Yards", category: UnitCategory::Length, 
        base_factor: 0.9144, offset: 0.0 
    });
    registry.insert("mi", UnitDefinition { 
        symbol: "mi", name: "Miles", category: UnitCategory::Length, 
        base_factor: 1609.344, offset: 0.0 
    });
    
    // Mass (base: grams)
    registry.insert("mg", UnitDefinition { 
        symbol: "mg", name: "Milligrams", category: UnitCategory::Mass, 
        base_factor: 0.001, offset: 0.0 
    });
    registry.insert("g", UnitDefinition { 
        symbol: "g", name: "Grams", category: UnitCategory::Mass, 
        base_factor: 1.0, offset: 0.0 
    });
    registry.insert("kg", UnitDefinition { 
        symbol: "kg", name: "Kilograms", category: UnitCategory::Mass, 
        base_factor: 1000.0, offset: 0.0 
    });
    registry.insert("oz", UnitDefinition { 
        symbol: "oz", name: "Ounces", category: UnitCategory::Mass, 
        base_factor: 28.3495, offset: 0.0 
    });
    registry.insert("lb", UnitDefinition { 
        symbol: "lb", name: "Pounds", category: UnitCategory::Mass, 
        base_factor: 453.592, offset: 0.0 
    });
    
    // Volume (base: liters)
    registry.insert("ml", UnitDefinition { 
        symbol: "ml", name: "Milliliters", category: UnitCategory::Volume, 
        base_factor: 0.001, offset: 0.0 
    });
    registry.insert("L", UnitDefinition { 
        symbol: "L", name: "Liters", category: UnitCategory::Volume, 
        base_factor: 1.0, offset: 0.0 
    });
    registry.insert("fl-oz", UnitDefinition { 
        symbol: "fl-oz", name: "Fluid Ounces", category: UnitCategory::Volume, 
        base_factor: 0.0295735, offset: 0.0 
    });
    registry.insert("cup", UnitDefinition { 
        symbol: "cup", name: "Cups", category: UnitCategory::Volume, 
        base_factor: 0.236588, offset: 0.0 
    });
    registry.insert("pint", UnitDefinition { 
        symbol: "pint", name: "Pints", category: UnitCategory::Volume, 
        base_factor: 0.473176, offset: 0.0 
    });
    registry.insert("quart", UnitDefinition { 
        symbol: "quart", name: "Quarts", category: UnitCategory::Volume, 
        base_factor: 0.946353, offset: 0.0 
    });
    registry.insert("gal", UnitDefinition { 
        symbol: "gal", name: "Gallons", category: UnitCategory::Volume, 
        base_factor: 3.78541, offset: 0.0 
    });
    
    // Temperature (base: Celsius)
    // For affine conversions: base_value = (value + offset) * base_factor
    registry.insert("C", UnitDefinition { 
        symbol: "C", name: "Celsius", category: UnitCategory::Temperature, 
        base_factor: 1.0, offset: 0.0 
    });
    registry.insert("F", UnitDefinition { 
        symbol: "F", name: "Fahrenheit", category: UnitCategory::Temperature, 
        base_factor: 5.0/9.0, offset: -32.0 
    });
    
    // Speed (base: m/s)
    registry.insert("km/h", UnitDefinition { 
        symbol: "km/h", name: "Kilometers/Hour", category: UnitCategory::Speed, 
        base_factor: 1.0/3.6, offset: 0.0 
    });
    registry.insert("m/h", UnitDefinition { 
        symbol: "m/h", name: "Miles/Hour", category: UnitCategory::Speed, 
        base_factor: 0.44704, offset: 0.0 
    });
    
    registry
});

/// Convert category enum to string
fn category_to_string(category: UnitCategory) -> String {
    match category {
        UnitCategory::Length => "length",
        UnitCategory::Mass => "mass",
        UnitCategory::Volume => "volume",
        UnitCategory::Temperature => "temperature",
        UnitCategory::Speed => "speed",
    }.to_string()
}

/// Generic conversion function using the unit registry
/// Supports both multiplicative (base_factor) and affine (offset) conversions
/// Also handles cross-category conversions (Mass ↔ Volume) using water density as bridge
fn convert_value(value: f64, from_unit: &str, to_unit: &str) -> Result<f64, String> {
    // Same unit, no conversion needed
    if from_unit == to_unit {
        return Ok(value);
    }

    // Look up both units in the registry
    let from_def = UNIT_REGISTRY.get(from_unit)
        .ok_or_else(|| format!("Unknown source unit: {}", from_unit))?;
    let to_def = UNIT_REGISTRY.get(to_unit)
        .ok_or_else(|| format!("Unknown target unit: {}", to_unit))?;

    // Check for category mismatch (Mass ↔ Volume bridge)
    let bridge_factor = match (&from_def.category, &to_def.category) {
        (UnitCategory::Mass, UnitCategory::Volume) => {
            // Mass to Volume: 1000 g = 1 L (water density)
            // So: grams / 1000 = liters
            println!("[convert_value] Bridge: Mass → Volume (water density: 1000 g/L)");
            Some(1.0 / 1000.0) // Convert grams to liters
        },
        (UnitCategory::Volume, UnitCategory::Mass) => {
            // Volume to Mass: 1 L = 1000 g (water density)
            // So: liters * 1000 = grams
            println!("[convert_value] Bridge: Volume → Mass (water density: 1000 g/L)");
            Some(1000.0) // Convert liters to grams
        },
        (from_cat, to_cat) if from_cat != to_cat => {
            return Err(format!(
                "Cannot convert between {:?} and {:?} (incompatible categories)",
                from_cat, to_cat
            ));
        },
        _ => None, // Same category, no bridge needed
    };

    // Step 1: Convert from source unit to base unit (with offset if applicable)
    let base_value = (value + from_def.offset) * from_def.base_factor;
    
    // Step 2: Apply bridge factor if cross-category conversion
    let bridged_value = if let Some(factor) = bridge_factor {
        base_value * factor
    } else {
        base_value
    };
    
    // Step 3: Convert from base unit to target unit (with offset if applicable)
    let result = (bridged_value / to_def.base_factor) - to_def.offset;

    println!(
        "[convert_value] {} {} → {} {} (base: {}, bridged: {}, result: {})",
        value, from_unit, result, to_unit, base_value, bridged_value, result
    );

    Ok(result)
}

// ============================================================================
// Feature Implementation
// ============================================================================


#[derive(Clone)]
pub struct UnitConverterFeature;

impl FeatureSync for UnitConverterFeature {
    fn id(&self) -> &'static str {
        "unit_converter"
    }
    
    fn widget_commands(&self) -> Vec<CommandItem> {
        vec![CommandItem {
            id: "widget_unit_converter".to_string(),
            label: "Unit Converter".to_string(),
            description: Some("Convert between units".to_string()),
            action_type: None,
            widget_type: Some("unit_converter".to_string()),
            category: None,
        }]
    }
    
    
    fn action_commands(&self) -> Vec<CommandItem> {
        // Generate action commands for all units in the registry
        // Users can quickly convert selected text via Command Palette
        vec![
            // Length conversions
            ("convert_to_mm", "Convert to Millimeters", "mm"),
            ("convert_to_cm", "Convert to Centimeters", "cm"),
            ("convert_to_m", "Convert to Meters", "m"),
            ("convert_to_km", "Convert to Kilometers", "km"),
            ("convert_to_in", "Convert to Inches", "in"),
            ("convert_to_ft", "Convert to Feet", "ft"),
            ("convert_to_yd", "Convert to Yards", "yd"),
            ("convert_to_mi", "Convert to Miles", "mi"),
            
            // Mass conversions
            ("convert_to_mg", "Convert to Milligrams", "mg"),
            ("convert_to_g", "Convert to Grams", "g"),
            ("convert_to_kg", "Convert to Kilograms", "kg"),
            ("convert_to_oz", "Convert to Ounces", "oz"),
            ("convert_to_lb", "Convert to Pounds", "lb"),
            
            // Volume conversions
            ("convert_to_ml", "Convert to Milliliters", "ml"),
            ("convert_to_l", "Convert to Liters", "L"),
            ("convert_to_fl_oz", "Convert to Fluid Ounces", "fl-oz"),
            ("convert_to_cup", "Convert to Cups", "cup"),
            ("convert_to_pint", "Convert to Pints", "pint"),
            ("convert_to_quart", "Convert to Quarts", "quart"),
            ("convert_to_gal", "Convert to Gallons", "gal"),
            
            // Temperature conversions
            ("convert_to_c", "Convert to Celsius", "C"),
            ("convert_to_f", "Convert to Fahrenheit", "F"),
            
            // Speed conversions
            ("convert_to_kmh", "Convert to Kilometers/Hour", "km/h"),
            ("convert_to_mph", "Convert to Miles/Hour", "m/h"),
        ]
        .into_iter()
        .map(|(id, label, target_unit)| CommandItem {
            id: id.to_string(),
            label: label.to_string(),
            description: None,
            action_type: Some(ActionType::ConvertUnit {
                target: target_unit.to_string(),
            }),
            widget_type: None,
            category: None, // Will be assigned by get_action_category
        })
        .collect()
    }

    fn get_context_boost(&self, _captured_text: &str) -> std::collections::HashMap<String, f64> {
        std::collections::HashMap::new()
    }
}

#[async_trait]
impl FeatureAsync for UnitConverterFeature {
        async fn execute_action(
        &self,
        action_type: &ActionType,
        params: &serde_json::Value,
    ) -> crate::shared::error::AppResult<ExecuteActionResponse> {
        // DEBUG: Log the received action type
        println!("[execute_action] DEBUG: Received Action: {:?}", action_type);
        println!("[execute_action] DEBUG: Params: {:?}", params);
        
        // Check action type FIRST before attempting any parsing
        match action_type {
            ActionType::ConvertUnit { target } => {
                // Extract text from params
                let text = params.get("text")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| crate::shared::error::AppError::Validation(ERR_MISSING_TEXT_PARAM.to_string()))?;

                // Parse amount and source unit from text
                let (amount, source_unit) = parse_unit_from_text(text)
                    .map_err(|e| crate::shared::error::AppError::Calculation(e))?;

                let result = convert_value(amount, &source_unit, target)
                    .map_err(|e| crate::shared::error::AppError::Calculation(e))?;
                
                let target_unit = target.as_str();
                let converted_value = result;

                // Format result with beautiful number formatting
                let formatted_value = format_number(converted_value);
                let result_string = format!("{} {}", formatted_value, target_unit);

                Ok(ExecuteActionResponse {
                    result: result_string,
                    metadata: Some(json!({
                        "from_unit": source_unit,
                        "target_unit": target_unit,
                        "original_amount": amount,
                        "converted_amount": converted_value,
                        "widget": "unit_converter"
                    })),
                })
            },
            _ => Err(crate::shared::error::AppError::Unknown(crate::shared::errors::ERR_UNSUPPORTED_ACTION.to_string())),
        }
    }
}

// ============================================================================
// Tauri Commands - Public API
// ============================================================================

/// Parse text input and extract amount, unit, and category
#[tauri::command]
pub async fn parse_text_command(text: String) -> crate::shared::error::AppResult<ParseUnitResponse> {
    let (amount, unit) = parse_unit_from_text(&text)
        .map_err(|e| crate::shared::error::AppError::Validation(e))?;
    
    let unit_def = UNIT_REGISTRY.get(unit.as_str())
        .ok_or_else(|| crate::shared::error::AppError::Validation(format!("Unknown unit: {}", unit)))?;
    
    let category = category_to_string(unit_def.category);
    
    Ok(ParseUnitResponse {
        amount,
        unit,
        category,
    })
}

/// Get all available units from the registry
/// Returns a list of UnitDTO objects for frontend consumption
#[tauri::command]
pub async fn get_all_units_command() -> crate::shared::error::AppResult<GetUnitsResponse> {
    use crate::shared::types::UnitDTO;
    
    let mut units: Vec<UnitDTO> = UNIT_REGISTRY
        .iter()
        .map(|(_, def)| UnitDTO {
            id: def.symbol.to_string(),
            label: def.name.to_string(),
            category: category_to_string(def.category.clone()),
        })
        .collect();
    
    // Sort by category, then by name
    units.sort_by(|a, b| {
        a.category.cmp(&b.category)
            .then_with(|| a.label.cmp(&b.label))
    });
    
    Ok(GetUnitsResponse { units })
}

/// Convert units using the registry
#[tauri::command]
pub async fn convert_units_command(request: ConvertUnitsRequest) -> crate::shared::error::AppResult<ConvertUnitsResponse> {
    let result = convert_value(request.amount, &request.from_unit, &request.to_unit)
        .map_err(|e| crate::shared::error::AppError::Calculation(e))?;
    
    let formatted_value = format_number(result);
    
    Ok(ConvertUnitsResponse {
        result,
        formatted_result: formatted_value,
        from_unit: request.from_unit,
        to_unit: request.to_unit,
    })
}

// ============================================================================
// Legacy Commands (Deprecated - kept for backward compatibility)
// ============================================================================

#[deprecated(note = "Use convert_units_command instead")]
#[tauri::command]
pub async fn convert_units(request: ConvertUnitsRequest) -> crate::shared::error::AppResult<ConvertUnitsResponse> {
    convert_units_command(request).await
}

#[deprecated(note = "Use get_all_units_command and filter by category")]
#[tauri::command]
pub async fn get_units_for_category(category: String) -> crate::shared::error::AppResult<Vec<String>> {
    let units = match category.as_str() {
        "length" => vec!["mm", "cm", "m", "km", "in", "ft", "yd", "mi"],
        "mass" => vec!["mg", "g", "kg", "oz", "lb"],
        "volume" => vec!["ml", "L", "fl-oz", "cup", "pint", "quart", "gal"],
        "temperature" => vec!["C", "F"],
        "speed" => vec!["km/h", "m/h"],
        _ => return Err(crate::shared::error::AppError::Validation(format!("Unknown category: {}", category))),
    };
    
    Ok(units.iter().map(|s| s.to_string()).collect())
}

#[deprecated(note = "Settings should be managed in frontend state")]
#[tauri::command]
pub async fn get_unit_settings() -> crate::shared::error::AppResult<serde_json::Value> {
    Ok(json!({
        "default_from_unit": "m",
        "default_to_unit": "ft",
        "default_category": "length"
    }))
}

// ============================================================================
// Helper Functions
// ============================================================================


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
// LAX PARSING: Removed ^ and $ anchors to allow extraction from anywhere in the string
static RE_PATTERN_1: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"([+-]?\d+(?:\.\d+)?)\s*([a-zA-Z°'/″³]+(?:\/[a-zA-Z]+)?)")
        .expect("Failed to compile regex pattern 1")
});

static RE_PATTERN_2: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"([a-zA-Z°'/″³]+(?:\/[a-zA-Z]+)?)\s*([+-]?\d+(?:\.\d+)?)")
        .expect("Failed to compile regex pattern 2")
});

static RE_PATTERN_3: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"([+-]?\d+(?:\.\d+)?)")
        .expect("Failed to compile regex pattern 3")
});

// Parse amount and unit from text (e.g., "100m", "12 km", "3.5 meters", "2km to miles")
// LAX PARSING: Extracts the first number/unit pair found anywhere in the string
fn parse_unit_from_text(text: &str) -> Result<(f64, String), String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("Empty text".to_string());
    }

    // Normalize comma decimal separators to dots
    let normalized_text = text.replace(',', ".");

    // Pattern 1: Number followed by unit (e.g., "12km", "12 km", "12 kilometers")
    // LAX: Use .find() to locate the pattern anywhere in the string
    if let Some(caps) = RE_PATTERN_1.captures(&normalized_text) {
        if let (Ok(amount), Some(unit_str)) = (caps[1].parse::<f64>(), caps.get(2)) {
            if let Some(canonical_unit) = normalize_unit(unit_str.as_str()) {
                println!("[parse_unit_from_text] ✓ Extracted: {} {} from '{}'", amount, canonical_unit, text);
                return Ok((amount, canonical_unit.to_string()));
            }
        }
    }

    // Pattern 2: Unit followed by number (e.g., "km12", "m 100")
    if let Some(caps) = RE_PATTERN_2.captures(&normalized_text) {
        if let (Some(unit_str), Ok(amount)) = (caps.get(1), caps[2].parse::<f64>()) {
            if let Some(canonical_unit) = normalize_unit(unit_str.as_str()) {
                println!("[parse_unit_from_text] ✓ Extracted: {} {} from '{}'", amount, canonical_unit, text);
                return Ok((amount, canonical_unit.to_string()));
            }
        }
    }

    // Pattern 3: Try to extract any number and any known unit from the text
    if let Some(caps) = RE_PATTERN_3.captures(&normalized_text) {
        if let Ok(amount) = caps[1].parse::<f64>() {
            let text_lower = normalized_text.to_lowercase();
            // Check common unit patterns (longest first to match "kilometers" before "km")
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
                    println!("[parse_unit_from_text] ✓ Extracted: {} {} from '{}'", amount, canonical, text);
                    return Ok((amount, canonical.to_string()));
                }
            }
        }
    }

    println!("[parse_unit_from_text] ✗ Failed to parse: '{}'", text);
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
