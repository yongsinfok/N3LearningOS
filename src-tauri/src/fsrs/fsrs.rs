use super::types::{Card, CardState, FSRSParameters, Rating, ReviewLog, SchedulingInfo};

pub fn fsrs_review(
    card: &Card,
    rating: Rating,
    params: &FSRSParameters,
) -> SchedulingInfo {
    let w = &params.w;
    let mut new_card = card.clone();

    if card.state == CardState::New {
        new_card.stability = w[0];
        new_card.difficulty = w[2];

        match rating {
            Rating::Again => {
                new_card.scheduled_days = 0;
                new_card.state = CardState::Learning;
            }
            Rating::Hard => {
                new_card.scheduled_days = 1;
                new_card.state = CardState::Learning;
            }
            Rating::Good => {
                new_card.scheduled_days = 1;
                new_card.state = CardState::Learning;
            }
            Rating::Easy => {
                new_card.scheduled_days = 4;
                new_card.state = CardState::Review;
            }
        }
    } else {
        let mut diff = card.difficulty;
        new_card.elapsed_days = card.scheduled_days;

        diff = diff - w[4] * (rating as i32 - 2) as f64;
        diff = constrain_difficulty(diff);
        new_card.difficulty = w[7] * w[2] + (1.0 - w[7]) * diff;

        let stab = card.stability.max(0.01);
        new_card.stability = match rating {
            Rating::Again => w[0],
            Rating::Hard => w[0] * stab.powf(w[1]) * w[5].powf(new_card.difficulty),
            Rating::Good => stab * (1.0 + w[2] * w[6].powf(new_card.difficulty) * stab.powf(w[3])),
            Rating::Easy => {
                stab * w[8] * w[6].powf(new_card.difficulty) * stab.powf(w[3])
            }
        };

        new_card.scheduled_days = match rating {
            Rating::Again => 0,
            Rating::Hard => (new_card.stability * 1.2).round() as i32,
            Rating::Good => new_card.stability.round() as i32,
            Rating::Easy => (new_card.stability * 1.3).round() as i32,
        }.max(1);

        match rating {
            Rating::Again => {
                new_card.lapses += 1;
                if card.state == CardState::Review {
                    new_card.state = CardState::Relearning;
                } else {
                    new_card.state = CardState::Learning;
                }
            }
            _ => {
                if card.state == CardState::Learning || card.state == CardState::Relearning {
                    new_card.state = CardState::Review;
                }
            }
        }
    }

    new_card.reps += 1;

    let review_log = ReviewLog {
        rating,
        elapsed_days: card.elapsed_days,
        scheduled_days: new_card.scheduled_days,
        review_duration_secs: 0,
    };

    SchedulingInfo {
        card: new_card,
        review_log,
    }
}

fn constrain_difficulty(d: f64) -> f64 {
    d.clamp(0.0, 10.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_card_first_review_good() {
        let card = Card::default();
        let params = FSRSParameters::default();
        let result = fsrs_review(&card, Rating::Good, &params);
        assert_eq!(result.card.state, CardState::Learning);
        assert_eq!(result.card.scheduled_days, 1);
        assert!(result.card.stability > 0.0);
        assert!(result.card.difficulty > 0.0);
    }

    #[test]
    fn test_new_card_review_easy() {
        let card = Card::default();
        let params = FSRSParameters::default();
        let result = fsrs_review(&card, Rating::Easy, &params);
        assert_eq!(result.card.state, CardState::Review);
        assert_eq!(result.card.scheduled_days, 4);
    }

    #[test]
    fn test_review_card_again_resets_stability() {
        let mut card = Card::default();
        card.state = CardState::Review;
        card.stability = 10.0;
        card.difficulty = 5.0;
        card.scheduled_days = 14;
        let params = FSRSParameters::default();
        let result = fsrs_review(&card, Rating::Again, &params);
        assert_eq!(result.card.state, CardState::Relearning);
        assert_eq!(result.card.stability, params.w[0]);
    }

    #[test]
    fn test_review_card_good_increases_interval() {
        let mut card = Card::default();
        card.state = CardState::Review;
        card.stability = 10.0;
        card.difficulty = 5.0;
        card.scheduled_days = 10;
        let params = FSRSParameters::default();
        let result = fsrs_review(&card, Rating::Good, &params);
        assert!(result.card.scheduled_days > 10);
        assert!(result.card.stability > 10.0);
    }

    #[test]
    fn test_difficulty_decreases_on_easy() {
        let mut card = Card::default();
        card.state = CardState::Review;
        card.difficulty = 5.0;
        card.stability = 10.0;
        let params = FSRSParameters::default();
        let result = fsrs_review(&card, Rating::Easy, &params);
        assert!(result.card.difficulty < 5.0);
    }
}
