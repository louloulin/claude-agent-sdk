//! Skills Integration Module
//!
//! This module provides integration between Agents and Skills in the investment system.

pub mod skill_agent;

pub use skill_agent::{
    SkillAgent,
    SkillAgentBuilder,
    SkillRegistry,
};
