---
name: reinforcement-learning
description: DQN强化学习交易agent,自适应学习最优交易策略
allowed-tools:
  - train_dqn_agent
  - dqn_predict_action
  - evaluate_dqn_performance
  - backtest_dqn_strategy
  - save_dqn_model
  - load_dqn_model
model: claude-sonnet-4-20250514
tags:
  - reinforcement-learning
  - dqn
  - trading-agent
  - deep-learning
---

# Reinforcement Learning Trading Agent Skill

基于DQN (Deep Q-Network) 强化学习的自主交易Agent,通过与市场环境交互学习最优交易策略。

## 能力

1. **DQN Agent训练**: 训练强化学习交易Agent
2. **策略执行**: Agent自动选择Buy/Sell/Hold动作
3. **性能评估**: 评估Agent的收益率和风险
4. **回测验证**: 在历史数据上回测策略
5. **模型保存**: 保存训练好的Agent
6. **模型加载**: 加载已保存的Agent

## DQN架构

```
Q-Network架构:
State (价格历史30天)
  ↓
Linear层1 (128个神经元) + ReLU
  ↓
Linear层2 (64个神经元) + ReLU
  ↓
Linear层3 (3个输出) → Q-Values
  ↓
Actions: Buy | Sell | Hold
```

## 训练流程

1. **初始化**: 创建Q-Network和Target-Network
2. **探索阶段**: Epsilon从1.0逐渐衰减到0.01
3. **经验收集**: Agent与环境交互,收集Experience
4. **经验回放**: 随机采样batch进行训练
5. **目标网络更新**: 定期同步Target-Network
6. **收敛**: 当epsilon达到最小值时停止训练

## 使用示例

### 训练Agent

```
使用AAPL过去3年的数据训练DQN Agent
训练1000个episode,初始资金10000美元
状态窗口30天,学习率0.001
```

### 执行交易

```
加载已训练的AAPL DQN Agent
根据当前市场状态自动选择交易动作
Buy、Sell或Hold
```

### 回测策略

```
对DQN Agent进行回测,评估其在历史数据上的表现
计算总收益率、最大回撤、夏普比率
```

## 超参数

- **State Size**: 30-60天价格历史
- **Learning Rate**: 0.001
- **Gamma**: 0.99 (折扣因子)
- **Epsilon**: 1.0 → 0.01 (探索率衰减)
- **Epsilon Decay**: 0.995
- **Batch Size**: 32
- **Replay Buffer**: 10000 experiences
- **Training Episodes**: 500-2000

## 环境设计

**State**:
- 归一化价格历史 (最近30天)
- 归一化成交量历史
- 当前持仓状态

**Actions**:
- Buy: 买入全部资金
- Sell: 卖出全部持仓
- Hold: 持有不动

**Reward**:
- 交易收益: (当前价值 - 初始资金) / 初始资金
- 惩罚: 无效交易动作 -0.01
- 风险控制: 收益下限 -100%

## 性能指标

预期性能 (在充分训练后):
- **总收益率**: >基准收益率
- **胜率**: >55%
- **最大回撤**: <20%
- **夏普比率**: >1.0

## 技术实现

- **PyTorch绑定**: tch-rs
- **Experience Replay**: 10000容量环形缓冲
- **Target Network**: 周期性更新
- **Epsilon-Greedy**: 平衡探索与利用

## 注意事项

1. DQN Agent需要大量训练数据 (建议1000+ episodes)
2. 训练时间可能较长 (建议GPU加速)
3. 定期重新训练以适应市场变化
4. 在实盘使用前进行充分回测验证
5. 设置合理的止损止盈机制
