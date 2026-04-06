//! Typed RPC payload enums for artifact operations.
//!
//! These enums represent the integer/string codes that Google's batchexecute
//! endpoint expects for artifact generation. Each variant maps 1:1 to the
//! values discovered via reverse engineering the NotebookLM web UI traffic.
//!
//! CRITICAL: The integer codes here are NOT arbitrary — they come from
//! reverse engineering. A single wrong code will cause a silent RPC failure.
//!
//! Reference: teng-lin/notebooklm-py (NotebookLmArtifactType, NotebookLmArtifactStatus)

use std::fmt;

// =========================================================================
// Core Type Enums
// =========================================================================

/// Internal artifact type code sent to Google's RPC.
/// Maps to the integer at position [2] of the artifact array.
///
/// NOTE: Type 4 (Quiz/Flashcards) is distinguished by `variant`, not type_code.
/// Type 5 (Mind Map) uses a COMPLETELY DIFFERENT RPC endpoint.
/// Type 6 appears UNUSED in the reference implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactTypeCode {
    Audio = 1,
    Report = 2,
    Video = 3,
    QuizFlashcards = 4, // Distinguished by variant: 1=Flashcards, 2=Quiz
    MindMap = 5,
    Infographic = 7,
    SlideDeck = 8,
    DataTable = 9,
}

impl ArtifactTypeCode {
    /// Convert to the integer code for wire format.
    pub fn code(self) -> i32 {
        self as i32
    }

    /// Parse from API integer response.
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            1 => Some(Self::Audio),
            2 => Some(Self::Report),
            3 => Some(Self::Video),
            4 => Some(Self::QuizFlashcards),
            5 => Some(Self::MindMap),
            7 => Some(Self::Infographic),
            8 => Some(Self::SlideDeck),
            9 => Some(Self::DataTable),
            _ => None,
        }
    }
}

/// Artifact generation/processing status.
/// Maps to the integer at position [4] of the artifact array.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactStatus {
    Processing = 1,
    Pending = 2,
    Completed = 3,
    Failed = 4,
}

impl ArtifactStatus {
    /// Convert to the integer code for wire format.
    pub fn code(self) -> i32 {
        self as i32
    }

    /// Parse from API integer response.
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            1 => Some(Self::Processing),
            2 => Some(Self::Pending),
            3 => Some(Self::Completed),
            4 => Some(Self::Failed),
            _ => None,
        }
    }
}

impl fmt::Display for ArtifactStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Processing => write!(f, "PROCESSING"),
            Self::Pending => write!(f, "PENDING"),
            Self::Completed => write!(f, "COMPLETED"),
            Self::Failed => write!(f, "FAILED"),
        }
    }
}

/// Result of an artifact generation request.
///
/// Wraps the task_id + initial status returned by CREATE_ARTIFACT,
/// plus optional error info for rate-limited or failed generations.
#[derive(Debug, Clone)]
pub struct GenerationStatus {
    /// The artifact/task ID returned by the API.
    pub task_id: String,
    /// The initial generation status.
    pub status: ArtifactStatus,
    /// Human-readable error message (present when rate-limited or failed).
    pub error: Option<String>,
    /// Machine-readable error code (e.g. "USER_DISPLAYABLE_ERROR" for rate limiting).
    pub error_code: Option<String>,
}

impl GenerationStatus {
    /// Create a successful generation status from task_id and status.
    pub fn new(task_id: String, status: ArtifactStatus) -> Self {
        Self {
            task_id,
            status,
            error: None,
            error_code: None,
        }
    }

    /// Create a rate-limited generation status.
    pub fn rate_limited(error: impl Into<String>) -> Self {
        let error = error.into();
        Self {
            task_id: "rate_limited".to_string(),
            status: ArtifactStatus::Failed,
            error: Some(error),
            error_code: Some("USER_DISPLAYABLE_ERROR".to_string()),
        }
    }

    /// Create a failed generation status with an error code.
    pub fn failed(
        task_id: String,
        error: impl Into<String>,
        error_code: impl Into<String>,
    ) -> Self {
        Self {
            task_id,
            status: ArtifactStatus::Failed,
            error: Some(error.into()),
            error_code: Some(error_code.into()),
        }
    }

    /// Whether this generation was rate-limited (retryable).
    pub fn is_rate_limited(&self) -> bool {
        self.error_code.as_deref() == Some("USER_DISPLAYABLE_ERROR")
    }

    /// Whether the artifact generation has completed successfully.
    pub fn is_complete(&self) -> bool {
        self.status == ArtifactStatus::Completed
    }

    /// Whether the artifact generation has failed.
    pub fn is_failed(&self) -> bool {
        self.status == ArtifactStatus::Failed
    }

    /// Whether the artifact is still being generated.
    pub fn is_in_progress(&self) -> bool {
        matches!(
            self.status,
            ArtifactStatus::Processing | ArtifactStatus::Pending
        )
    }
}

impl fmt::Display for GenerationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GenerationStatus(task_id={}, status={}",
            self.task_id, self.status
        )?;
        if let Some(ref code) = self.error_code {
            write!(f, ", error_code={}", code)?;
        }
        if let Some(ref err) = self.error {
            write!(f, ", error=\"{}\"", err)?;
        }
        write!(f, ")")
    }
}

/// Result of a mind map generation (two-step RPC).
///
/// Mind maps use GENERATE_MIND_MAP (yyryJe) + CREATE_NOTE (CYK0Xb),
/// returning both the parsed mind map data and the persisted note ID.
#[derive(Debug, Clone)]
pub struct MindMapResult {
    /// The note ID where the mind map was persisted.
    pub note_id: Option<String>,
    /// The parsed mind map JSON data.
    pub mind_map_data: Option<serde_json::Value>,
}

impl MindMapResult {
    /// Create a successful mind map result.
    pub fn new(note_id: String, mind_map_data: serde_json::Value) -> Self {
        Self {
            note_id: Some(note_id),
            mind_map_data: Some(mind_map_data),
        }
    }

    /// Create an empty/failed mind map result.
    pub fn empty() -> Self {
        Self {
            note_id: None,
            mind_map_data: None,
        }
    }
}

/// User-facing artifact type, resolved from type_code + variant.
/// This is what MCP tools expose to the LLM/user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactType {
    Audio,
    Video,
    Report,
    Quiz,
    Flashcards,
    MindMap,
    Infographic,
    SlideDeck,
    DataTable,
    Unknown,
}

impl ArtifactType {
    /// Resolve from API type_code and optional variant (for Quiz/Flashcards).
    /// variant is extracted from art[9][1][0].
    pub fn from_type_code_and_variant(type_code: ArtifactTypeCode, variant: Option<i32>) -> Self {
        match type_code {
            ArtifactTypeCode::Audio => Self::Audio,
            ArtifactTypeCode::Report => Self::Report,
            ArtifactTypeCode::Video => Self::Video,
            ArtifactTypeCode::QuizFlashcards => match variant {
                Some(1) => Self::Flashcards,
                Some(2) => Self::Quiz,
                _ => Self::Unknown,
            },
            ArtifactTypeCode::MindMap => Self::MindMap,
            ArtifactTypeCode::Infographic => Self::Infographic,
            ArtifactTypeCode::SlideDeck => Self::SlideDeck,
            ArtifactTypeCode::DataTable => Self::DataTable,
        }
    }

    /// String key for MCP tool output and CLI display.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Audio => "audio",
            Self::Video => "video",
            Self::Report => "report",
            Self::Quiz => "quiz",
            Self::Flashcards => "flashcards",
            Self::MindMap => "mind_map",
            Self::Infographic => "infographic",
            Self::SlideDeck => "slide_deck",
            Self::DataTable => "data_table",
            Self::Unknown => "unknown",
        }
    }

    /// Parse from string (case-insensitive). Used by MCP tool input.
    pub fn from_str_key(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "audio" => Some(Self::Audio),
            "video" => Some(Self::Video),
            "report" => Some(Self::Report),
            "quiz" => Some(Self::Quiz),
            "flashcards" => Some(Self::Flashcards),
            "mind_map" | "mindmap" => Some(Self::MindMap),
            "infographic" => Some(Self::Infographic),
            "slide_deck" | "slidedeck" => Some(Self::SlideDeck),
            "data_table" | "datatable" => Some(Self::DataTable),
            _ => None,
        }
    }
}

impl fmt::Display for ArtifactType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// =========================================================================
// Audio Configuration Enums
// =========================================================================

/// Audio format/presentation style.
/// Integer code at position [0] of the audio config sub-array.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    DeepDive = 1,
    Brief = 2,
    Critique = 3,
    Debate = 4,
}

impl AudioFormat {
    pub fn code(self) -> i32 {
        self as i32
    }

    pub fn from_str_key(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "deep_dive" | "deepdive" => Some(Self::DeepDive),
            "brief" => Some(Self::Brief),
            "critique" => Some(Self::Critique),
            "debate" => Some(Self::Debate),
            _ => None,
        }
    }
}

/// Audio length/duration.
/// Integer code at position [1] of the audio config sub-array.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioLength {
    Short = 1,
    Default = 2,
    Long = 3,
}

impl AudioLength {
    pub fn code(self) -> i32 {
        self as i32
    }

    pub fn from_str_key(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "short" => Some(Self::Short),
            "default" => Some(Self::Default),
            "long" => Some(Self::Long),
            _ => None,
        }
    }
}

// =========================================================================
// Video Configuration Enums
// =========================================================================

/// Video format/presentation style.
/// Integer code at position [4] of the video config sub-array.
///
/// CRITICAL: Cinematic (3) sets style=None. Uses Veo 3 AI. Requires Ultra subscription.
/// Generation takes ~30-40 minutes. Uses a MUCH longer timeout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoFormat {
    Explainer = 1,
    Brief = 2,
    Cinematic = 3,
}

impl VideoFormat {
    pub fn code(self) -> i32 {
        self as i32
    }

    pub fn from_str_key(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "explainer" => Some(Self::Explainer),
            "brief" => Some(Self::Brief),
            "cinematic" => Some(Self::Cinematic),
            _ => None,
        }
    }

    /// Cinematic videos require much longer timeouts (~30 min vs default 5 min).
    pub fn default_timeout_secs(self) -> u64 {
        match self {
            Self::Cinematic => 1800,
            _ => 300,
        }
    }
}

/// Video visual style.
/// Integer code at position [5] of the video config sub-array.
/// None for Cinematic format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoStyle {
    AutoSelect = 1,
    Classic = 2,
    Whiteboard = 3,
    Kawaii = 4,
    Anime = 5,
    Watercolor = 6,
    RetroPrint = 7,
    Heritage = 8,
    PaperCraft = 9,
    // 10 is referenced in exploration but no name found — skip
}

impl VideoStyle {
    pub fn code(self) -> i32 {
        self as i32
    }

    pub fn from_str_key(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "auto" => Some(Self::AutoSelect),
            "classic" => Some(Self::Classic),
            "whiteboard" => Some(Self::Whiteboard),
            "kawaii" => Some(Self::Kawaii),
            "anime" => Some(Self::Anime),
            "watercolor" => Some(Self::Watercolor),
            "retro_print" | "retroprint" => Some(Self::RetroPrint),
            "heritage" => Some(Self::Heritage),
            "paper_craft" | "papercraft" => Some(Self::PaperCraft),
            _ => None,
        }
    }
}

// =========================================================================
// Quiz & Flashcard Configuration Enums
// =========================================================================

/// Number of quiz/flashcard items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuizQuantity {
    Fewer = 1,
    Standard = 2,
}

impl QuizQuantity {
    pub fn code(self) -> i32 {
        self as i32
    }

    pub fn from_str_key(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "fewer" | "less" => Some(Self::Fewer),
            "standard" | "normal" => Some(Self::Standard),
            _ => None,
        }
    }
}

/// Difficulty level for quiz/flashcards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuizDifficulty {
    Easy = 1,
    Medium = 2,
    Hard = 3,
}

impl QuizDifficulty {
    pub fn code(self) -> i32 {
        self as i32
    }

    pub fn from_str_key(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "easy" | "simple" => Some(Self::Easy),
            "medium" | "normal" => Some(Self::Medium),
            "hard" | "difficult" => Some(Self::Hard),
            _ => None,
        }
    }
}

// =========================================================================
// Infographic Configuration Enums
// =========================================================================

/// Infographic orientation/layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfographicOrientation {
    Landscape = 1,
    Portrait = 2,
    Square = 3,
}

impl InfographicOrientation {
    pub fn code(self) -> i32 {
        self as i32
    }

    pub fn from_str_key(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "landscape" | "horizontal" => Some(Self::Landscape),
            "portrait" | "vertical" => Some(Self::Portrait),
            "square" => Some(Self::Square),
            _ => None,
        }
    }
}

/// Level of detail in infographic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfographicDetail {
    Concise = 1,
    Standard = 2,
    Detailed = 3,
}

impl InfographicDetail {
    pub fn code(self) -> i32 {
        self as i32
    }

    pub fn from_str_key(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "concise" | "minimal" => Some(Self::Concise),
            "standard" | "normal" => Some(Self::Standard),
            "detailed" | "comprehensive" => Some(Self::Detailed),
            _ => None,
        }
    }
}

/// Infographic visual style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfographicStyle {
    AutoSelect = 1,
    // Styles 2-11 exist but don't have confirmed names from reverse engineering.
    // Adding only confirmed values. The API accepts any int 1-11.
    // We'll use AutoSelect (1) as the safe default and allow raw int passthrough.
}

impl InfographicStyle {
    pub fn code(self) -> i32 {
        self as i32
    }
}

// =========================================================================
// Slide Deck Configuration Enums
// =========================================================================

/// Slide deck format/presentation style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlideDeckFormat {
    DetailedDeck = 1,
    PresenterSlides = 2,
}

impl SlideDeckFormat {
    pub fn code(self) -> i32 {
        self as i32
    }

    pub fn from_str_key(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "detailed" | "detailed_deck" => Some(Self::DetailedDeck),
            "presenter" | "presenter_slides" => Some(Self::PresenterSlides),
            _ => None,
        }
    }
}

/// Slide deck length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlideDeckLength {
    Default = 1,
    Short = 2,
}

impl SlideDeckLength {
    pub fn code(self) -> i32 {
        self as i32
    }

    pub fn from_str_key(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "default" | "normal" => Some(Self::Default),
            "short" => Some(Self::Short),
            _ => None,
        }
    }
}

// =========================================================================
// Report Format — String Enum (not integer!)
// =========================================================================

/// Report template format.
/// CRITICAL: Unlike other enums, ReportFormat maps to STRING values in the wire format,
/// not integers. Each template has a built-in prompt/title/description pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportFormat {
    BriefingDoc,
    StudyGuide,
    BlogPost,
    /// Custom report with user-provided prompt
    Custom {
        prompt: String,
    },
}

impl ReportFormat {
    /// Get the built-in prompt for template formats.
    /// Custom format uses the user-provided prompt directly.
    pub fn prompt(&self) -> Option<&str> {
        match self {
            Self::BriefingDoc => Some(
                "Create a briefing document that summarizes the key information from the sources.",
            ),
            Self::StudyGuide => Some(
                "Create a comprehensive study guide covering the main topics from the sources.",
            ),
            Self::BlogPost => {
                Some("Write an engaging blog post based on the content from the sources.")
            }
            Self::Custom { prompt } => Some(prompt),
        }
    }

    /// Get the built-in title for template formats.
    pub fn title(&self) -> &str {
        match self {
            Self::BriefingDoc => "Briefing Document",
            Self::StudyGuide => "Study Guide",
            Self::BlogPost => "Blog Post",
            Self::Custom { .. } => "Custom Report",
        }
    }

    /// Get the built-in description for template formats.
    pub fn description(&self) -> &str {
        match self {
            Self::BriefingDoc => "Key information summary",
            Self::StudyGuide => "Comprehensive topic coverage",
            Self::BlogPost => "Engaging content article",
            Self::Custom { .. } => "Custom format report",
        }
    }

    pub fn from_str_key(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "briefing_doc" | "briefingdoc" | "briefing" => Some(Self::BriefingDoc),
            "study_guide" | "studyguide" | "study" => Some(Self::StudyGuide),
            "blog_post" | "blogpost" | "blog" => Some(Self::BlogPost),
            "custom" => Some(Self::Custom {
                prompt: String::new(),
            }),
            _ => None,
        }
    }
}

impl fmt::Display for ReportFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BriefingDoc => write!(f, "briefing_doc"),
            Self::StudyGuide => write!(f, "study_guide"),
            Self::BlogPost => write!(f, "blog_post"),
            Self::Custom { .. } => write!(f, "custom"),
        }
    }
}

// =========================================================================
// Quiz/Flashcard Variant — distinguishes Quiz (2) from Flashcards (1)
// =========================================================================

/// Variant code for type 4 (Quiz/Flashcards).
/// Extracted from art[9][1][0] in the artifact response.
/// This is the ONLY way to distinguish Quiz from Flashcards — they share type_code 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuizVariant {
    Flashcards = 1,
    Quiz = 2,
}

impl QuizVariant {
    pub fn code(self) -> i32 {
        self as i32
    }

    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            1 => Some(Self::Flashcards),
            2 => Some(Self::Quiz),
            _ => None,
        }
    }
}

// =========================================================================
// RPC Endpoint IDs
// =========================================================================

/// Known RPC endpoint IDs for artifact operations.
/// These are the obfuscated method names Google uses internally.
pub mod rpc_ids {
    /// Generate any artifact type except mind map
    pub const CREATE_ARTIFACT: &str = "R7cb6c";
    /// List all artifacts in a notebook
    pub const LIST_ARTIFACTS: &str = "gArtLc";
    /// Delete an artifact
    pub const DELETE_ARTIFACT: &str = "V5N4be";
    /// Rename an artifact
    pub const RENAME_ARTIFACT: &str = "rc3d8d";
    /// Export artifact to Google Docs/Sheets
    pub const EXPORT_ARTIFACT: &str = "Krh3pd";
    /// Share artifact publicly
    pub const SHARE_ARTIFACT: &str = "RGP97b";
    /// Get quiz/flashcard interactive HTML
    pub const GET_INTERACTIVE_HTML: &str = "v9rmvd";
    /// Revise individual slide with prompt
    pub const REVISE_SLIDE: &str = "KmcKPe";
    /// Generate mind map (DIFFERENT RPC from CREATE_ARTIFACT!)
    pub const GENERATE_MIND_MAP: &str = "yyryJe";
    /// Persist mind map as note (second step after GENERATE_MIND_MAP)
    pub const CREATE_NOTE: &str = "CYK0Xb";
    /// List notes and mind maps
    pub const GET_NOTES_AND_MIND_MAPS: &str = "cFji9";
    /// Get AI-suggested report formats
    pub const GET_SUGGESTED_REPORTS: &str = "ciyUvf";
}

// =========================================================================
// Source ID Formatting Helpers
// =========================================================================

/// Format source IDs as triple-nested array: `[[[sid1]], [[sid2]], ...]`
///
/// This is the format Google expects at the `source_ids_triple` position
/// in most artifact generation payloads.
pub fn to_triple_nested(source_ids: &[String]) -> serde_json::Value {
    let nested: Vec<serde_json::Value> = source_ids
        .iter()
        .map(|sid| serde_json::json!([[sid]]))
        .collect();
    serde_json::json!(nested)
}

/// Format source IDs as double-nested array: `[[sid1], [sid2], ...]`
///
/// This is the format Google expects at the `source_ids_double` position
/// inside the config sub-array of most artifact payloads.
pub fn to_double_nested(source_ids: &[String]) -> serde_json::Value {
    let nested: Vec<serde_json::Value> = source_ids
        .iter()
        .map(|sid| serde_json::json!([sid]))
        .collect();
    serde_json::json!(nested)
}

// =========================================================================
// ArtifactConfig — Type-Safe Generation Configuration
// =========================================================================

/// Type-safe artifact generation configuration.
///
/// Each variant contains ONLY the configuration fields valid for that artifact type.
/// The compiler rejects invalid configs at compile time — e.g., Quiz requires
/// `difficulty` but Audio does not.
///
/// # Wire Format
///
/// All variants (except Mind Map) serialize to a positional JSON array via
/// `to_params_array()`. The array structure is:
/// ```text
/// [[2], notebook_id, [null, null, type_code, source_ids_triple, ..., [config]]]
/// ```
///
/// # Design Decision (AD-1)
///
/// This is the "Hybrid Enum + Dispatcher" pattern from the design doc:
/// - Compile-time safety for config fields (Quiz has difficulty, Audio doesn't)
/// - Single generation entry point: `generate_artifact()` for all types
/// - Each variant produces a predictable, testable JSON array
///
/// Mind Map is NOT a variant here — it uses a completely different RPC endpoint
/// (`GENERATE_MIND_MAP` / `yyryJe`) and is handled by `generate_mind_map()`.
#[derive(Debug, Clone)]
pub enum ArtifactConfig {
    Audio {
        format: AudioFormat,
        length: AudioLength,
        instructions: Option<String>,
        language: String,
        source_ids: Vec<String>,
    },
    Video {
        format: VideoFormat,
        /// None for Cinematic format (uses Veo 3 AI).
        style: Option<VideoStyle>,
        instructions: Option<String>,
        language: String,
        source_ids: Vec<String>,
    },
    Report {
        format: ReportFormat,
        language: String,
        source_ids: Vec<String>,
        /// Additional instructions appended to the template prompt.
        extra_instructions: Option<String>,
    },
    Quiz {
        difficulty: QuizDifficulty,
        quantity: QuizQuantity,
        instructions: Option<String>,
        source_ids: Vec<String>,
    },
    Flashcards {
        difficulty: QuizDifficulty,
        quantity: QuizQuantity,
        instructions: Option<String>,
        source_ids: Vec<String>,
    },
    Infographic {
        orientation: InfographicOrientation,
        detail: InfographicDetail,
        style: InfographicStyle,
        instructions: Option<String>,
        language: String,
        source_ids: Vec<String>,
    },
    SlideDeck {
        format: SlideDeckFormat,
        length: SlideDeckLength,
        instructions: Option<String>,
        language: String,
        source_ids: Vec<String>,
    },
    DataTable {
        instructions: String,
        language: String,
        source_ids: Vec<String>,
    },
}

impl ArtifactConfig {
    /// Build the complete RPC params array for `CREATE_ARTIFACT` (R7cb6c).
    ///
    /// Returns: `[[2], notebook_id, [inner_array]]`
    ///
    /// The `inner_array` is a positional array where:
    /// - `[2]` = type code position
    /// - `[3]` = source_ids_triple position
    /// - Config sub-array position varies by type (see exploration.md)
    ///
    /// # Panics
    ///
    /// Does not panic. All variants produce valid JSON arrays.
    pub fn to_params_array(&self, notebook_id: &str) -> serde_json::Value {
        match self {
            // -----------------------------------------------------------------
            // Audio (type 1) — config at index 6
            // Wire: [null, null, 1, triple, null, null,
            //        [null, [instructions, length_code, null, double, language, null, format_code]]]
            // -----------------------------------------------------------------
            Self::Audio {
                format,
                length,
                instructions,
                language,
                source_ids,
            } => {
                let triple = to_triple_nested(source_ids);
                let double = to_double_nested(source_ids);
                let instr = instructions.as_deref().unwrap_or("");
                serde_json::json!([
                    [2],
                    notebook_id,
                    [
                        null,
                        null,
                        1,
                        triple,
                        null,
                        null,
                        [
                            null,
                            [
                                instr,
                                length.code(),
                                null,
                                double,
                                language,
                                null,
                                format.code()
                            ]
                        ]
                    ]
                ])
            }

            // -----------------------------------------------------------------
            // Report (type 2) — config at index 7
            // Wire: [null, null, 2, triple, null, null, null,
            //        [null, [title, description, null, double, language, prompt, null, true]]]
            // -----------------------------------------------------------------
            Self::Report {
                format,
                language,
                source_ids,
                extra_instructions,
            } => {
                let triple = to_triple_nested(source_ids);
                let double = to_double_nested(source_ids);
                let title = format.title();
                let description = format.description();
                // Build prompt: template prompt + extra instructions
                let prompt = match (format.prompt(), extra_instructions) {
                    (Some(base), Some(extra)) if !extra.is_empty() => {
                        format!("{}\n\n{}", base, extra)
                    }
                    (Some(base), _) => base.to_string(),
                    (None, Some(extra)) => extra.clone(),
                    (None, None) => String::new(),
                };
                serde_json::json!([
                    [2],
                    notebook_id,
                    [
                        null,
                        null,
                        2,
                        triple,
                        null,
                        null,
                        null,
                        [
                            null,
                            [
                                title,
                                description,
                                null,
                                double,
                                language,
                                prompt,
                                null,
                                true
                            ]
                        ]
                    ]
                ])
            }

            // -----------------------------------------------------------------
            // Video (type 3) — config at index 8
            // Wire: [null, null, 3, triple, null, null, null, null,
            //        [null, null, [double, language, instructions, null, format_code, style_code]]]
            // -----------------------------------------------------------------
            Self::Video {
                format,
                style,
                instructions,
                language,
                source_ids,
            } => {
                let triple = to_triple_nested(source_ids);
                let double = to_double_nested(source_ids);
                let instr = instructions.as_deref().unwrap_or("");
                let style_val = style.map(|s| s.code());
                serde_json::json!([
                    [2],
                    notebook_id,
                    [
                        null,
                        null,
                        3,
                        triple,
                        null,
                        null,
                        null,
                        null,
                        [
                            null,
                            null,
                            [double, language, instr, null, format.code(), style_val]
                        ]
                    ]
                ])
            }

            // -----------------------------------------------------------------
            // Quiz (type 4, variant 2) — config at index 10
            // Wire: [null, null, 4, triple, null×6,
            //        [null, [2, null, instructions, null×3, [quantity_code, difficulty_code]]]]
            //
            // CRITICAL: [quantity, difficulty] at position [7] of variant sub-array
            // -----------------------------------------------------------------
            Self::Quiz {
                difficulty,
                quantity,
                instructions,
                source_ids,
            } => {
                let triple = to_triple_nested(source_ids);
                let instr = instructions.as_deref().unwrap_or("");
                serde_json::json!([
                    [2],
                    notebook_id,
                    [
                        null,
                        null,
                        4,
                        triple,
                        null,
                        null,
                        null,
                        null,
                        null,
                        null,
                        [
                            null,
                            [
                                2,
                                null,
                                instr,
                                null,
                                null,
                                null,
                                null,
                                [quantity.code(), difficulty.code()]
                            ]
                        ]
                    ]
                ])
            }

            // -----------------------------------------------------------------
            // Flashcards (type 4, variant 1) — config at index 10
            // Wire: [null, null, 4, triple, null×6,
            //        [null, [1, null, instructions, null×2, [difficulty_code, quantity_code]]]]
            //
            // CRITICAL: [difficulty, quantity] at position [6] of variant sub-array
            // *** REVERSED ORDER AND DIFFERENT POSITION vs Quiz! ***
            // -----------------------------------------------------------------
            Self::Flashcards {
                difficulty,
                quantity,
                instructions,
                source_ids,
            } => {
                let triple = to_triple_nested(source_ids);
                let instr = instructions.as_deref().unwrap_or("");
                serde_json::json!([
                    [2],
                    notebook_id,
                    [
                        null,
                        null,
                        4,
                        triple,
                        null,
                        null,
                        null,
                        null,
                        null,
                        null,
                        [
                            null,
                            [
                                1,
                                null,
                                instr,
                                null,
                                null,
                                null,
                                [difficulty.code(), quantity.code()]
                            ]
                        ]
                    ]
                ])
            }

            // -----------------------------------------------------------------
            // Infographic (type 7) — config at index 14
            // Wire: [null, null, 7, triple, null×10,
            //        [[instructions, language, null, orientation, detail, style]]]
            // -----------------------------------------------------------------
            Self::Infographic {
                orientation,
                detail,
                style,
                instructions,
                language,
                source_ids,
            } => {
                let triple = to_triple_nested(source_ids);
                let instr = instructions.as_deref().unwrap_or("");
                serde_json::json!([
                    [2],
                    notebook_id,
                    [
                        null,
                        null,
                        7,
                        triple,
                        null,
                        null,
                        null,
                        null,
                        null,
                        null,
                        null,
                        null,
                        null,
                        null,
                        [[
                            instr,
                            language,
                            null,
                            orientation.code(),
                            detail.code(),
                            style.code()
                        ]]
                    ]
                ])
            }

            // -----------------------------------------------------------------
            // Slide Deck (type 8) — config at index 16
            // Wire: [null, null, 8, triple, null×12,
            //        [[instructions, language, format_code, length_code]]]
            // -----------------------------------------------------------------
            Self::SlideDeck {
                format,
                length,
                instructions,
                language,
                source_ids,
            } => {
                let triple = to_triple_nested(source_ids);
                let instr = instructions.as_deref().unwrap_or("");
                serde_json::json!([
                    [2],
                    notebook_id,
                    [
                        null,
                        null,
                        8,
                        triple,
                        null,
                        null,
                        null,
                        null,
                        null,
                        null,
                        null,
                        null,
                        null,
                        null,
                        null,
                        null,
                        [[instr, language, format.code(), length.code()]]
                    ]
                ])
            }

            // -----------------------------------------------------------------
            // Data Table (type 9) — config at index 18
            // Wire: [null, null, 9, triple, null×14,
            //        [null, [instructions, language]]]
            // -----------------------------------------------------------------
            Self::DataTable {
                instructions,
                language,
                source_ids,
            } => {
                let triple = to_triple_nested(source_ids);
                serde_json::json!([
                    [2],
                    notebook_id,
                    [
                        null,
                        null,
                        9,
                        triple,
                        null,
                        null,
                        null,
                        null,
                        null,
                        null,
                        null,
                        null,
                        null,
                        null,
                        null,
                        null,
                        null,
                        null,
                        [null, [instructions, language]]
                    ]
                ])
            }
        }
    }
}

// =========================================================================
// Tests — Phase 2: ArtifactConfig & Payload Builders
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // ArtifactTypeCode
    // -----------------------------------------------------------------------

    #[test]
    fn test_artifact_type_code_values() {
        assert_eq!(ArtifactTypeCode::Audio.code(), 1);
        assert_eq!(ArtifactTypeCode::Report.code(), 2);
        assert_eq!(ArtifactTypeCode::Video.code(), 3);
        assert_eq!(ArtifactTypeCode::QuizFlashcards.code(), 4);
        assert_eq!(ArtifactTypeCode::MindMap.code(), 5);
        assert_eq!(ArtifactTypeCode::Infographic.code(), 7);
        assert_eq!(ArtifactTypeCode::SlideDeck.code(), 8);
        assert_eq!(ArtifactTypeCode::DataTable.code(), 9);
    }

    #[test]
    fn test_artifact_type_code_from_code_roundtrip() {
        let codes = [1, 2, 3, 4, 5, 7, 8, 9];
        for code in codes {
            let parsed = ArtifactTypeCode::from_code(code).unwrap();
            assert_eq!(parsed.code(), code, "Roundtrip failed for code {}", code);
        }
    }

    #[test]
    fn test_artifact_type_code_from_code_invalid() {
        assert!(ArtifactTypeCode::from_code(0).is_none());
        assert!(ArtifactTypeCode::from_code(6).is_none()); // Unused
        assert!(ArtifactTypeCode::from_code(10).is_none());
        assert!(ArtifactTypeCode::from_code(-1).is_none());
    }

    // -----------------------------------------------------------------------
    // ArtifactStatus
    // -----------------------------------------------------------------------

    #[test]
    fn test_artifact_status_values() {
        assert_eq!(ArtifactStatus::Processing.code(), 1);
        assert_eq!(ArtifactStatus::Pending.code(), 2);
        assert_eq!(ArtifactStatus::Completed.code(), 3);
        assert_eq!(ArtifactStatus::Failed.code(), 4);
    }

    #[test]
    fn test_artifact_status_from_code_roundtrip() {
        for code in 1..=4 {
            let parsed = ArtifactStatus::from_code(code).unwrap();
            assert_eq!(parsed.code(), code);
        }
    }

    #[test]
    fn test_artifact_status_display() {
        assert_eq!(ArtifactStatus::Processing.to_string(), "PROCESSING");
        assert_eq!(ArtifactStatus::Completed.to_string(), "COMPLETED");
        assert_eq!(ArtifactStatus::Failed.to_string(), "FAILED");
    }

    // -----------------------------------------------------------------------
    // ArtifactType (user-facing)
    // -----------------------------------------------------------------------

    #[test]
    fn test_artifact_type_from_type_code() {
        assert_eq!(
            ArtifactType::from_type_code_and_variant(ArtifactTypeCode::Audio, None),
            ArtifactType::Audio
        );
        assert_eq!(
            ArtifactType::from_type_code_and_variant(ArtifactTypeCode::Video, None),
            ArtifactType::Video
        );
        assert_eq!(
            ArtifactType::from_type_code_and_variant(ArtifactTypeCode::Report, None),
            ArtifactType::Report
        );
        assert_eq!(
            ArtifactType::from_type_code_and_variant(ArtifactTypeCode::MindMap, None),
            ArtifactType::MindMap
        );
        assert_eq!(
            ArtifactType::from_type_code_and_variant(ArtifactTypeCode::Infographic, None),
            ArtifactType::Infographic
        );
        assert_eq!(
            ArtifactType::from_type_code_and_variant(ArtifactTypeCode::SlideDeck, None),
            ArtifactType::SlideDeck
        );
        assert_eq!(
            ArtifactType::from_type_code_and_variant(ArtifactTypeCode::DataTable, None),
            ArtifactType::DataTable
        );
    }

    #[test]
    fn test_artifact_type_quiz_flashcard_variant_distinction() {
        // Variant 1 = Flashcards
        assert_eq!(
            ArtifactType::from_type_code_and_variant(ArtifactTypeCode::QuizFlashcards, Some(1)),
            ArtifactType::Flashcards
        );
        // Variant 2 = Quiz
        assert_eq!(
            ArtifactType::from_type_code_and_variant(ArtifactTypeCode::QuizFlashcards, Some(2)),
            ArtifactType::Quiz
        );
        // No variant = Unknown
        assert_eq!(
            ArtifactType::from_type_code_and_variant(ArtifactTypeCode::QuizFlashcards, None),
            ArtifactType::Unknown
        );
        // Invalid variant = Unknown
        assert_eq!(
            ArtifactType::from_type_code_and_variant(ArtifactTypeCode::QuizFlashcards, Some(99)),
            ArtifactType::Unknown
        );
    }

    #[test]
    fn test_artifact_type_as_str() {
        assert_eq!(ArtifactType::Audio.as_str(), "audio");
        assert_eq!(ArtifactType::MindMap.as_str(), "mind_map");
        assert_eq!(ArtifactType::SlideDeck.as_str(), "slide_deck");
        assert_eq!(ArtifactType::DataTable.as_str(), "data_table");
        assert_eq!(ArtifactType::Unknown.as_str(), "unknown");
    }

    #[test]
    fn test_artifact_type_from_str_key() {
        assert_eq!(
            ArtifactType::from_str_key("audio"),
            Some(ArtifactType::Audio)
        );
        assert_eq!(
            ArtifactType::from_str_key("VIDEO"),
            Some(ArtifactType::Video)
        );
        assert_eq!(
            ArtifactType::from_str_key("mind_map"),
            Some(ArtifactType::MindMap)
        );
        assert_eq!(
            ArtifactType::from_str_key("mindmap"),
            Some(ArtifactType::MindMap)
        );
        assert_eq!(
            ArtifactType::from_str_key("slide_deck"),
            Some(ArtifactType::SlideDeck)
        );
        assert_eq!(
            ArtifactType::from_str_key("slidedeck"),
            Some(ArtifactType::SlideDeck)
        );
        assert_eq!(
            ArtifactType::from_str_key("data_table"),
            Some(ArtifactType::DataTable)
        );
        assert_eq!(
            ArtifactType::from_str_key("datatable"),
            Some(ArtifactType::DataTable)
        );
        assert_eq!(ArtifactType::from_str_key("invalid"), None);
    }

    // -----------------------------------------------------------------------
    // Audio enums
    // -----------------------------------------------------------------------

    #[test]
    fn test_audio_format_codes() {
        assert_eq!(AudioFormat::DeepDive.code(), 1);
        assert_eq!(AudioFormat::Brief.code(), 2);
        assert_eq!(AudioFormat::Critique.code(), 3);
        assert_eq!(AudioFormat::Debate.code(), 4);
    }

    #[test]
    fn test_audio_format_from_str_key() {
        assert_eq!(
            AudioFormat::from_str_key("deep_dive"),
            Some(AudioFormat::DeepDive)
        );
        assert_eq!(
            AudioFormat::from_str_key("deepdive"),
            Some(AudioFormat::DeepDive)
        );
        assert_eq!(AudioFormat::from_str_key("BRIEF"), Some(AudioFormat::Brief));
        assert_eq!(AudioFormat::from_str_key("invalid"), None);
    }

    #[test]
    fn test_audio_length_codes() {
        assert_eq!(AudioLength::Short.code(), 1);
        assert_eq!(AudioLength::Default.code(), 2);
        assert_eq!(AudioLength::Long.code(), 3);
    }

    #[test]
    fn test_audio_length_from_str_key() {
        assert_eq!(AudioLength::from_str_key("short"), Some(AudioLength::Short));
        assert_eq!(
            AudioLength::from_str_key("default"),
            Some(AudioLength::Default)
        );
        assert_eq!(AudioLength::from_str_key("long"), Some(AudioLength::Long));
    }

    // -----------------------------------------------------------------------
    // Video enums
    // -----------------------------------------------------------------------

    #[test]
    fn test_video_format_codes() {
        assert_eq!(VideoFormat::Explainer.code(), 1);
        assert_eq!(VideoFormat::Brief.code(), 2);
        assert_eq!(VideoFormat::Cinematic.code(), 3);
    }

    #[test]
    fn test_video_format_from_str_key() {
        assert_eq!(
            VideoFormat::from_str_key("explainer"),
            Some(VideoFormat::Explainer)
        );
        assert_eq!(
            VideoFormat::from_str_key("cinematic"),
            Some(VideoFormat::Cinematic)
        );
    }

    #[test]
    fn test_video_format_timeout() {
        assert_eq!(VideoFormat::Cinematic.default_timeout_secs(), 1800);
        assert_eq!(VideoFormat::Explainer.default_timeout_secs(), 300);
        assert_eq!(VideoFormat::Brief.default_timeout_secs(), 300);
    }

    #[test]
    fn test_video_style_codes() {
        assert_eq!(VideoStyle::AutoSelect.code(), 1);
        assert_eq!(VideoStyle::Classic.code(), 2);
        assert_eq!(VideoStyle::Whiteboard.code(), 3);
        assert_eq!(VideoStyle::Kawaii.code(), 4);
        assert_eq!(VideoStyle::Anime.code(), 5);
        assert_eq!(VideoStyle::Watercolor.code(), 6);
        assert_eq!(VideoStyle::RetroPrint.code(), 7);
        assert_eq!(VideoStyle::Heritage.code(), 8);
        assert_eq!(VideoStyle::PaperCraft.code(), 9);
    }

    #[test]
    fn test_video_style_from_str_key() {
        assert_eq!(
            VideoStyle::from_str_key("auto"),
            Some(VideoStyle::AutoSelect)
        );
        assert_eq!(
            VideoStyle::from_str_key("whiteboard"),
            Some(VideoStyle::Whiteboard)
        );
        assert_eq!(
            VideoStyle::from_str_key("paper_craft"),
            Some(VideoStyle::PaperCraft)
        );
        assert_eq!(
            VideoStyle::from_str_key("papercraft"),
            Some(VideoStyle::PaperCraft)
        );
        assert_eq!(VideoStyle::from_str_key("invalid"), None);
    }

    // -----------------------------------------------------------------------
    // Quiz/Flashcard enums
    // -----------------------------------------------------------------------

    #[test]
    fn test_quiz_quantity_codes() {
        assert_eq!(QuizQuantity::Fewer.code(), 1);
        assert_eq!(QuizQuantity::Standard.code(), 2);
    }

    #[test]
    fn test_quiz_difficulty_codes() {
        assert_eq!(QuizDifficulty::Easy.code(), 1);
        assert_eq!(QuizDifficulty::Medium.code(), 2);
        assert_eq!(QuizDifficulty::Hard.code(), 3);
    }

    #[test]
    fn test_quiz_variant_from_code() {
        assert_eq!(QuizVariant::from_code(1), Some(QuizVariant::Flashcards));
        assert_eq!(QuizVariant::from_code(2), Some(QuizVariant::Quiz));
        assert!(QuizVariant::from_code(3).is_none());
    }

    // -----------------------------------------------------------------------
    // Infographic enums
    // -----------------------------------------------------------------------

    #[test]
    fn test_infographic_orientation_codes() {
        assert_eq!(InfographicOrientation::Landscape.code(), 1);
        assert_eq!(InfographicOrientation::Portrait.code(), 2);
        assert_eq!(InfographicOrientation::Square.code(), 3);
    }

    #[test]
    fn test_infographic_detail_codes() {
        assert_eq!(InfographicDetail::Concise.code(), 1);
        assert_eq!(InfographicDetail::Standard.code(), 2);
        assert_eq!(InfographicDetail::Detailed.code(), 3);
    }

    #[test]
    fn test_infographic_style_code() {
        assert_eq!(InfographicStyle::AutoSelect.code(), 1);
    }

    // -----------------------------------------------------------------------
    // Slide Deck enums
    // -----------------------------------------------------------------------

    #[test]
    fn test_slide_deck_format_codes() {
        assert_eq!(SlideDeckFormat::DetailedDeck.code(), 1);
        assert_eq!(SlideDeckFormat::PresenterSlides.code(), 2);
    }

    #[test]
    fn test_slide_deck_length_codes() {
        assert_eq!(SlideDeckLength::Default.code(), 1);
        assert_eq!(SlideDeckLength::Short.code(), 2);
    }

    // -----------------------------------------------------------------------
    // ReportFormat (string enum)
    // -----------------------------------------------------------------------

    #[test]
    fn test_report_format_has_prompts() {
        assert!(ReportFormat::BriefingDoc.prompt().is_some());
        assert!(ReportFormat::StudyGuide.prompt().is_some());
        assert!(ReportFormat::BlogPost.prompt().is_some());
        assert!(ReportFormat::Custom {
            prompt: "test".into()
        }
        .prompt()
        .is_some());
    }

    #[test]
    fn test_report_format_custom_prompt_passthrough() {
        let custom = ReportFormat::Custom {
            prompt: "My custom prompt".to_string(),
        };
        assert_eq!(custom.prompt(), Some("My custom prompt"));
    }

    #[test]
    fn test_report_format_display() {
        assert_eq!(ReportFormat::BriefingDoc.to_string(), "briefing_doc");
        assert_eq!(ReportFormat::StudyGuide.to_string(), "study_guide");
        assert_eq!(ReportFormat::BlogPost.to_string(), "blog_post");
        assert_eq!(
            ReportFormat::Custom {
                prompt: String::new()
            }
            .to_string(),
            "custom"
        );
    }

    #[test]
    fn test_report_format_from_str_key() {
        assert_eq!(
            ReportFormat::from_str_key("briefing_doc"),
            Some(ReportFormat::BriefingDoc)
        );
        assert_eq!(
            ReportFormat::from_str_key("briefingdoc"),
            Some(ReportFormat::BriefingDoc)
        );
        assert_eq!(
            ReportFormat::from_str_key("study_guide"),
            Some(ReportFormat::StudyGuide)
        );
        assert_eq!(
            ReportFormat::from_str_key("blog_post"),
            Some(ReportFormat::BlogPost)
        );
        assert_eq!(
            ReportFormat::from_str_key("custom"),
            Some(ReportFormat::Custom {
                prompt: String::new()
            })
        );
        assert_eq!(ReportFormat::from_str_key("invalid"), None);
    }

    // -----------------------------------------------------------------------
    // RPC IDs
    // -----------------------------------------------------------------------

    #[test]
    fn test_rpc_ids_are_non_empty() {
        assert!(!rpc_ids::CREATE_ARTIFACT.is_empty());
        assert!(!rpc_ids::LIST_ARTIFACTS.is_empty());
        assert!(!rpc_ids::DELETE_ARTIFACT.is_empty());
        assert!(!rpc_ids::GET_INTERACTIVE_HTML.is_empty());
        assert!(!rpc_ids::GENERATE_MIND_MAP.is_empty());
        assert!(!rpc_ids::CREATE_NOTE.is_empty());
        assert!(!rpc_ids::GET_NOTES_AND_MIND_MAPS.is_empty());
    }

    #[test]
    fn test_mind_map_uses_different_rpc() {
        // Mind map generation uses a DIFFERENT endpoint than other artifacts
        assert_ne!(rpc_ids::GENERATE_MIND_MAP, rpc_ids::CREATE_ARTIFACT);
    }

    // -----------------------------------------------------------------------
    // Source ID formatting helpers
    // -----------------------------------------------------------------------

    #[test]
    fn test_to_triple_nested() {
        let ids = vec!["sid1".to_string(), "sid2".to_string()];
        let result = to_triple_nested(&ids);
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        // [[[sid1]], [[sid2]]]
        assert_eq!(
            arr[0].as_array().unwrap()[0].as_array().unwrap()[0]
                .as_str()
                .unwrap(),
            "sid1"
        );
        assert_eq!(
            arr[1].as_array().unwrap()[0].as_array().unwrap()[0]
                .as_str()
                .unwrap(),
            "sid2"
        );
    }

    #[test]
    fn test_to_double_nested() {
        let ids = vec!["sid1".to_string(), "sid2".to_string()];
        let result = to_double_nested(&ids);
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        // [[sid1], [sid2]]
        assert_eq!(arr[0].as_array().unwrap()[0].as_str().unwrap(), "sid1");
        assert_eq!(arr[1].as_array().unwrap()[0].as_str().unwrap(), "sid2");
    }

    #[test]
    fn test_to_triple_nested_empty() {
        let result = to_triple_nested(&[]);
        assert!(result.as_array().unwrap().is_empty());
    }

    // -----------------------------------------------------------------------
    // ArtifactConfig::to_params_array — one test per variant
    // -----------------------------------------------------------------------

    fn source_ids() -> Vec<String> {
        vec!["src-aaa-111".to_string(), "src-bbb-222".to_string()]
    }

    /// Helper: extract the inner array from params: [[2], nb_id, [inner]]
    fn inner_array(params: &serde_json::Value) -> Vec<&serde_json::Value> {
        params.as_array().unwrap()[2]
            .as_array()
            .unwrap()
            .iter()
            .collect()
    }

    // --- Audio ---

    #[test]
    fn test_audio_params_structure() {
        let config = ArtifactConfig::Audio {
            format: AudioFormat::DeepDive,
            length: AudioLength::Default,
            instructions: Some("Focus on key concepts".to_string()),
            language: "en".to_string(),
            source_ids: source_ids(),
        };
        let params = config.to_params_array("nb-123");

        // Outer: [[2], "nb-123", [inner]]
        let outer = params.as_array().unwrap();
        assert_eq!(outer.len(), 3);
        assert_eq!(outer[0].as_array().unwrap()[0].as_i64().unwrap(), 2);
        assert_eq!(outer[1].as_str().unwrap(), "nb-123");

        // Inner: type_code at [2]
        let inner = inner_array(&params);
        assert_eq!(inner.len(), 7); // indices 0-6
        assert_eq!(inner[2].as_i64().unwrap(), 1); // type code
        assert!(inner[3].is_array()); // triple nested source_ids

        // Config at index 6
        let config_arr = inner[6].as_array().unwrap();
        assert_eq!(config_arr.len(), 2);
        let config_inner = config_arr[1].as_array().unwrap();
        assert_eq!(config_inner[0].as_str().unwrap(), "Focus on key concepts"); // instructions
        assert_eq!(config_inner[1].as_i64().unwrap(), 2); // length=Default
        assert_eq!(config_inner[3].is_array(), true); // double nested source_ids
        assert_eq!(config_inner[4].as_str().unwrap(), "en"); // language
        assert_eq!(config_inner[6].as_i64().unwrap(), 1); // format=DeepDive
    }

    // --- Report ---

    #[test]
    fn test_report_params_structure() {
        let config = ArtifactConfig::Report {
            format: ReportFormat::StudyGuide,
            language: "en".to_string(),
            source_ids: source_ids(),
            extra_instructions: None,
        };
        let params = config.to_params_array("nb-456");

        let inner = inner_array(&params);
        assert_eq!(inner.len(), 8); // indices 0-7
        assert_eq!(inner[2].as_i64().unwrap(), 2); // type code

        // Config at index 7
        let config_inner = inner[7].as_array().unwrap()[1].as_array().unwrap();
        assert_eq!(config_inner[0].as_str().unwrap(), "Study Guide"); // title
        assert_eq!(
            config_inner[1].as_str().unwrap(),
            "Comprehensive topic coverage"
        ); // description
        assert!(config_inner[2].is_null()); // padding
        assert_eq!(config_inner[4].as_str().unwrap(), "en"); // language
        assert!(config_inner[5].as_str().unwrap().contains("study guide")); // prompt
        assert!(config_inner[7].as_bool().unwrap()); // true
    }

    #[test]
    fn test_report_custom_with_extra_instructions() {
        let config = ArtifactConfig::Report {
            format: ReportFormat::Custom {
                prompt: "Summarize in bullet points".to_string(),
            },
            language: "es".to_string(),
            source_ids: source_ids(),
            extra_instructions: Some("Focus on chapter 3".to_string()),
        };
        let params = config.to_params_array("nb-789");
        let config_inner = inner_array(&params)[7].as_array().unwrap()[1]
            .as_array()
            .unwrap();

        // Prompt should combine base + extra
        let prompt = config_inner[5].as_str().unwrap();
        assert!(prompt.contains("Summarize in bullet points"));
        assert!(prompt.contains("Focus on chapter 3"));
    }

    // --- Video ---

    #[test]
    fn test_video_params_structure() {
        let config = ArtifactConfig::Video {
            format: VideoFormat::Explainer,
            style: Some(VideoStyle::Whiteboard),
            instructions: None,
            language: "en".to_string(),
            source_ids: source_ids(),
        };
        let params = config.to_params_array("nb-vid");

        let inner = inner_array(&params);
        assert_eq!(inner.len(), 9); // indices 0-8
        assert_eq!(inner[2].as_i64().unwrap(), 3);

        // Config at index 8: [null, null, [double, language, instr, null, format, style]]
        let config_arr = inner[8].as_array().unwrap();
        let config_inner = config_arr[2].as_array().unwrap();
        assert!(config_inner[0].is_array()); // double nested
        assert_eq!(config_inner[1].as_str().unwrap(), "en");
        assert_eq!(config_inner[2].as_str().unwrap(), ""); // no instructions
        assert_eq!(config_inner[4].as_i64().unwrap(), 1); // Explainer
        assert_eq!(config_inner[5].as_i64().unwrap(), 3); // Whiteboard
    }

    #[test]
    fn test_video_cinematic_no_style() {
        let config = ArtifactConfig::Video {
            format: VideoFormat::Cinematic,
            style: None, // CRITICAL: Cinematic has no style
            instructions: Some("Make it epic".to_string()),
            language: "en".to_string(),
            source_ids: source_ids(),
        };
        let params = config.to_params_array("nb-cin");

        let inner = inner_array(&params);
        let config_inner = inner[8].as_array().unwrap()[2].as_array().unwrap();
        assert_eq!(config_inner[4].as_i64().unwrap(), 3); // Cinematic
        assert!(config_inner[5].is_null()); // NO style
        assert_eq!(config_inner[2].as_str().unwrap(), "Make it epic");
    }

    // --- Quiz ---

    #[test]
    fn test_quiz_params_structure() {
        let config = ArtifactConfig::Quiz {
            difficulty: QuizDifficulty::Hard,
            quantity: QuizQuantity::Standard,
            instructions: None,
            source_ids: source_ids(),
        };
        let params = config.to_params_array("nb-quiz");

        let inner = inner_array(&params);
        assert_eq!(inner.len(), 11); // indices 0-10
        assert_eq!(inner[2].as_i64().unwrap(), 4); // QuizFlashcards type

        // Config at index 10: [null, [2, null, instr, null×3, [quantity, difficulty]]]
        let config_arr = inner[10].as_array().unwrap();
        let variant_arr = config_arr[1].as_array().unwrap();
        assert_eq!(variant_arr[0].as_i64().unwrap(), 2); // variant=Quiz
        assert_eq!(variant_arr[7].as_array().unwrap()[0].as_i64().unwrap(), 2); // quantity=Standard
        assert_eq!(variant_arr[7].as_array().unwrap()[1].as_i64().unwrap(), 3); // difficulty=Hard
    }

    // --- Flashcards ---

    #[test]
    fn test_flashcards_params_structure() {
        let config = ArtifactConfig::Flashcards {
            difficulty: QuizDifficulty::Easy,
            quantity: QuizQuantity::Fewer,
            instructions: None,
            source_ids: source_ids(),
        };
        let params = config.to_params_array("nb-flash");

        let inner = inner_array(&params);
        assert_eq!(inner.len(), 11);
        assert_eq!(inner[2].as_i64().unwrap(), 4); // QuizFlashcards type

        // Config at index 10: [null, [1, null, instr, null×2, [difficulty, quantity]]]
        let config_arr = inner[10].as_array().unwrap();
        let variant_arr = config_arr[1].as_array().unwrap();
        assert_eq!(variant_arr[0].as_i64().unwrap(), 1); // variant=Flashcards

        // CRITICAL: [difficulty, quantity] at position [6] — REVERSED from Quiz!
        let pair = variant_arr[6].as_array().unwrap();
        assert_eq!(pair[0].as_i64().unwrap(), 1); // difficulty=Easy FIRST
        assert_eq!(pair[1].as_i64().unwrap(), 1); // quantity=Fewer SECOND
    }

    #[test]
    fn test_quiz_flashcard_order_reversal() {
        // This is THE most critical test in the entire module.
        // Quiz: [quantity, difficulty] at variant[7]
        // Flashcards: [difficulty, quantity] at variant[6]

        let quiz = ArtifactConfig::Quiz {
            difficulty: QuizDifficulty::Hard,
            quantity: QuizQuantity::Fewer,
            instructions: None,
            source_ids: source_ids(),
        };
        let quiz_params = quiz.to_params_array("nb");
        let quiz_variant = inner_array(&quiz_params)[10].as_array().unwrap()[1]
            .as_array()
            .unwrap();
        let quiz_pair = quiz_variant[7].as_array().unwrap();
        // Quiz: [quantity=Fewer=1, difficulty=Hard=3]
        assert_eq!(quiz_pair[0].as_i64().unwrap(), 1); // quantity FIRST
        assert_eq!(quiz_pair[1].as_i64().unwrap(), 3); // difficulty SECOND

        let flash = ArtifactConfig::Flashcards {
            difficulty: QuizDifficulty::Hard,
            quantity: QuizQuantity::Fewer,
            instructions: None,
            source_ids: source_ids(),
        };
        let flash_params = flash.to_params_array("nb");
        let flash_variant = inner_array(&flash_params)[10].as_array().unwrap()[1]
            .as_array()
            .unwrap();
        let flash_pair = flash_variant[6].as_array().unwrap();
        // Flashcards: [difficulty=Hard=3, quantity=Fewer=1]
        assert_eq!(flash_pair[0].as_i64().unwrap(), 3); // difficulty FIRST
        assert_eq!(flash_pair[1].as_i64().unwrap(), 1); // quantity SECOND

        // Prove they are DIFFERENT
        assert_ne!(quiz_pair[0], flash_pair[0]);
        assert_ne!(quiz_pair[1], flash_pair[1]);
    }

    // --- Infographic ---

    #[test]
    fn test_infographic_params_structure() {
        let config = ArtifactConfig::Infographic {
            orientation: InfographicOrientation::Landscape,
            detail: InfographicDetail::Standard,
            style: InfographicStyle::AutoSelect,
            instructions: Some("Colorful".to_string()),
            language: "en".to_string(),
            source_ids: source_ids(),
        };
        let params = config.to_params_array("nb-inf");

        let inner = inner_array(&params);
        assert_eq!(inner.len(), 15); // indices 0-14
        assert_eq!(inner[2].as_i64().unwrap(), 7);

        // Config at index 14: [[instructions, language, null, orientation, detail, style]]
        let config_arr = inner[14].as_array().unwrap()[0].as_array().unwrap();
        assert_eq!(config_arr[0].as_str().unwrap(), "Colorful");
        assert_eq!(config_arr[1].as_str().unwrap(), "en");
        assert!(config_arr[2].is_null());
        assert_eq!(config_arr[3].as_i64().unwrap(), 1); // Landscape
        assert_eq!(config_arr[4].as_i64().unwrap(), 2); // Standard
        assert_eq!(config_arr[5].as_i64().unwrap(), 1); // AutoSelect
    }

    // --- Slide Deck ---

    #[test]
    fn test_slide_deck_params_structure() {
        let config = ArtifactConfig::SlideDeck {
            format: SlideDeckFormat::PresenterSlides,
            length: SlideDeckLength::Short,
            instructions: None,
            language: "en".to_string(),
            source_ids: source_ids(),
        };
        let params = config.to_params_array("nb-slide");

        let inner = inner_array(&params);
        assert_eq!(inner.len(), 17); // indices 0-16
        assert_eq!(inner[2].as_i64().unwrap(), 8);

        // Config at index 16: [[instructions, language, format, length]]
        let config_arr = inner[16].as_array().unwrap()[0].as_array().unwrap();
        assert_eq!(config_arr[0].as_str().unwrap(), ""); // no instructions
        assert_eq!(config_arr[1].as_str().unwrap(), "en");
        assert_eq!(config_arr[2].as_i64().unwrap(), 2); // PresenterSlides
        assert_eq!(config_arr[3].as_i64().unwrap(), 2); // Short
    }

    // --- Data Table ---

    #[test]
    fn test_data_table_params_structure() {
        let config = ArtifactConfig::DataTable {
            instructions: "Extract all numerical data".to_string(),
            language: "en".to_string(),
            source_ids: source_ids(),
        };
        let params = config.to_params_array("nb-data");

        let inner = inner_array(&params);
        assert_eq!(inner.len(), 19); // indices 0-18
        assert_eq!(inner[2].as_i64().unwrap(), 9);

        // Config at index 18: [null, [instructions, language]]
        let config_arr = inner[18].as_array().unwrap();
        let config_inner = config_arr[1].as_array().unwrap();
        assert_eq!(
            config_inner[0].as_str().unwrap(),
            "Extract all numerical data"
        );
        assert_eq!(config_inner[1].as_str().unwrap(), "en");
    }

    // -----------------------------------------------------------------------
    // Edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn test_audio_no_instructions_defaults_to_empty_string() {
        let config = ArtifactConfig::Audio {
            format: AudioFormat::Brief,
            length: AudioLength::Short,
            instructions: None,
            language: "en".to_string(),
            source_ids: vec!["only-src".to_string()],
        };
        let params = config.to_params_array("nb");
        let inner = inner_array(&params);
        let config_inner = inner[6].as_array().unwrap()[1].as_array().unwrap();
        assert_eq!(config_inner[0].as_str().unwrap(), ""); // defaults to ""
    }

    #[test]
    fn test_all_params_have_correct_outer_structure() {
        // Verify that ALL variants produce [[2], notebook_id, [inner...]]
        let nb = "nb-test";
        let sources = source_ids();

        let configs = vec![
            ArtifactConfig::Audio {
                format: AudioFormat::DeepDive,
                length: AudioLength::Default,
                instructions: None,
                language: "en".into(),
                source_ids: sources.clone(),
            },
            ArtifactConfig::Video {
                format: VideoFormat::Brief,
                style: Some(VideoStyle::Classic),
                instructions: None,
                language: "en".into(),
                source_ids: sources.clone(),
            },
            ArtifactConfig::Report {
                format: ReportFormat::BlogPost,
                language: "en".into(),
                source_ids: sources.clone(),
                extra_instructions: None,
            },
            ArtifactConfig::Quiz {
                difficulty: QuizDifficulty::Medium,
                quantity: QuizQuantity::Standard,
                instructions: None,
                source_ids: sources.clone(),
            },
            ArtifactConfig::Flashcards {
                difficulty: QuizDifficulty::Medium,
                quantity: QuizQuantity::Standard,
                instructions: None,
                source_ids: sources.clone(),
            },
            ArtifactConfig::Infographic {
                orientation: InfographicOrientation::Square,
                detail: InfographicDetail::Concise,
                style: InfographicStyle::AutoSelect,
                instructions: None,
                language: "en".into(),
                source_ids: sources.clone(),
            },
            ArtifactConfig::SlideDeck {
                format: SlideDeckFormat::DetailedDeck,
                length: SlideDeckLength::Default,
                instructions: None,
                language: "en".into(),
                source_ids: sources.clone(),
            },
            ArtifactConfig::DataTable {
                instructions: "table".into(),
                language: "en".into(),
                source_ids: sources,
            },
        ];

        for config in configs {
            let params = config.to_params_array(nb);
            let outer = params.as_array().unwrap();
            assert_eq!(outer.len(), 3, "Outer array must have 3 elements");
            assert_eq!(
                outer[0].as_array().unwrap()[0].as_i64().unwrap(),
                2,
                "First element must be [2]"
            );
            assert_eq!(
                outer[1].as_str().unwrap(),
                nb,
                "Second element must be notebook_id"
            );
            assert!(outer[2].is_array(), "Third element must be inner array");
        }
    }

    // -----------------------------------------------------------------------
    // GenerationStatus
    // -----------------------------------------------------------------------

    #[test]
    fn test_generation_status_new() {
        let s = GenerationStatus::new("task-123".to_string(), ArtifactStatus::Processing);
        assert_eq!(s.task_id, "task-123");
        assert_eq!(s.status, ArtifactStatus::Processing);
        assert!(s.error.is_none());
        assert!(s.error_code.is_none());
        assert!(s.is_in_progress());
        assert!(!s.is_complete());
        assert!(!s.is_failed());
        assert!(!s.is_rate_limited());
    }

    #[test]
    fn test_generation_status_new_completed() {
        let s = GenerationStatus::new("task-456".to_string(), ArtifactStatus::Completed);
        assert!(s.is_complete());
        assert!(!s.is_in_progress());
        assert!(!s.is_failed());
    }

    #[test]
    fn test_generation_status_rate_limited() {
        let s = GenerationStatus::rate_limited("Too many requests");
        assert_eq!(s.task_id, "rate_limited");
        assert_eq!(s.status, ArtifactStatus::Failed);
        assert_eq!(s.error.as_deref(), Some("Too many requests"));
        assert_eq!(s.error_code.as_deref(), Some("USER_DISPLAYABLE_ERROR"));
        assert!(s.is_rate_limited());
        assert!(s.is_failed());
    }

    #[test]
    fn test_generation_status_failed() {
        let s = GenerationStatus::failed(
            "task-789".to_string(),
            "Something went wrong",
            "INTERNAL_ERROR",
        );
        assert_eq!(s.task_id, "task-789");
        assert!(s.is_failed());
        assert!(!s.is_rate_limited());
        assert_eq!(s.error_code.as_deref(), Some("INTERNAL_ERROR"));
    }

    #[test]
    fn test_generation_status_display() {
        let s = GenerationStatus::new("abc".to_string(), ArtifactStatus::Processing);
        assert_eq!(
            format!("{}", s),
            "GenerationStatus(task_id=abc, status=PROCESSING)"
        );

        let r = GenerationStatus::rate_limited("slow down");
        let display = format!("{}", r);
        assert!(display.contains("USER_DISPLAYABLE_ERROR"));
        assert!(display.contains("slow down"));
    }

    #[test]
    fn test_generation_status_pending_is_in_progress() {
        let s = GenerationStatus::new("task-p".to_string(), ArtifactStatus::Pending);
        assert!(s.is_in_progress());
        assert!(!s.is_complete());
    }

    // -----------------------------------------------------------------------
    // MindMapResult
    // -----------------------------------------------------------------------

    #[test]
    fn test_mind_map_result_new() {
        let data = serde_json::json!({"name": "Test Map", "children": []});
        let r = MindMapResult::new("note-123".to_string(), data.clone());
        assert_eq!(r.note_id.as_deref(), Some("note-123"));
        assert_eq!(r.mind_map_data.as_ref(), Some(&data));
    }

    #[test]
    fn test_mind_map_result_empty() {
        let r = MindMapResult::empty();
        assert!(r.note_id.is_none());
        assert!(r.mind_map_data.is_none());
    }
}
