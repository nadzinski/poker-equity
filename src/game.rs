use crate::cards::Card;
use crate::hands::Hand;
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashSet;
use std::iter::FromIterator;

// A GameSpec represents incomplete information about a game situation
// which can be used to construct a Game by randomly filling in the
// unknown cards.
pub struct GameSpec {
    pub board: Vec<Card>,
    pub hole_cards: Vec<(Card, Card)>,
}

pub struct Game {
    deck: Vec<Card>,
    board: Vec<Card>,
    hole_cards: Vec<(Card, Card)>,
}

impl Game {
    pub fn from_spec(spec: &GameSpec) -> Game {
        let mut cards_set: HashSet<Card> = HashSet::from_iter(Card::create_deck());
        let mut board = Vec::new();
        let mut hole_cards = Vec::new();

        // Set up board
        for spec_card in &spec.board {
            let card = cards_set.take(&spec_card).unwrap();
            board.push(card);
        }

        // Set up hole cards
        for (spec_card_1, spec_card_2) in &spec.hole_cards {
            let card_1 = cards_set.take(&spec_card_1).unwrap();
            let card_2 = cards_set.take(&spec_card_2).unwrap();
            hole_cards.push((card_1, card_2));
        }

        // Put remaining cards into deck and shuffle
        let mut deck: Vec<Card> = cards_set.into_iter().collect();
        deck.shuffle(&mut thread_rng());

        Game {
            deck,
            board,
            hole_cards,
        }
    }

    pub fn deal_down_to_river(&mut self) {
        let cards_to_deal = 5 - self.board.len();
        for _ in 0..cards_to_deal {
            self.board.push(self.deck.pop().unwrap());
        }
    }

    pub fn get_player_hands(&self) -> Vec<Hand> {
        (0..self.hole_cards.len())
            .map(|player| self.get_scoring_hand_for_player(player))
            .collect()
    }

    pub fn get_winning_players_and_hands(&self) -> Vec<(usize, Hand)> {
        let player_hands = self.get_player_hands();
        let best_hand = player_hands.iter().max().unwrap().clone();
        let winning_players_and_hands: Vec<(usize, Hand)> = player_hands
            .into_iter()
            .enumerate()
            .filter(|(_, hand)| *hand == best_hand)
            .collect();
        winning_players_and_hands
    }

    fn get_scoring_hand_for_player(&self, player: usize) -> Hand {
        // Out of the 7 cards that can be used in a hand (2 hole cards plus 5 board cards), get
        // every possible 5 card hand combination, then find the highest-scoring hand.
        let mut all_cards: Vec<&Card> = Vec::new();
        all_cards.push(&self.hole_cards[player].0);
        all_cards.push(&self.hole_cards[player].1);
        all_cards.append(&mut self.board.iter().collect());

        let scoring_hand = all_cards
            .into_iter()
            .combinations(5)
            .map(|cards| Hand::new(cards))
            .max()
            .unwrap();
        scoring_hand
    }
}
