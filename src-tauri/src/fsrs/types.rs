use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Rating {
    Again = 0,
    Hard = 1,
    Good = 2,
    Easy = 3,
}

impl Rating {
    pub fn from_i32(n: i32) -> Option<Self> {
        match n {
            0 => Some(Self::Again),
            1 => Some(Self::Hard),
            2 => Some(Self::Good),
            3 => Some(Self::Easy),
            _ => None,
        }
    }
}

pub const DEFAULT_PARAMS: [f64; 17] = [
    0.4, 0.6, 2.4, 5.8, 4.93, 0.94, 0.86, 0.01, 1.49, 0.14, 0.94, 2.18, 0.05, 0.34, 1.26, 0.29,
    2.61,
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FSRSParameters {
    pub w: [f64; 17],
}

impl Default for FSRSParameters {
    fn default() -> Self {
        Self { w: DEFAULT_PARAMS }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CardState {
    New = 0,
    Learning = 1,
    Review = 2,
    Relearning = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub stability: f64,
    pub difficulty: f64,
    pub elapsed_days: i32,
    pub scheduled_days: i32,
    pub reps: i32,
    pub lapses: i32,
    pub state: CardState,
}

impl Default for Card {
    fn default() -> Self {
        Self {
            stability: 0.0,
            difficulty: 0.0,
            elapsed_days: 0,
            scheduled_days: 0,
            reps: 0,
            lapses: 0,
            state: CardState::New,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewLog {
    pub rating: Rating,
    pub elapsed_days: i32,
    pub scheduled_days: i32,
    pub review_duration_secs: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingInfo {
    pub card: Card,
    pub review_log: ReviewLog,
}
