#[path = "../src/dtos/agent.rs"]
mod agent_dto;
use agent_dto::{AgentRequest, InteractiveElementDto};

pub fn format_interactive_elements(elements: &[InteractiveElementDto]) -> String {
    elements
        .iter()
        .map(|e| format!("- Ref {}: {} ({})", e.id, e.name, e.role))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_request_with_interactive_elements() {
        let json_data = r#"{
          "query": "Click Edit Profile",
          "interactive_elements": [
            {"id": 1, "role": "button", "name": "Edit Profile"},
            {"id": 2, "role": "link", "name": "Settings"}
          ]
        }"#;

        let req: AgentRequest =
            serde_json::from_str(json_data).expect("Should deserialize with interactive_elements");

        assert!(req.interactive_elements.is_some());
        let elements = req.interactive_elements.as_ref().unwrap();
        assert_eq!(elements.len(), 2);
        assert_eq!(elements[0].name, "Edit Profile");
        assert_eq!(elements[1].role, "link");
    }

    #[test]
    fn test_format_interactive_elements_for_prompt() {
        // This test defines the expected formatting for interactive elements in the prompt.
        let elements = vec![
            InteractiveElementDto {
                id: 1,
                role: "button".to_string(),
                name: "Edit Profile".to_string(),
            },
            InteractiveElementDto {
                id: 2,
                role: "link".to_string(),
                name: "Settings".to_string(),
            },
        ];

        let formatted = format_interactive_elements(&elements);
        let expected = "- Ref 1: Edit Profile (button)\n- Ref 2: Settings (link)";
        assert_eq!(formatted, expected);
    }
}
