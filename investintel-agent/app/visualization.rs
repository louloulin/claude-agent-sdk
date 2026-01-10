// visualization.rs - Chart and visualization generation using plotters
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use plotters::prelude::*;
use plotters::style::{Color, RGBAColor};
use std::path::Path;

/// Chart configuration
#[derive(Debug, Clone)]
pub struct ChartConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub show_grid: bool,
    pub show_legend: bool,
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            title: "InvestIntel Chart".to_string(),
            show_grid: true,
            show_legend: true,
        }
    }
}

/// Price data point for plotting
#[derive(Debug, Clone)]
pub struct PricePoint {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
}

/// Chart generator
pub struct ChartGenerator {
    config: ChartConfig,
}

impl ChartGenerator {
    /// Create a new chart generator
    pub fn new(config: ChartConfig) -> Self {
        Self { config }
    }

    /// Generate line chart for price data
    pub fn generate_line_chart(
        &self,
        data: &[(DateTime<Utc>, f64)],
        output_path: &Path,
    ) -> Result<()> {
        let root = BitMapBackend::new(output_path, (self.config.width, self.config.height)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption(&self.config.title, ("sans-serif", 40))
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(
                data.first().unwrap().0.timestamp() as f64..data.last().unwrap().0.timestamp() as f64,
                self.find_min_y(data)..self.find_max_y(data),
            )?;

        chart.configure_mesh().draw()?;

        chart
            .draw_series(LineSeries::new(
                data.iter().map(|(dt, price)| (dt.timestamp() as f64, *price)),
                &BLUE,
            ))?
            .label("Price")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &BLUE));

        if self.config.show_legend {
            chart.configure_series_labels().draw()?;
        }

        root.present()?;

        Ok(())
    }

    /// Generate candlestick chart
    pub fn generate_candlestick_chart(
        &self,
        data: &[PricePoint],
        output_path: &Path,
    ) -> Result<()> {
        let root = BitMapBackend::new(output_path, (self.config.width, self.config.height)).into_drawing_area();
        root.fill(&WHITE)?;

        let (min_price, max_price) = self.find_price_range(data);
        let (start_time, end_time) = self.find_time_range(data);

        let mut chart = ChartBuilder::on(&root)
            .caption(&self.config.title, ("sans-serif", 40))
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(start_time..end_time, min_price..max_price)?;

        if self.config.show_grid {
            chart.configure_mesh().draw()?;
        }

        // Draw candlesticks
        for candle in data {
            let x = candle.timestamp.timestamp() as f64;
            let open = candle.open;
            let close = candle.close;
            let high = candle.high;
            let low = candle.low;

            let color = if close > open { &GREEN } else { &RED };

            // Draw wick (high to low)
            chart.draw_series(std::iter::once(PathElement::new(
                vec![(x, high), (x, low)],
                color,
            )))?;

            // Draw body (open to close)
            let body_top = open.max(close);
            let body_bottom = open.min(close);
            chart.draw_series(std::iter::once(Rectangle::new(
                [(x - 0.5, body_bottom), (x + 0.5, body_top)],
                color.filled(),
            )))?;
        }

        root.present()?;

        Ok(())
    }

    /// Generate equity curve chart
    pub fn generate_equity_curve(
        &self,
        equity_data: &[(DateTime<Utc>, f64)],
        output_path: &Path,
    ) -> Result<()> {
        let root = BitMapBackend::new(output_path, (self.config.width, self.config.height)).into_drawing_area();
        root.fill(&WHITE)?;

        let initial_equity = equity_data.first().unwrap().1;

        let mut chart = ChartBuilder::on(&root)
            .caption("Portfolio Equity Curve", ("sans-serif", 40))
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(
                equity_data.first().unwrap().0.timestamp() as f64..equity_data.last().unwrap().0.timestamp() as f64,
                initial_equity * 0.95..self.find_max_y(equity_data) * 1.05,
            )?;

        chart.configure_mesh().draw()?;

        // Draw equity line
        chart
            .draw_series(LineSeries::new(
                equity_data.iter().map(|(dt, equity)| (dt.timestamp() as f64, *equity)),
                &BLUE.stroke_width(2),
            ))?
            .label("Portfolio Value")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &BLUE));

        // Draw initial capital reference line
        chart
            .draw_series(LineSeries::new(
                vec![
                    (equity_data.first().unwrap().0.timestamp() as f64, initial_equity),
                    (equity_data.last().unwrap().0.timestamp() as f64, initial_equity),
                ],
                &RED.stroke_width(1).mix(0.5),
            ))?
            .label("Initial Capital")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &RED));

        if self.config.show_legend {
            chart.configure_series_labels().draw()?;
        }

        root.present()?;

        Ok(())
    }

    /// Generate drawdown chart
    pub fn generate_drawdown_chart(
        &self,
        equity_data: &[(DateTime<Utc>, f64)],
        output_path: &Path,
    ) -> Result<()> {
        let root = BitMapBackend::new(output_path, (self.config.width, self.config.height)).into_drawing_area();
        root.fill(&WHITE)?;

        // Calculate drawdown
        let mut drawdown_data = Vec::new();
        let mut peak = equity_data.first().unwrap().1;

        for (dt, equity) in equity_data {
            if *equity > peak {
                peak = *equity;
            }
            let drawdown = (peak - equity) / peak * 100.0;
            drawdown_data.push((*dt, drawdown));
        }

        let mut chart = ChartBuilder::on(&root)
            .caption("Drawdown Chart", ("sans-serif", 40))
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(
                drawdown_data.first().unwrap().0.timestamp() as f64..drawdown_data.last().unwrap().0.timestamp() as f64,
                0.0..self.find_max_y(&drawdown_data) * 1.1,
            )?;

        chart.configure_mesh().draw()?;

        // Fill drawdown area
        chart.draw_series(std::iter::once(PathElement::new(
            drawdown_data
                .iter()
                .map(|(dt, dd)| (*dt, 0.0))
                .chain(drawdown_data.iter().map(|(dt, dd)| (*dt, *dd)))
                .chain(std::iter::once((
                    drawdown_data.last().unwrap().0.timestamp() as f64,
                    0.0,
                ))),
            &RED.mix(0.3).filled(),
        )))?;

        // Draw drawdown line
        chart
            .draw_series(LineSeries::new(
                drawdown_data.iter().map(|(dt, dd)| (dt.timestamp() as f64, *dd)),
                &RED.stroke_width(2),
            ))?
            .label("Drawdown %")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &RED));

        if self.config.show_legend {
            chart.configure_series_labels().draw()?;
        }

        root.present()?;

        Ok(())
    }

    /// Generate returns distribution histogram
    pub fn generate_returns_histogram(
        &self,
        returns: &[f64],
        output_path: &Path,
    ) -> Result<()> {
        let root = BitMapBackend::new(output_path, (self.config.width, self.config.height)).into_drawing_area();
        root.fill(&WHITE)?;

        // Calculate histogram bins
        let num_bins = 50;
        let min_return = returns.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_return = returns.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let bin_width = (max_return - min_return) / num_bins as f64;

        let mut bins = vec![0; num_bins];
        for ret in returns {
            let bin_idx = ((ret - min_return) / bin_width).floor() as usize;
            if bin_idx < num_bins {
                bins[bin_idx] += 1;
            }
        }

        let max_count = *bins.iter().max().unwrap_or(&1) as f64;

        let mut chart = ChartBuilder::on(&root)
            .caption("Returns Distribution", ("sans-serif", 40))
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(min_return..max_return, 0.0..max_count)?;

        chart.configure_mesh().draw()?;

        // Draw bars
        chart.draw_series(
            bins.iter()
                .enumerate()
                .map(|(i, count)| {
                    let bin_start = min_return + i as f64 * bin_width;
                    let bin_end = bin_start + bin_width;
                    let bar_center = (bin_start + bin_end) / 2.0;

                    Rectangle::new(
                        [(bin_start, 0.0), (bin_end, *count as f64)],
                        BLUE.filled(),
                    )
                })
        )?;

        root.present()?;

        Ok(())
    }

    /// Generate technical indicators chart
    pub fn generate_indicators_chart(
        &self,
        price_data: &[(DateTime<Utc>, f64)],
        sma_20: &[f64],
        sma_50: &[f64],
        output_path: &Path,
    ) -> Result<()> {
        let root = BitMapBackend::new(output_path, (self.config.width, self.config.height)).into_drawing_area();
        root.fill(&WHITE)?;

        let max_price = self.find_max_y(price_data);
        let min_price = self.find_min_y(price_data);

        let mut chart = ChartBuilder::on(&root)
            .caption("Technical Indicators", ("sans-serif", 40))
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(
                price_data.first().unwrap().0.timestamp() as f64..price_data.last().unwrap().0.timestamp() as f64,
                min_price..max_price,
            )?;

        chart.configure_mesh().draw()?;

        // Draw price line
        chart
            .draw_series(LineSeries::new(
                price_data.iter().map(|(dt, price)| (dt.timestamp() as f64, *price)),
                &BLACK.stroke_width(1),
            ))?
            .label("Price")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &BLACK));

        // Draw SMA 20
        if sma_20.len() == price_data.len() {
            chart
                .draw_series(LineSeries::new(
                    price_data.iter().enumerate().map(|(i, (dt, _))| {
                        (dt.timestamp() as f64, sma_20[i])
                    }),
                    &BLUE.stroke_width(2),
                ))?
                .label("SMA 20")
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &BLUE));
        }

        // Draw SMA 50
        if sma_50.len() == price_data.len() {
            chart
                .draw_series(LineSeries::new(
                    price_data.iter().enumerate().map(|(i, (dt, _))| {
                        (dt.timestamp() as f64, sma_50[i])
                    }),
                    &RED.stroke_width(2),
                ))?
                .label("SMA 50")
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &RED));
        }

        if self.config.show_legend {
            chart.configure_series_labels().draw()?;
        }

        root.present()?;

        Ok(())
    }

    // Helper functions
    fn find_min_y(&self, data: &[(DateTime<Utc>, f64)]) -> f64 {
        data.iter().map(|(_, y)| *y).fold(f64::INFINITY, f64::min)
    }

    fn find_max_y(&self, data: &[(DateTime<Utc>, f64)]) -> f64 {
        data.iter().map(|(_, y)| *y).fold(f64::NEG_INFINITY, f64::max)
    }

    fn find_price_range(&self, data: &[PricePoint]) -> (f64, f64) {
        let min = data.iter().map(|p| p.low).fold(f64::INFINITY, f64::min);
        let max = data.iter().map(|p| p.high).fold(f64::NEG_INFINITY, f64::max);
        (min, max)
    }

    fn find_time_range(&self, data: &[PricePoint]) -> (f64, f64) {
        let min = data.first().unwrap().timestamp.timestamp() as f64;
        let max = data.last().unwrap().timestamp.timestamp() as f64;
        (min, max)
    }
}

// Define colors
const WHITE: RGBAColor = RGBAColor(255, 255, 255, 1.0);
const BLACK: RGBAColor = RGBAColor(0, 0, 0, 1.0);
const RED: RGBAColor = RGBAColor(255, 0, 0, 1.0);
const GREEN: RGBAColor = RGBAColor(0, 255, 0, 1.0);
const BLUE: RGBAColor = RGBAColor(0, 0, 255, 1.0);

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_generate_line_chart() {
        let config = ChartConfig {
            title: "Test Line Chart".to_string(),
            ..Default::default()
        };

        let generator = ChartGenerator::new(config);

        let data: Vec<(DateTime<Utc>, f64)> = (0..100)
            .map(|i| {
                (
                    Utc::now() - Duration::days(100 - i),
                    100.0 + i as f64 + (rand::random::<f64>() - 0.5) * 10.0,
                )
            })
            .collect();

        let output_path = Path::new("/tmp/test_line_chart.png");
        generator.generate_line_chart(&data, output_path).unwrap();

        assert!(output_path.exists());
        println!("✅ Line chart generated successfully");
    }

    #[test]
    fn test_generate_candlestick_chart() {
        let config = ChartConfig {
            title: "Test Candlestick Chart".to_string(),
            ..Default::default()
        };

        let generator = ChartGenerator::new(config);

        let data: Vec<PricePoint> = (0..50)
            .map(|i| {
                let base = 150.0 + i as f64 * 0.5;
                let range = 2.0;
                PricePoint {
                    timestamp: Utc::now() - Duration::days(50 - i),
                    open: base + (rand::random::<f64>() - 0.5) * range,
                    high: base + rand::random::<f64>() * range,
                    low: base - rand::random::<f64>() * range,
                    close: base + (rand::random::<f64>() - 0.5) * range,
                    volume: 1000000,
                }
            })
            .collect();

        let output_path = Path::new("/tmp/test_candlestick_chart.png");
        generator.generate_candlestick_chart(&data, output_path).unwrap();

        assert!(output_path.exists());
        println!("✅ Candlestick chart generated successfully");
    }
}
