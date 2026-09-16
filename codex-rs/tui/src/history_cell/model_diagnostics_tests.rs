use super::*;
use crate::history_cell::HistoryCell;
use codex_app_server_protocol::ModelRerouteReason;
use codex_app_server_protocol::ModelReroutedNotification;
use codex_app_server_protocol::ModelSafetyBufferingUpdatedNotification;
use codex_app_server_protocol::WarningNotification;
use pretty_assertions::assert_eq;
use ratatui::style::Color;

#[test]
fn model_diagnostics_are_red_and_render_observed_values() {
    let notifications = [
        ServerNotification::Warning(WarningNotification {
            thread_id: Some("thread".into()),
            message: format!("{ROUTING_HINT_WARNING_PREFIX}model=requested-model;tier=priority"),
        }),
        ServerNotification::ModelRerouted(ModelReroutedNotification {
            thread_id: "thread".into(),
            turn_id: "turn".into(),
            from_model: "requested-model".into(),
            to_model: "fallback-model".into(),
            reason: ModelRerouteReason::HighRiskCyberActivity,
        }),
        ServerNotification::ModelSafetyBufferingUpdated(ModelSafetyBufferingUpdatedNotification {
            thread_id: "thread".into(),
            turn_id: "turn".into(),
            model: "requested-model".into(),
            use_cases: vec!["cyber".into()],
            reasons: vec!["review".into()],
            show_buffering_ui: false,
            faster_model: None,
        }),
    ];
    let lines = notifications
        .iter()
        .flat_map(|notification| {
            new_model_diagnostic(notification)
                .expect("diagnostic")
                .display_lines(/*width*/ 200)
        })
        .collect::<Vec<_>>();
    for line in &lines {
        for span in &line.spans {
            assert_eq!(span.style.fg.or(line.style.fg), Some(Color::Red));
        }
    }
    insta::assert_snapshot!(
        lines.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n"),
        @r#"
    • x-codex-routing-hint (request): model=requested-model;tier=priority
    • ModelReroute: requested-model -> fallback-model (reason: HighRiskCyberActivity)
    • SafetyBuffering: model=requested-model; use_cases=["cyber"]; reasons=["review"]; show_buffering_ui=false; faster_model=None
    "#
    );
}

#[test]
fn routing_hint_wraps_and_cannot_inject_terminal_colors() {
    assert!(
        new_model_diagnostic(&ServerNotification::Warning(WarningNotification {
            thread_id: Some("thread".into()),
            message: "unrelated warning".into(),
        }))
        .is_none()
    );
    let notification = ServerNotification::Warning(WarningNotification {
        thread_id: Some("thread".into()),
        message: format!("{ROUTING_HINT_WARNING_PREFIX}\x1b[32mmodel=requested-model\x1b[0m"),
    });
    let cell = new_model_diagnostic(&notification).expect("diagnostic");
    let lines = cell.display_lines(/*width*/ 40);
    assert!(lines.iter().all(|line| line.width() <= 40));
    insta::assert_snapshot!(
        lines.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n"),
        @"
    • x-codex-routing-hint (request):
      model=requested-model
    "
    );
}
