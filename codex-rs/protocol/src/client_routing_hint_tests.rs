use super::ClientRoutingHintEvent;
use pretty_assertions::assert_eq;

#[test]
fn only_unambiguous_model_mismatches_create_events() {
    for (header, selected, routing) in [
        ("model=selected", "selected", None),
        ("model=selected;tier=priority", "selected", None),
        ("tier=priority; model = selected ", "selected", None),
        ("model=other;tier=priority", "selected", Some("other")),
        ("tier=priority; model = other ", "selected", Some("other")),
        ("model=selected-extra", "selected", Some("selected-extra")),
        ("tier=priority", "selected", None),
        ("model", "selected", None),
        ("model=", "selected", None),
        ("model=other;model=selected", "selected", None),
        ("model=other;model=other", "selected", None),
        ("model=other", "", None),
    ] {
        assert_eq!(
            ClientRoutingHintEvent::from_header(header.to_string(), selected),
            routing.map(|model| ClientRoutingHintEvent {
                header_value: header.to_string(),
                selected_model: selected.to_string(),
                routing_model: model.to_string(),
            }),
            "{header}",
        );
    }
}
