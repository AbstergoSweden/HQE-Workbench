use hqe_flow::FlowEngine;
use hqe_mcp::ToolRegistry;
use hqe_protocol::models::{MCPToolDefinition, WorkflowDefinition, WorkflowStep};
use serde_json::json;

#[tokio::test]
async fn test_flow_execution() -> anyhow::Result<()> {
    // 1. Setup Registry and Engine
    let registry = ToolRegistry::new();
    let engine = FlowEngine::new(registry.clone());

    // 2. Register a mock tool "get_stock_price"
    let tool_def = MCPToolDefinition {
        name: "get_stock_price".to_string(),
        description: "Mock stock price".to_string(),
        input_schema: json!({}),
    };

    registry
        .register_tool(
            "finance",
            tool_def,
            Box::new(|args| {
                // Mock implementation - now async
                Box::pin(async move {
                    let ticker = args
                        .get("ticker")
                        .and_then(|s| s.as_str())
                        .unwrap_or("UNKNOWN");
                    Ok(json!({
                        "symbol": ticker,
                        "price": 150.00
                    }))
                })
            }),
        )
        .await
        .expect("Failed to register tool");

    // 3. Register a Flow that uses the tool
    let flow = WorkflowDefinition {
        id: "daily_briefing".to_string(),
        name: "Daily Market Briefing".to_string(),
        steps: vec![WorkflowStep {
            id: "step1".to_string(),
            action: "call_tool".to_string(),
            params: json!({
                "tool": "get_stock_price", // Registry lookup needs to match this.
                // In registry.rs, keys are "topic__name".
                // We need to fix the lookup or the registration key.
                "ticker": "AAPL"
            }),
        }],
    };
    engine.register_flow(flow).await;

    // 4. Execute Flow
    // Note: The registry key logic in `hqe-mcp` prefixes topic_id.
    // The flow calls "get_stock_price".
    // We need to ensure consistency.
    // Let's check `hqe-mcp/src/registry.rs`: `key = format!("{}__{}", topic_id, def.name);`
    // So if I register with topic "finance" and tool "get_stock_price", key is "finance__get_stock_price".
    // But my flow calls "get_stock_price".
    // The `call_tool` method needs to handle this or I need to register with "" topic?

    // For this test, let's fix the registration to match what the flow expects,
    // OR update the flow to use the namespaced name.
    // Let's update the Flow to use "finance__get_stock_price" for correctness.

    // WAIT, I can't easily change the Flow definition inside the `json!` macro above without string manipulation
    // or changing the `call_tool` implementation.

    // Let's assume for now that `call_tool` argument IS the full key.

    // Re-registering flow with namespaced tool name:
    let flow_fixed = WorkflowDefinition {
        id: "daily_briefing".to_string(),
        name: "Daily Market Briefing".to_string(),
        steps: vec![WorkflowStep {
            id: "step1".to_string(),
            action: "call_tool".to_string(),
            params: json!({
                "tool": "finance__get_stock_price",
                "ticker": "AAPL"
            }),
        }],
    };

    // Re-create engine to clear previous
    let engine = FlowEngine::new(registry.clone());
    engine.register_flow(flow_fixed).await;

    let result = engine.execute_flow("daily_briefing", json!({})).await;

    assert!(result.is_ok());
    let value = result?;
    assert_eq!(value["price"], 150.00);
    assert_eq!(value["symbol"], "AAPL");
    Ok(())
}
