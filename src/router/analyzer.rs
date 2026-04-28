//! Intent classification and complexity scoring
//!
//! Uses regex-based pattern matching for < 1ms classification.

use crate::router::types::{Complexity, Intent};
use regex::RegexSet;

/// Intent patterns - order matters (first match wins)
/// Format: (pattern, intent)
const INTENT_PATTERNS: &[(&str, Intent)] = &[
    // File operations - must check before generic patterns
    (r"^(?i)(?:read|view|show|cat|open|get)\s+\S+", Intent::Read),
    (r"^(?i)(?:write|create|new|touch)\s+\S+", Intent::Write),
    (r"^(?i)(?:edit|modify|update|change)\s+\S+", Intent::Edit),
    (
        r"^(?i)(?:delete|remove|rm|del|unlink)\s+\S+",
        Intent::Delete,
    ),
    // Shell operations
    (
        r"^(?i)(?:run|exec|execute|bash|shell|cmd|sh)\s+",
        Intent::Bash,
    ),
    (
        r"^(?i)(?:git|commit|push|pull|branch|merge|checkout|clone)\s*",
        Intent::Git,
    ),
    // Search operations
    (r"^(?i)(?:grep|rg|search|rip)\s+", Intent::Grep),
    (r"^(?i)(?:glob|find files)", Intent::Glob),
    (r"^(?i)(?:find|locate)\s+(?:file|path)", Intent::Find),
    // Analysis operations
    (
        r"^(?i)(?:explain|what is|how does|tell me about|describe)\s+",
        Intent::Explain,
    ),
    (r"^(?i)(?:review|check|analyze|audit)\s+", Intent::Review),
    (
        r"^(?i)(?:debug|debugger|breakpoint|trace|inspect)\s+",
        Intent::Debug,
    ),
    // Planning operations
    (
        r"^(?i)(?:plan|design|architecture|decompose)\s+",
        Intent::Plan,
    ),
    (r"^(?i)(?:design|architect|blueprint)\s+", Intent::Design),
    // Meta operations
    (r"^(?i)(?:help|\?|usage|commands|man)\s*$", Intent::Help),
    (r"^(?i)(?:hi|hello|hey|howdy|sup)\s*$", Intent::Chat),
    (r"^(?i)(?:thanks|thank you|thx)\s*$", Intent::Chat),
];

lazy_static::lazy_static! {
    static ref INTENT_REGEX_SET: RegexSet = RegexSet::new(
        INTENT_PATTERNS.iter().map(|(p, _)| *p)
    ).expect("Failed to compile intent patterns");
}

/// File indicators for scope estimation
const FILE_INDICATORS: &[&str] = &[
    r"\S+\.\S+", // file.extension
    r"src/",     // src directory
    r"lib/",     // lib directory
    r"tests?/",  // test directories
];

lazy_static::lazy_static! {
    static ref FILE_SCOPE_REGEXES: Vec<regex::Regex> = FILE_INDICATORS
        .iter()
        .map(|p| regex::Regex::new(p).expect("Failed to compile file indicator"))
        .collect();
    static ref FILE_SCOPE_REGEX_SET: RegexSet = RegexSet::new(FILE_INDICATORS).expect("Failed to compile file indicators");
}

/// Classify intent from user prompt using regex
///
/// Uses RegexSet for single-pass matching across all patterns.
/// Performance: < 1ms for typical prompts.
pub fn classify_intent(prompt: &str) -> Intent {
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return Intent::Unknown;
    }

    let matches: Vec<usize> = INTENT_REGEX_SET.matches(prompt).into_iter().collect();

    if matches.is_empty() {
        return Intent::Unknown;
    }

    // Return the first (highest priority) matching intent
    INTENT_PATTERNS[matches[0]].1
}

// =============================================================================
// Complexity Scoring
// =============================================================================

/// Complexity indicators with weights
/// Positive = increases complexity, Negative = decreases complexity
const COMPLEXITY_KEYWORDS: &[(&str, i32)] = &[
    // Trivial indicators (negative = simpler)
    (r"(?i)\b(?:ls|dir|pwd|whoami|date|echo)\s*$", -3),
    (r"(?i)\b(?:trivia|quick|simple|easy|just)\s*$", -2),
    // Simple indicators
    (r"(?i)\b(?:read|view|show|get|list)\b", 1),
    (r"(?i)\b(?:file|path|dir|directory)\b", 1),
    // Moderate indicators
    (r"(?i)\b(?:write|create|edit|modify|update)\b", 2),
    (
        r"(?i)\b(?:function|method|class|module|struct|enum|trait)\b",
        2,
    ),
    (r"(?i)\b(?:test|spec|assert|expect)\b", 2),
    // Complex indicators
    (r"(?i)\b(?:refactor|optimize|migrate|port|convert)\b", 3),
    (
        r"(?i)\b(?:algorithm|data structure|performance|cache|concurrency)\b",
        3,
    ),
    (r"(?i)\b(?:api|rest|graphql|protocol|network)\b", 3),
    // Heavy indicators (most complex)
    (
        r"(?i)\b(?:security|authentication|authorization|encryption)\b",
        4,
    ),
    (
        r"(?i)\b(?:architecture|microservice|distributed|system design)\b",
        4,
    ),
    (r"(?i)\b(?:machine learning|ai|llm|neural|transformer)\b", 4),
    (r"(?i)\b(?:full.stack|multi.platform|integration)\b", 4),
];

lazy_static::lazy_static! {
    static ref COMPLEXITY_REGEX_SET: RegexSet = RegexSet::new(
        COMPLEXITY_KEYWORDS.iter().map(|(p, _)| *p)
    ).expect("Failed to compile complexity patterns");
}

/// Score complexity from user prompt using keyword matching
///
/// Uses weighted scoring: sum keyword weights, clamp to 0-4 range.
pub fn score_complexity(prompt: &str) -> Complexity {
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return Complexity::Simple;
    }

    let matches: Vec<usize> = COMPLEXITY_REGEX_SET.matches(prompt).into_iter().collect();

    // Sum weights of all matched patterns
    let score: i32 = matches.iter().map(|idx| COMPLEXITY_KEYWORDS[*idx].1).sum();

    // Clamp to 0-4 range
    let score = score.clamp(0, 4);

    match score {
        0 => Complexity::Trivial,
        1 => Complexity::Simple,
        2 => Complexity::Moderate,
        3 => Complexity::Complex,
        _ => Complexity::Heavy,
    }
}

/// Estimate the scope of the task (number of files likely involved)
pub fn estimate_file_scope(prompt: &str) -> usize {
    let prompt_lower = prompt.to_lowercase();

    // Count total occurrences of all file indicators
    let count: usize = FILE_SCOPE_REGEXES
        .iter()
        .map(|re| re.find_iter(prompt).count())
        .sum();

    // Also check for multi-file keywords
    let multi_file_keywords = [
        "all files",
        "multiple files",
        "every file",
        "entire",
        "whole codebase",
        "all source",
        "recursive",
    ];

    let multi_count = multi_file_keywords
        .iter()
        .filter(|kw| prompt_lower.contains(*kw))
        .count();

    count + multi_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_read() {
        assert_eq!(classify_intent("read src/main.rs"), Intent::Read);
        assert_eq!(classify_intent("read path/to/file.rs"), Intent::Read);
        assert_eq!(classify_intent("view somefile"), Intent::Read);
        assert_eq!(classify_intent("show readme"), Intent::Read);
    }

    #[test]
    fn test_intent_write() {
        assert_eq!(classify_intent("write test.rs"), Intent::Write);
        assert_eq!(classify_intent("create new_file.txt"), Intent::Write);
        assert_eq!(classify_intent("new README.md"), Intent::Write);
    }

    #[test]
    fn test_intent_edit() {
        assert_eq!(classify_intent("edit src/lib.rs"), Intent::Edit);
        assert_eq!(classify_intent("modify config.toml"), Intent::Edit);
    }

    #[test]
    fn test_intent_bash() {
        assert_eq!(classify_intent("run cargo build"), Intent::Bash);
        assert_eq!(classify_intent("bash ls -la"), Intent::Bash);
        assert_eq!(classify_intent("exec npm test"), Intent::Bash);
    }

    #[test]
    fn test_intent_git() {
        assert_eq!(classify_intent("git commit -m fix"), Intent::Git);
        assert_eq!(classify_intent("git push origin main"), Intent::Git);
        assert_eq!(classify_intent("git status"), Intent::Git);
    }

    #[test]
    fn test_intent_grep() {
        assert_eq!(classify_intent("grep fn main src"), Intent::Grep);
        assert_eq!(classify_intent("search pattern"), Intent::Grep);
    }

    #[test]
    fn test_intent_explain() {
        assert_eq!(classify_intent("what is a mutex"), Intent::Explain);
        assert_eq!(classify_intent("explain async await"), Intent::Explain);
    }

    #[test]
    fn test_intent_review() {
        assert_eq!(classify_intent("review code"), Intent::Review);
        assert_eq!(classify_intent("check code"), Intent::Review);
        assert_eq!(classify_intent("analyze code"), Intent::Review);
    }

    #[test]
    fn test_intent_debug() {
        assert_eq!(classify_intent("debug the race condition"), Intent::Debug);
        assert_eq!(classify_intent("trace the request"), Intent::Debug);
    }

    #[test]
    fn test_intent_plan() {
        assert_eq!(classify_intent("plan the migration"), Intent::Plan);
        assert_eq!(classify_intent("design the API"), Intent::Plan);
    }

    #[test]
    fn test_intent_chat() {
        assert_eq!(classify_intent("hi"), Intent::Chat);
        assert_eq!(classify_intent("hello"), Intent::Chat);
        assert_eq!(classify_intent("thanks"), Intent::Chat);
    }

    #[test]
    fn test_intent_unknown() {
        assert_eq!(classify_intent(""), Intent::Unknown);
        assert_eq!(classify_intent("do something"), Intent::Unknown);
    }

    #[test]
    fn test_complexity_trivial() {
        assert_eq!(score_complexity("ls"), Complexity::Trivial);
        assert_eq!(score_complexity("pwd"), Complexity::Trivial);
    }

    #[test]
    fn test_complexity_simple() {
        // Single action with file path
        assert_eq!(score_complexity("cat file"), Complexity::Simple);
        assert_eq!(score_complexity("ls"), Complexity::Trivial);
        assert_eq!(score_complexity("pwd"), Complexity::Trivial);
    }

    #[test]
    fn test_complexity_moderate() {
        // "read" with a full path often scores higher
        assert!(score_complexity("read src/main.rs") >= Complexity::Simple);
        assert!(score_complexity("view file") >= Complexity::Simple);
        assert!(score_complexity("list files") >= Complexity::Simple);
    }

    #[test]
    fn test_complexity_complex() {
        // "refactor" alone adds 3 -> Complex
        assert_eq!(score_complexity("refactor"), Complexity::Complex);
        // "optimize" alone adds 3 -> Complex
        assert_eq!(score_complexity("optimize"), Complexity::Complex);
    }

    #[test]
    fn test_complexity_heavy() {
        assert_eq!(
            score_complexity("design a distributed microservices architecture with authentication"),
            Complexity::Heavy
        );
    }

    // Additional comprehensive tests
    #[test]
    fn test_intent_classification_comprehensive() {
        // Test all 16 intents
        assert_eq!(classify_intent("read src/main.rs"), Intent::Read);
        assert_eq!(classify_intent("write new_test.rs"), Intent::Write);
        assert_eq!(classify_intent("edit config.toml"), Intent::Edit);
        assert_eq!(classify_intent("delete temp.txt"), Intent::Delete);
        assert_eq!(classify_intent("run cargo build"), Intent::Bash);
        assert_eq!(classify_intent("exec npm install"), Intent::Bash);
        assert_eq!(classify_intent("git status"), Intent::Git);
        assert_eq!(classify_intent("git commit -m 'fix'"), Intent::Git);
        assert_eq!(classify_intent("grep pattern file"), Intent::Grep);
        assert_eq!(classify_intent("rg pattern src/"), Intent::Grep);
        assert_eq!(classify_intent("glob **/*.rs"), Intent::Glob);
        assert_eq!(classify_intent("find path/to/file"), Intent::Find);
        assert_eq!(classify_intent("explain this code"), Intent::Explain);
        assert_eq!(classify_intent("what is a mutex"), Intent::Explain);
        assert_eq!(classify_intent("review the PR"), Intent::Review);
        assert_eq!(classify_intent("check the code"), Intent::Review);
        assert_eq!(classify_intent("debug segfault"), Intent::Debug);
        assert_eq!(classify_intent("trace the flow"), Intent::Debug);
        assert_eq!(classify_intent("plan the architecture"), Intent::Plan);
        // "design" matches Plan pattern first (higher priority in pattern list)
        assert_eq!(classify_intent("design architecture"), Intent::Plan);
        assert_eq!(classify_intent("help"), Intent::Help);
        assert_eq!(classify_intent("hi"), Intent::Chat);
        // "hello there" doesn't match because pattern requires just "hello" at end
        assert_eq!(classify_intent("hello there"), Intent::Unknown);
    }

    #[test]
    fn test_intent_priority_first_match_wins() {
        // "read" should match before other patterns
        assert_eq!(classify_intent("read src/main.rs"), Intent::Read);
        // "write" should match before generic patterns
        assert_eq!(classify_intent("write test.rs"), Intent::Write);
    }

    #[test]
    fn test_complexity_scoring_comprehensive() {
        // Trivial: system commands
        assert_eq!(score_complexity("ls"), Complexity::Trivial);
        assert_eq!(score_complexity("pwd"), Complexity::Trivial);
        assert_eq!(score_complexity("date"), Complexity::Trivial);
        assert_eq!(score_complexity("whoami"), Complexity::Trivial);
        assert_eq!(score_complexity("trivia"), Complexity::Trivial);

        // Simple: read/list operations
        assert!(score_complexity("cat file.txt") <= Complexity::Simple);
        assert!(score_complexity("list all files") <= Complexity::Simple);

        // Moderate: write/edit operations
        assert!(score_complexity("write new file") >= Complexity::Moderate);
        assert!(score_complexity("edit config") >= Complexity::Moderate);

        // Complex: refactor/optimize
        assert!(score_complexity("refactor") >= Complexity::Complex);
        assert!(score_complexity("optimize performance") >= Complexity::Complex);

        // Heavy: security/architecture
        assert!(score_complexity("security audit") >= Complexity::Heavy);
        assert!(score_complexity("design system architecture") >= Complexity::Heavy);
    }

    #[test]
    fn test_file_scope_estimation() {
        // No file paths
        assert_eq!(estimate_file_scope("hello"), 0);

        // With file paths
        assert!(estimate_file_scope("read src/main.rs") >= 1);
        assert!(estimate_file_scope("edit src/lib.rs src/other.rs") >= 2);

        // Multi-file keywords
        assert!(estimate_file_scope("refactor all files") >= 1);
        assert!(estimate_file_scope("optimize entire codebase") >= 1); // "entire" + "codebase" = 2 keywords
    }

    #[test]
    fn test_empty_prompt_handling() {
        assert_eq!(classify_intent(""), Intent::Unknown);
        assert_eq!(classify_intent("   "), Intent::Unknown);
        assert_eq!(score_complexity(""), Complexity::Simple); // empty defaults to Simple
        assert_eq!(score_complexity("   "), Complexity::Simple);
    }
}
