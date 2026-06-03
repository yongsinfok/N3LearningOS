use super::types::{Card, CardState, Rating, ReviewLog, SchedulingInfo};

pub fn sm2_review(card: &Card, rating: Rating) -> SchedulingInfo {
    let mut new_card = card.clone();
    let quality = rating_to_quality(rating);

    new_card.reps += 1;
    new_card.elapsed_days = new_card.scheduled_days;

    if rating == Rating::Again {
        new_card.lapses += 1;
        new_card.scheduled_days = 1;
        if card.state != CardState::New {
            new_card.state = CardState::Relearning;
        }
    } else {
        if card.state == CardState::New || card.state == CardState::Learning {
            new_card.state = CardState::Learning;
        }
        match new_card.reps {
            1 => new_card.scheduled_days = 1,
            2 => new_card.scheduled_days = 6,
            _ => {
                let interval = (card.scheduled_days as f64 * ease_factor(quality)).round() as i32;
                new_card.scheduled_days = interval.max(1);
            }
        }
    }

    if card.state == CardState::New && rating != Rating::Again {
        new_card.state = CardState::Learning;
    }
    if new_card.state == CardState::Learning && new_card.reps >= 2 {
        new_card.state = CardState::Review;
    }

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

fn rating_to_quality(rating: Rating) -> f64 {
    match rating {
        Rating::Again => 1.0,
        Rating::Hard => 2.0,
        Rating::Good => 3.0,
        Rating::Easy => 4.0,
    }
}

fn ease_factor(quality: f64) -> f64 {
    let ef = 2.5 + (0.1 - (5.0 - quality) * (0.08 + (5.0 - quality) * 0.02));
    ef.clamp(1.3, 2.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_card_first_review_good() {
        let card = Card::default();
        let result = sm2_review(&card, Rating::Good);
        assert_eq!(result.card.state, CardState::Learning);
        assert_eq!(result.card.scheduled_days, 1);
        assert_eq!(result.card.reps, 1);
    }

    #[test]
    fn test_again_resets_interval() {
        let mut card = Card::default();
        card.state = CardState::Review;
        card.scheduled_days = 14;
        card.reps = 5;
        let result = sm2_review(&card, Rating::Again);
        assert_eq!(result.card.state, CardState::Relearning);
        assert_eq!(result.card.scheduled_days, 1);
        assert_eq!(result.card.lapses, 1);
    }

    #[test]
    fn test_spaced_review_intervals_grow() {
        let mut card = Card::default();
        card.state = CardState::Review;
        card.scheduled_days = 10;
        card.reps = 3;
        let result = sm2_review(&card, Rating::Good);
        assert!(result.card.scheduled_days > 10);
    }
}
