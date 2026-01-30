use anyhow::{anyhow, Result};
use hqe_mcp::ToolRegistry;
use hqe_protocol::models::{WorkflowDefinition, WorkflowStep};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument};

/// Engine for managing and executing workflows
#[derive(Clone)]
pub struct FlowEngine {
    flows: Arc<RwLock<HashMap<String, WorkflowDefinition>>>,
    tool_registry: ToolRegistry,
}

impl FlowEngine {
    /// Create a new flow engine
    pub fn new(tool_registry: ToolRegistry) -> Self {
        Self {
            flows: Arc::new(RwLock::new(HashMap::new())),
            tool_registry,
        }
    }

    /// Register a new workflow definition
    pub async fn register_flow(&self, flow: WorkflowDefinition) {
        info!("Registering flow: {}", flow.id);
        let mut flows = self.flows.write().await;
        flows.insert(flow.id.clone(), flow);
    }

    /// List all registered workflows
    pub async fn list_flows(&self) -> Vec<WorkflowDefinition> {
        let flows = self.flows.read().await;
        flows.values().cloned().collect()
    }

    /// Execute a workflow by ID
    #[instrument(skip(self, input))]
    pub async fn execute_flow(&self, flow_id: &str, input: Value) -> Result<Value> {
        let flow = {
            let flows = self.flows.read().await;
            flows
                .get(flow_id)
                .ok_or_else(|| anyhow!("Flow not found: {}", flow_id))?
                .clone()
        };

        info!("Starting flow execution: {}", flow.name);

        let mut context = input;

        for step in &flow.steps {
            context = self.execute_step(step, context).await?;
        }

        Ok(context)
    }

    async fn execute_step(&self, step: &WorkflowStep, _input: Value) -> Result<Value> {
        match step.action.as_str() {
            "call_tool" => {
                let tool_name = step
                    .params
                    .get("tool")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Step {} missing 'tool' param", step.id))?;

                let args = step.params.clone();

                info!("Step {}: Calling tool {}", step.id, tool_name);
                self.tool_registry
                    .call_tool(tool_name, args)
                    .await
                    .map_err(|e| anyhow!(e))
            }
            _ => Err(anyhow!("Unknown action: {}", step.action)),
        }
    }
}
