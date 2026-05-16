use serde::{Deserialize, Serialize};

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
}
