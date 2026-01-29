use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
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
    Text {
        text: String,
    },
    InlineData {
        inline_data: InlineData,
    },
    FunctionResponse {
        #[serde(rename = "functionResponse")]
        function_response: FunctionResponseData,
    },
    FunctionCall {
        #[serde(rename = "functionCall")]
        function_call: FunctionCallData,
    },
}

#[derive(Debug, Serialize, Clone)]
struct FunctionResponseData {
    name: String,
    response: serde_json::Value,
}

#[derive(Debug, Serialize, Clone)]
struct InlineData {
    mime_type: String,
    data: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Tool {
    GoogleSearch {
        google_search: GoogleSearchTool,
    },
    FunctionDeclarations {
        function_declarations: Vec<FunctionDeclaration>,
    },
}

#[derive(Debug, Serialize)]
struct GoogleSearchTool {}

#[derive(Debug, Serialize)]
struct FunctionDeclaration {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

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
    Text {
        text: String,
    },
    FunctionCall {
        #[serde(rename = "functionCall")]
        function_call: FunctionCallData,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct FunctionCallData {
    name: String,
    args: serde_json::Value,
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
        pool: &SqlitePool,
        context: Option<&SanitizedContext>,
        user_message: &str,
        custom_instruction: Option<&str>,
        user_image: Option<&str>,
    ) -> Result<(String, Option<UsageMetadata>), String> {
        // Fetch memories
        let memories = crate::memory::get_recent_memories(pool, 10)
            .await
            .map_err(|e| format!("Failed to fetch memories: {}", e))?;

        let system_prompt = self.build_system_prompt(context, custom_instruction, &memories);
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
                tracing::info!(
                    "Screenshot data received, length: {} bytes",
                    screenshot.len()
                );

                let image_added = if let Some(base64_data) =
                    screenshot.strip_prefix("data:image/jpeg;base64,")
                {
                    parts.push(Part::InlineData {
                        inline_data: InlineData {
                            mime_type: "image/jpeg".to_string(),
                            data: base64_data.to_string(),
                        },
                    });
                    tracing::info!("Including screenshot (JPEG) in AI request");
                    true
                } else if let Some(base64_data) = screenshot.strip_prefix("data:image/png;base64,")
                {
                    parts.push(Part::InlineData {
                        inline_data: InlineData {
                            mime_type: "image/png".to_string(),
                            data: base64_data.to_string(),
                        },
                    });
                    tracing::info!("Including screenshot (PNG) in AI request");
                    true
                } else if let Some(base64_data) = screenshot.strip_prefix("data:image/webp;base64,")
                {
                    parts.push(Part::InlineData {
                        inline_data: InlineData {
                            mime_type: "image/webp".to_string(),
                            data: base64_data.to_string(),
                        },
                    });
                    tracing::info!("Including screenshot (WebP) in AI request");
                    true
                } else {
                    let prefix: String = screenshot.chars().take(50).collect();
                    tracing::warn!("Screenshot has unrecognized format. Prefix: {}", prefix);
                    false
                };

                if !image_added {
                    tracing::warn!(
                        "Screenshot was NOT included in AI request due to format mismatch"
                    );
                }
            } else {
                tracing::debug!("No screenshot in context");
            }
        }

        // Define save_memory tool
        let _save_memory_tool = FunctionDeclaration {
            name: "save_memory".to_string(),
            description: "Save important information about the user for future reference. Use this when the user asks you to remember something.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "The information to remember"
                    }
                },
                "required": ["content"]
            }),
        };

        let mut contents = vec![Content {
            role: Some("user".to_string()),
            parts,
        }];

        // Tool loop (max 5 iterations)
        for _ in 0..5 {
            let request_tools = vec![
                Tool::GoogleSearch {
                    google_search: GoogleSearchTool {},
                },
                Tool::FunctionDeclarations {
                    function_declarations: vec![FunctionDeclaration {
                        name: "save_memory".to_string(),
                        description: "Save important information about the user for future reference. Use this when the user asks you to remember something.".to_string(),
                        parameters: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "content": {
                                    "type": "string",
                                    "description": "The information to remember"
                                }
                            },
                            "required": ["content"]
                        }),
                    }],
                },
            ];

            let request = GeminiRequest {
                contents: contents.clone(),
                tools: Some(request_tools),
            };

            // API call
            let response = self.call_gemini(&request).await?;

            if let Some(candidates) = &response.candidates {
                if let Some(candidate) = candidates.first() {
                    let mut function_calls_to_execute = Vec::new();
                    let mut text_response = String::new();

                    for part in &candidate.content.parts {
                        match part {
                            ResponsePart::FunctionCall { function_call } => {
                                function_calls_to_execute.push(function_call.clone());
                            }
                            ResponsePart::Text { text } => {
                                text_response.push_str(text);
                            }
                        }
                    }

                    if function_calls_to_execute.is_empty() {
                        return Ok((text_response, response.usage_metadata));
                    }

                    // Process function calls
                    // 1. Add model's turn to history
                    let mut model_parts = Vec::new();
                    if !text_response.is_empty() {
                        model_parts.push(Part::Text {
                            text: text_response.clone(),
                        });
                    }
                    for fc in &function_calls_to_execute {
                        model_parts.push(Part::FunctionCall {
                            function_call: fc.clone(),
                        });
                    }
                    contents.push(Content {
                        role: Some("model".to_string()),
                        parts: model_parts,
                    });

                    // 2. Execute functions and add tool outputs
                    for fc in function_calls_to_execute {
                        if fc.name == "save_memory" {
                            let content = fc
                                .args
                                .get("content")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");

                            tracing::info!("Executing save_memory tool: {}", content);

                            let result = match crate::memory::add_memory(pool, content).await {
                                Ok(id) => serde_json::json!({ "success": true, "id": id }),
                                Err(e) => {
                                    serde_json::json!({ "success": false, "error": e.to_string() })
                                }
                            };

                            contents.push(Content {
                                role: Some("function".to_string()),
                                parts: vec![Part::FunctionResponse {
                                    function_response: FunctionResponseData {
                                        name: "save_memory".to_string(),
                                        response: result,
                                    },
                                }],
                            });
                        } else {
                            // Unknown function
                            contents.push(Content {
                                role: Some("function".to_string()),
                                parts: vec![Part::FunctionResponse {
                                    function_response: FunctionResponseData {
                                        name: fc.name.clone(),
                                        response: serde_json::json!({ "error": "Unknown function" })
                                    }
                                }]
                            });
                        }
                    }

                    // Loop continues to send tool outputs back to model
                    continue;
                }
            }

            if let Some(error) = response.error {
                return Err(format!("API error: {}", error.message));
            }
        }

        Err("Max iterations reached or no response".to_string())
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

    fn build_system_prompt(
        &self,
        context: Option<&SanitizedContext>,
        custom_instruction: Option<&str>,
        memories: &[crate::memory::Memory],
    ) -> String {
        let mut prompt = String::from(
            "Kamu adalah asisten browser yang membantu. Kamu bisa melihat apa yang sedang dijelajahi pengguna dan membantu mereka memahami kontennya.\n\n",
        );

        if let Some(instruction) = custom_instruction {
            prompt.push_str(&format!("INSTRUKSI TAMBAHAN: {}\n\n", instruction));
        }

        // Inject memories
        if !memories.is_empty() {
            prompt.push_str("MEMORI PENGGUNA (hal-hal yang kamu ingat):\n");
            for memory in memories {
                prompt.push_str(&format!("- [{}] {}\n", memory.created_at, memory.content));
            }
            prompt.push_str("\n");
        }

        prompt.push_str(
            "PENTING: Kamu memiliki akses ke:\n\
            1. Konten teks halaman browser\n\
            2. Screenshot dari tampilan browser saat ini\n\
            3. Google Search - gunakan ini untuk mencari informasi terkini di internet\n\
            4. save_memory - gunakan ini untuk menyimpan informasi penting tentang pengguna jika diminta\n\n\
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
