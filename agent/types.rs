use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPrompt {
    pub id: String,
    pub natural_language: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub context: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub error_message: Option<String>,
    pub result: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    pub step_number: u32,
    pub agent_name: String,
    pub agent_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepStatus {
    Planned,
    Executing,
    Completed,
    Failed(String),
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPlanResponse {
    pub number_of_steps: u32,
    pub steps: Vec<AgentStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPlan {
    pub id: String,
    pub prompt: UserPrompt,
    pub steps: Vec<AgentStep>,
    pub max_steps: u32,
    pub current_step: u32,
    // TODO: Add status
    //pub status: PlanStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPlanError {
    pub error_message: String,
    pub replan: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanStatus {
    Planning,
    Executing,
    Completed,
    Failed(String),
    MaxStepsReached,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationScoreResponse {
    pub score: u32,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub plan_id: String,
    pub original_prompt: String,
    pub score: u32,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub description: String,
    pub success_criteria: Vec<String>,
    pub constraints: Vec<String>,
    pub priority: Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deviation {
    pub id: String,
    pub goal_id: String,
    pub description: String,
    pub severity: DeviationSeverity,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub correction_attempts: Vec<CorrectionAttempt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionAttempt {
    pub id: String,
    pub description: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub success: bool,
    pub error_message: Option<String>,
} 