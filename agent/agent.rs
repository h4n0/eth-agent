use crate::{mcp_client::FoundryMcpClient, tools::*, types::*};
use anyhow::Result;
use tracing::{info, warn, error};
use uuid::Uuid;
use std::{f64::consts::E, sync::Arc};
use tokio::sync::Mutex;

use rig::{client::{CompletionClient, ProviderClient}, completion::{Completion, Prompt}};


pub struct EthAgent<T: CompletionClient + ProviderClient + Send + Sync> {
    provider_client: T,
    brave_search_api_key: String,
    planning_model: String,
    execution_model: String,
    evaluation_model: String,
    evaluation_threshold: u32,
}

impl<T: CompletionClient + ProviderClient + Send + Sync> EthAgent<T> {
    pub fn new(
        brave_search_api_key: &str,
        planning_model: &str,
        execution_model: &str,
        evaluation_model: &str,
        evaluation_threshold: u32,
    ) -> Result<Self> {
        let provider_client = T::from_env();

        Ok(Self {
            provider_client,
            brave_search_api_key: brave_search_api_key.to_string(),
            planning_model: planning_model.to_string(),
            execution_model: execution_model.to_string(),
            evaluation_model: evaluation_model.to_string(),
            evaluation_threshold: evaluation_threshold,
        })
    }

    pub async fn run(&mut self, prompt: UserPrompt) -> Result<AgentResult> {
        info!("Starting agent execution for prompt: {}", prompt.natural_language);

        let mut plan_counter = 0;

        const MAX_PLAN_RETRIES: u32 = 3;

        // TODO: refactor
        let mut replan_reason: Option<String> = None;

        while plan_counter < MAX_PLAN_RETRIES {


            // Step 1: Plan
            plan_counter += 1;
            let plan = match self.plan(&prompt, &replan_reason).await {
                Ok(plan) => plan,
                Err(e) => {
                    error!("Plan creation failed: {}", e);
                    return Err(e);
                }
            };
            info!("Plan created: {:?}", plan);

            // Step 2: Agent loop
            let res = match self.agent_loop(&prompt, &plan).await {
                Ok(result) => result,
                Err(e) => {
                    error!("Agent loop failed: {}", e.error_message);
                    // If replan is true, continue to the next plan
                    if e.replan {
                        replan_reason = Some(e.error_message.clone());
                        continue;
                    }
                    return Err(anyhow::anyhow!("Agent loop failed: {}", e.error_message));
                }
            };

            info!("Agent loop completed with result: {:?}", res);
            return Ok(res);
        }

        return Err(anyhow::anyhow!("Agent loop failed with max retries"));
    }

    async fn plan(&self, prompt: &UserPrompt, replan_reason: &Option<String>) -> Result<AgentPlan> {
        info!("Creating execution plan for prompt: {}", prompt.natural_language);

        const PREAMBLE: &str = r#"
        You are a helpful assistant that creates execution plans for Ethereum transactions.

        The output MUST be a valid JSON object in the following format:
        {{
            "number_of_steps": 1-10,
            "steps": [
                {{
                    "step_number": 1-10,
                    "agent_name": "ethereum_agent",
                    "agent_prompt": "Prompt for the agent to execute",
                }}
            ]
        }}


        Sub-agents:
        - ethereum_agent: An agent that can send transactions to the Ethereum network, with the following tools:
            - send_transaction: Send a transaction to the Ethereum network
            - validate_address: Validate an Ethereum address
            - balance: Get the balance of an Ethereum address
            - get_contract_code: Get the contract code of an Ethereum address
        - search_agent: An agent that can search the web for information
            - search: Search the web for information
        

        Example prompt:
        - Send 0.001 ETH from Alice to Bob
        - What is the balance of Alice?
        - What is the current price of ETH?

        Known addresses:
        Alice: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
        Bob: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8

        "#;

        let planner_client = self.provider_client.agent(&self.planning_model)
        .preamble(PREAMBLE)
        .build();

        info!("Planner client initialized");

        // If replan reason is provided, add it to the user prompt
        let user_prompt = if let Some(replan_reason_str) = replan_reason {
            format!("This a replan request. The user prompt is: {} and the replan reason is: {}", prompt.natural_language.clone(), replan_reason_str)
        } else {
            prompt.natural_language.clone()
        };

        info!("User prompt: {}", user_prompt);

        let prompt_request = planner_client.prompt(user_prompt);


        let plan_response = prompt_request.await?;

        info!("Plan response: {}", plan_response.clone());

        let actual_plan = if plan_response.contains("```json") {
            // Keep the content between ```json and ``` from the plan response
            plan_response.split("```json").nth(1).unwrap().split("```").nth(0).unwrap().to_string()

        } else {
            plan_response
        };

        info!("Actual plan: {}", actual_plan);

        let agent_plan: AgentPlanResponse = serde_json::from_str(&actual_plan)?;

        Ok(AgentPlan {
            id: Uuid::new_v4().to_string(),
            prompt: prompt.clone(),
            steps: agent_plan.steps,
            max_steps: agent_plan.number_of_steps,
            current_step: 0,
        })
    }

    async fn agent_loop(&self, prompt: &UserPrompt, agent_plan: &AgentPlan) -> Result<AgentResult, AgentPlanError> {
        info!("Creating execution plan for prompt: {}", prompt.natural_language);

        const ETHEREUM_PREAMBLE: &str = "
        You are a helpful assistant that creates execution plans for Ethereum transactions.


        Tools:
        - send_transaction: Send a transaction to the Ethereum network
        - balance: Get the balance of an Ethereum address
        - validate_address: Validate an Ethereum address
        - get_contract_code: Get the contract code of an Ethereum address

        Known addresses:
        Alice: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
        Bob: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8

        ";

        const SEARCH_PREAMBLE: &str = "
        You are a helpful assistant that can search the web for information.

        Tools:
        - web_search: Search the web for information
        ";

        info!("Initializing MCP client...");
        let client = match FoundryMcpClient::new().await {
            Ok(client) => {
                info!("MCP client initialized successfully");
                Arc::new(Mutex::new(client))
            }
            Err(e) => {
                error!("Failed to initialize MCP client: {}", e);
                return Err(AgentPlanError {
                    error_message: format!("MCP client initialization failed: {}", e),
                    replan: false,
                });
            }
        };

        info!("Looping through steps...");

        let ethereum_agent = self.provider_client.agent(&self.execution_model)
        .preamble(ETHEREUM_PREAMBLE)
        .tool(SendTransactionTool::new(client.clone()))
        .tool(BalanceTool::new(client.clone()))
        .tool(GetContractCodeTool::new(client.clone()))
        .tool(ValidateAddressTool::new(client.clone()))
        .temperature(0.7)
        .build();

        let search_agent = self.provider_client.agent(&self.execution_model)
        .preamble(SEARCH_PREAMBLE)
        .tool(WebSearchTool::new(self.brave_search_api_key.clone()))
        .temperature(0.7)
        .build();

        // Implement memory
        let mut memory = vec![];

        for step in &agent_plan.steps {
            info!("Step: {}", step.step_number);
            match step.agent_name.as_str() {
                "ethereum_agent" => {
                    let response = match ethereum_agent.prompt(step.agent_prompt.clone() + "Previous steps: " + &memory.join("\n")).await {
                        Ok(response) => {
                            info!("Response: {}", response);
                            response
                        }
                        Err(e) => {
                            error!("Failed to get response from ethereum agent: {}", e);
                            return Err(AgentPlanError {
                                error_message: format!("Failed to get response from ethereum agent: {}", e),
                                replan: false,
                            });
                        }
                    };

                    memory.push(response.clone());
                    info!("Response: {}", response);
                }
                "search_agent" => {
                    let response = match search_agent.prompt(step.agent_prompt.clone() + "Previous steps: " + &memory.join("\n")).multi_turn(3).await {
                        Ok(response) => {
                            info!("Response: {}", response);
                            response
                        }
                        Err(e) => {
                            error!("Failed to get response from search agent: {}", e);
                            return Err(AgentPlanError {
                                error_message: format!("Failed to get response from search agent: {}", e),
                                replan: false,
                            });
                        }
                    };

                    memory.push(response.clone());
                    info!("Response: {}", response);
                }
                _ => {
                    error!("Unknown agent name: {}", step.agent_name);
                    return Err(AgentPlanError {
                        error_message: format!("Unknown agent name: {}", step.agent_name),
                        replan: true,
                    });
                }
            }



            if let Ok(evaluation) = self.evaluate_result(&prompt, &step.agent_prompt, &memory.last().unwrap().clone()).await {
                info!("Evaluation: {:?}", evaluation);

                if evaluation.score < self.evaluation_threshold {
                    error!("Evaluation score is below threshold: {}, returning error", evaluation.score);
                    return Err(AgentPlanError {
                        error_message: format!("Evaluation score is below threshold: {}", evaluation.score),
                        replan: true,
                    });
                }
            } else {
                error!("Evaluation failed");
                return Err(AgentPlanError {
                    error_message: format!("Evaluation failed"),
                    replan: true,
                });
            }
        }

        let result = memory.last().unwrap_or(&"Failed to get response from agent".to_string()).clone();

        info!("Agent loop response: {:?}", result);

        Ok(AgentResult {
            error_message: None,
            result: result,
        })
    }

    async fn evaluate_result(&self, original_prompt: &UserPrompt, agent_prompt: &str, result: &str) -> Result<EvaluationResult> {
        info!("Evaluating execution result against original prompt");

        const EVALUATION_PREAMBLE: &str = r#"
        You are an evaluator of agent execution results.
        You will be given a result from an agent and an agent prompt that the agent was given, and a user prompt that the agent was given.
        You will need to evaluate the result and determine if it is aligned with the agent prompt and user prompt.
        You will need to return a score between 0 and 100.

        You should only output a valid JSON object in the following format:
        {{
            "score": 0-100,
            "reasoning": "Reasoning for the score"
        }}
        "#;
        let evaluation_client = self.provider_client.agent(&self.evaluation_model)
        .preamble(EVALUATION_PREAMBLE)
        .build();

        let evaluation_response = evaluation_client.prompt(format!("Evaluate the following result: {} against the current agent prompt: {} and user prompt: {}", result, agent_prompt, original_prompt.natural_language)).await?;

        // Remove ```json and ``` from the evaluation response if they exist
        let evaluation_response = evaluation_response.replace("```json", "").replace("```", "");

        let evaluation_response: EvaluationScoreResponse = serde_json::from_str(&evaluation_response)?;

        info!("Evaluation score: {} and reasoning: {}", evaluation_response.score, evaluation_response.reasoning);

        Ok(EvaluationResult {
            plan_id: Uuid::new_v4().to_string(),
            original_prompt: original_prompt.natural_language.clone(),
            score: evaluation_response.score,
            reasoning: evaluation_response.reasoning,
        })
    }
} 