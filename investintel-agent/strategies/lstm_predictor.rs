//! LSTM Price Prediction Model
//!
//! Uses Long Short-Term Memory (LSTM) neural networks to predict stock prices
//! Based on tch-rs (PyTorch bindings for Rust)

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

use tch::{nn, Device, Tensor, Kind};
use tch::nn::{LSTMConfig, LinearConfig};

/// LSTM price predictor
pub struct LSTMPredictor {
    device: Device,
    lstm: nn::LSTM,
    fc: nn::Linear,
    vs: nn::VarStore,
    input_size: i64,
    hidden_size: i64,
    num_layers: i64,
}

impl LSTMPredictor {
    /// Create a new LSTM predictor
    ///
    /// # Arguments
    /// * `input_size` - Number of input features (e.g., 5 for OHLCV)
    /// * `hidden_size` - Number of LSTM hidden units
    /// * `num_layers` - Number of LSTM layers
    ///
    /// # Example
    /// ```no_run
    /// # use anyhow::Result;
    /// # fn example() -> Result<()> {
    /// let predictor = LSTMPredictor::new(5, 64, 2);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(input_size: i64, hidden_size: i64, num_layers: i64) -> Self {
        let device = Device::cuda_if_available();
        let vs = nn::VarStore::new(device);

        let lstm_config = LSTMConfig {
            input_size,
            hidden_size,
            num_layers,
            bidirectional: false,
            dropout: 0.2,  // 20% dropout for regularization
            ..Default::default()
        };

        let lstm = nn::lstm(&vs, input_size, hidden_size, num_layers, Default::default());

        let fc_config = LinearConfig {
            input_size: hidden_size,
            output_size: 1,
            bias: true,
            ..Default::default()
        };
        let fc = nn::linear(&vs, hidden_size, 1, fc_config);

        Self {
            device,
            lstm,
            fc,
            vs,
            input_size,
            hidden_size,
            num_layers,
        }
    }

    /// Train the LSTM model
    ///
    /// # Arguments
    /// * `data` - Training data with sequences and targets
    /// * `epochs` - Number of training epochs
    /// * `learning_rate` - Learning rate for optimizer
    /// * `log_interval` - Print loss every N epochs
    pub fn train(
        &mut self,
        data: &TrainingData,
        epochs: usize,
        learning_rate: f64,
        log_interval: usize,
    ) -> Result<TrainingMetrics> {
        let mut opt = nn::Adam::default()
            .build(&self.vs, learning_rate)
            .context("Failed to create Adam optimizer")?;

        let mut losses = Vec::new();
        let start_time = std::time::Instant::now();

        for epoch in 0..epochs {
            // Forward pass
            let output = self.forward(&data.inputs)?;
            let loss = output.mse_loss(&data.targets, tch::Reduction::Mean);

            // Backward pass
            opt.backward_step(&loss);

            let loss_value = f64::from(loss);
            losses.push(loss_value);

            if epoch % log_interval == 0 || epoch == epochs - 1 {
                println!(
                    "Epoch {}/{}: Loss = {:.6}",
                    epoch + 1,
                    epochs,
                    loss_value
                );
            }
        }

        let training_time = start_time.elapsed();

        Ok(TrainingMetrics {
            final_loss: *losses.last().unwrap_or(&0.0),
            avg_loss: losses.iter().sum::<f64>() / losses.len() as f64,
            epochs,
            training_time_secs: training_time.as_secs_f64(),
        })
    }

    /// Forward pass through LSTM
    fn forward(&self, inputs: &Tensor) -> Result<Tensor> {
        let seq_len = inputs.size()[0] as i64;
        let batch_size = inputs.size()[1] as i64;

        // Initialize hidden state
        let (h0, c0) = self.init_hidden(batch_size);

        // LSTM forward
        let lstm_out = self.lstm.seq(inputs, (h0, c0))?;

        // Take the last output
        let last_output = lstm_out.get(lstm_out.size()[0] - 1);

        // Fully connected layer
        let prediction = last_output.apply(&self.fc);
        Ok(prediction)
    }

    /// Initialize hidden state
    fn init_hidden(&self, batch_size: i64) -> (Tensor, Tensor) {
        let h0 = Tensor::zeros(
            &[self.num_layers, batch_size, self.hidden_size],
            (Kind::Float, self.device),
        );
        let c0 = Tensor::zeros(
            &[self.num_layers, batch_size, self.hidden_size],
            (Kind::Float, self.device),
        );
        (h0, c0)
    }

    /// Predict next price
    ///
    /// # Arguments
    /// * `sequence` - Input sequence (seq_len x input_size)
    ///
    /// # Returns
    /// Predicted price
    pub fn predict(&self, sequence: &Tensor) -> Result<f64> {
        let output = self.forward(sequence)?;
        let prediction = f64::from(output.double_value(&[]));
        Ok(prediction)
    }

    /// Save model to disk
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.vs.save(path)
            .context("Failed to save model")?;
        Ok(())
    }

    /// Load model from disk
    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.vs.load(path)
            .context("Failed to load model")?;
        Ok(())
    }
}

/// Training data for LSTM
#[derive(Debug, Clone)]
pub struct TrainingData {
    pub inputs: Tensor,  // Shape: [seq_len, batch, input_size]
    pub targets: Tensor, // Shape: [batch, 1]
}

impl TrainingData {
    /// Create training data from OHLCV prices
    ///
    /// # Arguments
    /// * `prices` - Historical prices (OHLCV data)
    /// * `seq_len` - Sequence length (number of time steps)
    /// * `prediction_horizon` - Days ahead to predict
    pub fn from_prices(
        prices: &[f64],
        seq_len: usize,
        prediction_horizon: usize,
    ) -> Result<Self> {
        if prices.len() < seq_len + prediction_horizon {
            anyhow::bail!(
                "Insufficient data: need at least {} prices, got {}",
                seq_len + prediction_horizon,
                prices.len()
            );
        }

        let mut sequences = Vec::new();
        let mut targets = Vec::new();

        // Create sliding window sequences
        for i in 0..=(prices.len() - seq_len - prediction_horizon) {
            let sequence: Vec<f32> = prices[i..i + seq_len]
                .iter()
                .map(|&p| p as f32)
                .collect();

            let target = prices[i + seq_len + prediction_horizon - 1] as f32;

            sequences.push(sequence);
            targets.push(target);
        }

        // Convert to tensors
        let num_sequences = sequences.len();
        let inputs = Tensor::of_slice(
            &sequences.into_iter().flatten().collect::<Vec<_>>()[..]
        )
        .view([seq_len as i64, num_sequences as i64, 1])
        .to(self::Device::Cpu);

        let targets_tensor = Tensor::of_slice(&targets)
            .view([num_sequences as i64, 1]);

        Ok(Self {
            inputs,
            targets: targets_tensor,
        })
    }

    /// Create training data from multiple features
    ///
    /// # Arguments
    /// * `features` - 2D array [time_steps, num_features]
    /// * `targets` - Target prices
    /// * `seq_len` - Sequence length
    pub fn from_features(
        features: &[Vec<f64>],
        targets: &[f64],
        seq_len: usize,
    ) -> Result<Self> {
        if features.len() < seq_len {
            anyhow::bail!(
                "Insufficient feature data: need at least {} time steps, got {}",
                seq_len,
                features.len()
            );
        }

        let num_features = features[0].len();
        let mut sequences = Vec::new();
        let mut target_values = Vec::new();

        for i in 0..=(features.len() - seq_len) {
            let sequence: Vec<f32> = features[i..i + seq_len]
                .iter()
                .flat_map(|v| v.iter().map(|&x| x as f32))
                .collect();

            let target = targets[i + seq_len - 1] as f32;

            sequences.push(sequence);
            target_values.push(target);
        }

        let num_sequences = sequences.len();
        let inputs = Tensor::of_slice(&sequences.into_iter().flatten().collect::<Vec<_>>()[..])
            .view([seq_len as i64, num_sequences as i64, num_features as i64]);

        let targets_tensor = Tensor::of_slice(&target_values)
            .view([num_sequences as i64, 1]);

        Ok(Self {
            inputs,
            targets: targets_tensor,
        })
    }
}

/// Training metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingMetrics {
    pub final_loss: f64,
    pub avg_loss: f64,
    pub epochs: usize,
    pub training_time_secs: f64,
}

/// Prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    pub symbol: String,
    pub current_price: f64,
    pub predicted_price: f64,
    pub change_percent: f64,
    pub confidence: f64,  // 0.0 - 1.0
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lstm_creation() {
        let predictor = LSTMPredictor::new(5, 64, 2);
        assert_eq!(predictor.input_size, 5);
        assert_eq!(predictor.hidden_size, 64);
        assert_eq!(predictor.num_layers, 2);
    }

    #[test]
    fn test_training_data_from_prices() {
        let prices = vec![
            100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0,
            110.0, 111.0, 112.0, 113.0, 114.0, 115.0, 116.0, 117.0, 118.0, 119.0,
        ];

        let data = TrainingData::from_prices(&prices, 10, 1).unwrap();
        assert_eq!(data.inputs.size()[0], 10);  // seq_len
        assert!(data.inputs.size()[1] > 0);     // batch_size
    }

    #[test]
    fn test_training_data_insufficient() {
        let prices = vec![100.0, 101.0, 102.0];
        let result = TrainingData::from_prices(&prices, 10, 1);
        assert!(result.is_err());
    }
}
