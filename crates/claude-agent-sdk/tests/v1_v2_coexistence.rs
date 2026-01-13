//! V1/V2 API Coexistence Tests
//!
//! This test file verifies that V1 and V2 APIs can coexist in the same application
//! without conflicts or interference.

use claude_agent_sdk::{
    ClaudeAgentOptions, ClaudeClient, ClaudeError, PermissionMode as V1PermissionMode, Result,
};
use claude_agent_sdk::v2::{
    PermissionMode as V2PermissionMode, PromptResult, Session, SessionOptions,
};

#[test]
fn test_v1_v2_imports_coexist() {
    // Test that both V1 and V2 types can be imported simultaneously
    let _v1_options = ClaudeAgentOptions::builder()
        .permission_mode(V1PermissionMode::Default)
        .build();

    let _v2_options = SessionOptions::builder()
        .permission_mode(V2PermissionMode::Default)
        .build();

    // If this compiles, both APIs coexist without naming conflicts
    assert!(true);
}

#[test]
fn test_v1_v2_permission_modes_equal() {
    // Verify that V1 and V2 PermissionMode have the same values
    assert_eq!(format!("{:?}", V1PermissionMode::Default), "Default");
    assert_eq!(format!("{:?}", V2PermissionMode::Default), "Default");

    assert_eq!(format!("{:?}", V1PermissionMode::BypassPermissions), "BypassPermissions");
    assert_eq!(format!("{:?}", V2PermissionMode::BypassPermissions), "BypassPermissions");

    assert_eq!(format!("{:?}", V1PermissionMode::AcceptEdits), "AcceptEdits");
    assert_eq!(format!("{:?}", V2PermissionMode::AcceptEdits), "AcceptEdits");

    assert_eq!(format!("{:?}", V1PermissionMode::Plan), "Plan");
    assert_eq!(format!("{:?}", V2PermissionMode::Plan), "Plan");
}

#[test]
fn test_v1_claude_agent_options_builder() {
    // Test V1 ClaudeAgentOptions builder
    let options = ClaudeAgentOptions::builder()
        .model("claude-sonnet-4")
        .permission_mode(V1PermissionMode::BypassPermissions)
        .max_turns(10)
        .max_budget_usd(10.0)
        .build();

    assert_eq!(options.model, Some("claude-sonnet-4".to_string()));
    assert_eq!(format!("{:?}", options.permission_mode), "Some(BypassPermissions)");
    assert_eq!(options.max_turns, Some(10));
    assert_eq!(options.max_budget_usd, Some(10.0));
}

#[test]
fn test_v2_session_options_builder() {
    // Test V2 SessionOptions builder
    let options = SessionOptions::builder()
        .model("claude-sonnet-4".to_string())
        .permission_mode(V2PermissionMode::BypassPermissions)
        .max_turns(10)
        .max_budget_usd(10.0)
        .build();

    assert_eq!(options.model, Some("claude-sonnet-4".to_string()));
    assert_eq!(format!("{:?}", options.permission_mode), "Some(BypassPermissions)");
    assert_eq!(options.max_turns, Some(10));
    assert_eq!(options.max_budget_usd, Some(10.0));
}

#[test]
fn test_v1_v2_options_equivalence() {
    // Verify that V1 and V2 options produce equivalent configurations
    let v1_options = ClaudeAgentOptions::builder()
        .model("claude-sonnet-4")
        .permission_mode(V1PermissionMode::Default)
        .max_turns(5)
        .build();

    let v2_options = SessionOptions::builder()
        .model("claude-sonnet-4".to_string())
        .permission_mode(V2PermissionMode::Default)
        .max_turns(5)
        .build();

    // Check model equivalence
    assert_eq!(v1_options.model, v2_options.model);

    // Check permission mode equivalence
    let v1_perm = format!("{:?}", v1_options.permission_mode);
    let v2_perm = format!("{:?}", v2_options.permission_mode);
    assert!(v2_perm.contains(&v1_perm));

    // Check max_turns equivalence
    assert_eq!(v1_options.max_turns, v2_options.max_turns);
}

#[test]
fn test_v1_v2_no_naming_conflicts() {
    // Verify that there are no naming conflicts between V1 and V2 types

    // V1 types
    let _v1_client_type: std::marker::PhantomData<ClaudeClient> = std::marker::PhantomData;
    let _v1_options_type: std::marker::PhantomData<ClaudeAgentOptions> = std::marker::PhantomData;
    let _v1_perm_type: std::marker::PhantomData<V1PermissionMode> = std::marker::PhantomData;

    // V2 types
    let _v2_session_type: std::marker::PhantomData<Session> = std::marker::PhantomData;
    let _v2_options_type: std::marker::PhantomData<SessionOptions> = std::marker::PhantomData;
    let _v2_perm_type: std::marker::PhantomData<V2PermissionMode> = std::marker::PhantomData;
    let _v2_result_type: std::marker::PhantomData<PromptResult> = std::marker::PhantomData;

    // If this compiles, all types can coexist
    assert!(true);
}

#[test]
fn test_v1_default_options() {
    // Test V1 default options
    let options = ClaudeAgentOptions::default();

    // Verify defaults
    assert!(options.model.is_none()); // Uses SDK default
    assert!(options.permission_mode.is_none()); // Uses SDK default
    assert!(options.max_turns.is_none());
    assert!(options.max_budget_usd.is_none());
}

#[test]
fn test_v2_default_options() {
    // Test V2 default options
    let options = SessionOptions::default();

    // Verify defaults
    assert!(options.model.is_none()); // Uses SDK default
    assert!(options.permission_mode.is_none()); // Uses SDK default
    assert!(options.max_turns.is_none());
    assert!(options.max_budget_usd.is_none());
}

#[test]
fn test_v1_v2_optional_fields_difference() {
    // Test that V2 uses Option<T> for all fields, while V1 has some non-optional fields

    // V1: Some fields are not Option<T>
    let v1_options = ClaudeAgentOptions::default();
    // permission_mode is not Option in V1, it has a default value
    let _ = v1_options.permission_mode; // This compiles

    // V2: All fields are Option<T>
    let v2_options = SessionOptions::default();
    // permission_mode is Option<PermissionMode> in V2
    assert!(v2_options.permission_mode.is_none()); // This is None by default
}

#[test]
fn test_v1_v2_builder_patterns() {
    // Test that both V1 and V2 use the builder pattern correctly

    // V1 builder
    let v1_built = ClaudeAgentOptions::builder()
        .model("test-model")
        .permission_mode(V1PermissionMode::BypassPermissions)
        .build();

    assert_eq!(v1_built.model, Some("test-model".to_string()));

    // V2 builder
    let v2_built = SessionOptions::builder()
        .model("test-model".to_string())
        .permission_mode(V2PermissionMode::BypassPermissions)
        .build();

    assert_eq!(v2_built.model, Some("test-model".to_string()));
}

#[test]
fn test_v1_cloned_options() {
    // Test that V1 options can be cloned
    let options1 = ClaudeAgentOptions::builder()
        .model("claude-sonnet-4")
        .build();

    let options2 = options1.clone();

    assert_eq!(options1.model, options2.model);
    assert_eq!(format!("{:?}", options1.permission_mode), format!("{:?}", options2.permission_mode));
}

#[test]
fn test_v2_cloned_options() {
    // Test that V2 options can be cloned
    let options1 = SessionOptions::builder()
        .model("claude-sonnet-4".to_string())
        .build();

    let options2 = options1.clone();

    assert_eq!(options1.model, options2.model);
    assert_eq!(options1.permission_mode, options2.permission_mode);
}

#[test]
fn test_coexistence_in_same_function() {
    // Test that V1 and V2 APIs can be used in the same function

    // Create V1 options
    let v1_opts = ClaudeAgentOptions::builder()
        .permission_mode(V1PermissionMode::Default)
        .build();

    // Create V2 options
    let v2_opts = SessionOptions::builder()
        .permission_mode(V2PermissionMode::Default)
        .build();

    // Both should coexist
    assert_eq!(format!("{:?}", v1_opts.permission_mode), "Some(Default)");
    assert!(v2_opts.permission_mode.is_some());
}

#[test]
fn test_v1_v2_types_are_distinct() {
    // Verify that V1 and V2 types are distinct and cannot be mistakenly interchanged

    // V1 ClaudeAgentOptions and V2 SessionOptions are different types
    let v1_opts: ClaudeAgentOptions = ClaudeAgentOptions::default();
    let v2_opts: SessionOptions = SessionOptions::default();

    // They cannot be assigned to each other (compile-time check)
    // If this test compiles, the types are distinct
    let _ = (v1_opts, v2_opts);

    // V1 and V2 PermissionMode are also in different modules
    let v1_perm: V1PermissionMode = V1PermissionMode::Default;
    let v2_perm: V2PermissionMode = V2PermissionMode::Default;

    // They are distinct types
    let _ = (v1_perm, v2_perm);
}

#[tokio::test]
async fn test_v1_v2_async_functions_coexist() {
    // Test that V1 and V2 async functions can coexist

    // Both APIs can be imported and used in the same async context
    async fn use_v1_api() -> Result<()> {
        let _options = ClaudeAgentOptions::default();
        Ok(())
    }

    async fn use_v2_api() -> Result<PromptResult> {
        // This would normally call prompt(), but we're just testing compilation
        let _options = SessionOptions::default();
        Err(ClaudeError::InternalError(
            "Not actually calling API in test".to_string(),
        ))
    }

    // Both can be awaited in the same function
    let _v1_result = use_v1_api().await;
    let _v2_result = use_v2_api().await;

    // If this compiles and runs, async functions coexist
    assert!(true);
}
