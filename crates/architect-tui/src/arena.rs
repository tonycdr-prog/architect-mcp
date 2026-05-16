use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArenaReviewStatus {
    Pass,
    Warn,
    Fail,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArenaCandidateInput {
    pub adapter: String,
    pub review_status: ArenaReviewStatus,
    pub verification_passed: bool,
    pub diff_size: u64,
    pub contract_drift: bool,
    pub crashed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RankedArenaCandidate {
    pub rank: usize,
    pub adapter: String,
    pub score: u8,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArenaCandidateRecord {
    pub adapter: String,
    pub worktree: Option<String>,
    pub review_status: ArenaReviewStatus,
    pub verification_passed: bool,
    pub diff_size: u64,
    pub contract_drift: bool,
    pub crashed: bool,
    pub changed_files: Vec<Value>,
    pub summary: Vec<String>,
}

impl ArenaCandidateRecord {
    pub fn as_input(&self) -> ArenaCandidateInput {
        ArenaCandidateInput {
            adapter: self.adapter.clone(),
            review_status: self.review_status.clone(),
            verification_passed: self.verification_passed,
            diff_size: self.diff_size,
            contract_drift: self.contract_drift,
            crashed: self.crashed,
        }
    }
}

pub fn rank_arena_candidates(candidates: Vec<ArenaCandidateInput>) -> Vec<RankedArenaCandidate> {
    let mut ranked = candidates
        .into_iter()
        .map(score_candidate)
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.adapter.cmp(&right.adapter))
    });
    for (index, candidate) in ranked.iter_mut().enumerate() {
        candidate.rank = index + 1;
    }
    ranked
}

pub fn rank_arena_records(candidates: &[ArenaCandidateRecord]) -> Vec<RankedArenaCandidate> {
    rank_arena_candidates(
        candidates
            .iter()
            .map(ArenaCandidateRecord::as_input)
            .collect(),
    )
}

fn score_candidate(candidate: ArenaCandidateInput) -> RankedArenaCandidate {
    let mut score = 100_i16;
    let mut reasons = Vec::new();

    match candidate.review_status {
        ArenaReviewStatus::Pass => reasons.push("implementation review passed".to_string()),
        ArenaReviewStatus::Warn => {
            score -= 18;
            reasons.push("implementation review has warnings".to_string());
        }
        ArenaReviewStatus::Fail => {
            score -= 55;
            reasons.push("implementation review failed".to_string());
        }
        ArenaReviewStatus::Unknown => {
            score -= 25;
            reasons.push("implementation review pending".to_string());
        }
    }
    if !candidate.verification_passed {
        score -= 18;
        reasons.push("verification not passed".to_string());
    }
    if candidate.contract_drift {
        score -= 35;
        reasons.push("contract drift detected".to_string());
    }
    if candidate.crashed {
        score -= 45;
        reasons.push("adapter crashed or timed out".to_string());
    }
    let diff_penalty = (candidate.diff_size / 500).min(15) as i16;
    if diff_penalty > 0 {
        score -= diff_penalty;
        reasons.push(format!("large diff penalty: {diff_penalty}"));
    }

    RankedArenaCandidate {
        rank: 0,
        adapter: candidate.adapter,
        score: score.clamp(0, 100) as u8,
        reasons,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arena_ranking_rewards_verified_reviewed_candidates() {
        let ranked = rank_arena_candidates(vec![
            ArenaCandidateInput {
                adapter: "codex".to_string(),
                review_status: ArenaReviewStatus::Pass,
                verification_passed: true,
                diff_size: 150,
                contract_drift: false,
                crashed: false,
            },
            ArenaCandidateInput {
                adapter: "claude".to_string(),
                review_status: ArenaReviewStatus::Warn,
                verification_passed: false,
                diff_size: 2400,
                contract_drift: true,
                crashed: false,
            },
            ArenaCandidateInput {
                adapter: "shell".to_string(),
                review_status: ArenaReviewStatus::Unknown,
                verification_passed: false,
                diff_size: 0,
                contract_drift: false,
                crashed: true,
            },
        ]);
        assert_eq!(ranked[0].adapter, "codex");
        assert_eq!(ranked[0].rank, 1);
        assert!(ranked[0].score > ranked[1].score);
        assert!(
            ranked[2]
                .reasons
                .iter()
                .any(|reason| reason == "adapter crashed or timed out")
        );
    }

    #[test]
    fn arena_records_rank_with_same_scoring_model() {
        let ranked = rank_arena_records(&[
            ArenaCandidateRecord {
                adapter: "codex".to_string(),
                worktree: Some(".architect-mcp/worktrees/session/codex".to_string()),
                review_status: ArenaReviewStatus::Pass,
                verification_passed: true,
                diff_size: 12,
                contract_drift: false,
                crashed: false,
                changed_files: Vec::new(),
                summary: Vec::new(),
            },
            ArenaCandidateRecord {
                adapter: "shell".to_string(),
                worktree: Some(".architect-mcp/worktrees/session/shell".to_string()),
                review_status: ArenaReviewStatus::Fail,
                verification_passed: false,
                diff_size: 0,
                contract_drift: true,
                crashed: false,
                changed_files: Vec::new(),
                summary: Vec::new(),
            },
        ]);
        assert_eq!(ranked[0].adapter, "codex");
        assert_eq!(ranked[1].adapter, "shell");
    }
}
