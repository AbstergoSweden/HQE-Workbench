use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use thiserror::Error;
use hqe_protocol::models::MCPToolDefinition;
use serde_json::Value;
use jsonschema::JSONSchema;
use tracing::{debug, warn};

/// A handler function for a tool.
/// Now async to avoid blocking the runtime.
pub type ToolHandler = Box<dyn Fn(Value) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>> + Send + Sync>;

/// Registry for all available MCP tools across all topics.
#[derive(Default, Clone)]
pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, RegisteredTool>>>,
}

/// A registered tool with its schema validator
pub struct RegisteredTool {
    /// Tool definition including input schema
    pub definition: MCPToolDefinition,
    /// Async handler function
    pub handler: ToolHandler,
    /// Topic that registered this tool
    pub topic_id: String,
    /// Compiled JSON schema for validation
    schema_validator: Option<JSONSchema>,
}

/// Errors that can occur during tool operations
#[derive(Debug, Error)]
pub enum ToolError {
    /// Tool not found
    #[error("Tool not found: {0}")]
    NotFound(String),
    /// Invalid arguments provided
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),
    /// Schema compilation failed
    #[error("Schema error: {0}")]
    SchemaError(String),
    /// Handler execution failed
    #[error("Execution error: {0}")]
    ExecutionError(String),
}

impl ToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Compile a JSON schema for validation
    fn compile_schema(schema: &Value) -> Result<JSONSchema, ToolError> {
        JSONSchema::compile(schema)
            .map_err(|e| ToolError::SchemaError(e.to_string()))
    }

    /// Register a new tool from a topic.
    /// Compiles the input schema for validation.
    /// Returns error if schema compilation fails.
    pub async fn register_tool(&self, topic_id: &str, def: MCPToolDefinition, handler: ToolHandler) -> Result<(), ToolError> {
        let mut tools = self.tools.write().await;
        let key = format!("{}__{}", topic_id, def.name);
        
        // Compile the schema for validation
        let schema_validator = match Self::compile_schema(&def.input_schema) {
            Ok(validator) => {
                debug!("Compiled schema for tool: {}", def.name);
                Some(validator)
            }
            Err(e) => {
                warn!(
                    "Failed to compile schema for tool {}: {}. Tool cannot be registered.",
                    def.name, e
                );
                return Err(e);
            }
        };
        
        tools.insert(key, RegisteredTool {
            definition: def,
            handler,
            topic_id: topic_id.to_string(),
            schema_validator,
        });

        Ok(())
    }

    /// List all registered tools.
    pub async fn list_tools(&self) -> Vec<MCPToolDefinition> {
        let tools = self.tools.read().await;
        tools.values().map(|t| t.definition.clone()).collect()
    }

    /// Validate arguments against the tool's input schema
    fn validate_args(tool: &RegisteredTool, args: &Value) -> Result<(), ToolError> {
        if let Some(validator) = &tool.schema_validator {
            match validator.validate(args) {
                Ok(()) => {
                    debug!("Arguments validated successfully for tool: {}", tool.definition.name);
                    Ok(())
                }
                Err(errors) => {
                    let error_messages: Vec<String> = errors
                        .map(|e| format!("{}: {}", e.instance_path, e))
                        .collect();
                    let error_msg = error_messages.join("; ");
                    warn!(
                        "Validation failed for tool {}: {}",
                        tool.definition.name, error_msg
                    );
                    Err(ToolError::InvalidArguments(error_msg))
                }
            }
        } else {
            // No schema validator available, skip validation
            debug!(
                "No schema validator for tool {}, skipping validation",
                tool.definition.name
            );
            Ok(())
        }
    }

    /// Call a tool by name (format: "topic__toolname" or just "toolname" if unique).
    /// Validates arguments against the tool's input schema before calling.
    pub async fn call_tool(&self, name: &str, args: Value) -> Result<Value, ToolError> {
        let tools = self.tools.read().await;
        
        // Simple lookup for now
        if let Some(tool) = tools.get(name) {
            // Validate arguments against schema
            Self::validate_args(tool, &args)?;
            
            // Call the handler
            return (tool.handler)(args).await.map_err(|e| ToolError::ExecutionError(e.to_string()));
        }
        
        Err(ToolError::NotFound(name.to_string()))
    }

    /// Get a tool's definition by name
    pub async fn get_tool(&self, name: &str) -> Option<MCPToolDefinition> {
        let tools = self.tools.read().await;
        tools.get(name).map(|t| t.definition.clone())
    }

    /// Unregister a tool by name
    pub async fn unregister_tool(&self, name: &str) -> bool {
        let mut tools = self.tools.write().await;
        tools.remove(name).is_some()
    }

    /// Clear all registered tools
    pub async fn clear(&self) {
        let mut tools = self.tools.write().await;
        tools.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_handler() -> ToolHandler {
        Box::new(|args| {
            Box::pin(async move {
                Ok(json!({ "received": args }))
            })
        })
    }

    #[tokio::test]
    async fn test_register_and_call_tool() {
        let registry = ToolRegistry::new();
        
        let def = MCPToolDefinition {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" }
                },
                "required": ["name"]
            }),
        };
        
        registry.register_tool("test_topic", def, create_test_handler()).await.expect("Failed to register tool");
        
        let tools = registry.list_tools().await;
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "test_tool");
    }

    #[tokio::test]
    async fn test_validate_args_success() {
        let registry = ToolRegistry::new();
        
        let def = MCPToolDefinition {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "count": { "type": "integer", "minimum": 0 }
                },
                "required": ["count"]
            }),
        };
        
        registry.register_tool("test_topic", def, create_test_handler()).await.expect("Failed to register tool");
        
        // Valid args
        let result = registry.call_tool(
            "test_topic__test_tool",
            json!({ "count": 5 })
        ).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_args_failure() {
        let registry = ToolRegistry::new();
        
        let def = MCPToolDefinition {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "count": { "type": "integer", "minimum": 0 }
                },
                "required": ["count"]
            }),
        };
        
        registry.register_tool("test_topic", def, create_test_handler()).await.expect("Failed to register tool");
        
        // Invalid args - negative count violates minimum
        let result = registry.call_tool(
            "test_topic__test_tool",
            json!({ "count": -1 })
        ).await;
        
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid arguments"));
    }

    #[tokio::test]
    async fn test_tool_not_found() {
        let registry = ToolRegistry::new();
        
        let result = registry.call_tool(
            "nonexistent__tool",
            json!({})
        ).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Tool not found"));
    }

    #[tokio::test]
    async fn test_register_invalid_schema() {
        let registry = ToolRegistry::new();
        
        let def = MCPToolDefinition {
            name: "invalid_tool".to_string(),
            description: "A tool with invalid schema".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "count": { "type": "unknown_type" } // "unknown_type" is not valid JSON schema
                }
            }),
        };
        
        // registration should fail
        let result = registry.register_tool("test_topic", def, create_test_handler()).await;
        
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            ToolError::SchemaError(_) => {}, // Expected
            _ => panic!("Expected SchemaError, got {:?}", err),
        }
    }
}
