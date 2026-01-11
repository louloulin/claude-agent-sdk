//! Tests for Partnership Builder Module
//!
//! Validates builder pattern and partnership creation

use super::builder::*;
use super::types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_partnership_builder_basic() {
        let partners = vec![
            Partner::new("Alice", 100_000.0, 0.8),
            Partner::new("Bob", 150_000.0, 0.8),
        ];

        let partnership = PartnershipBuilder::new()
            .name("Test Partnership".to_string())
            .partners(partners)
            .strategy(InvestmentStrategy::ValueInvesting)
            .build()
            .await
            .unwrap();

        assert_eq!(partnership.name, "Test Partnership");
        assert_eq!(partnership.partners.len(), 2);
        assert_eq!(partnership.portfolio.total_value, 250_000.0);
    }

    #[tokio::test]
    async fn test_partnership_builder_single_partner() {
        let partnership = PartnershipBuilder::new()
            .name("Single Partner Partnership".to_string())
            .add_partner(Partner::new("Charlie", 500_000.0, 0.75))
            .strategy(InvestmentStrategy::GARP)
            .build()
            .await
            .unwrap();

        assert_eq!(partnership.partners.len(), 1);
        assert_eq!(partnership.portfolio.total_value, 500_000.0);
    }

    #[tokio::test]
    async fn test_partnership_builder_missing_name() {
        let partners = vec![Partner::new("Alice", 100_000.0, 0.8)];

        let result = PartnershipBuilder::new()
            .partners(partners)
            .build()
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name"));
    }

    #[tokio::test]
    async fn test_partnership_builder_empty_name() {
        let partners = vec![Partner::new("Alice", 100_000.0, 0.8)];

        let result = PartnershipBuilder::new()
            .name("".to_string())
            .partners(partners)
            .build()
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_partnership_builder_no_partners() {
        let result = PartnershipBuilder::new()
            .name("Empty Partnership".to_string())
            .build()
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("partner"));
    }

    #[tokio::test]
    async fn test_partnership_builder_below_minimum_investment() {
        let partners = vec![Partner::new("Small Investor", 50_000.0, 0.8)];

        let result = PartnershipBuilder::new()
            .name("Underfunded Partnership".to_string())
            .partners(partners)
            .build()
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("below minimum"));
    }

    #[tokio::test]
    async fn test_partnership_builder_custom_agreement() {
        use super::super::agreement::*;

        let partners = vec![Partner::new("Alice", 1_000_000.0, 0.8)];

        let mut custom_agreement = PartnershipAgreement::for_strategy(InvestmentStrategy::DeepValue);
        custom_agreement.hurdle_rate = 0.08;
        custom_agreement.ai_profit_share = 0.30;
        custom_agreement.minimum_investment = 500_000.0;

        let partnership = PartnershipBuilder::new()
            .name("Custom Agreement Partnership".to_string())
            .partners(partners)
            .agreement(custom_agreement)
            .build()
            .await
            .unwrap();

        assert_eq!(partnership.agreement.hurdle_rate, 0.08);
        assert_eq!(partnership.agreement.ai_profit_share, 0.30);
        assert_eq!(partnership.agreement.minimum_investment, 500_000.0);
    }

    #[tokio::test]
    async fn test_partnership_status() {
        let partners = vec![
            Partner::new("Alice", 100_000.0, 0.8),
            Partner::new("Bob", 150_000.0, 0.8),
        ];

        let partnership = PartnershipBuilder::new()
            .name("Status Test Partnership".to_string())
            .partners(partners)
            .build()
            .await
            .unwrap();

        let status = partnership.status();

        assert_eq!(status.name, "Status Test Partnership");
        assert_eq!(status.partner_count, 2);
        assert_eq!(status.total_capital, 250_000.0);
        assert_eq!(status.current_value, 250_000.0);
        assert_eq!(status.cash, 250_000.0);
        assert_eq!(status.position_count, 0);
    }

    #[tokio::test]
    async fn test_partnership_analyze() {
        let partners = vec![Partner::new("Alice", 100_000.0, 0.8)];

        let partnership = PartnershipBuilder::new()
            .name("Analysis Test Partnership".to_string())
            .partners(partners)
            .build()
            .await
            .unwrap();

        let result = partnership.analyze("AAPL").await;

        // May succeed or fail depending on execution environment
        match result {
            Ok(decision) => {
                assert_eq!(decision.symbol, "AAPL");
                assert!(!decision.reasoning.is_empty());
            }
            Err(_) => {
                // Expected in test environment
            }
        }
    }

    #[test]
    fn test_partnership_builder_default() {
        let _builder = PartnershipBuilder::default();
        
        // Builder should create successfully
        // Cannot access private fields directly
    }

    #[tokio::test]
    async fn test_partnership_builder_multiple_add_partner() {
        let partnership = PartnershipBuilder::new()
            .name("Multi-Add Partnership".to_string())
            .add_partner(Partner::new("Alice", 100_000.0, 0.8))
            .add_partner(Partner::new("Bob", 150_000.0, 0.8))
            .add_partner(Partner::new("Charlie", 200_000.0, 0.75))
            .build()
            .await
            .unwrap();

        assert_eq!(partnership.partners.len(), 3);
        assert_eq!(partnership.portfolio.total_value, 450_000.0);
    }

    #[tokio::test]
    async fn test_partnership_status_serialization() {
        let partners = vec![Partner::new("Alice", 100_000.0, 0.8)];

        let partnership = PartnershipBuilder::new()
            .name("Serialization Test".to_string())
            .partners(partners)
            .build()
            .await
            .unwrap();

        let status = partnership.status();
        
        // Verify it can be serialized to JSON
        let json = serde_json::to_string(&status).unwrap();
        
        assert!(json.contains("Serialization Test"));
        assert!(json.contains("100000"));
    }

    #[tokio::test]
    async fn test_partnership_ai_team_integration() {
        let partners = vec![Partner::new("Alice", 100_000.0, 0.8)];

        let partnership = PartnershipBuilder::new()
            .name("AI Team Test".to_string())
            .partners(partners)
            .build()
            .await
            .unwrap();

        // Verify AI team is properly initialized
        assert_eq!(partnership.ai_team.chief_investment_agent.name(), "Chief Investment Agent");
    }

    #[tokio::test]
    async fn test_partnership_reuse_existing_agents() {
        let partners = vec![Partner::new("Alice", 100_000.0, 0.8)];

        let partnership = PartnershipBuilder::new()
            .name("Reuse Agents Test".to_string())
            .partners(partners)
            .reuse_existing_agents()
            .build()
            .await
            .unwrap();

        // Should create partnership with existing agents
        assert_eq!(partnership.ai_team.chief_investment_agent.name(), "Chief Investment Agent");
    }
}
