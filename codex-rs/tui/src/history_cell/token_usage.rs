//! Token usage transcript cells.

use super::*;
use crate::token_usage::TokenUsage;
use codex_protocol::num_format::format_with_separators;

pub(crate) fn new_turn_token_usage(usage: &TokenUsage) -> PlainHistoryCell {
    let reasoning_tokens = usage.reasoning_output_tokens;
    let reasoning = format_with_separators(reasoning_tokens);
    let reasoning = if matches!(reasoning_tokens, 516 | 1_034 | 1_552) {
        reasoning.red()
    } else if reasoning_tokens > 1_000 {
        reasoning.green()
    } else {
        reasoning.dim()
    };

    PlainHistoryCell::new(vec![
        vec![
            "• ".dim(),
            "tokens: ".dim(),
            "output ".dim(),
            format_with_separators(usage.output_tokens).dim(),
            " · ".dim(),
            "reasoning ".dim(),
            reasoning,
        ]
        .into(),
    ])
}
