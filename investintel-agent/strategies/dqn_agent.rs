//! DQN Reinforcement Learning Trading Agent
//!
//! Deep Q-Network (DQN) agent for automated trading using reinforcement learning
//! Based on tch-rs (PyTorch bindings for Rust)

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use tch::{nn, Device, Tensor, Kind};
use tch::nn::{LinearConfig, Module};

/// Trading actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TradingAction {
    Buy,
    Sell,
    Hold,
}

impl TradingAction {
    pub fn to_index(&self) -> usize {
        match self {
            TradingAction::Buy => 0,
            TradingAction::Sell => 1,
            TradingAction::Hold => 2,
        }
    }

    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => TradingAction::Buy,
            1 => TradingAction::Sell,
            2 => TradingAction::Hold,
            _ => panic!("Invalid action index: {}", idx),
        }
    }

    pub fn all_actions() -> Vec<Self> {
        vec![
            TradingAction::Buy,
            TradingAction::Sell,
            TradingAction::Hold,
        ]
    }
}

/// Experience tuple for replay buffer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    pub state: Vec<f32>,
    pub action: usize,
    pub reward: f32,
    pub next_state: Vec<f32>,
    pub done: bool,
}

/// DQN trading agent
pub struct DQNTradingAgent {
    q_network: nn::Sequential,
    target_network: nn::Sequential,
    optimizer: nn::Optimizer<nn::Adam>,
    epsilon: f64,
    epsilon_min: f64,
    epsilon_decay: f64,
    gamma: f64,  // Discount factor
    replay_buffer: VecDeque<Experience>,
    replay_buffer_size: usize,
    batch_size: usize,
    state_size: usize,
    device: Device,
    vs: nn::VarStore,
}

impl DQNTradingAgent {
    /// Create a new DQN trading agent
    ///
    /// # Arguments
    /// * `state_size` - Size of the state vector (e.g., 30 for 30-day price history)
    /// * `learning_rate` - Learning rate for optimizer
    ///
    /// # Example
    /// ```no_run
    /// # use anyhow::Result;
    /// # fn example() -> Result<()> {
    /// let agent = DQNTradingAgent::new(30, 0.001);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(state_size: usize, learning_rate: f64) -> Self {
        let device = Device::cuda_if_available();
        let vs = nn::VarStore::new(device);

        // Q-Network architecture
        let q_network = nn::seq()
            .add(nn::linear(
                &vs,
                LinearConfig {
                    in_size: state_size,
                    out_size: 128,
                    bias: true,
                },
            ))
            .add_fn(|x| x.relu())
            .add(nn::linear(
                &vs,
                LinearConfig {
                    in_size: 128,
                    out_size: 64,
                    bias: true,
                },
            ))
            .add_fn(|x| x.relu())
            .add(nn::linear(
                &vs,
                LinearConfig {
                    in_size: 64,
                    out_size: 3,  // 3 actions: Buy, Sell, Hold
                    bias: true,
                },
            ));

        // Target network (same architecture)
        let target_network = q_network.clone();

        let optimizer = nn::Adam::default()
            .build(&vs, learning_rate)
            .context("Failed to create Adam optimizer")?;

        Self {
            q_network,
            target_network,
            optimizer,
            epsilon: 1.0,        // Initial epsilon (100% exploration)
            epsilon_min: 0.01,    // Minimum epsilon (1% exploration)
            epsilon_decay: 0.995, // Epsilon decay rate
            gamma: 0.99,          // Discount factor
            replay_buffer: VecDeque::with_capacity(10000),
            replay_buffer_size: 10000,
            batch_size: 32,
            state_size,
            device,
            vs,
        }
    }

    /// Select action using epsilon-greedy policy
    ///
    /// # Arguments
    /// * `state` - Current state vector
    ///
    /// # Returns
    /// Selected trading action
    pub fn select_action(&mut self, state: &[f32]) -> TradingAction {
        // Epsilon-greedy action selection
        if rand::random::<f64>() < self.epsilon {
            // Explore: random action
            let idx = rand::thread_rng().gen_range(0..3);
            TradingAction::from_index(idx)
        } else {
            // Exploit: best action from Q-network
            self.select_best_action(state)
        }
    }

    /// Select best action (no exploration)
    pub fn select_best_action(&self, state: &[f32]) -> TradingAction {
        let state_tensor = Tensor::of_slice(state)
            .view([1, self.state_size as i64])
            .to(self.device);

        let q_values = self.q_network.forward(&state_tensor);
        let action_idx = q_values.argmax(-1, false).int64_value(&[0, 0]) as usize;

        TradingAction::from_index(action_idx)
    }

    /// Train the agent on a batch of experiences
    ///
    /// # Returns
    /// Training loss
    pub fn train_step(&mut self) -> Result<f64> {
        if self.replay_buffer.len() < self.batch_size {
            return Ok(0.0);  // Not enough experiences yet
        }

        // Sample random batch from replay buffer
        let batch: Vec<_> = (0..self.batch_size)
            .map(|_| {
                let idx = rand::thread_rng().gen_range(0..self.replay_buffer.len());
                self.replay_buffer[idx].clone()
            })
            .collect();

        // Prepare tensors
        let states: Vec<f32> = batch.iter().flat_map(|e| e.state.clone()).collect();
        let actions: Vec<i64> = batch.iter().map(|e| e.action as i64).collect();
        let rewards: Vec<f32> = batch.iter().map(|e| e.reward).collect();
        let next_states: Vec<f32> = batch.iter().flat_map(|e| e.next_state.clone()).collect();
        let dones: Vec<f32> = batch.iter().map(|e| if e.done { 1.0 } else { 0.0 }).collect();

        let states_tensor = Tensor::of_slice(&states)
            .view([self.batch_size as i64, self.state_size as i64])
            .to(self.device);

        let actions_tensor = Tensor::of_slice(&actions)
            .to(self.device);

        let rewards_tensor = Tensor::of_slice(&rewards)
            .view([self.batch_size as i64, 1])
            .to(self.device);

        let next_states_tensor = Tensor::of_slice(&next_states)
            .view([self.batch_size as i64, self.state_size as i64])
            .to(self.device);

        let dones_tensor = Tensor::of_slice(&dones)
            .view([self.batch_size as i64, 1])
            .to(self.device);

        // Compute Q-values
        let q_values = self.q_network.forward(&states_tensor);
        let next_q_values = self.target_network.forward(&next_states_tensor);

        // Gather Q-values for taken actions
        let actions_one_hot = Tensor::zeros([self.batch_size as i64, 3], (Kind::Float, self.device));
        let indices = Tensor::of_slice(&actions).unsqueeze(1);
        actions_one_hot = actions_one_hot.scatter_(1, &indices, 1.0);

        let current_q = (q_values * actions_one_hot).sum_dim_int(&[1], false, true);

        // Compute target Q-values
        let next_q_max = next_q_values.max_dim_keepdim(-1, false);
        let target_q = rewards_tensor + self.gamma * next_q_max * (1.0 - dones_tensor);

        // Compute loss
        let loss = current_q.mse_loss(&target_q, tch::Reduction::Mean);

        // Backward pass
        self.optimizer.backward_step(&loss);

        // Decay epsilon
        self.epsilon = (self.epsilon * self.epsilon_decay).max(self.epsilon_min);

        Ok(f64::from(loss))
    }

    /// Store experience in replay buffer
    pub fn remember(&mut self, experience: Experience) {
        self.replay_buffer.push_back(experience);
    }

    /// Update target network with current Q-network weights
    pub fn update_target_network(&mut self) {
        self.target_network = self.q_network.clone();
    }

    /// Get current epsilon (exploration rate)
    pub fn epsilon(&self) -> f64 {
        self.epsilon
    }

    /// Save model to disk
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        self.vs.save(path)
            .context("Failed to save model")?;
        Ok(())
    }

    /// Load model from disk
    pub fn load<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        self.vs.load(path)
            .context("Failed to load model")?;
        Ok(())
    }
}

/// Trading environment for training DQN agent
pub struct TradingEnv {
    data: Vec<EnvState>,
    current_idx: usize,
    balance: f64,
    position: f64,
    initial_balance: f64,
    window_size: usize,
}

#[derive(Debug, Clone)]
pub struct EnvState {
    pub price: f64,
    pub volume: f64,
    pub timestamp: DateTime<Utc>,
}

impl TradingEnv {
    /// Create a new trading environment
    ///
    /// # Arguments
    /// * `data` - Historical price data
    /// * `initial_balance` - Starting balance
    /// * `window_size` - Number of time steps in state
    pub fn new(data: Vec<EnvState>, initial_balance: f64, window_size: usize) -> Self {
        Self {
            data,
            current_idx: window_size,
            balance: initial_balance,
            position: 0.0,
            initial_balance,
            window_size,
        }
    }

    /// Reset environment to initial state
    pub fn reset(&mut self) -> Vec<f32> {
        self.current_idx = self.window_size;
        self.balance = self.initial_balance;
        self.position = 0.0;
        self.get_state()
    }

    /// Get current state
    pub fn get_state(&self) -> Vec<f32> {
        let start = self.current_idx.saturating_sub(self.window_size);
        let end = self.current_idx;

        let mut state = Vec::new();

        for i in start..end {
            if let Some(env_state) = self.data.get(i) {
                // Normalize price by most recent price
                let recent_price = self.data.get(end).map(|s| s.price).unwrap_or(1.0);
                state.push((env_state.price / recent_price) as f32);
                state.push((env_state.volume / 1000000.0) as f32);  // Normalize volume
            }
        }

        // Pad with zeros if not enough data
        while state.len() < self.window_size * 2 {
            state.push(0.0);
        }

        state
    }

    /// Execute one step in the environment
    ///
    /// # Arguments
    /// * `action` - Trading action to execute
    ///
    /// # Returns
    /// (next_state, reward, done) tuple
    pub fn step(&mut self, action: TradingAction) -> (Vec<f32>, f32, bool) {
        if self.current_idx >= self.data.len() {
            return (self.get_state(), 0.0, true);
        }

        let current_price = self.data[self.current_idx].price;
        let reward = match action {
            TradingAction::Buy => {
                if self.position == 0.0 && self.balance > current_price {
                    self.position = self.balance / current_price;
                    self.balance = 0.0;
                    0.0  // No immediate reward
                } else {
                    -0.01  // Penalty for invalid buy
                }
            }
            TradingAction::Sell => {
                if self.position > 0.0 {
                    let pnl = self.position * current_price;
                    let reward = (pnl - self.initial_balance) / self.initial_balance;
                    self.balance = pnl;
                    self.position = 0.0;
                    reward.max(-1.0)  // Cap loss at -100%
                } else {
                    -0.01  // Penalty for invalid sell
                }
            }
            TradingAction::Hold => {
                if self.position > 0.0 {
                    let current_value = self.position * current_price;
                    (current_value - self.initial_balance) / self.initial_balance
                } else {
                    0.0
                }
            }
        };

        self.current_idx += 1;
        let done = self.current_idx >= self.data.len() - 1;

        (self.get_state(), reward, done)
    }

    /// Get current portfolio value
    pub fn portfolio_value(&self) -> f64 {
        let current_price = self.data.get(self.current_idx.min(self.data.len() - 1))
            .map(|s| s.price)
            .unwrap_or(1.0);
        self.balance + self.position * current_price
    }

    /// Get total return
    pub fn total_return(&self) -> f64 {
        (self.portfolio_value() - self.initial_balance) / self.initial_balance
    }
}

/// Training statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingStats {
    pub episode: usize,
    pub total_reward: f64,
    pub total_return: f64,
    pub final_portfolio_value: f64,
    pub epsilon: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dqn_creation() {
        let agent = DQNTradingAgent::new(30, 0.001);
        assert_eq!(agent.state_size, 30);
        assert_eq!(agent.epsilon, 1.0);
    }

    #[test]
    fn test_trading_action_index() {
        assert_eq!(TradingAction::Buy.to_index(), 0);
        assert_eq!(TradingAction::Sell.to_index(), 1);
        assert_eq!(TradingAction::Hold.to_index(), 2);

        assert_eq!(TradingAction::from_index(0), TradingAction::Buy);
        assert_eq!(TradingAction::from_index(1), TradingAction::Sell);
        assert_eq!(TradingAction::from_index(2), TradingAction::Hold);
    }

    #[test]
    fn test_select_action() {
        let mut agent = DQNTradingAgent::new(10, 0.001);
        let state = vec![0.5f32; 10];
        let action = agent.select_action(&state);
        // Should return a valid action
        match action {
            TradingAction::Buy | TradingAction::Sell | TradingAction::Hold => {}
        }
    }
}
