//! Transport layer for communicating with Claude Code CLI

pub mod pooled;
pub mod subprocess;
mod trait_def;

pub use pooled::{PooledTransport, get_pool, init_pool};
pub use subprocess::{BufferMetricsSnapshot, SubprocessTransport};
pub use trait_def::Transport;
