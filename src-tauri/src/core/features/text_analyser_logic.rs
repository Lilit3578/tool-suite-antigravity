use unicode_segmentation::UnicodeSegmentation;
use crate::shared::types::TextAnalysisResponse;

/// Perform text analysis (pure logic)
///
/// This function is CPU-bound but generally fast enough for typical input sizes.
/// For very large text, it should be run in `spawn_blocking`.
pub fn perform_analysis(text: &str) -> TextAnalysisResponse {
    let word_count = text.unicode_words().count();
    let char_count = text.chars().count();
    let char_count_no_spaces = text.chars().filter(|c| !c.is_whitespace()).count();
    let grapheme_count = text.graphemes(true).count();
    let line_count = text.lines().count();
    
    // Average reading speed: 200 wpm
    // words / 200 = minutes
    // minutes * 60 = seconds
    let reading_time_sec = if word_count > 0 {
        (word_count as f64 / 200.0) * 60.0
    } else {
        0.0
    };

    TextAnalysisResponse {
        word_count,
        char_count,
        char_count_no_spaces,
        grapheme_count,
        line_count,
        reading_time_sec,
    }
}
