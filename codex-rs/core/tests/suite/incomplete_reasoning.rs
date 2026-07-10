use anyhow::Result;
use core_test_support::responses;
use core_test_support::skip_if_no_network;
use core_test_support::test_codex::test_codex;
use pretty_assertions::assert_eq;
use serde_json::Value;

const INCOMPLETE_REASONING_FOLLOW_UP: &str = "Your previous reasoning was incomplete. Think deeply again, and do not send optional commentary.";

fn ev_completed_with_reasoning_tokens(id: &str, reasoning_tokens: i64) -> Value {
    serde_json::json!({
        "type": "response.completed",
        "response": {
            "id": id,
            "usage": {
                "input_tokens": 0,
                "input_tokens_details": null,
                "output_tokens": reasoning_tokens,
                "output_tokens_details": {
                    "reasoning_tokens": reasoning_tokens,
                },
                "total_tokens": reasoning_tokens,
            }
        }
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn incomplete_reasoning_tokens_trigger_one_follow_up_request() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let server = responses::start_mock_server().await;
    let response_mock = responses::mount_sse_sequence(
        &server,
        vec![
            responses::sse(vec![
                responses::ev_response_created("resp-1"),
                responses::ev_assistant_message("msg-1", "first answer"),
                ev_completed_with_reasoning_tokens("resp-1", 1_034),
            ]),
            responses::sse(vec![
                responses::ev_response_created("resp-2"),
                responses::ev_assistant_message("msg-2", "final answer"),
                responses::ev_completed("resp-2"),
            ]),
        ],
    )
    .await;
    let test = test_codex().build_with_auto_env(&server).await?;

    test.submit_turn("solve carefully").await?;

    let requests = response_mock.requests();
    assert_eq!(requests.len(), 2);
    let user_texts = requests[1].message_input_texts("user");
    assert_eq!(
        &user_texts[user_texts.len() - 2..],
        &[
            "solve carefully".to_string(),
            INCOMPLETE_REASONING_FOLLOW_UP.to_string(),
        ]
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn incomplete_reasoning_tokens_during_tool_call_trigger_follow_up_on_next_request()
-> Result<()> {
    skip_if_no_network!(Ok(()));

    let server = responses::start_mock_server().await;
    let call_id = "plan-call";
    let plan_args = serde_json::json!({
        "explanation": "Tool chain check",
        "plan": [
            {"step": "Run tool", "status": "in_progress"},
        ],
    })
    .to_string();
    let response_mock = responses::mount_sse_sequence(
        &server,
        vec![
            responses::sse(vec![
                responses::ev_response_created("resp-1"),
                responses::ev_function_call(call_id, "update_plan", &plan_args),
                ev_completed_with_reasoning_tokens("resp-1", 516),
            ]),
            responses::sse(vec![
                responses::ev_response_created("resp-2"),
                responses::ev_assistant_message("msg-2", "tool chain finished"),
                responses::ev_completed("resp-2"),
            ]),
        ],
    )
    .await;
    let test = test_codex().build_with_auto_env(&server).await?;

    test.submit_turn("solve carefully").await?;

    let requests = response_mock.requests();
    assert_eq!(requests.len(), 2);
    let _ = requests[1].function_call_output(call_id);
    let user_texts = requests[1].message_input_texts("user");
    assert_eq!(
        &user_texts[user_texts.len() - 2..],
        &[
            "solve carefully".to_string(),
            INCOMPLETE_REASONING_FOLLOW_UP.to_string(),
        ]
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn incomplete_reasoning_follow_up_is_not_count_limited() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let server = responses::start_mock_server().await;
    let mut response_sequence: Vec<_> = (1..=7)
        .map(|response_index| {
            responses::sse(vec![
                responses::ev_response_created(&format!("resp-{response_index}")),
                responses::ev_assistant_message(
                    &format!("msg-{response_index}"),
                    &format!("answer {response_index}"),
                ),
                ev_completed_with_reasoning_tokens(&format!("resp-{response_index}"), 516),
            ])
        })
        .collect();
    response_sequence.push(responses::sse(vec![
        responses::ev_response_created("resp-8"),
        responses::ev_assistant_message("msg-8", "final answer"),
        responses::ev_completed("resp-8"),
    ]));
    let response_mock = responses::mount_sse_sequence(&server, response_sequence).await;
    let test = test_codex().build_with_auto_env(&server).await?;

    test.submit_turn("solve carefully").await?;

    let requests = response_mock.requests();
    assert_eq!(requests.len(), 8);
    let user_texts = requests[7].message_input_texts("user");
    assert_eq!(
        user_texts,
        std::iter::once("solve carefully".to_string())
            .chain(std::iter::repeat_n(
                INCOMPLETE_REASONING_FOLLOW_UP.to_string(),
                7,
            ))
            .collect::<Vec<_>>()
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reasoning_tokens_that_do_not_match_do_not_trigger_follow_up() -> Result<()> {
    skip_if_no_network!(Ok(()));

    for reasoning_tokens in [1_001, 1_032, 1_548, 3_096] {
        let server = responses::start_mock_server().await;
        let response_mock = responses::mount_sse_once(
            &server,
            responses::sse(vec![
                responses::ev_response_created("resp-1"),
                responses::ev_assistant_message("msg-1", "final answer"),
                ev_completed_with_reasoning_tokens("resp-1", reasoning_tokens),
            ]),
        )
        .await;
        let test = test_codex().build_with_auto_env(&server).await?;

        test.submit_turn("solve carefully").await?;

        response_mock.single_request();
    }

    Ok(())
}
