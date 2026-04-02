//! Cost tracking for LLM API usage.
//!
//! Tracks cumulative token usage and estimated cost per session,
//! with configurable per-token rates for different models.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Per-token pricing for a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Model identifier (e.g., "gpt-4o", "claude-sonnet-4-20250514").
    pub model: String,
    /// Cost per input (prompt) token in USD.
    pub input_cost_per_token: f64,
    /// Cost per output (completion) token in USD.
    pub output_cost_per_token: f64,
}

/// Token usage for a single request.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Number of input (prompt) tokens.
    pub input_tokens: u64,
    /// Number of output (completion) tokens.
    pub output_tokens: u64,
}

impl TokenUsage {
    /// Total tokens (input + output).
    pub fn total(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }
}

/// Session-level cost tracker.
///
/// Accumulates token usage across multiple requests and computes
/// estimated cost based on model pricing.
pub struct CostTracker {
    /// Cumulative token usage for the session.
    cumulative: TokenUsage,
    /// Per-request history.
    requests: Vec<TokenUsage>,
    /// Pricing table keyed by model name.
    pricing: HashMap<String, ModelPricing>,
    /// The current model being used.
    current_model: String,
}

impl CostTracker {
    /// Create a new cost tracker for the given model.
    pub fn new(model: &str) -> Self {
        let mut tracker = Self {
            cumulative: TokenUsage::default(),
            requests: Vec::new(),
            pricing: HashMap::new(),
            current_model: model.to_string(),
        };
        tracker.load_default_pricing();
        tracker
    }

    /// Load default pricing for well-known models.
    fn load_default_pricing(&mut self) {
        let defaults = vec![
            ModelPricing {
                model: "gpt-4o".to_string(),
                input_cost_per_token: 2.5e-6,
                output_cost_per_token: 10.0e-6,
            },
            ModelPricing {
                model: "gpt-4o-mini".to_string(),
                input_cost_per_token: 0.15e-6,
                output_cost_per_token: 0.6e-6,
            },
            ModelPricing {
                model: "gpt-4-turbo".to_string(),
                input_cost_per_token: 10.0e-6,
                output_cost_per_token: 30.0e-6,
            },
            ModelPricing {
                model: "claude-sonnet-4-20250514".to_string(),
                input_cost_per_token: 3.0e-6,
                output_cost_per_token: 15.0e-6,
            },
            ModelPricing {
                model: "claude-opus-4-20250514".to_string(),
                input_cost_per_token: 15.0e-6,
                output_cost_per_token: 75.0e-6,
            },
            ModelPricing {
                model: "claude-haiku-3-5".to_string(),
                input_cost_per_token: 0.25e-6,
                output_cost_per_token: 1.25e-6,
            },
        ];

        for p in defaults {
            self.pricing.insert(p.model.clone(), p);
        }
    }

    /// Set custom pricing for a model.
    pub fn set_pricing(&mut self, pricing: ModelPricing) {
        self.pricing.insert(pricing.model.clone(), pricing);
    }

    /// Set the current model.
    pub fn set_model(&mut self, model: &str) {
        self.current_model = model.to_string();
    }

    /// Record token usage from a single request.
    pub fn record(&mut self, usage: TokenUsage) {
        self.cumulative.input_tokens += usage.input_tokens;
        self.cumulative.output_tokens += usage.output_tokens;
        self.requests.push(usage);
    }

    /// Get cumulative token usage.
    pub fn cumulative_usage(&self) -> &TokenUsage {
        &self.cumulative
    }

    /// Get the number of requests recorded.
    pub fn request_count(&self) -> usize {
        self.requests.len()
    }

    /// Estimate the total cost in USD for the session.
    ///
    /// Returns `None` if no pricing is available for the current model.
    pub fn estimated_cost(&self) -> Option<f64> {
        let pricing = self.pricing.get(&self.current_model)?;
        let cost = (self.cumulative.input_tokens as f64 * pricing.input_cost_per_token)
            + (self.cumulative.output_tokens as f64 * pricing.output_cost_per_token);
        Some(cost)
    }

    /// Estimate cost for a specific usage.
    pub fn estimate_usage_cost(&self, usage: &TokenUsage) -> Option<f64> {
        let pricing = self.pricing.get(&self.current_model)?;
        let cost = (usage.input_tokens as f64 * pricing.input_cost_per_token)
            + (usage.output_tokens as f64 * pricing.output_cost_per_token);
        Some(cost)
    }

    /// Format a cost summary for display.
    pub fn format_summary(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("Model: {}", self.current_model));
        lines.push(format!("Requests: {}", self.requests.len()));
        lines.push(format!(
            "Tokens: {} input, {} output, {} total",
            self.cumulative.input_tokens,
            self.cumulative.output_tokens,
            self.cumulative.total(),
        ));

        match self.estimated_cost() {
            Some(cost) => {
                lines.push(format!("Estimated cost: ${:.6}", cost));
            }
            None => {
                lines.push(format!(
                    "Estimated cost: unknown (no pricing for '{}')",
                    self.current_model
                ));
            }
        }

        lines.join("\n")
    }

    /// Format cost as a short inline string (e.g., for status bars).
    pub fn format_short(&self) -> String {
        let total = self.cumulative.total();
        match self.estimated_cost() {
            Some(cost) => format!("{}tok ${:.4}", total, cost),
            None => format!("{}tok", total),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_cumulative() {
        let mut tracker = CostTracker::new("gpt-4o");
        tracker.record(TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
        });
        tracker.record(TokenUsage {
            input_tokens: 200,
            output_tokens: 100,
        });

        assert_eq!(tracker.cumulative_usage().input_tokens, 300);
        assert_eq!(tracker.cumulative_usage().output_tokens, 150);
        assert_eq!(tracker.cumulative_usage().total(), 450);
        assert_eq!(tracker.request_count(), 2);
    }

    #[test]
    fn test_estimated_cost() {
        let mut tracker = CostTracker::new("gpt-4o");
        tracker.record(TokenUsage {
            input_tokens: 1000,
            output_tokens: 500,
        });

        let cost = tracker.estimated_cost().unwrap();
        // 1000 * 2.5e-6 + 500 * 10e-6 = 0.0025 + 0.005 = 0.0075
        assert!((cost - 0.0075).abs() < 1e-10);
    }

    #[test]
    fn test_unknown_model_cost() {
        let mut tracker = CostTracker::new("my-custom-model");
        tracker.record(TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
        });
        assert!(tracker.estimated_cost().is_none());
    }

    #[test]
    fn test_custom_pricing() {
        let mut tracker = CostTracker::new("my-model");
        tracker.set_pricing(ModelPricing {
            model: "my-model".to_string(),
            input_cost_per_token: 1e-6,
            output_cost_per_token: 2e-6,
        });
        tracker.record(TokenUsage {
            input_tokens: 1000,
            output_tokens: 500,
        });
        let cost = tracker.estimated_cost().unwrap();
        // 1000 * 1e-6 + 500 * 2e-6 = 0.001 + 0.001 = 0.002
        assert!((cost - 0.002).abs() < 1e-10);
    }

    #[test]
    fn test_format_summary() {
        let mut tracker = CostTracker::new("gpt-4o");
        tracker.record(TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
        });
        let summary = tracker.format_summary();
        assert!(summary.contains("gpt-4o"));
        assert!(summary.contains("100 input"));
        assert!(summary.contains("50 output"));
        assert!(summary.contains("Estimated cost: $"));
    }

    #[test]
    fn test_format_short() {
        let mut tracker = CostTracker::new("gpt-4o");
        tracker.record(TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
        });
        let short = tracker.format_short();
        assert!(short.contains("150tok"));
        assert!(short.contains("$"));
    }
}
