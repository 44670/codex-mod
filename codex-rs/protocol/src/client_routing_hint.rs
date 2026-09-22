use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;

/// An observed client-side routing header, not a server model-routing decision.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, JsonSchema, TS)]
pub struct ClientRoutingHintEvent {
    pub header_value: String,
    pub selected_model: String,
    pub routing_model: String,
}

impl ClientRoutingHintEvent {
    /// Observes an unambiguous model alongside the model selected for this request.
    pub fn from_header(header_value: String, selected_model: &str) -> Option<Self> {
        let mut models = header_value.split(';').filter_map(|field| {
            let (name, value) = field.split_once('=')?;
            (name.trim() == "model").then_some(value.trim())
        });
        let routing_model = models.next()?;
        if routing_model.is_empty() || selected_model.is_empty() || models.next().is_some() {
            return None;
        }
        let routing_model = routing_model.to_string();
        Some(Self {
            header_value,
            selected_model: selected_model.to_string(),
            routing_model,
        })
    }
}

#[cfg(test)]
#[path = "client_routing_hint_tests.rs"]
mod tests;
