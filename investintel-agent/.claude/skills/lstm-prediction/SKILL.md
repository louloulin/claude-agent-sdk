---
name: lstm-prediction
description: LSTM神经网络价格预测,基于历史数据预测未来价格走势
allowed-tools:
  - train_lstm_model
  - lstm_predict_price
  - evaluate_lstm_performance
  - save_lstm_model
  - load_lstm_model
model: claude-sonnet-4-20250514
tags:
  - machine-learning
  - lstm
  - price-prediction
  - deep-learning
---

# LSTM Price Prediction Skill

基于LSTM(Long Short-Term Memory)神经网络的价格预测模型,使用历史价格数据预测未来价格走势。

## 能力

1. **LSTM模型训练**: 自动训练LSTM预测模型
2. **价格预测**: 预测未来1-7天的价格
3. **性能评估**: 计算预测准确率和误差
4. **模型保存**: 保存训练好的模型
5. **模型加载**: 加载已保存的模型

## 架构

```
LSTM网络架构:
Input (历史价格序列)
  ↓
LSTM层1 (64个隐藏单元)
  ↓
LSTM层2 (64个隐藏单元)
  ↓
Dropout (20%)
  ↓
全连接层
  ↓
Output (预测价格)
```

## 使用示例

### 训练模型

```
使用AAPL过去2年的数据训练LSTM模型,序列长度为30天,预测未来1天价格
训练100个epoch,学习率0.001
```

### 预测价格

```
加载已训练的AAPL LSTM模型,预测未来7天的价格
给出预测结果和置信区间
```

### 评估性能

```
评估AAPL LSTM模型的预测性能
计算MAPE、RMSE、方向准确率
```

## 模型参数

- **输入特征**: 5个 (Open, High, Low, Close, Volume)
- **序列长度**: 10-60天 (可配置)
- **预测范围**: 1-7天 (可配置)
- **LSTM层数**: 2层
- **隐藏单元**: 64个
- **Dropout**: 20%
- **优化器**: Adam
- **学习率**: 0.001
- **训练轮数**: 50-200

## 性能指标

预期性能 (在充分训练后):
- **MAPE**: <5% (平均绝对百分比误差)
- **RMSE**: <2% (均方根误差)
- **方向准确率**: >55% (涨跌方向预测)

## 技术实现

- **PyTorch绑定**: tch-rs
- **GPU加速**: 自动使用CUDA (如果可用)
- **模型保存**: .pt格式
- **批量预测**: 支持多symbol并发预测

## 注意事项

1. LSTM预测仅供参考,不构成投资建议
2. 历史表现不代表未来收益
3. 建议结合其他技术指标使用
4. 定期重新训练模型以适应市场变化
