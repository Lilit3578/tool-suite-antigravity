//! Time Zone Converter Feature
//! 
//! Consolidated logic for time zone conversion, including parsing, detection, and timezone data.

use std::sync::OnceLock;
use std::collections::HashMap;
use regex::Regex;
use async_trait::async_trait;
use chrono::{Local, TimeZone, Offset};
use chrono_tz::Tz;
use chrono_english::{parse_date_string, Dialect};
use crate::shared::types::{
    ConvertTimeRequest, ConvertTimeResponse, ParsedTimeInput, 
    CommandItem, ActionType, TimePayload, ExecuteActionResponse, TimezoneInfo
};
use super::{FeatureSync, FeatureAsync};

// ==================================================================================
// CONSTANTS
// ==================================================================================

/// Timezone data: (Label, IANA ID, Search Keywords)
pub const ALL_TIMEZONES: &[(&str, &str, &str)] = &[
    // --- ASIA ---
    ("Beijing/China", "Asia/Shanghai", "china beijing shanghai shenzhen guangzhou chengdu chongqing wuhan tianjin hangzhou nanjing xi'an harbin shenyang urumqi lhasa hong kong macau"),
    ("Tokyo/Japan", "Asia/Tokyo", "japan tokyo osaka kyoto yokohama sapporo nagoya kobe fukuoka kawasaki"),
    ("New Delhi/India", "Asia/Kolkata", "india delhi mumbai bengaluru bangalore chennai kolkata calcutta hyderabad ahmedabad pune jaipur kochi patna nagpur guwahati"),
    ("Seoul/South Korea", "Asia/Seoul", "south korea seoul busan incheon daegu daejeon gwangju"),
    ("Singapore/Singapore", "Asia/Singapore", "singapore"),
    ("Dubai/United Arab Emirates", "Asia/Dubai", "uae united arab emirates dubai abu dhabi sharjah"),
    ("Bangkok/Thailand", "Asia/Bangkok", "thailand bangkok chiang mai phuket"),
    ("Jakarta/Indonesia", "Asia/Jakarta", "indonesia jakarta surabaya bandung"),
    ("Kabul/Afghanistan", "Asia/Kabul", "afghanistan asia kabul"),
    ("Yerevan/Armenia", "Asia/Yerevan", "armenia asia yerevan"),
    ("Dhaka/Bangladesh", "Asia/Dhaka", "bangladesh asia dhaka"),
    ("Thimphu/Bhutan", "Asia/Thimphu", "bhutan asia thimphu"),
    ("Baghdad/Iraq", "Asia/Baghdad", "iraq asia baghdad"),
    ("Jerusalem/Israel", "Asia/Jerusalem", "israel asia jerusalem"),
    ("Amman/Jordan", "Asia/Amman", "jordan asia amman"),
    ("Almaty/Kazakhstan", "Asia/Almaty", "kazakhstan asia almaty astana"),
    ("Kuwait/Kuwait", "Asia/Kuwait", "kuwait asia middle east"),
    ("Bishkek/Kyrgyzstan", "Asia/Bishkek", "kyrgyzstan asia bishkek"),
    ("Vientiane/Laos", "Asia/Vientiane", "laos asia vientiane"),
    ("Beirut/Lebanon", "Asia/Beirut", "lebanon asia beirut"),
    ("Kuala Lumpur/Malaysia", "Asia/Kuala_Lumpur", "malaysia asia kuala lumpur"),
    ("Colombo/Sri Lanka", "Asia/Colombo", "sri lanka asia colombo"),
    ("Manila/Philippines", "Asia/Manila", "philippines asia manila"),
    ("Muscat/Oman", "Asia/Muscat", "oman asia muscat"),
    ("Karachi/Pakistan", "Asia/Karachi", "pakistan asia karachi islamabad"),
    ("Tehran/Iran", "Asia/Tehran", "iran tehran mashhad isfahan karaj shiraz"),
    ("Riyadh/Saudi Arabia", "Asia/Riyadh", "saudi arabia riyadh jeddah mecca medina"),
    ("Dushanbe/Tajikistan", "Asia/Dushanbe", "tajikistan asia dushanbe"),
    ("Ashgabat/Turkmenistan", "Asia/Ashgabat", "turkmenistan asia ashgabat"),
    ("Tashkent/Uzbekistan", "Asia/Tashkent", "uzbekistan asia tashkent"),
    ("Aden/Yemen", "Asia/Aden", "yemen asia aden"),
    ("Yangon/Myanmar", "Asia/Yangon", "myanmar burma asia yangon rangoon"),
    ("Dili/Timor-Leste", "Asia/Dili", "timor leste east asia dili"),
    ("Pyongyang/North Korea", "Asia/Pyongyang", "north korea asia pyongyang dprk"),
    ("Ho Chi Minh/Vietnam", "Asia/Ho_Chi_Minh", "vietnam asia ho chi minh saigon hanoi"),

    // --- EUROPE ---
    ("London/United Kingdom", "Europe/London", "uk united kingdom great britain england scotland wales london manchester birmingham liverpool edinburgh glasgow belfast leeds sheffield bristol newcastle cardiff"),
    ("Paris/France", "Europe/Paris", "france paris marseille lyon toulouse nice nantes strasbourg montpellier bordeaux lille"),
    ("Berlin/Germany", "Europe/Berlin", "germany berlin hamburg munich cologne frankfurt stuttgart dusseldorf dortmund essen leipzig"),
    ("Rome/Italy", "Europe/Rome", "italy rome milan naples turin palermo genoa bologna florence"),
    ("Madrid/Spain", "Europe/Madrid", "spain madrid barcelona valencia seville zaragoza malaga"),
    ("Moscow/Russia", "Europe/Moscow", "russia moscow st petersburg novosibirsk yekaterinburg kazan nizhny novgorod chelyabinsk samara omsk"),
    ("Amsterdam/Netherlands", "Europe/Amsterdam", "netherlands amsterdam rotterdam the hague utrecht"),
    ("Zurich/Switzerland", "Europe/Zurich", "switzerland zurich geneva basel lausanne bern"),
    ("Kyiv/Ukraine", "Europe/Kiev", "ukraine kyiv kiev kharkiv odessa dnipro donetsk zaporizhzhia lviv"),
    ("Tirane/Albania", "Europe/Tirane", "albania europe tirane tirana"),
    ("Vienna/Austria", "Europe/Vienna", "austria europe vienna"),
    ("Andorra la Vella/Andorra", "Europe/Andorra", "andorra europe"),
    ("Sarajevo/Bosnia and Herzegovina", "Europe/Sarajevo", "bosnia herzegovina europe sarajevo"),
    ("Sofia/Bulgaria", "Europe/Sofia", "bulgaria europe sofia"),
    ("Zagreb/Croatia", "Europe/Zagreb", "croatia europe zagreb"),
    ("Prague/Czech Republic", "Europe/Prague", "czech republic europe prague"),
    ("Copenhagen/Denmark", "Europe/Copenhagen", "denmark europe copenhagen"),
    ("Tallinn/Estonia", "Europe/Tallinn", "estonia europe tallinn"),
    ("Helsinki/Finland", "Europe/Helsinki", "finland europe helsinki"),
    ("Athens/Greece", "Europe/Athens", "greece europe athens"),
    ("Budapest/Hungary", "Europe/Budapest", "hungary europe budapest"),
    ("Reykjavik/Iceland", "Atlantic/Reykjavik", "iceland atlantic reykjavik"),
    ("Riga/Latvia", "Europe/Riga", "latvia europe riga"),
    ("Vaduz/Liechtenstein", "Europe/Vaduz", "liechtenstein europe vaduz"),
    ("Vilnius/Lithuania", "Europe/Vilnius", "lithuania europe vilnius"),
    ("Luxembourg/Luxembourg", "Europe/Luxembourg", "luxembourg europe"),
    ("Valletta/Malta", "Europe/Malta", "malta europe"),
    ("Chisinau/Moldova", "Europe/Chisinau", "moldova europe chisinau"),
    ("Monaco/Monaco", "Europe/Monaco", "monaco europe"),
    ("Podgorica/Montenegro", "Europe/Podgorica", "montenegro europe podgorica"),
    ("Oslo/Norway", "Europe/Oslo", "norway europe oslo"),
    ("Skopje/North Macedonia", "Europe/Skopje", "north macedonia europe skopje"),
    ("Lisbon/Portugal", "Europe/Lisbon", "portugal europe lisbon"),
    ("Bucharest/Romania", "Europe/Bucharest", "romania europe bucharest"),
    ("San Marino/San Marino", "Europe/San_Marino", "san marino europe"),
    ("Belgrade/Serbia", "Europe/Belgrade", "serbia europe belgrade"),
    ("Bratislava/Slovakia", "Europe/Bratislava", "slovakia europe bratislava"),
    ("Ljubljana/Slovenia", "Europe/Ljubljana", "slovenia europe ljubljana"),
    ("Stockholm/Sweden", "Europe/Stockholm", "sweden europe stockholm"),
    ("Vatican City/Vatican", "Europe/Vatican", "vatican city europe"),
    ("Istanbul/Turkey", "Europe/Istanbul", "turkey istanbul ankara izmir bursa adana"),

    // --- AMERICAS ---
    ("New York/United States", "America/New_York", "usa united states eastern est edt new york nyc washington dc miami atlanta boston philadelphia detroit indianapolis louisville monticello columbus charlotte orlando"),
    ("Chicago/United States", "America/Chicago", "usa united states central cst cdt chicago houston dallas austin san antonio nashville memphis kansas city minneapolis st louis new orleans"),
    ("Denver/United States", "America/Denver", "usa united states mountain mst mdt denver salt lake city albuquerque el paso"),
    ("Phoenix/United States", "America/Phoenix", "usa united states arizona phoenix tucson flagstaff"),
    ("Los Angeles/United States", "America/Los_Angeles", "usa united states pacific pst pdt los angeles la san francisco sf san diego seattle portland las vegas vancouver sacramento san jose"),
    ("Anchorage/United States", "America/Anchorage", "usa united states alaska anchorage juneau fairbanks"),
    ("Honolulu/United States", "Pacific/Honolulu", "usa united states hawaii honolulu adak hilo"),
    ("Toronto/Canada", "America/Toronto", "canada eastern toronto montreal ottawa quebec city hamilton london"),
    ("Vancouver/Canada", "America/Vancouver", "canada pacific vancouver victoria surrey burnaby"),
    ("Sao Paulo/Brazil", "America/Sao_Paulo", "brazil sao paulo rio de janeiro brasilia salvador fortaleza belo horizonte"),
    ("Buenos Aires/Argentina", "America/Argentina/Buenos_Aires", "argentina america buenos aires"),
    ("Antigua/Antigua and Barbuda", "America/Antigua", "antigua barbuda america caribbean"),
    ("Nassau/Bahamas", "America/Nassau", "bahamas america nassau caribbean"),
    ("Managua/Nicaragua", "America/Managua", "nicaragua america managua central"),
    ("Mexico City/Mexico", "America/Mexico_City", "mexico america mexico city"),
    ("Guatemala/Guatemala", "America/Guatemala", "guatemala america central"),
    ("Havana/Cuba", "America/Havana", "cuba america havana caribbean"),
    ("Port of Spain/Trinidad and Tobago", "America/Port_of_Spain", "trinidad tobago america port spain caribbean"),
    ("Paramaribo/Suriname", "America/Paramaribo", "suriname america paramaribo"),
    ("Caracas/Venezuela", "America/Caracas", "venezuela america caracas"),
    ("Montevideo/Uruguay", "America/Montevideo", "uruguay america montevideo"),
    ("Asuncion/Paraguay", "America/Asuncion", "paraguay america asuncion"),
    ("Lima/Peru", "America/Lima", "peru america lima"),
    ("Bogota/Colombia", "America/Bogota", "colombia america bogota"),
    ("Guayaquil/Ecuador", "America/Guayaquil", "ecuador america guayaquil quito"),
    ("Port-au-Prince/Haiti", "America/Port-au-Prince", "haiti america port au prince caribbean"),
    ("Tegucigalpa/Honduras", "America/Tegucigalpa", "honduras america tegucigalpa central"),
    ("Belize City/Belize", "America/Belize", "belize america central"),
    ("St Kitts/Saint Kitts and Nevis", "America/St_Kitts", "saint kitts nevis america caribbean"),
    ("Castries/Saint Lucia", "America/St_Lucia", "saint lucia america caribbean"),
    ("Kingstown/Saint Vincent and the Grenadines", "America/St_Vincent", "saint vincent grenadines america caribbean"),
    ("Grenada/Grenada", "America/Grenada", "grenada america caribbean"),
    ("Georgetown/Guyana", "America/Guyana", "guyana america south"),

    // --- AUSTRALIA & PACIFIC ---
    ("Sydney/Australia", "Australia/Sydney", "australia eastern aedt aest sydney melbourne brisbane canberra hobart gold coast newcastle wollongong"),
    ("Adelaide/Australia", "Australia/Adelaide", "australia central acst acdt adelaide darwin"),
    ("Perth/Australia", "Australia/Perth", "australia western awst perth"),
    ("Auckland/New Zealand", "Pacific/Auckland", "new zealand auckland wellington christchurch"),
    ("Apia/Samoa", "Pacific/Apia", "samoa pacific apia"),
    ("Majuro/Marshall Islands", "Pacific/Majuro", "marshall islands pacific majuro"),
    ("Port Moresby/Papua New Guinea", "Pacific/Port_Moresby", "papua new guinea pacific port moresby"),
    ("Palau/Palau", "Pacific/Palau", "palau pacific"),
    ("Guadalcanal/Solomon Islands", "Pacific/Guadalcanal", "solomon islands pacific guadalcanal"),
    ("Nauru/Nauru", "Pacific/Nauru", "nauru pacific"),
    ("Efate/Vanuatu", "Pacific/Efate", "vanuatu pacific efate"),
    ("Funafuti/Tuvalu", "Pacific/Funafuti", "tuvalu pacific funafuti"),
    ("Tongatapu/Tonga", "Pacific/Tongatapu", "tonga pacific tongatapu"),

    // --- AFRICA ---
    ("Cairo/Egypt", "Africa/Cairo", "egypt cairo alexandria giza"),
    ("Johannesburg/South Africa", "Africa/Johannesburg", "south africa johannesburg cape town durban pretoria"),
    ("Asmara/Eritrea", "Africa/Asmara", "eritrea africa asmara"),
    ("Addis Ababa/Ethiopia", "Africa/Addis_Ababa", "ethiopia africa addis ababa"),
    ("Gaborone/Botswana", "Africa/Gaborone", "botswana africa gaborone"),
    ("Algiers/Algeria", "Africa/Algiers", "algeria africa algiers"),
    ("Luanda/Angola", "Africa/Luanda", "angola africa luanda"),
    ("Bangui/Central African Republic", "Africa/Bangui", "central african republic africa bangui"),
    ("Ndjamena/Chad", "Africa/Ndjamena", "chad africa ndjamena"),
    ("Libreville/Gabon", "Africa/Libreville", "gabon africa libreville"),
    ("Banjul/Gambia", "Africa/Banjul", "gambia africa banjul"),
    ("Accra/Ghana", "Africa/Accra", "ghana africa accra"),
    ("Conakry/Guinea", "Africa/Conakry", "guinea africa conakry"),
    ("Bissau/Guinea-Bissau", "Africa/Bissau", "guinea bissau africa"),
    ("Kigali/Rwanda", "Africa/Kigali", "rwanda africa kigali"),
    ("Monrovia/Liberia", "Africa/Monrovia", "liberia africa monrovia"),
    ("Maseru/Lesotho", "Africa/Maseru", "lesotho africa maseru"),
    ("Tripoli/Libya", "Africa/Tripoli", "libya africa tripoli"),
    ("Nouakchott/Mauritania", "Africa/Nouakchott", "mauritania africa nouakchott"),
    ("Porto-Novo/Benin", "Africa/Porto-Novo", "benin africa porto novo"),
    ("Ouagadougou/Burkina Faso", "Africa/Ouagadougou", "burkina faso africa ouagadougou"),
    ("Maputo/Mozambique", "Africa/Maputo", "mozambique africa maputo"),
    ("Lusaka/Zambia", "Africa/Lusaka", "zambia africa lusaka"),
    ("Harare/Zimbabwe", "Africa/Harare", "zimbabwe africa harare"),
    ("Freetown/Sierra Leone", "Africa/Freetown", "sierra leone africa freetown"),
    ("Khartoum/Sudan", "Africa/Khartoum", "sudan africa khartoum"),
    ("Lome/Togo", "Africa/Lome", "togo africa lome"),
];

/// Timezone IANA ID to Abbreviation mapping
/// Format: (IANA_ID, Abbreviation)
pub const TIMEZONE_ABBREVIATIONS_MAP: &[(&str, &str)] = &[
    ("Kabul/Afghanistan", "AFT"),
    ("Tirana/Albania", "CET"),
    ("Algiers/Algeria", "CET"),
    ("Andorra/Andorra", "CET"),
    ("Luanda/Angola", "WAT"),
    ("St. John's/Antigua and Barbuda", "AST"),
    ("Buenos Aires/Argentina", "ART"),
    ("Yerevan/Armenia", "AMT"),
    ("Sydney/Australia", "AEDT"),
    ("Vienna/Austria", "CET"),
    ("Baku/Azerbaijan", "AZT"),
    ("Nassau/Bahamas", "EST"),
    ("Manama/Bahrain", "AST"),
    ("Dhaka/Bangladesh", "BST"),
    ("Bridgetown/Barbados", "AST"),
    ("Minsk/Belarus", "MSK"),
    ("Brussels/Belgium", "CET"),
    ("Belmopan/Belize", "CST"),
    ("Porto-Novo/Benin", "WAT"),
    ("Thimphu/Bhutan", "BTT"),
    ("La Paz/Bolivia", "BOT"),
    ("Sarajevo/Bosnia and Herzegovina", "CET"),
    ("Gaborone/Botswana", "CAT"),
    ("Brasília/Brazil", "BRT"),
    ("Bandar Seri Begawan/Brunei", "BNT"),
    ("Sofia/Bulgaria", "EET"),
    ("Ouagadougou/Burkina Faso", "GMT"),
    ("Bujumbura/Burundi", "CAT"),
    ("Praia/Cape Verde", "CVT"),
    ("Phnom Penh/Cambodia", "ICT"),
    ("Douala/Cameroon", "WAT"),
    ("Toronto/Canada", "EST"),
    ("Bangui/Central African Republic", "WAT"),
    ("N'Djamena/Chad", "WAT"),
    ("Santiago/Chile", "CLT"),
    ("Beijing/China", "CST"),
    ("Bogotá/Colombia", "COT"),
    ("Moroni/Comoros", "EAT"),
    ("Brazzaville/Congo", "WAT"),
    ("San José/Costa Rica", "CST"),
    ("Zagreb/Croatia", "CET"),
    ("Havana/Cuba", "CST"),
    ("Nicosia/Cyprus", "EET"),
    ("Prague/Czech Republic", "CET"),
    ("Copenhagen/Denmark", "CET"),
    ("Djibouti/Djibouti", "EAT"),
    ("Roseau/Dominica", "AST"),
    ("Santo Domingo/Dominican Republic", "AST"),
    ("Kinshasa/DR Congo", "WAT"),
    ("Guayaquil/Ecuador", "ECT"),
    ("Cairo/Egypt", "EET"),
    ("San Salvador/El Salvador", "CST"),
    ("Malabo/Equatorial Guinea", "WAT"),
    ("Asmara/Eritrea", "EAT"),
    ("Tallinn/Estonia", "EET"),
    ("Mbabane/Eswatini", "SAST"),
    ("Addis Ababa/Ethiopia", "EAT"),
    ("Suva/Fiji", "FJT"),
    ("Helsinki/Finland", "EET"),
    ("Paris/France", "CET"),
    ("Libreville/Gabon", "WAT"),
    ("Banjul/Gambia", "GMT"),
    ("Tbilisi/Georgia", "GET"),
    ("Berlin/Germany", "CET"),
    ("Accra/Ghana", "GMT"),
    ("Athens/Greece", "EET"),
    ("St. George's/Grenada", "AST"),
    ("Guatemala City/Guatemala", "CST"),
    ("Conakry/Guinea", "GMT"),
    ("Bissau/Guinea-Bissau", "GMT"),
    ("Georgetown/Guyana", "GYT"),
    ("Port-au-Prince/Haiti", "EST"),
    ("Tegucigalpa/Honduras", "CST"),
    ("Budapest/Hungary", "CET"),
    ("Reykjavik/Iceland", "GMT"),
    ("New Delhi/India", "IST"),
    ("Jakarta/Indonesia", "WIB"),
    ("Tehran/Iran", "IRST"),
    ("Baghdad/Iraq", "AST"),
    ("Dublin/Ireland", "GMT"),
    ("Jerusalem/Israel", "IST"),
    ("Rome/Italy", "CET"),
    ("Kingston/Jamaica", "EST"),
    ("Tokyo/Japan", "JST"),
    ("Amman/Jordan", "EET"),
    ("Almaty/Kazakhstan", "ALMT"),
    ("Nairobi/Kenya", "EAT"),
    ("Tarawa/Kiribati", "GILT"),
    ("Kuwait City/Kuwait", "AST"),
    ("Bishkek/Kyrgyzstan", "KGT"),
    ("Vientiane/Laos", "ICT"),
    ("Riga/Latvia", "EET"),
    ("Beirut/Lebanon", "EET"),
    ("Maseru/Lesotho", "SAST"),
    ("Monrovia/Liberia", "GMT"),
    ("Tripoli/Libya", "EET"),
    ("Vaduz/Liechtenstein", "CET"),
    ("Vilnius/Lithuania", "EET"),
    ("Luxembourg/Luxembourg", "CET"),
    ("Antananarivo/Madagascar", "EAT"),
    ("Blantyre/Malawi", "CAT"),
    ("Kuala Lumpur/Malaysia", "MYT"),
    ("Malé/Maldives", "MVT"),
    ("Bamako/Mali", "GMT"),
    ("Valletta/Malta", "CET"),
    ("Majuro/Marshall Islands", "MHT"),
    ("Nouakchott/Mauritania", "GMT"),
    ("Port Louis/Mauritius", "MUT"),
    ("Mexico City/Mexico", "CST"),
    ("Pohnpei/Micronesia", "PONT"),
    ("Chișinău/Moldova", "EET"),
    ("Monaco/Monaco", "CET"),
    ("Ulaanbaatar/Mongolia", "ULAT"),
    ("Podgorica/Montenegro", "CET"),
    ("Rabat/Morocco", "WET"),
    ("Maputo/Mozambique", "CAT"),
    ("Yangon/Myanmar", "MMT"),
    ("Windhoek/Namibia", "CAT"),
    ("Nauru/Nauru", "NRT"),
    ("Kathmandu/Nepal", "NPT"),
    ("Amsterdam/Netherlands", "CET"),
    ("Auckland/New Zealand", "NZDT"),
    ("Managua/Nicaragua", "CST"),
    ("Niamey/Niger", "WAT"),
    ("Lagos/Nigeria", "WAT"),
    ("Pyongyang/North Korea", "KST"),
    ("Skopje/North Macedonia", "CET"),
    ("Oslo/Norway", "CET"),
    ("Muscat/Oman", "GST"),
    ("Karachi/Pakistan", "PKT"),
    ("Ngerulmud/Palau", "PWT"),
    ("Hebron/Palestine", "EET"),
    ("Panama City/Panama", "EST"),
    ("Port Moresby/Papua New Guinea", "PGT"),
    ("Asunción/Paraguay", "PYT"),
    ("Lima/Peru", "PET"),
    ("Manila/Philippines", "PHT"),
    ("Warsaw/Poland", "CET"),
    ("Lisbon/Portugal", "WET"),
    ("Doha/Qatar", "AST"),
    ("Bucharest/Romania", "EET"),
    ("Moscow/Russia", "MSK"),
    ("Kigali/Rwanda", "CAT"),
    ("Basseterre/Saint Kitts and Nevis", "AST"),
    ("Castries/Saint Lucia", "AST"),
    ("Kingstown/Saint Vincent and the Grenadines", "AST"),
    ("Apia/Samoa", "WST"),
    ("San Marino/San Marino", "CET"),
    ("Riyadh/Saudi Arabia", "AST"),
    ("Dakar/Senegal", "GMT"),
    ("Belgrade/Serbia", "CET"),
    ("Victoria/Seychelles", "SCT"),
    ("Freetown/Sierra Leone", "GMT"),
    ("Singapore/Singapore", "SGT"),
    ("Bratislava/Slovakia", "CET"),
    ("Ljubljana/Slovenia", "CET"),
    ("Honiara/Solomon Islands", "SBT"),
    ("Mogadishu/Somalia", "EAT"),
    ("Johannesburg/South Africa", "SAST"),
    ("Seoul/South Korea", "KST"),
    ("Juba/South Sudan", "EAT"),
    ("Madrid/Spain", "CET"),
    ("Colombo/Sri Lanka", "IST"),
    ("Khartoum/Sudan", "CAT"),
    ("Paramaribo/Suriname", "SRT"),
    ("Stockholm/Sweden", "CET"),
    ("Zurich/Switzerland", "CET"),
    ("Damascus/Syria", "EET"),
    ("Taipei/Taiwan", "CST"),
    ("Dushanbe/Tajikistan", "TJT"),
    ("Dar es Salaam/Tanzania", "EAT"),
    ("Bangkok/Thailand", "ICT"),
    ("Dili/Timor-Leste", "TLT"),
    ("Lomé/Togo", "GMT"),
    ("Nuku'alofa/Tonga", "TOT"),
    ("Port of Spain/Trinidad and Tobago", "AST"),
    ("Tunis/Tunisia", "CET"),
    ("Istanbul/Turkey", "TRT"),
    ("Ashgabat/Turkmenistan", "TMT"),
    ("Funafuti/Tuvalu", "TVT"),
    ("Kampala/Uganda", "EAT"),
    ("Kyiv/Ukraine", "EET"),
    ("Dubai/United Arab Emirates", "GST"),
    ("London/United Kingdom", "GMT"),
    ("New York/United States", "EST"),
    ("Montevideo/Uruguay", "UYT"),
    ("Tashkent/Uzbekistan", "UZT"),
    ("Port Vila/Vanuatu", "VUT"),
    ("Vatican City/Vatican", "CET"),
    ("Caracas/Venezuela", "VET"),
    ("Ho Chi Minh City/Vietnam", "ICT"),
    ("Aden/Yemen", "AST"),
    ("Lusaka/Zambia", "CAT"),
    ("Harare/Zimbabwe", "CAT"),
];

// ==================================================================================
// LOGIC: PARSING & DETECTION
// ==================================================================================

/// Static Regex Patterns using OnceLock for performance
static IANA_REGEX: OnceLock<Regex> = OnceLock::new();
static FORMATTED_DATE_REGEX: OnceLock<Regex> = OnceLock::new();
static TIME_FORMAT_REGEX: OnceLock<Regex> = OnceLock::new();
static WHITESPACE_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_iana_regex() -> &'static Regex {
    IANA_REGEX.get_or_init(|| Regex::new(r"(?i)\b([A-Za-z]+/[A-Za-z_]+)\b").unwrap())
}

fn get_formatted_date_regex() -> &'static Regex {
    FORMATTED_DATE_REGEX.get_or_init(|| Regex::new(r"(?i)\d{1,2}\s+(jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)").unwrap())
}

fn get_time_format_regex() -> &'static Regex {
    TIME_FORMAT_REGEX.get_or_init(|| Regex::new(r"(?i)\d{1,2}:\d{2}\s*(am|pm)").unwrap())
}

fn get_whitespace_regex() -> &'static Regex {
    WHITESPACE_REGEX.get_or_init(|| Regex::new(r"\s+").unwrap())
}

/// Detect if text is a conversion result to avoid re-parsing
fn is_conversion_result(text: &str) -> bool {
    // Pattern 1: Contains " - " separator with timezone format
    let has_dash_separator = text.contains(" - ");
    let has_timezone_format = text.contains('/') && text.contains('(') && text.contains(')');
    
    // Pattern 2: Contains formatted date like "04 dec" or "04 Dec"
    let has_formatted_date = get_formatted_date_regex().is_match(text);
    
    // Pattern 3: Contains time with am/pm and formatted date
    let has_time_format = get_time_format_regex().is_match(text);
    
    // If it has the dash separator AND either timezone format OR (time format AND date format)
    has_dash_separator && (has_timezone_format || (has_time_format && has_formatted_date))
}

/// Helper: Check if `word` exists in `text` as a whole word (surrounded by boundaries)
fn has_whole_word(text: &str, word: &str) -> bool {
    let text_len = text.len();
    let word_len = word.len();
    
    if word_len == 0 || word_len > text_len {
        return false;
    }
    
    for (idx, _) in text.match_indices(word) {
        // Check character before
        let boundary_start = if idx == 0 {
            true
        } else {
            let char_before = text[..idx].chars().last();
             match char_before {
                Some(c) => !c.is_alphanumeric(),
                None => true,
            }
        };
        
        // Check character after
        let boundary_end = if idx + word_len >= text_len {
            true
        } else {
            let char_after = text[idx+word_len..].chars().next();
            match char_after {
                Some(c) => !c.is_alphanumeric(),
                None => true,
            }
        };
        
        if boundary_start && boundary_end {
            return true;
        }
    }
    false
}

/// Detect timezone from text
/// Returns: Option<(iana_id, matched_keyword)>
fn detect_timezone_from_text(text: &str) -> Option<(String, Option<String>)> {
    let text_lower = text.to_lowercase();
    
    if is_conversion_result(text) {
        return None;
    }
    
    // Strategy 1: Check for IANA timezone IDs
    if let Some(caps) = get_iana_regex().captures(text) {
        if let Some(iana_match) = caps.get(1) {
            let iana_id = iana_match.as_str();
            for (_, id, _) in ALL_TIMEZONES {
                if id.eq_ignore_ascii_case(iana_id) {
                    return Some((id.to_string(), None));
                }
            }
        }
    }
    
    // Strategy 2: Check for timezone abbreviations using static map
    let preferred_timezones = [
        "America/New_York", // EST/EDT
        "America/Los_Angeles", // PST/PDT
        "America/Chicago", // CST/CDT
        "America/London", // GMT/BST
        "Europe/London",
        "Europe/Paris", // CET
        "Asia/Tokyo", // JST
    ];

    let mut candidates = Vec::new();

    for (label, abbr) in TIMEZONE_ABBREVIATIONS_MAP {
       // Check for whole word match without Regex overhead
       if has_whole_word(&text_lower, &abbr.to_lowercase()) {
             // Abbreviation matched! Find IANA ID.
             if let Some((_, iana_id, _)) = ALL_TIMEZONES.iter().find(|(l, _, _)| l == label) {
                 candidates.push((iana_id.to_string(), Some(abbr.to_string())));
             }
       }
    }
    
    // Sort candidates to prioritize preferred timezones
    if !candidates.is_empty() {
        candidates.sort_by(|(id_a, _), (id_b, _)| {
            let rank_a = preferred_timezones.iter().position(|&p| p == id_a).unwrap_or(999);
            let rank_b = preferred_timezones.iter().position(|&p| p == id_b).unwrap_or(999);
            rank_a.cmp(&rank_b)
        });
        return Some(candidates[0].clone());
    }
    
    // Strategy 3: Check for city/country names
    for (label, iana_id, keywords) in ALL_TIMEZONES {
        let label_lower = label.to_lowercase();
        
        if text_lower.contains(&label_lower) {
            return Some((iana_id.to_string(), Some(label.to_string())));
        }
        
        // Optimize keyword search
        for keyword in keywords.split_whitespace() {
            let kw_lower = keyword.to_lowercase();
            
            if kw_lower.len() > 3 {
                 if text_lower.contains(&kw_lower) {
                     // Fast path for long keywords
                     let original_keyword = keywords
                        .split_whitespace()
                        .find(|k| k.to_lowercase() == kw_lower)
                        .unwrap_or(keyword);
                     return Some((iana_id.to_string(), Some(original_keyword.to_string()))); 
                 }
            } else {
                 // Strict match for short keywords
                 if has_whole_word(&text_lower, &kw_lower) {
                    let original_keyword = keywords
                        .split_whitespace()
                        .find(|k| k.to_lowercase() == kw_lower)
                        .unwrap_or(keyword);
                    return Some((iana_id.to_string(), Some(original_keyword.to_string())));
                 }
            }
        }
    }
    
    None
}

fn extract_time_portion(text: &str, detected_timezone: &Option<String>) -> String {
    let mut cleaned_text = text.to_string();
    
    if let Some(tz) = detected_timezone {
        // Remove the IANA ID
        let tz_regex = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(tz))).ok();
        if let Some(re) = tz_regex {
             cleaned_text = re.replace_all(&cleaned_text, "").to_string();
        }

        // Remove timezone abbreviations
        for (abbr, _) in TIMEZONE_ABBREVIATIONS_MAP {
             if let Ok(re) = Regex::new(&format!(r"(?i)\b{}\b", abbr)) {
                 cleaned_text = re.replace_all(&cleaned_text, "").to_string();
             }
        }
        
        // Remove the detected timezone's label and keywords
        for (label, iana_id, keywords) in ALL_TIMEZONES {
            if iana_id == tz {
                 if let Ok(re) = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(label))) {
                     cleaned_text = re.replace_all(&cleaned_text, "").to_string();
                 }
                
                for keyword in keywords.split_whitespace() {
                    // Clean ALL keywords associated with this timezone
                     if let Ok(re) = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(keyword))) {
                         cleaned_text = re.replace_all(&cleaned_text, "").to_string();
                     }
                }
                break;
            }
        }
        
        // Remove common phrases
        let phrases = vec![r"(?i)\bin\s+\w+", r"(?i)\w+\s+time"];
        for pattern in phrases {
             if let Ok(re) = Regex::new(pattern) {
                 cleaned_text = re.replace_all(&cleaned_text, "").to_string();
             }
        }
    }
    
    cleaned_text = cleaned_text.trim().to_string();
    cleaned_text = get_whitespace_regex().replace_all(&cleaned_text, " ").to_string();
    
    if cleaned_text.is_empty() {
        "now".to_string()
    } else {
        cleaned_text
    }
}

pub fn parse_time_from_text(text: &str) -> Option<ParsedTimeInput> {
    if is_conversion_result(text) {
        return None;
    }
    
    let detected_result = detect_timezone_from_text(text);
    let (detected_timezone, matched_keyword) = match detected_result {
        Some((tz, kw)) => (Some(tz), kw),
        None => (None, None),
    };
    
    let time_input = extract_time_portion(text, &detected_timezone);
    
    Some(ParsedTimeInput {
        time_input,
        source_timezone: detected_timezone,
        matched_keyword,
    })
}

// ==================================================================================
// HELPERS
// ==================================================================================

/// Get timezone abbreviation for a given IANA ID
pub fn get_timezone_abbreviation(iana_id: &str) -> &str {
    // First, find the display label for this IANA ID
    let display_label = ALL_TIMEZONES
        .iter()
        .find(|(_, id, _)| *id == iana_id)
        .map(|(label, _, _)| *label);
    
    // Then look up the abbreviation using the display label
    if let Some(label) = display_label {
        TIMEZONE_ABBREVIATIONS_MAP
            .iter()
            .find(|(map_label, _)| *map_label == label)
            .map(|(_, abbr)| *abbr)
            .unwrap_or("UTC")
    } else {
        "UTC"
    }
}

fn format_utc_offset(offset_seconds: i32) -> String {
    let hours = offset_seconds / 3600;
    let minutes = (offset_seconds % 3600).abs() / 60;
    format!("UTC{:+03}:{:02}", hours, minutes)
}

fn format_timezone_label(iana_id: &str, display_label: &str) -> String {
    let abbr = get_timezone_abbreviation(iana_id);
    format!("{} ({})", display_label, abbr)
}

fn format_timezone_label_with_abbr(iana_id: &str, abbr: &str) -> String {
    // Find the display label from constants
    let display_label = ALL_TIMEZONES
        .iter()
        .find(|(_, id, _)| *id == iana_id)
        .map(|(label, _, _)| *label)
        .unwrap_or("Unknown");
    
    format!("{} ({})", display_label, abbr)
}

pub fn get_all_timezones() -> Vec<TimezoneInfo> {
    ALL_TIMEZONES
        .iter()
        .map(|(country_label, iana_id, keywords)| {
            let formatted_label = format_timezone_label(iana_id, country_label);
            TimezoneInfo {
                label: formatted_label,
                iana_id: iana_id.to_string(),
                keywords: keywords.to_string(),
            }
        })
        .collect()
}

pub fn generate_timezone_commands() -> Vec<CommandItem> {
    ALL_TIMEZONES
        .iter()
        .map(|(label, iana_id, _keywords)| {
            let formatted_label = format_timezone_label(iana_id, label);
            CommandItem {
                id: format!("convert_time_{}", iana_id.to_lowercase().replace('/', "_")),
                label: format!("Convert time to {}", formatted_label),
                description: None,
                action_type: Some(ActionType::ConvertTimeAction(TimePayload {
                    target_timezone: iana_id.to_string(),
                })),
                widget_type: None,
                category: None,
            }
        })
        .collect()
}

// ==================================================================================
// FEATURE IMPLEMENTATION
// ==================================================================================

#[derive(Clone)]
pub struct TimeConverterFeature;

impl FeatureSync for TimeConverterFeature {
    fn id(&self) -> &str {
        "time_converter"
    }
    
    fn widget_commands(&self) -> Vec<CommandItem> {
        vec![CommandItem {
            id: "widget_time_converter".to_string(),
            label: "Time Zone Converter".to_string(),
            description: Some("Open time zone converter widget".to_string()),
            action_type: None,
            widget_type: Some("time_converter".to_string()),
            category: None,
        }]
    }
    
    fn action_commands(&self) -> Vec<CommandItem> {
        generate_timezone_commands()
    }
    
    fn get_context_boost(&self, _captured_text: &str) -> HashMap<String, f64> {
        HashMap::new()
    }
}

#[async_trait]
impl FeatureAsync for TimeConverterFeature {
    async fn execute_action(
        &self,
        action: &ActionType,
        params: &serde_json::Value,
    ) -> crate::shared::error::AppResult<ExecuteActionResponse> {
        match action {
            ActionType::ConvertTimeAction(payload) => {
                let text_input = params.get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("now");
                
                let parsed = parse_time_from_text(text_input)
                    .ok_or_else(|| crate::shared::error::AppError::Validation("Failed to parse time from text, likely a conversion result.".to_string()))?;
                
                let request = ConvertTimeRequest {
                    time_input: parsed.time_input,
                    target_timezone: payload.target_timezone.clone(),
                    source_timezone: parsed.source_timezone,
                    matched_keyword: parsed.matched_keyword,
                };
                
                let response = parse_and_convert_time(request)?;
                let formatted_result = response.target_time;
                
                Ok(ExecuteActionResponse {
                    result: formatted_result,
                    metadata: Some(serde_json::json!({
                        "offset_description": response.offset_description,
                        "source_timezone": response.source_timezone,
                        "target_timezone": response.target_timezone,
                    })),
                })
            }
            _ => Err(crate::shared::error::AppError::Unknown(crate::shared::errors::ERR_UNSUPPORTED_ACTION.to_string())),
        }
    }
}

/// Parse natural language time input and convert to target timezone
pub fn parse_and_convert_time(request: ConvertTimeRequest) -> crate::shared::error::AppResult<ConvertTimeResponse> {
    let source_tz_str = request.source_timezone.as_deref().unwrap_or("UTC");
    let now = Local::now();
    
    let parsed_local_dt = parse_date_string(&request.time_input, now, Dialect::Us)
        .map_err(|e| crate::shared::error::AppError::Validation(format!("Failed to parse time input '{}': {}", request.time_input, e)))?;
    
    // Parse source timezone
    let source_tz: Tz = source_tz_str.parse()
        .map_err(|_| crate::shared::error::AppError::Validation(format!("Invalid source timezone: {}", source_tz_str)))?;
    
    // Interpret this naive datetime AS BEING IN the source timezone
    let naive = parsed_local_dt.naive_local();
    let source_dt = source_tz.from_local_datetime(&naive)
        .single()
        .ok_or_else(|| crate::shared::error::AppError::Validation(format!("Ambiguous or invalid time in timezone {}", source_tz_str)))?;
    
    // Parse target timezone
    let target_tz: Tz = request.target_timezone.parse()
        .map_err(|e| crate::shared::error::AppError::Validation(format!("Invalid target timezone '{}': {:?}", request.target_timezone, e)))?;
    
    // Convert
    let target_dt = source_dt.with_timezone(&target_tz);
    
    let source_offset_seconds = source_dt.offset().fix().local_minus_utc();
    let target_offset_seconds = target_dt.offset().fix().local_minus_utc();
    
    let source_utc_offset = format_utc_offset(source_offset_seconds);
    let target_utc_offset = format_utc_offset(target_offset_seconds);
    
    let source_zone_abbr = source_dt.format("%Z").to_string();
    let target_zone_abbr = target_dt.format("%Z").to_string();
    
    let source_label = format_timezone_label_with_abbr(&source_tz_str, &source_zone_abbr);
    let target_label = format_timezone_label_with_abbr(&request.target_timezone, &target_zone_abbr);
    
    let diff_seconds = target_offset_seconds - source_offset_seconds;
    let abs_diff_seconds = diff_seconds.abs();
    let hours = abs_diff_seconds / 3600;
    let minutes = (abs_diff_seconds % 3600) / 60;
    let sign = if diff_seconds >= 0 { "+" } else { "-" };
    
    let relative_offset = if diff_seconds == 0 {
        "Same time".to_string()
    } else {
        format!("{}{}h {}m", sign, hours, minutes)
    };
    
    let source_date = source_dt.date_naive();
    let target_date = target_dt.date_naive();
    let date_change_indicator = if target_date > source_date {
        Some("Next day".to_string())
    } else if target_date < source_date {
        Some("Previous day".to_string())
    } else {
        None
    };
    
    let offset_hours = target_offset_seconds as f64 / 3600.0;
    let source_offset_hours = source_offset_seconds as f64 / 3600.0;
    let diff_hours = offset_hours - source_offset_hours;
    
    let mut offset_description = if diff_hours > 0.0 {
        format!("{:.1} hours ahead", diff_hours)
    } else if diff_hours < 0.0 {
        format!("{:.1} hours behind", diff_hours.abs())
    } else {
        "Same time".to_string()
    };
    
    // SMART CITY DETECTION logic
    if let Some(ref keyword) = request.matched_keyword {
        if request.source_timezone.is_some() {
            for (display_label, iana_id, _keywords) in ALL_TIMEZONES {
                if iana_id == &source_tz_str {
                    let primary_city = display_label
                        .split('/')
                        .next()
                        .unwrap_or(display_label)
                        .to_lowercase();
                    
                    let keyword_lower = keyword.to_lowercase();
                    if keyword_lower != primary_city && keyword_lower.len() > 3 {
                         let keyword_capitalized = if let Some(first_char) = keyword.chars().next() {
                            first_char.to_uppercase().collect::<String>() + &keyword[1..]
                        } else {
                            keyword.clone()
                        };
                        offset_description = format!("{} • Uses the same timezone as {}", offset_description, keyword_capitalized);
                    }
                    break;
                }
            }
        }
    }
    
    let source_formatted = source_dt.format("%I:%M%P, %d %b").to_string();
    let target_formatted = target_dt.format("%I:%M%P, %d %b").to_string();
    
    Ok(ConvertTimeResponse {
        source_time: source_formatted,
        target_time: target_formatted,
        offset_description,
        source_timezone: source_label,
        target_timezone: target_label,
        target_utc_offset,
        target_zone_abbr,
        relative_offset,
        date_change_indicator,
        source_zone_abbr,
        source_utc_offset,
    })
}

// ==================================================================================
// COMMANDS
// ==================================================================================

#[tauri::command]
pub async fn parse_time_from_selection(text: String) -> crate::shared::error::AppResult<ParsedTimeInput> {
    parse_time_from_text(&text)
        .ok_or_else(|| crate::shared::error::AppError::Validation("Failed to parse time from selection, likely a conversion result.".to_string()))
}

#[tauri::command]
pub async fn get_system_timezone() -> crate::shared::error::AppResult<String> {
    match iana_time_zone::get_timezone() {
        Ok(tz) => Ok(tz),
        Err(_) => Ok("UTC".to_string())
    }
}

#[tauri::command]
pub async fn convert_time(request: ConvertTimeRequest) -> crate::shared::error::AppResult<ConvertTimeResponse> {
    parse_and_convert_time(request)
}

#[tauri::command]
pub async fn get_timezones() -> crate::shared::error::AppResult<Vec<TimezoneInfo>> {
    Ok(get_all_timezones())
}