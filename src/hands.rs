use crate::cards::Card;
use itertools::Itertools;
use std::cmp::Ordering;

// Logic for types of hands and their relative value

// need to write a whole bunch of test cases, lots of edge tests
// (e.g. multi-way straights, etc.)

// We want HandType to be a thing that can be copied, not moved
// i.e. ht1 = ht2 is a copy and means that both ht1 and ht2 remain valid.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HandType {
    StraightFlush,
    Quads,
    FullHouse,
    Flush,
    Straight,
    ThreeOfAKind,
    TwoPair,
    Pair,
    HighCard,
}

// Since `Hand` contains a vector of references to cards, it needs a lifetime specifier.
// This says: for a `Hand` with an associated lifetime 'a, we guarantee that the associated
// lifetimes of the Card references will each live at least as long as the Hand lifetime 'a.
// That is to say, the references to the Cards must refer to valid existing things
// for as long as the Hand exists.
#[derive(Clone, Eq)]
pub struct Hand<'a> {
    pub cards: Vec<&'a Card>,
    pub hand_type: HandType,
    level: u8,
    score: u64,
}

impl<'a> Ord for Hand<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.level, self.score).cmp(&(other.level, other.score))
    }
}

impl<'a> PartialOrd for Hand<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> PartialEq for Hand<'a> {
    fn eq(&self, other: &Self) -> bool {
        (self.level, self.score).eq(&(other.level, other.score))
    }
}

impl<'a> Hand<'a> {
    pub fn new(cards: Vec<&Card>) -> Hand {
        let (hand_type, level, score) = get_hand_type_level_and_score(&cards);
        Hand {
            cards,
            hand_type,
            level,
            score,
        }
    }

    // todo: do printing better, impl Display instead
    #[allow(dead_code)]
    pub fn print_hand(&self) {
        // Q - why do we need the & on &self below?
        for card in &self.cards {
            print!("{} ", card);
        }
        println!("");
        println!(
            "Hand type: {:?}, level: {}, score: {}",
            &self.hand_type, &self.level, &self.score
        );
    }
}

/// This function computes a (level, score) pair for a 5-card hand. Pairs for any two hands can be
/// compared to determine the stronger hand (or if the hands are a draw). The "level" represents
/// how strong the "Hand Type" is, e.g. Flush = 6 is stronger than Straight = 5. The "score" is a
/// number that depends on the Hand Type and can be compared within hands of the same type to
/// determine the stronger hand.
#[rustfmt::skip]
fn get_hand_type_level_and_score(cards: &Vec<&Card>) -> (HandType, u8, u64) {
    let all_same_suit = all_same_suit(cards);
    let straight_score = straight_score(cards);
    let (groupings, groupings_score) = get_groupings_and_score(cards);

    match (groupings.as_slice(), all_same_suit, straight_score) {
        ([1, 1, 1, 1, 1], true,  Some(ss))  => (HandType::StraightFlush, 9, ss),
        ([4, 1],          false, None)      => (HandType::Quads, 8, groupings_score),
        ([3, 2],          false, None)      => (HandType::FullHouse, 7, groupings_score),
        ([1, 1, 1, 1, 1], true,  None)      => (HandType::Flush, 6, groupings_score),
        ([1, 1, 1, 1, 1], false, Some(ss))  => (HandType::Straight, 5, ss),
        ([3, 1, 1],       false, None)      => (HandType::ThreeOfAKind, 4, groupings_score),
        ([2, 2, 1],       false, None)      => (HandType::TwoPair, 3, groupings_score),
        ([2, 1, 1, 1],    false, None)      => (HandType::Pair, 2, groupings_score),
        ([1, 1, 1, 1, 1], false, None)      => (HandType::HighCard, 1, groupings_score),
        _                                   => panic!("No valid hand type for hand!"),
    }
}

fn get_groupings_and_score(cards: &Vec<&Card>) -> (Vec<u8>, u64) {
    // generalized rank-grouping and scoring function for non-straight hands
    // sort the cards
    // group by (requires previous sort)
    // re-sort by (group size, rank)
    // unzip grouping sizes and ranks
    // score by BASE * grouping1_rank + BASE^2 * grouping2_rank + ...
    const BASE: u64 = 16;

    let mut ordered_cards = cards.to_vec();
    ordered_cards.sort_by_key(|card| card.rank);

    let mut group_sizes_and_ranks: Vec<(u8, u8)> = ordered_cards
        .iter()
        .chunk_by(|card| card.rank)
        .into_iter()
        .map(|(rank, chunk)| (chunk.count() as u8, rank))
        .collect();

    group_sizes_and_ranks.sort();
    let (mut group_sizes, ranks): (Vec<u8>, Vec<u8>) = group_sizes_and_ranks.into_iter().unzip();

    group_sizes.reverse();

    let score = ranks
        .into_iter()
        .enumerate()
        .map(|(n, rank)| rank as u64 * BASE.pow(n as u32))
        .sum();

    (group_sizes, score)
}

fn all_same_suit(cards: &Vec<&Card>) -> bool {
    let first_suit = cards[0].suit.clone();
    cards.iter().all(|card| card.suit == first_suit)
}

fn straight_score(cards: &Vec<&Card>) -> Option<u64> {
    // special scoring function for straights or straight flushes
    // the score is the rank of the low card of the straight
    let mut ordered_ranks: Vec<u8> = cards.iter().map(|card| card.rank).collect();
    ordered_ranks.sort();

    // ace-2-3-4-5 "the wheel" special case
    if ordered_ranks.as_slice() == [2, 3, 4, 5, 14] {
        return Some(1);
    }

    let lowest = ordered_ranks[0];
    let desired_straight: Vec<u8> = (0..5).map(|i| lowest + i).collect();
    if ordered_ranks == desired_straight {
        return Some(lowest as u64);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cards(cards_str: &str) -> Vec<Card> {
        cards_str
            .split_whitespace()
            .map(|card_str| Card::from_str(card_str))
            .collect()
    }

    fn make_hand<'a>(cards: &'a Vec<Card>) -> Hand<'a> {
        Hand::new(cards.iter().collect())
    }

    #[test]
    fn test_hand_type_straight_flush() {
        let cards = &make_cards("Jh Th Ah Kh Qh");
        let hand = make_hand(cards);
        assert_eq!(hand.hand_type, HandType::StraightFlush);
    }

    #[test]
    fn test_hand_type_straight_flush_low() {
        let cards = &make_cards("5d 2d Ad 3d 4d");
        let hand = make_hand(cards);
        assert_eq!(hand.hand_type, HandType::StraightFlush);
    }

    #[test]
    fn test_hand_type_quads() {
        let cards = &make_cards("Jh Jd Js Jc 7d");
        let hand = make_hand(cards);
        assert_eq!(hand.hand_type, HandType::Quads);
    }

    #[test]
    fn test_hand_type_full_house() {
        let cards = &make_cards("3h 2s 3c 2c 3d");
        let hand = make_hand(cards);
        assert_eq!(hand.hand_type, HandType::FullHouse);
    }

    #[test]
    fn test_hand_type_flush() {
        let cards = &make_cards("3h 2h 4h 5h 7h");
        let hand = make_hand(cards);
        assert_eq!(hand.hand_type, HandType::Flush);
    }

    #[test]
    fn test_hand_type_straight() {
        let cards = &make_cards("8d 7h 9d 6s Th");
        let hand = make_hand(cards);
        assert_eq!(hand.hand_type, HandType::Straight);
    }

    #[test]
    fn test_hand_type_straight_low() {
        let cards = &make_cards("5d 2s Ah 3d 4c");
        let hand = make_hand(cards);
        assert_eq!(hand.hand_type, HandType::Straight);
    }

    #[test]
    fn test_hand_type_three_of_a_kind() {
        let cards = &make_cards("2s 2s 3h Qs 2c");
        let hand = make_hand(cards);
        assert_eq!(hand.hand_type, HandType::ThreeOfAKind);
    }

    #[test]
    fn test_hand_type_two_pair() {
        let cards = &make_cards("As 3c Ah Kd 3h");
        let hand = make_hand(cards);
        assert_eq!(hand.hand_type, HandType::TwoPair);
    }

    #[test]
    fn test_hand_type_pair() {
        let cards = &make_cards("As Kc 7s Kd 9d");
        let hand = make_hand(cards);
        assert_eq!(hand.hand_type, HandType::Pair);
    }

    #[test]
    fn test_hand_type_high_card() {
        let cards = &make_cards("2s Qh 7c Kd 8d");
        let hand = make_hand(cards);
        assert_eq!(hand.hand_type, HandType::HighCard);
    }

    #[test]
    fn test_straight_flush_vs_straight_flush() {
        let sf_1 = &make_cards("9d Td Jd Kd Qd");
        let sf_2 = &make_cards("8s 9s Ts Js Qs");
        assert!(make_hand(sf_1) > make_hand(sf_2));
    }

    #[test]
    fn test_straight_flush_vs_low_straight_flush() {
        let sf_1 = &make_cards("9d Td Jd Kd Qd");
        let sf_2 = &make_cards("4s 5s 3s 2s As");
        assert!(make_hand(sf_1) > make_hand(sf_2));
    }

    #[test]
    fn test_straight_flush_vs_straight_flush_equal() {
        let sf_1 = &make_cards("2d 3d 4d 5d 6d");
        let sf_2 = &make_cards("6s 5s 4s 3s 2s");
        assert!(make_hand(sf_1) == make_hand(sf_2));
    }

    #[test]
    fn test_straight_flush_vs_quads() {
        let sf = &make_cards("9d Td Jd Kd Qd");
        let q = &make_cards("Td Th Ts Tc Qd");
        assert!(make_hand(sf) > make_hand(q));
    }

    #[test]
    fn test_quads_vs_quads() {
        let q_1 = &make_cards("Jd Jh Ks Jc Jd");
        let q_2 = &make_cards("Td Th Ts Tc Qd");
        assert!(make_hand(q_1) > make_hand(q_2));
    }

    #[test]
    fn test_quads_vs_quads_kicker() {
        let q_1 = &make_cards("Jd Jh Ks Jc Jd");
        let q_2 = &make_cards("Jd Jh Qc Jc Jd");
        assert!(make_hand(q_1) > make_hand(q_2));
    }

    #[test]
    fn test_quads_vs_quads_equal() {
        let q_1 = &make_cards("Jd Jh Ks Jc Jd");
        let q_2 = &make_cards("Jd Jh Kc Jc Jd");
        assert!(make_hand(q_1) == make_hand(q_2));
    }

    // ... todo: more tests

    #[test]
    fn test_flush_versus_flush_lower_cards() {
        let fl_1 = &make_cards("Ks Js Ts 7s 5s");
        let fl_2 = &make_cards("Ks Js Ts 7s 4s");
        assert!(make_hand(fl_1) > make_hand(fl_2));
    }

    // ... todo: more tests

    #[test]
    fn test_straight_versus_wheel_straight() {
        let q_1 = &make_cards("2c 3h 4d 5c 6d");
        let q_2 = &make_cards("Ac 2c 3h 4d 5c");
        assert!(make_hand(q_1) > make_hand(q_2));
    }

    // ... todo: more tests

    #[test]
    fn test_pair_vs_pair() {
        let p_1 = &make_cards("7s Js 6h 7d Qh");
        let p_2 = &make_cards("6c Js 6h 7d Ah");
        assert!(make_hand(p_1) > make_hand(p_2));
    }

    #[test]
    fn test_pair_vs_pair_kicker() {
        let p_1 = &make_cards("6s Js Th 6d 4h");
        let p_2 = &make_cards("6c Js Th 6d 2h");
        assert!(make_hand(p_1) > make_hand(p_2));
    }

    #[test]
    fn test_pair_vs_high_card() {
        let p = &make_cards("2c Js Ks 6d 2h");
        let hc = &make_cards("Ac Jc Ks 6d 4h");
        assert!(make_hand(p) > make_hand(hc));
    }

    #[test]
    fn test_high_card_vs_high_card() {
        let hc_1 = &make_cards("Ac Jc Ks 6d 4h");
        let hc_2 = &make_cards("Ad Js Ks 6d 2h");
        assert!(make_hand(hc_1) > make_hand(hc_2));
    }
}
