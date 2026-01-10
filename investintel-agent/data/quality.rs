//! Data Quality Validation
//!
//! Validates financial data for completeness, accuracy, timeliness, and consistency

use anyhow::{Context, Result};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::data::{yahoo::QuoteData, alpha_vantage::GlobalQuote};

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub score: f64,  // 0.0 - 1.0
    pub issues: Vec<ValidationIssue>,
    pub warnings: Vec<String>,
}

/// Validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueCategory {
    MissingData,
    InvalidValue,
    OutOfRange,
    StaleData,
    Inconsistent,
}

/// Data quality validator
pub struct DataQualityValidator {
    strict_mode: bool,
    max_price_change_percent: f64,
    max_volume_multiplier: f64,
    max_data_age_seconds: i64,
}

impl DataQualityValidator {
    pub fn new() -> Self {
        Self {
            strict_mode: false,
            max_price_change_percent: 50.0,  // Allow 50% swing
            max_volume_multiplier: 100.0,     // Allow 100x volume spike
            max_data_age_seconds: 3600,       // 1 hour
        }
    }

    pub fn strict() -> Self {
        Self {
            strict_mode: true,
            max_price_change_percent: 20.0,
            max_volume_multiplier: 10.0,
            max_data_age_seconds: 300,  // 5 minutes
        }
    }

    /// Validate Yahoo Finance quote
    pub fn validate_quote(&self, quote: &QuoteData) -> ValidationResult {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();
        let mut valid = true;

        // Check for missing critical data
        if quote.regular_market_price <= 0.0 {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                category: IssueCategory::InvalidValue,
                message: format!("Price is invalid: {}", quote.regular_market_price),
            });
            valid = false;
        }

        if quote.volume == 0 {
            warnings.push("Volume is zero, might be outside trading hours".to_string());
        }

        // Check for reasonable price range
        if quote.regular_market_price < 0.01 {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                category: IssueCategory::InvalidValue,
                message: "Price is too low, possible data error".to_string(),
            });
            valid = false;
        }

        if quote.regular_market_price > 1_000_000.0 {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                category: IssueCategory::OutOfRange,
                message: "Price is unusually high".to_string(),
            });
        }

        // Check day high/low consistency
        if quote.day_high > 0.0 && quote.day_low > 0.0 {
            if quote.day_high < quote.day_low {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    category: IssueCategory::Inconsistent,
                    message: format!("Day high ({}) is less than day low ({})", quote.day_high, quote.day_low),
                });
                valid = false;
            }

            if quote.regular_market_price > quote.day_high || quote.regular_market_price < quote.day_low {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning,
                    category: IssueCategory::Inconsistent,
                    message: "Current price is outside daily range".to_string(),
                });
            }
        }

        // Check for stale data
        let data_age = Utc::now() - quote.timestamp;
        if data_age.num_seconds() > self.max_data_age_seconds {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                category: IssueCategory::StaleData,
                message: format!("Data is {} seconds old", data_age.num_seconds()),
            });
        }

        // Check 52-week range
        if quote.week_high > 0.0 && quote.week_low > 0.0 {
            if quote.week_high < quote.week_low {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    category: IssueCategory::Inconsistent,
                    message: "52-week high is less than 52-week low".to_string(),
                });
                valid = false;
            }
        }

        // Calculate quality score
        let error_count = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Error)).count();
        let warning_count = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Warning)).count();

        let score = if valid {
            1.0 - (warning_count as f64 * 0.1) - (error_count as f64 * 0.5)
        } else {
            0.0
        }.max(0.0);

        ValidationResult {
            is_valid: valid,
            score,
            issues,
            warnings,
        }
    }

    /// Validate OHLCV data
    pub fn validate_ohlcv(&self, data: &[crate::data::yahoo::OHLCV]) -> Result<ValidationReport> {
        if data.is_empty() {
            return Ok(ValidationReport {
                is_valid: false,
                total_records: 0,
                valid_records: 0,
                issues: vec![ValidationIssue {
                    severity: IssueSeverity::Error,
                    category: IssueCategory::MissingData,
                    message: "No OHLCV data provided".to_string(),
                }],
                quality_score: 0.0,
            });
        }

        let mut valid_records = 0;
        let mut all_issues = Vec::new();

        for (i, bar) in data.iter().enumerate() {
            // Check OHLC consistency
            if bar.high < bar.low {
                all_issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    category: IssueCategory::Inconsistent,
                    message: format!("Record {}: high ({}) < low ({})", i, bar.high, bar.low),
                });
                continue;
            }

            if bar.close > bar.high || bar.close < bar.low {
                all_issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning,
                    category: IssueCategory::Inconsistent,
                    message: format!("Record {}: close outside high-low range", i),
                });
            }

            if bar.open > bar.high || bar.open < bar.low {
                all_issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning,
                    category: IssueCategory::Inconsistent,
                    message: format!("Record {}: open outside high-low range", i),
                });
            }

            // Check for zero values
            if bar.open == 0.0 || bar.high == 0.0 || bar.low == 0.0 || bar.close == 0.0 {
                all_issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    category: IssueCategory::InvalidValue,
                    message: format!("Record {}: contains zero values", i),
                });
                continue;
            }

            // Check for negative values
            if bar.open < 0.0 || bar.high < 0.0 || bar.low < 0.0 || bar.close < 0.0 {
                all_issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    category: IssueCategory::InvalidValue,
                    message: format!("Record {}: contains negative values", i),
                });
                continue;
            }

            valid_records += 1;
        }

        // Detect gaps
        let mut gaps = Vec::new();
        for i in 1..data.len() {
            let time_diff = data[i].timestamp - data[i-1].timestamp;
            if time_diff.num_days() > 7 {  // More than a week gap
                gaps.push((i-1, i, time_diff));
            }
        }

        for (start, end, duration) in gaps {
            all_issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                category: IssueCategory::MissingData,
                message: format!("Gap between records {} and {}: {} days", start, end, duration.num_days()),
            });
        }

        let quality_score = if data.len() > 0 {
            valid_records as f64 / data.len() as f64
        } else {
            0.0
        };

        Ok(ValidationReport {
            is_valid: quality_score >= 0.8,
            total_records: data.len(),
            valid_records,
            issues: all_issues,
            quality_score,
        })
    }

    /// Detect anomalies in price data
    pub fn detect_price_anomalies(
        &self,
        current_price: f64,
        historical_prices: &[f64],
    ) -> Vec<AnomalyInfo> {
        let mut anomalies = Vec::new();

        if historical_prices.len() < 20 {
            return anomalies;
        }

        let mean: f64 = historical_prices.iter().sum::<f64>() / historical_prices.len() as f64;
        let variance = historical_prices.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / historical_prices.len() as f64;
        let std_dev = variance.sqrt();

        // Z-score based anomaly detection
        let z_score = (current_price - mean) / std_dev;

        if z_score.abs() > 3.0 {
            anomalies.push(AnomalyInfo {
                anomaly_type: if z_score > 0.0 {
                    AnomalyType::PriceSpike
                } else {
                    AnomalyType::PriceDrop
                },
                severity: if z_score.abs() > 5.0 {
                    AnomalySeverity::Critical
                } else if z_score.abs() > 4.0 {
                    AnomalySeverity::High
                } else {
                    AnomalySeverity::Medium
                },
                z_score,
                description: format!(
                    "Price z-score: {:.2} (current: {:.2}, mean: {:.2}, std: {:.2})",
                    z_score, current_price, mean, std_dev
                ),
            });
        }

        anomalies
    }
}

impl Default for DataQualityValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation report for bulk data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub total_records: usize,
    pub valid_records: usize,
    pub issues: Vec<ValidationIssue>,
    pub quality_score: f64,  // 0.0 - 1.0
}

/// Anomaly information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyInfo {
    pub anomaly_type: AnomalyType,
    pub severity: AnomalySeverity,
    pub z_score: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    PriceSpike,
    PriceDrop,
    VolumeAnomaly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_validator_creation() {
        let validator = DataQualityValidator::new();
        assert!(!validator.strict_mode);
    }

    #[test]
    fn test_validate_quote_valid() {
        let validator = DataQualityValidator::new();
        let quote = QuoteData {
            symbol: "AAPL".to_string(),
            company_name: "Apple Inc.".to_string(),
            regular_market_price: 150.0,
            previous_close: 148.0,
            change: 2.0,
            change_percent: 1.35,
            day_high: 152.0,
            day_low: 147.0,
            volume: 50_000_000,
            week_high: 180.0,
            week_low: 120.0,
            market_cap: 2_500_000_000_000,
            currency: "USD".to_string(),
            timestamp: Utc::now(),
        };

        let result = validator.validate_quote(&quote);
        assert!(result.is_valid);
        assert!(result.score > 0.8);
    }

    #[test]
    fn test_validate_quote_invalid_price() {
        let validator = DataQualityValidator::new();
        let mut quote = QuoteData {
            symbol: "TEST".to_string(),
            company_name: "Test".to_string(),
            regular_market_price: -1.0,
            previous_close: 0.0,
            change: 0.0,
            change_percent: 0.0,
            day_high: 0.0,
            day_low: 0.0,
            volume: 0,
            week_high: 0.0,
            week_low: 0.0,
            market_cap: 0,
            currency: "USD".to_string(),
            timestamp: Utc::now(),
        };

        let result = validator.validate_quote(&quote);
        assert!(!result.is_valid);
        assert!(result.score == 0.0);
    }
}
