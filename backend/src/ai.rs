use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

use crate::privacy::SanitizedContext;

// ============ Request Structures ============

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
}

#[derive(Debug, Serialize, Clone)]
struct Content {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    parts: Vec<Part>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
enum Part {
    Text { text: String },
    InlineData { inline_data: InlineData },
}

#[derive(Debug, Serialize, Clone)]
struct InlineData {
    mime_type: String,
    data: String,
}

#[derive(Debug, Serialize)]
struct Tool {
    google_search: GoogleSearchTool,
}

#[derive(Debug, Serialize)]
struct GoogleSearchTool {}

// ============ Response Structures ============

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<UsageMetadata>,
    error: Option<GeminiError>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    pub prompt_token_count: i32,
    #[serde(rename = "candidatesTokenCount")]
    pub candidates_token_count: Option<i32>,
    #[serde(rename = "totalTokenCount")]
    pub total_token_count: i32,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: CandidateContent,
}

#[derive(Debug, Deserialize)]
struct CandidateContent {
    parts: Vec<ResponsePart>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ResponsePart {
    Text { text: String },
}

#[derive(Debug, Deserialize)]
struct GeminiError {
    message: String,
}

// ============ AI Client ============

pub struct AiClient {
    client: Client,
    api_key: String,
}

impl AiClient {
    pub fn new() -> Result<Self, String> {
        let api_key = env::var("GOOGLE_API_KEY")
            .or_else(|_| env::var("GEMINI_API_KEY"))
            .map_err(|_| "GOOGLE_API_KEY or GEMINI_API_KEY environment variable not set")?;

        Ok(Self {
            client: Client::new(),
            api_key,
        })
    }

    pub async fn ask(
        &self,
        context: Option<&SanitizedContext>,
        user_message: &str,
        custom_instruction: Option<&str>,
        user_image: Option<&str>,
    ) -> Result<(String, Option<UsageMetadata>), String> {
        let system_prompt = self.build_system_prompt(context, custom_instruction);
        let full_prompt = format!("{}\n\nUser: {}", system_prompt, user_message);

        // Build parts - text first, then image if available
        let mut parts: Vec<Part> = vec![Part::Text { text: full_prompt }];

        // Add user uploaded image if available
        if let Some(img_data) = user_image {
            // Assume JPEG for now or detect from prefix
            let (mime, data) = if img_data.starts_with("data:image/png;base64,") {
                ("image/png", &img_data[22..])
            } else if img_data.starts_with("data:image/jpeg;base64,") {
                ("image/jpeg", &img_data[23..])
            } else if img_data.starts_with("data:image/webp;base64,") {
                ("image/webp", &img_data[23..])
            } else {
                // Default fallback or raw base64
                ("image/jpeg", img_data)
            };
            
            parts.push(Part::InlineData {
                inline_data: InlineData {
                    mime_type: mime.to_string(),
                    data: data.to_string(),
                },
            });
            tracing::info!("Including user uploaded image in AI request");
        }

        // Add screenshot as image if available
        if let Some(ctx) = context {
            if let Some(screenshot) = &ctx.screenshot {
                if let Some(base64_data) = screenshot.strip_prefix("data:image/jpeg;base64,") {
                    parts.push(Part::InlineData {
                        inline_data: InlineData {
                            mime_type: "image/jpeg".to_string(),
                            data: base64_data.to_string(),
                        },
                    });
                    tracing::info!("Including screenshot in AI request");
                } else if let Some(base64_data) = screenshot.strip_prefix("data:image/png;base64,")
                {
                    parts.push(Part::InlineData {
                        inline_data: InlineData {
                            mime_type: "image/png".to_string(),
                            data: base64_data.to_string(),
                        },
                    });
                    tracing::info!("Including screenshot (PNG) in AI request");
                }
            }
        }

        // Define native google_search tool
        let tools = vec![Tool {
            google_search: GoogleSearchTool {},
        }];

        let initial_content = Content {
            role: Some("user".to_string()),
            parts,
        };

        let request = GeminiRequest {
            contents: vec![initial_content],
            tools: Some(tools),
        };

        // API call - google_search is handled automatically by Gemini
        let response = self.call_gemini(&request).await?;

        // Extract text response (search results are embedded automatically)
        if let Some(candidates) = &response.candidates {
            if let Some(candidate) = candidates.first() {
                for part in &candidate.content.parts {
                    let ResponsePart::Text { text } = part;
                    return Ok((text.clone(), response.usage_metadata));
                }
            }
        }

        if let Some(error) = response.error {
            return Err(format!("API error: {}", error.message));
        }

        Err("Tidak ada respons dari AI".to_string())
    }

    async fn call_gemini(&self, request: &GeminiRequest) -> Result<GeminiResponse, String> {
        let url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent";

        let response = self
            .client
            .post(url)
            .header("x-goog-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        if !status.is_success() {
            return Err(format!("API error ({}): {}", status, body));
        }

        serde_json::from_str(&body).map_err(|e| {
            format!(
                "Failed to parse response: {} - Body: {}",
                e,
                &body[..body.len().min(500)]
            )
        })
    }

    fn build_system_prompt(&self, context: Option<&SanitizedContext>, custom_instruction: Option<&str>) -> String {
        let mut prompt = String::from(
            "Kamu adalah asisten browser yang membantu. Kamu bisa melihat apa yang sedang dijelajahi pengguna dan membantu mereka memahami kontennya.\n\n",
        );

        if let Some(instruction) = custom_instruction {
            prompt.push_str(&format!("INSTRUKSI TAMBAHAN: {}\n\n", instruction));
        }

        prompt.push_str(
            "PENTING: Kamu memiliki akses ke:\n\
            1. Konten teks halaman browser\n\
            2. Screenshot dari tampilan browser saat ini\n\
            3. Google Search - gunakan ini untuk mencari informasi terkini di internet\n\n\
            Gunakan screenshot untuk memahami elemen visual, layout, gambar, grafik, dan hal-hal yang mungkin tidak tertangkap dalam teks.\n\
            Gunakan Google Search ketika pengguna bertanya tentang informasi yang tidak ada di halaman, berita terkini, atau meminta kamu mencari sesuatu.\n\n\
            WAJIB: Selalu jawab dalam Bahasa Indonesia, kecuali pengguna secara eksplisit meminta bahasa lain.\n\n",
        );

        if let Some(ctx) = context {
            prompt.push_str("Konteks browser saat ini:\n");

            if let Some(url) = &ctx.url {
                prompt.push_str(&format!("URL: {}\n", url));
            }

            if let Some(title) = &ctx.title {
                prompt.push_str(&format!("Judul Halaman: {}\n", title));
            }

            if let Some(content) = &ctx.content {
                let truncated = if content.len() > 12000 {
                    format!("{}... [terpotong]", &content[..12000])
                } else {
                    content.clone()
                };
                prompt.push_str(&format!("\nKonten Halaman:\n{}\n", truncated));
            }

            if ctx.screenshot.is_some() {
                prompt.push_str("\n[Screenshot halaman saat ini terlampir di bawah]\n");
            }

            prompt.push_str("\n");
        } else {
            prompt
                .push_str("Tidak ada konteks browser. Pengguna belum membuka halaman apapun.\n\n");
        }

        prompt.push_str("Jawab pertanyaan pengguna berdasarkan konten halaman, screenshot, dan hasil pencarian web jika relevan. Jawab dengan ringkas dan membantu dalam Bahasa Indonesia.");

        prompt
    }
}
