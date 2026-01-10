---
name: sentiment-analysis
description: 金融情感分析，包括新闻情感、社交媒体情绪、研报观点提取、情感聚合与趋势分析。在分析市场舆情、提取情感信号、判断市场情绪时使用。
allowed-tools:
  - Bash(python:*, curl:*)
  - Read
  - Write
  - WebFetch
model: claude-sonnet-4-20250514
tags:
  - sentiment
  - news
  - social-media
  - nlp
dependencies: []
---

# Sentiment Analysis Skill

## 核心能力

### 1. 新闻情感分析

#### 单条新闻情感

```python
from transformers import pipeline

# 使用FinBERT模型 (金融领域微调的BERT)
sentiment_pipeline = pipeline("sentiment-analysis",
                             model="ProsusAI/finbert")

def analyze_news_sentiment(text):
    """
    分析新闻文本的情感

    Returns:
        {
            'label': 'positive' | 'negative' | 'neutral',
            'score': 置信度 (0-1)
        }
    """
    result = sentiment_pipeline(text)[0]
    return result
```

#### 批量新闻情感

```python
def batch_analyze_news(news_list):
    """
    批量分析多条新闻的情感

    Args:
        news_list: 新闻文本列表

    Returns:
        情感得分列表
    """
    sentiments = []
    for news in news_list:
        result = sentiment_pipeline(news)
        # 转换为数值得分
        if result['label'] == 'positive':
            score = result['score']
        elif result['label'] == 'negative':
            score = -result['score']
        else:
            score = 0
        sentiments.append(score)

    return sentiments
```

#### 新闻聚合情感

```python
def aggregate_news_sentiment(news_sentiments):
    """
    聚合多条新闻的情感

    Returns:
        {
            'mean': 平均情感,
            'std': 标准差,
            'positive_ratio': 正面新闻比例,
            'negative_ratio': 负面新闻比例
        }
    """
    return {
        'mean': np.mean(news_sentiments),
        'std': np.std(news_sentiments),
        'positive_ratio': sum(1 for s in news_sentiments if s > 0) / len(news_sentiments),
        'negative_ratio': sum(1 for s in news_sentiments if s < 0) / len(news_sentiments)
    }
```

### 2. 社交媒体监控

#### Twitter情感分析

```python
import tweepy

def fetch_twitter_sentiment(ticker, count=100):
    """
    获取Twitter上关于某股票的情感

    Args:
        ticker: 股票代码
        count: 获取推文数量

    Returns:
        平均情感得分
    """
    # 搜索相关推文
    tweets = tweepy.Cursor(api.search_tweets,
                          q=f"${ticker}",
                          lang="en").items(count)

    # 分析情感
    sentiments = []
    for tweet in tweets:
        sentiment = sentiment_pipeline(tweet.text)
        # 转换为数值
        if sentiment['label'] == 'positive':
            score = sentiment['score']
        elif sentiment['label'] == 'negative':
            score = -sentiment['score']
        else:
            score = 0
        sentiments.append(score)

    return np.mean(sentiments)
```

#### Reddit情感分析

```python
import praw

def fetch_reddit_sentiment(ticker, subreddit="wallstreetbets"):
    """
    获取Reddit上关于某股票的情感
    """
    reddit = praw.Reddit(client_id=CLIENT_ID,
                       client_secret=CLIENT_SECRET,
                       user_agent=USER_AGENT)

    subreddit = reddit.subreddit(subreddit)
    posts = subreddit.search(f"${ticker}", limit=100)

    sentiments = []
    for post in posts:
        sentiment = sentiment_pipeline(post.title + " " + post.selftext)
        # 转换为数值
        if sentiment['label'] == 'positive':
            score = sentiment['score']
        elif sentiment['label'] == 'negative':
            score = -sentiment['score']
        else:
            score = 0
        sentiments.append(score)

    return {
        'mean': np.mean(sentiments),
        'post_count': len(sentiments),
        'upvote_ratio': post.upvote_ratio
    }
```

#### StockTwits监控

```python
def fetch_stocktwits_sentiment(ticker):
    """
    从StockTwits获取情感数据
    """
    import requests
    url = f"https://api.stocktwits.com/api/2/streams/symbol/{ticker}.json"
    response = requests.get(url)
    data = response.json()

    sentiments = []
    for message in data['messages']:
        # StockTwits自带情感标签
        if message['entities']['sentiment']:
            if message['entities']['sentiment']['basic'] == 'Bullish':
                score = 1
            elif message['entities']['sentiment']['basic'] == 'Bearish':
                score = -1
            else:
                score = 0
            sentiments.append(score)

    return {
        'bullish_ratio': sum(1 for s in sentiments if s > 0) / len(sentiments),
        'bearish_ratio': sum(1 for s in sentiments if s < 0) / len(sentiments),
        'mean': np.mean(sentiments)
    }
```

### 3. 情感时间序列

#### 构建情感时间序列

```python
def build_sentiment_timeseries(ticker, start_date, end_date, freq='D'):
    """
    构建情感得分时间序列

    Args:
        ticker: 股票代码
        start_date: 开始日期
        end_date: 结束日期
        freq: 频率 ('D'=日, 'W'=周, 'M'=月)

    Returns:
        pandas Series: 情感得分时间序列
    """
    dates = pd.date_range(start_date, end_date, freq=freq)
    sentiments = []

    for date in dates:
        # 获取该日期的情感
        news_sentiment = fetch_news_sentiment_for_date(ticker, date)
        twitter_sentiment = fetch_twitter_sentiment_for_date(ticker, date)
        reddit_sentiment = fetch_reddit_sentiment_for_date(ticker, date)

        # 加权平均
        composite_sentiment = 0.4 * news_sentiment + 0.3 * twitter_sentiment + 0.3 * reddit_sentiment
        sentiments.append(composite_sentiment)

    return pd.Series(sentiments, index=dates)
```

#### 情感趋势分析

```python
def detect_sentiment_trend(sentiment_series, window=7):
    """
    检测情感趋势

    Returns:
        'uptrend' | 'downtrend' | 'stable'
    """
    rolling_mean = sentiment_series.rolling(window).mean()

    if rolling_mean.iloc[-1] > rolling_mean.iloc[-2] > rolling_mean.iloc[-3]:
        return 'uptrend'
    elif rolling_mean.iloc[-1] < rolling_mean.iloc[-2] < rolling_mean.iloc[-3]:
        return 'downtrend'
    else:
        return 'stable'
```

### 4. 异常情感检测

#### 突发情感变化

```python
def detect_sentiment_spike(sentiment_series, threshold=2.5):
    """
    检测情感异常波动

    Args:
        sentiment_series: 情感时间序列
        threshold: 标准差倍数阈值

    Returns:
        异常日期列表
    """
    mean = sentiment_series.mean()
    std = sentiment_series.std()

    outliers = sentiment_series[(sentiment_series < mean - threshold * std) |
                               (sentiment_series > mean + threshold * std)]

    return outliers.index.tolist()
```

## 工作流程

### 情感分析流程

1. **数据收集**
```bash
# 获取新闻
python scripts/fetch_news.py --ticker AAPL --days 30 --output data/news.json

# 获取社交媒体数据
python scripts/fetch_social.py --ticker AAPL --sources twitter reddit --days 7
```

2. **情感分析**
```python
# 批量分析情感
python scripts/analyze_sentiment.py --input data/news.json --model finbert
```

3. **情感聚合**
```python
# 聚合多源情感
python scripts/aggregate_sentiment.py --ticker AAPL --weights news:0.4 twitter:0.3 reddit:0.3
```

4. **情感可视化**
```python
# 生成情感时间序列图
python scripts/visualize_sentiment.py --ticker AAPL --period 1M --output sentiment_chart.png
```

## 最佳实践

### ✅ 推荐做法

1. **多源验证**
   - 结合新闻、社交媒体、分析师报告
   - 交叉验证情感信号

2. **情感与价格结合**
   - 情感是领先指标，不是绝对指标
   - 结合技术分析确认

3. **关注极端情感**
   - 极度贪婪时可能要警惕
   - 极度恐惧时可能是机会

### ❌ 避免错误

1. **过度依赖情感**
   - 情感可能误导
   - 基本面更重要

2. **忽视情感滞后**
   - 新闻反映的是过去
   - 市场可能已经消化

3. **样本偏差**
   - 社交媒体用户不代表所有投资者
   - 注意数据偏差

## 相关资源

- [新闻情感详解](news-sentiment.md)
- [社交媒体监控](social-media-monitoring.md)
- [情感聚合方法](sentiment-aggregation.md)

---

**版本**: 1.0.0
**最后更新**: 2026-01-10
