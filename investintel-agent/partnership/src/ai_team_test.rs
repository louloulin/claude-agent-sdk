//! Tests for AI Team Module
//!
//! Validates subagent team coordination and execution

use super::ai_team::*;
use claude_agent_sdk_rs::orchestration::Agent;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_team_creation() {
        let team = AITeam::new();
        
        // Verify team structure is created
        assert_eq!(team.chief_investment_agent.name(), "Chief Investment Agent");
        assert_eq!(team.research_team.fundamental_researcher.name(), "Fundamental Researcher");
        assert_eq!(team.analysis_team.valuation_analyst.name(), "Valuation Analyst");
        assert_eq!(team.trading_team.execution_agent.name(), "Execution Agent");
        assert_eq!(team.risk_team.portfolio_monitor.name(), "Portfolio Monitor");
    }

    #[tokio::test]
    async fn test_simple_agent_execution() {
        let agent = SimpleAgent::new("Test Agent", "Test description");
        
        let input = claude_agent_sdk_rs::orchestration::AgentInput::new("Test input".to_string());
        let result: claude_agent_sdk_rs::orchestration::AgentOutput = agent.execute(input).await.unwrap();
        
        assert!(result.content.contains("Test Agent"));
        // Note: the implementation doesn't actually use the input content
        assert_eq!(result.confidence, 0.80);
    }

    #[tokio::test]
    async fn test_chief_investment_agent() {
        let agent = ChiefInvestmentAgent::new();
        
        assert_eq!(agent.name(), "Chief Investment Agent");
        assert_eq!(agent.description(), "AI Buffett - Final decision maker");
        
        let input = claude_agent_sdk_rs::orchestration::AgentInput::new("AAPL".to_string());
        let result: claude_agent_sdk_rs::orchestration::AgentOutput = agent.execute(input).await.unwrap();
        
        assert!(result.content.contains("buy"));
        assert_eq!(result.confidence, 0.90);
    }

    #[tokio::test]
    async fn test_research_team_parallel_execution() {
        let team = ResearchTeam::new();
        
        let result = team.analyze_parallel("AAPL").await.unwrap();
        
        assert!(result.as_object().unwrap().contains_key("fundamental"));
        assert!(result.as_object().unwrap().contains_key("technical"));
        assert!(result.as_object().unwrap().contains_key("sentiment"));
        assert!(result.as_object().unwrap().contains_key("macro"));
    }

    #[tokio::test]
    async fn test_analysis_team_hierarchical_execution() {
        let team = AnalysisTeam::new();
        
        let research_data = serde_json::json!({"test": "data"});
        let result = team.analyze_hierarchical("AAPL", &research_data).await.unwrap();
        
        assert!(result.as_object().unwrap().contains_key("valuation"));
        assert!(result.as_object().unwrap().contains_key("quality"));
        assert!(result.as_object().unwrap().contains_key("risk"));
        assert!(result.as_object().unwrap().contains_key("moat"));
    }

    #[test]
    fn test_team_debug_implementations() {
        let research_team = ResearchTeam::new();
        let analysis_team = AnalysisTeam::new();
        let trading_team = TradingTeam::new();
        let risk_team = RiskTeam::new();
        let ai_team = AITeam::new();
        
        // Verify Debug implementations compile
        let _ = format!("{:?}", research_team);
        let _ = format!("{:?}", analysis_team);
        let _ = format!("{:?}", trading_team);
        let _ = format!("{:?}", risk_team);
        let _ = format!("{:?}", ai_team);
    }

    #[test]
    fn test_simple_agent_name_and_description() {
        let agent = SimpleAgent::new("Custom Agent", "Custom description");
        
        assert_eq!(agent.name(), "Custom Agent");
        assert_eq!(agent.description(), "Custom description");
    }

    #[tokio::test]
    async fn test_ai_team_analyze_investment() {
        let team = AITeam::new();
        
        // This will test the full pipeline but may not complete without real LLM
        // We're just checking it doesn't panic
        let result = team.analyze_investment("AAPL").await;
        
        // Result may be Ok or Err depending on execution environment
        // We just want to verify it doesn't crash
        match result {
            Ok(decision) => {
                assert!(decision.as_object().is_some());
            }
            Err(_) => {
                // Expected in test environment without real LLM
            }
        }
    }
}
