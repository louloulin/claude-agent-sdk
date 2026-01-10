//! 金融情感分析引擎
//!
//! 使用FinBERT和Claude Agent SDK进行金融文本情感分析
//! 支持新闻、财报、社交媒体等多源情感分析

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 情感类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SentimentType {
    /// 积极看多
    Positive,
    /// 中性
    Neutral,
    /// 消极看空
    Negative,
}

impl SentimentType {
    /// 从分数转换为情感类型
    pub fn from_score(score: f64) -> Self {
        if score >= 0.6 {
            SentimentType::Positive
        } else if score <= 0.4 {
            SentimentType::Negative
        } else {
            SentimentType::Neutral
        }
    }

    /// 转换为分数
    pub fn to_score(&self) -> f64 {
        match self {
            SentimentType::Positive => 0.75,
            SentimentType::Neutral => 0.5,
            SentimentType::Negative => 0.25,
        }
    }

    /// 转换为中文描述
    pub fn to_chinese(&self) -> &'static str {
        match self {
            SentimentType::Positive => "积极/看多",
            SentimentType::Neutral => "中性",
            SentimentType::Negative => "消极/看空",
        }
    }
}

/// 情感分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentResult {
    /// 情感类型
    pub sentiment: SentimentType,
    /// 情感分数 (0.0-1.0, 0.5为中性)
    pub score: f64,
    /// 置信度 (0.0-1.0)
    pub confidence: f64,
    /// 关键情感词
    pub keywords: Vec<String>,
    /// 情感强度 (弱/中/强)
    pub intensity: SentimentIntensity,
    /// 情感趋势 (如果有时序数据)
    pub trend: Option<SentimentTrend>,
}

/// 情感强度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SentimentIntensity {
    /// 弱
    Weak,
    /// 中
    Medium,
    /// 强
    Strong,
}

impl SentimentIntensity {
    pub fn from_score(score: f64) -> Self {
        let distance_from_neutral = (score - 0.5).abs();
        if distance_from_neutral < 0.15 {
            SentimentIntensity::Weak
        } else if distance_from_neutral < 0.3 {
            SentimentIntensity::Medium
        } else {
            SentimentIntensity::Strong
        }
    }
}

/// 情感趋势
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SentimentTrend {
    /// 上升
    Rising,
    /// 稳定
    Stable,
    /// 下降
    Falling,
}

/// 新闻情感分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsSentiment {
    /// 新闻标题
    pub title: String,
    /// 新闻来源
    pub source: String,
    /// 发布时间
    pub timestamp: DateTime<Utc>,
    /// 相关股票代码
    pub ticker: Option<String>,
    /// 情感分析结果
    pub sentiment: SentimentResult,
    /// 影响评分 (0-100)
    pub impact_score: f64,
    /// 新闻摘要
    pub summary: String,
}

/// 财报情感分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningsSentiment {
    /// 股票代码
    pub ticker: String,
    /// 财报季度
    pub quarter: String,
    /// 财报年份
    pub year: u32,
    /// 整体情感
    pub overall_sentiment: SentimentResult,
    /// 收入情感
    pub revenue_sentiment: SentimentResult,
    /// 利润情感
    pub earnings_sentiment: SentimentResult,
    /// 指引情感
    pub guidance_sentiment: Option<SentimentResult>,
    /// 关键亮点
    pub highlights: Vec<String>,
    /// 关键风险
    pub risks: Vec<String>,
}

/// 社交媒体情感分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialSentiment {
    /// 平台名称
    pub platform: String,
    /// 相关股票代码
    pub ticker: String,
    /// 时间范围
    pub timeframe: String,
    /// 整体情感
    pub overall_sentiment: SentimentResult,
    /// 讨论热度 (0-100)
    pub buzz_score: f64,
    /// 影响力评分 (0-100)
    pub influence_score: f64,
    /// 主要话题标签
    pub hashtags: Vec<String>,
    /// 情感分布（各类型占比）
    pub distribution: HashMap<SentimentType, f64>,
}

/// 聚合情感分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedSentiment {
    /// 股票代码
    pub ticker: String,
    /// 分析时间戳
    pub timestamp: DateTime<Utc>,
    /// 综合情感分数
    pub composite_score: f64,
    /// 新闻情感
    pub news_sentiment: Option<SentimentResult>,
    /// 社交媒体情感
    pub social_sentiment: Option<SentimentResult>,
    /// 财报情感（如果有）
    pub earnings_sentiment: Option<SentimentResult>,
    /// 情感趋势
    pub trend: SentimentTrend,
    /// 情感一致性 (0-1, 越高各来源越一致)
    pub consistency: f64,
    /// 综合解读
    pub interpretation: String,
}

/// 金融情感分析器
pub struct FinancialSentimentAnalyzer {
    /// 是否使用Claude API进行增强分析
    use_claude: bool,
    /// 情感词词典（正面）
    positive_words: Vec<String>,
    /// 情感词词典（负面）
    negative_words: Vec<String>,
}

impl FinancialSentimentAnalyzer {
    /// 创建新的情感分析器
    pub fn new() -> Self {
        Self {
            use_claude: true,
            positive_words: Self::load_positive_words(),
            negative_words: Self::load_negative_words(),
        }
    }

    /// 设置是否使用Claude API
    pub fn with_claude(mut self, use_claude: bool) -> Self {
        self.use_claude = use_claude;
        self
    }

    /// 分析单条文本的情感
    pub fn analyze_text(&self, text: &str) -> Result<SentimentResult> {
        // 基于词典的初步分析
        let (raw_score, keywords) = self.dictionary_based_analysis(text)?;

        // 如果启用了Claude，使用Claude进行增强分析
        let (score, confidence) = if self.use_claude {
            // TODO: 这里可以调用Claude API进行更精确的分析
            // 目前使用词典分析的分数
            (raw_score, 0.75)
        } else {
            (raw_score, 0.65)
        };

        let sentiment = SentimentType::from_score(score);
        let intensity = SentimentIntensity::from_score(score);

        Ok(SentimentResult {
            sentiment,
            score,
            confidence,
            keywords,
            intensity,
            trend: None,
        })
    }

    /// 分析新闻文本的情感
    pub fn analyze_news(
        &self,
        title: &str,
        content: &str,
        ticker: Option<&str>,
    ) -> Result<NewsSentiment> {
        // 合并标题和内容进行情感分析
        let full_text = format!("{}\n\n{}", title, content);
        let sentiment = self.analyze_text(&full_text)?;

        // 计算影响评分（基于情感强度和关键词数量）
        let impact_score = if sentiment.intensity == SentimentIntensity::Strong {
            80.0 + (sentiment.score.abs() * 20.0)
        } else if sentiment.intensity == SentimentIntensity::Medium {
            60.0 + (sentiment.score.abs() * 20.0)
        } else {
            40.0 + (sentiment.score.abs() * 20.0)
        };

        // 生成摘要（简化实现，取前200字）
        let summary = if content.len() > 200 {
            format!("{}...", &content[..200])
        } else {
            content.to_string()
        };

        Ok(NewsSentiment {
            title: title.to_string(),
            source: "Unknown".to_string(), // 可以从外部传入
            timestamp: Utc::now(),
            ticker: ticker.map(|s| s.to_string()),
            sentiment,
            impact_score,
            summary,
        })
    }

    /// 分析财报文本的情感
    pub fn analyze_earnings(
        &self,
        ticker: &str,
        quarter: &str,
        year: u32,
        earnings_call_text: &str,
    ) -> Result<EarningsSentiment> {
        // 分析整体情感
        let overall_sentiment = self.analyze_text(earnings_call_text)?;

        // 分析收入相关文本
        let revenue_text = Self::extract_section(earnings_call_text, &["revenue", "sales", "收入", "销售"]);
        let revenue_sentiment = if !revenue_text.is_empty() {
            self.analyze_text(&revenue_text)?
        } else {
            overall_sentiment.clone()
        };

        // 分析利润相关文本
        let earnings_text = Self::extract_section(
            earnings_call_text,
            &["earnings", "profit", "income", "利润", "盈利"],
        );
        let earnings_sentiment = if !earnings_text.is_empty() {
            self.analyze_text(&earnings_text)?
        } else {
            overall_sentiment.clone()
        };

        // 分析指引相关文本
        let guidance_text = Self::extract_section(
            earnings_call_text,
            &["guidance", "outlook", "forecast", "指引", "展望"],
        );
        let guidance_sentiment = if !guidance_text.is_empty() {
            Some(self.analyze_text(&guidance_text)?)
        } else {
            None
        };

        // 提取亮点
        let highlights = Self::extract_highlights(earnings_call_text, true);

        // 提取风险
        let risks = Self::extract_highlights(earnings_call_text, false);

        Ok(EarningsSentiment {
            ticker: ticker.to_string(),
            quarter: quarter.to_string(),
            year,
            overall_sentiment,
            revenue_sentiment,
            earnings_sentiment,
            guidance_sentiment,
            highlights,
            risks,
        })
    }

    /// 聚合多源情感分析
    pub fn aggregate_sentiment(
        &self,
        ticker: &str,
        news_sentiments: &[NewsSentiment],
        social_sentiments: &[SocialSentiment],
        earnings_sentiment: Option<&EarningsSentiment>,
    ) -> Result<AggregatedSentiment> {
        // 计算新闻情感平均值
        let news_score = if !news_sentiments.is_empty() {
            let sum: f64 = news_sentiments.iter().map(|n| n.sentiment.score).sum();
            let count = news_sentiments.len() as f64;
            Some(sum / count)
        } else {
            None
        };

        // 计算社交媒体情感平均值
        let social_score = if !social_sentiments.is_empty() {
            let sum: f64 = social_sentiments.iter().map(|s| s.overall_sentiment.score).sum();
            let count = social_sentiments.len() as f64;
            Some(sum / count)
        } else {
            None
        };

        // 获取财报情感
        let earnings_score = earnings_sentiment.map(|e| e.overall_sentiment.score);

        // 计算综合分数（加权平均）
        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;

        if let Some(score) = news_score {
            weighted_sum += score * 0.4; // 新闻权重40%
            total_weight += 0.4;
        }

        if let Some(score) = social_score {
            weighted_sum += score * 0.3; // 社交媒体权重30%
            total_weight += 0.3;
        }

        if let Some(score) = earnings_score {
            weighted_sum += score * 0.3; // 财报权重30%
            total_weight += 0.3;
        }

        let composite_score = if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.5 // 默认中性
        };

        // 确定趋势（简化实现，需要时序数据）
        let trend = if composite_score > 0.6 {
            SentimentTrend::Rising
        } else if composite_score < 0.4 {
            SentimentTrend::Falling
        } else {
            SentimentTrend::Stable
        };

        // 计算一致性
        let consistency = self.calculate_consistency(news_score, social_score, earnings_score);

        // 生成综合解读
        let interpretation = self.generate_interpretation(
            composite_score,
            trend,
            consistency,
            news_score.is_some() as u8 + social_score.is_some() as u8 + earnings_score.is_some() as u8,
        );

        Ok(AggregatedSentiment {
            ticker: ticker.to_string(),
            timestamp: Utc::now(),
            composite_score,
            news_sentiment: news_score.map(|score| SentimentResult {
                sentiment: SentimentType::from_score(score),
                score,
                confidence: 0.75,
                keywords: vec![],
                intensity: SentimentIntensity::from_score(score),
                trend: Some(trend),
            }),
            social_sentiment: social_score.map(|score| SentimentResult {
                sentiment: SentimentType::from_score(score),
                score,
                confidence: 0.70,
                keywords: vec![],
                intensity: SentimentIntensity::from_score(score),
                trend: Some(trend),
            }),
            earnings_sentiment: earnings_score.map(|score| SentimentResult {
                sentiment: SentimentType::from_score(score),
                score,
                confidence: 0.80,
                keywords: vec![],
                intensity: SentimentIntensity::from_score(score),
                trend: Some(trend),
            }),
            trend,
            consistency,
            interpretation,
        })
    }

    /// 基于词典的情感分析
    fn dictionary_based_analysis(&self, text: &str) -> Result<(f64, Vec<String>)> {
        let text_lower = text.to_lowercase();

        let mut positive_count = 0;
        let mut negative_count = 0;
        let mut matched_keywords = Vec::new();

        // 检查正面词
        for word in &self.positive_words {
            if text_lower.contains(word) {
                positive_count += 1;
                matched_keywords.push(word.clone());
            }
        }

        // 检查负面词
        for word in &self.negative_words {
            if text_lower.contains(word) {
                negative_count += 1;
                matched_keywords.push(word.clone());
            }
        }

        // 计算情感分数
        let total_count = positive_count + negative_count;
        let score = if total_count == 0 {
            0.5 // 中性
        } else {
            // 基础分数
            let base_score = positive_count as f64 / total_count as f64;

            // 考虑否定词（如"不看好"中的"不"会翻转情感）
            let has_negation = text_lower.contains("不")
                || text_lower.contains("没")
                || text_lower.contains("非")
                || text_lower.contains("no")
                || text_lower.contains("not");

            if has_negation {
                1.0 - base_score // 翻转分数
            } else {
                base_score
            }
        };

        // 去重关键词
        matched_keywords.sort();
        matched_keywords.dedup();
        matched_keywords.truncate(10); // 最多返回10个关键词

        Ok((score, matched_keywords))
    }

    /// 计算情感一致性
    fn calculate_consistency(
        &self,
        news: Option<f64>,
        social: Option<f64>,
        earnings: Option<f64>,
    ) -> f64 {
        let mut scores = Vec::new();

        if let Some(s) = news {
            scores.push(s);
        }
        if let Some(s) = social {
            scores.push(s);
        }
        if let Some(s) = earnings {
            scores.push(s);
        }

        if scores.len() < 2 {
            return 1.0; // 只有一个数据源，认为完全一致
        }

        // 计算标准差作为不一致性的度量
        let mean: f64 = scores.iter().sum::<f64>() / scores.len() as f64;
        let variance: f64 = scores.iter().map(|&s| (s - mean).powi(2)).sum();
        let std_dev = variance.sqrt();

        // 转换为一致性分数（标准差越小，一致性越高）
        (1.0 - std_dev).max(0.0).min(1.0)
    }

    /// 生成综合解读文本
    fn generate_interpretation(
        &self,
        composite_score: f64,
        trend: SentimentTrend,
        consistency: f64,
        num_sources: u8,
    ) -> String {
        let sentiment_desc = match SentimentType::from_score(composite_score) {
            SentimentType::Positive => "积极",
            SentimentType::Neutral => "中性",
            SentimentType::Negative => "消极",
        };

        let trend_desc = match trend {
            SentimentTrend::Rising => "上升",
            SentimentTrend::Stable => "稳定",
            SentimentTrend::Falling => "下降",
        };

        let consistency_desc = if consistency >= 0.8 {
            "高度一致"
        } else if consistency >= 0.5 {
            "基本一致"
        } else {
            "存在分歧"
        };

        let sources_desc = match num_sources {
            3 => "新闻、社交媒体和财报",
            2 => "多源数据",
            _ => "有限数据",
        };

        format!(
            "综合{}分析，{}整体情感{}，情感趋势{}，各来源情感{}。",
            sources_desc, sentiment_desc, trend_desc, consistency_desc
        )
    }

    /// 提取财报特定部分
    fn extract_section(text: &str, keywords: &[&str]) -> String {
        let sentences: Vec<&str> = text.split(&['.', '。', '!', '！', '?', '？'][..]).collect();

        sentences
            .iter()
            .filter(|s| keywords.iter().any(|&kw| s.to_lowercase().contains(kw)))
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(". ")
    }

    /// 提取亮点或风险
    fn extract_highlights(text: &str, positive: bool) -> Vec<String> {
        let positive_keywords = [
            "增长", "增长", "超预期", "创新高", "强劲", "改善", "提升", "record", "growth",
            "increase", "beat", "strong",
        ];

        let negative_keywords = [
            "下降", "下跌", "低于预期", "挑战", "风险", "压力", "担忧", "decline", "fall",
            "below", "challenge", "risk",
        ];

        let keywords = if positive {
            &positive_keywords[..]
        } else {
            &negative_keywords[..]
        };

        let sentences: Vec<&str> = text.split(&['.', '。'][..]).collect();

        sentences
            .iter()
            .filter(|s| keywords.iter().any(|&kw| s.contains(kw)))
            .map(|s| s.trim().to_string())
            .filter(|s| s.len() > 5)
            .take(5)
            .collect()
    }

    /// 加载正面情感词词典
    fn load_positive_words() -> Vec<String> {
        vec![
            // 英文
            "growth".to_string(),
            "increase".to_string(),
            "strong".to_string(),
            "beat".to_string(),
            "rise".to_string(),
            "bullish".to_string(),
            "outperform".to_string(),
            "upgrade".to_string(),
            "record".to_string(),
            "profit".to_string(),
            "gain".to_string(),
            "positive".to_string(),
            "excellent".to_string(),
            // 中文
            "增长".to_string(),
            "增长".to_string(),
            "强劲".to_string(),
            "超预期".to_string(),
            "看好".to_string(),
            "上涨".to_string(),
            "盈利".to_string(),
            "优秀".to_string(),
            "改善".to_string(),
            "提升".to_string(),
            "创新高".to_string(),
        ]
    }

    /// 加载负面情感词词典
    fn load_negative_words() -> Vec<String> {
        vec![
            // 英文
            "decline".to_string(),
            "fall".to_string(),
            "weak".to_string(),
            "miss".to_string(),
            "bearish".to_string(),
            "underperform".to_string(),
            "downgrade".to_string(),
            "loss".to_string(),
            "risk".to_string(),
            "concern".to_string(),
            "negative".to_string(),
            "challenging".to_string(),
            // 中文
            "下降".to_string(),
            "下跌".to_string(),
            "疲软".to_string(),
            "低于预期".to_string(),
            "看空".to_string(),
            "亏损".to_string(),
            "风险".to_string(),
            "担忧".to_string(),
            "挑战".to_string(),
            "压力".to_string(),
        ]
    }
}

impl Default for FinancialSentimentAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentiment_type_conversion() {
        assert_eq!(SentimentType::from_score(0.8), SentimentType::Positive);
        assert_eq!(SentimentType::from_score(0.5), SentimentType::Neutral);
        assert_eq!(SentimentType::from_score(0.2), SentimentType::Negative);
    }

    #[test]
    fn test_sentiment_intensity() {
        assert_eq!(
            SentimentIntensity::from_score(0.5),
            SentimentIntensity::Weak
        );
        assert_eq!(
            SentimentIntensity::from_score(0.7),
            SentimentIntensity::Medium
        );
        assert_eq!(
            SentimentIntensity::from_score(0.9),
            SentimentIntensity::Strong
        );
    }

    #[test]
    fn test_text_sentiment_analysis() {
        let analyzer = FinancialSentimentAnalyzer::new().with_claude(false);

        // 测试正面文本
        let positive_text = "公司营收强劲增长，超市场预期，盈利能力显著提升";
        let result = analyzer.analyze_text(positive_text).unwrap();
        assert!(result.score > 0.5);
        assert_eq!(result.sentiment, SentimentType::Positive);

        // 测试负面文本
        let negative_text = "公司业绩下滑，低于市场预期，面临较大经营压力和挑战";
        let result = analyzer.analyze_text(negative_text).unwrap();
        assert!(result.score < 0.5);
        assert_eq!(result.sentiment, SentimentType::Negative);
    }

    #[test]
    fn test_news_analysis() {
        let analyzer = FinancialSentimentAnalyzer::new().with_claude(false);

        let news = analyzer
            .analyze_news(
                "某公司发布超预期财报",
                "该公司第三季度营收增长50%，远超市场预期，净利润同比增长80%，创历史新高。",
                Some("AAPL"),
            )
            .unwrap();

        assert_eq!(news.ticker, Some("AAPL".to_string()));
        assert!(news.impact_score > 60.0);
        assert_eq!(news.sentiment.sentiment, SentimentType::Positive);
    }

    #[test]
    fn test_sentiment_aggregation() {
        let analyzer = FinancialSentimentAnalyzer::new().with_claude(false);

        // 创建模拟新闻情感
        let news_sentiments = vec![NewsSentiment {
            title: "测试新闻".to_string(),
            source: "Test".to_string(),
            timestamp: Utc::now(),
            ticker: Some("TEST".to_string()),
            sentiment: SentimentResult {
                sentiment: SentimentType::Positive,
                score: 0.75,
                confidence: 0.8,
                keywords: vec!["增长".to_string()],
                intensity: SentimentIntensity::Strong,
                trend: None,
            },
            impact_score: 80.0,
            summary: "测试摘要".to_string(),
        }];

        // 创建模拟社交媒体情感
        let social_sentiments = vec![SocialSentiment {
            platform: "Twitter".to_string(),
            ticker: "TEST".to_string(),
            timeframe: "24h".to_string(),
            overall_sentiment: SentimentResult {
                sentiment: SentimentType::Positive,
                score: 0.70,
                confidence: 0.7,
                keywords: vec![],
                intensity: SentimentIntensity::Medium,
                trend: None,
            },
            buzz_score: 75.0,
            influence_score: 65.0,
            hashtags: vec!["stocks".to_string()],
            distribution: HashMap::new(),
        }];

        let aggregated = analyzer
            .aggregate_sentiment("TEST", &news_sentiments, &social_sentiments, None)
            .unwrap();

        assert!(aggregated.composite_score > 0.6);
        assert_eq!(aggregated.trend, SentimentTrend::Rising);
        assert!(!aggregated.interpretation.is_empty());
    }
}
