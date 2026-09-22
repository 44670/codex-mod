//! Local observations, rendered independently of assistant messages and safety UI.

use super::PrefixedWrappedHistoryCell;
use super::sanitize_user_text;
use codex_app_server_protocol::ServerNotification;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use std::borrow::Cow;

pub(crate) fn new_model_diagnostic(
    notification: &ServerNotification,
) -> Option<PrefixedWrappedHistoryCell> {
    let message = match notification {
        ServerNotification::ResponseModel(event) => format!(
            "response.completed.response.model: {}",
            event.response_model,
        ),
        ServerNotification::ClientRoutingHint(event) => {
            format!("x-codex-routing-hint: {}", event.header_value,)
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
    let style = match notification {
        ServerNotification::ResponseModel(event)
            if event.response_model == event.selected_model =>
        {
            Style::new().dim()
        }
        ServerNotification::ClientRoutingHint(event)
            if event.routing_model == event.selected_model =>
        {
            Style::new().dim()
        }
        _ => Style::new().red(),
    };
    let message = sanitize_user_text(Cow::Owned(message)).into_owned();
    Some(PrefixedWrappedHistoryCell::new(
        Line::from(Span::styled(message, style)),
        Span::styled("• ", style),
        Span::styled("  ", style),
    ))
}

#[cfg(test)]
#[path = "model_diagnostics_tests.rs"]
mod tests;
