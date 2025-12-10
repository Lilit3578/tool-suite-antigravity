/**
 * Text manipulation utilities
 */

/**
 * Strips HTML tags from a string to create a plain text preview.
 * This is a simple implementation for preview generation.
 * 
 * @param html The HTML string to strip
 * @returns Plain text content
 */
export function stripHtmlTags(html: string): string {
    let result = "";
    let inTag = false;

    for (let i = 0; i < html.length; i++) {
        const c = html[i];
        if (c === '<') {
            inTag = true;
        } else if (c === '>') {
            inTag = false;
        } else if (!inTag) {
            result += c;
        }
    }

    return result.trim();
}

/**
 * Truncates text to a maximum length, appending ellipsis if needed.
 * 
 * @param text The text to truncate
 * @param maxLength Maximum length
 * @returns Truncated text
 */
export function truncateText(text: string, maxLength: number): string {
    if (text.length <= maxLength) {
        return text;
    }
    return text.substring(0, maxLength) + "...";
}
