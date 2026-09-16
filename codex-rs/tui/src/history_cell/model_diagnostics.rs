//! Local observations, rendered independently of assistant messages and safety UI.

use super::PrefixedWrappedHistoryCell;
use super::sanitize_user_text;
use codex_app_server_protocol::ServerNotification;
use codex_protocol::protocol::ROUTING_HINT_WARNING_PREFIX;
use ratatui::style::Stylize;
use ratatui::text::Line;
use std::borrow::Cow;

pub(crate) fn new_model_diagnostic(
    notification: &ServerNotification,
) -> Option<PrefixedWrappedHistoryCell> {
    let message = match notification {
        ServerNotification::Warning(event)
            if event.message.starts_with(ROUTING_HINT_WARNING_PREFIX) =>
        {
            event.message.clone()
        }
        ServerNotification::ModelRerouted(event) => format!(
            "ModelReroute: {} -> {} (reason: {:?})",
            event.from_model, event.to_model, event.reason,
        ),
        ServerNotification::ModelSafetyBufferingUpdated(event) => format!(
            "SafetyBuffering: model={}; use_cases={:?}; reasons={:?}; show_buffering_ui={}; faster_model={:?}",
            event.model,
            event.use_cases,
            event.reasons,
            event.show_buffering_ui,
            event.faster_model,
        ),
        // All other notifications retain their existing presentation.
        _ => return None,
    };
    let message = sanitize_user_text(Cow::Owned(message)).into_owned();
    Some(PrefixedWrappedHistoryCell::new(
        Line::from(message.red()),
        "• ".red(),
        "  ".red(),
    ))
}

#[cfg(test)]
#[path = "model_diagnostics_tests.rs"]
mod tests;
