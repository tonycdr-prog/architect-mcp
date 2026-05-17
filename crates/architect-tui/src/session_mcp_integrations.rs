use serde_json::Value;

use crate::interactive_integrations_support::{
    mcp_install_apply_summary, mcp_install_plan_summary, mcp_install_review_summary,
    mcp_recommendation_summary,
};
use crate::session::{TuiSession, unix_timestamp};

const MCP_INTEGRATION_GATES: &[&str] = &[
    "recommend_mcp_servers",
    "create_mcp_install_plan",
    "review_mcp_install_plan",
    "apply_mcp_install_plan",
];

impl TuiSession {
    pub fn set_mcp_recommendation(&mut self, value: Value) {
        let value = mcp_recommendation_summary(&value);
        self.clear_mcp_integration_state();
        self.gates
            .insert("recommend_mcp_servers".to_string(), value.clone());
        self.mcp_recommendation = Some(value);
        self.updated_at = unix_timestamp();
    }

    pub fn set_mcp_install_plan(&mut self, value: Value) {
        let value = mcp_install_plan_summary(&value);
        self.gates
            .insert("create_mcp_install_plan".to_string(), value.clone());
        self.mcp_install_plan = Some(value);
        self.mcp_install_review = None;
        self.mcp_install_approved = false;
        self.mcp_install_approval_reason = None;
        self.gates.remove("review_mcp_install_plan");
        self.gates.remove("apply_mcp_install_plan");
        self.updated_at = unix_timestamp();
    }

    pub fn set_mcp_install_review(&mut self, value: Value) {
        let value = mcp_install_review_summary(&value);
        self.gates
            .insert("review_mcp_install_plan".to_string(), value.clone());
        self.mcp_install_review = Some(value);
        self.mcp_install_approved = false;
        self.mcp_install_approval_reason = None;
        self.gates.remove("apply_mcp_install_plan");
        self.updated_at = unix_timestamp();
    }

    pub fn approve_mcp_install(&mut self, reason: impl Into<String>) {
        self.mcp_install_approved = true;
        self.mcp_install_approval_reason = Some(reason.into());
        self.updated_at = unix_timestamp();
    }

    pub fn clear_mcp_install_approval(&mut self) {
        self.mcp_install_approved = false;
        self.mcp_install_approval_reason = None;
        self.updated_at = unix_timestamp();
    }

    pub fn set_mcp_install_apply_result(&mut self, value: Value) {
        self.gates.insert(
            "apply_mcp_install_plan".to_string(),
            mcp_install_apply_summary(&value),
        );
        self.updated_at = unix_timestamp();
    }

    pub fn clear_mcp_integration_state(&mut self) {
        self.mcp_recommendation = None;
        self.mcp_install_plan = None;
        self.mcp_install_review = None;
        self.mcp_install_approved = false;
        self.mcp_install_approval_reason = None;
        for gate in MCP_INTEGRATION_GATES {
            self.gates.remove(*gate);
        }
        self.updated_at = unix_timestamp();
    }
}
