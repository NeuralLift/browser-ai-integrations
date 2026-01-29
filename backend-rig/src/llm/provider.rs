use rig::completion::Prompt;
use rig::message::{ImageMediaType, Message, UserContent};
use rig::providers::gemini;
use rig::OneOrMany;
use rig::prelude::*;

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
