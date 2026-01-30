use rig::agent::MultiTurnStreamItem;
use rig::completion::Prompt;
use rig::message::{ImageMediaType, Message, UserContent};
use rig::providers::gemini;
use rig::streaming::{StreamedAssistantContent, StreamingPrompt};
use rig::OneOrMany;
use rig::prelude::*;

use futures::stream::{Stream, StreamExt};
use std::pin::Pin;
use async_stream::stream;

pub struct GeminiProvider {
    client: gemini::Client,
}

impl GeminiProvider {
    pub fn new(client: gemini::Client) -> Self {
        Self { client }
    }
    
    pub async fn complete(&self, message: &str, custom_instruction: Option<&str>, image: Option<&str>) -> Result<String, String> {
        let mut preamble = "WAJIB: Selalu jawab dalam Bahasa Indonesia kecuali diminta lain.".to_string();
        if let Some(instruction) = custom_instruction {
            preamble.push_str(&format!("\n\nINSTRUKSI TAMBAHAN: {}", instruction));
        }
        
        let agent = self.client
            .agent(gemini::completion::GEMINI_2_5_FLASH)
            .preamble(&preamble)
            .build();
        
        let mut parts = vec![UserContent::text(message.to_string())];
        
        if let Some(img_data) = image {
            let (media_type, data) = parse_image_data(img_data);
            parts.push(UserContent::image_base64(data, Some(media_type), None));
        }
        
        let prompt = Message::User {
            content: OneOrMany::many(parts).expect("Parts list is not empty"),
        };
        
        agent.prompt(prompt).await.map_err(|e| e.to_string())
    }

    pub fn stream(
        &self,
        message: &str,
        custom_instruction: Option<&str>,
        image: Option<&str>,
    ) -> Pin<Box<dyn Stream<Item = Result<String, String>> + Send + 'static>> {
        let mut preamble =
            "WAJIB: Selalu jawab dalam Bahasa Indonesia kecuali diminta lain.".to_string();
        if let Some(instruction) = custom_instruction {
            preamble.push_str(&format!("\n\nINSTRUKSI TAMBAHAN: {}", instruction));
        }

        let client = self.client.clone();
        let message = message.to_string();
        let image = image.map(|s| s.to_string());

        Box::pin(stream! {
            let agent = client
                .agent(gemini::completion::GEMINI_2_5_FLASH)
                .preamble(&preamble)
                .build();

            let mut parts = vec![UserContent::text(message)];

            if let Some(img_data) = image {
                let (media_type, data) = parse_image_data(&img_data);
                parts.push(UserContent::image_base64(data.to_string(), Some(media_type), None));
            }

            let prompt = Message::User {
                content: OneOrMany::many(parts).expect("Parts list is not empty"),
            };

            let mut rig_stream = agent.stream_prompt(prompt).await;

            while let Some(chunk) = rig_stream.next().await {
                match chunk {
                    Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text))) => {
                        yield Ok::<String, String>(text.text);
                    }
                    Ok(_) => {}
                    Err(e) => yield Err::<String, String>(e.to_string()),
                }
            }
        })
    }
}

pub fn parse_image_data(img_data: &str) -> (ImageMediaType, &str) {
    if let Some(stripped) = img_data.strip_prefix("data:image/png;base64,") {
        (ImageMediaType::PNG, stripped)
    } else if let Some(stripped) = img_data.strip_prefix("data:image/jpeg;base64,") {
        (ImageMediaType::JPEG, stripped)
    } else if let Some(stripped) = img_data.strip_prefix("data:image/webp;base64,") {
        (ImageMediaType::WEBP, stripped)
    } else {
        if let Some(comma_pos) = img_data.find(',') {
            (ImageMediaType::JPEG, &img_data[comma_pos + 1..])
        } else {
            (ImageMediaType::JPEG, img_data)
        }
    }
}
