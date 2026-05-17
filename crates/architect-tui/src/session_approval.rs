use crate::promotion_receipt::PromotionReceipt;
use crate::session::{ApprovalStatus, TuiSession, unix_timestamp};

impl TuiSession {
    pub fn approve(&mut self, reason: impl Into<String>) {
        self.promotion_receipt = None;
        self.approval_status = ApprovalStatus::Approved;
        self.approval_reason = Some(reason.into());
        self.updated_at = unix_timestamp();
    }

    pub fn approve_execution(&mut self, reason: impl Into<String>) {
        self.execution_approved = true;
        self.execution_approval_reason = Some(reason.into());
        self.updated_at = unix_timestamp();
    }

    pub fn clear_execution_approval(&mut self) {
        self.execution_approved = false;
        self.execution_approval_reason = None;
        self.updated_at = unix_timestamp();
    }

    pub fn reject(&mut self, reason: impl Into<String>) {
        self.promotion_receipt = None;
        self.approval_status = ApprovalStatus::Rejected;
        self.approval_reason = Some(reason.into());
        self.updated_at = unix_timestamp();
    }

    pub fn override_approval(&mut self, reason: impl Into<String>) {
        self.promotion_receipt = None;
        self.approval_status = ApprovalStatus::Override;
        self.approval_reason = Some(reason.into());
        self.updated_at = unix_timestamp();
    }

    pub fn mark_promoted(&mut self, receipt: PromotionReceipt) {
        self.promotion_receipt = Some(receipt);
        self.approval_status = ApprovalStatus::Promoted;
        self.updated_at = unix_timestamp();
    }

    pub fn can_promote(&self) -> bool {
        matches!(
            self.approval_status,
            ApprovalStatus::Approved | ApprovalStatus::Override
        )
    }
}
