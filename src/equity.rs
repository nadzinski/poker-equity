use crate::game::Game;
use crate::game::GameSpec;

const DEFAULT_NUM_SIMULATIONS: u64 = 100000;

pub struct EquityResult {
    pub equity: f64,
    pub win_percentage: f64,
    pub draw_percentage: f64,
}

pub fn simulate_equity_from_game_spec(
    game_spec: GameSpec,
    num_simulations: Option<u64>,
) -> Vec<EquityResult> {
    let num_simulations = num_simulations.unwrap_or(DEFAULT_NUM_SIMULATIONS);
    let num_players = game_spec.hole_cards.len();
    let mut player_win_counts: Vec<u64> = vec![0; num_players];
    let mut player_draw_counts: Vec<u64> = vec![0; num_players];
    let mut player_equity_sums: Vec<f64> = vec![0.; num_players];

    for _ in 0..num_simulations {
        let mut game = Game::from_spec(&game_spec);
        game.deal_down_to_river();
        let winning_players_and_hands = game.get_winning_players_and_hands();
        let winners = winning_players_and_hands.len();
        for (player, _) in winning_players_and_hands {
            player_equity_sums[player] += 1. / winners as f64;
            if winners > 1 {
                player_draw_counts[player] += 1;
            } else {
                player_win_counts[player] += 1;
            }
        }
    }

    (0..num_players)
        .map(|p| EquityResult {
            equity: player_equity_sums[p] / num_simulations as f64,
            win_percentage: 100. * player_win_counts[p] as f64 / num_simulations as f64,
            draw_percentage: 100. * player_draw_counts[p] as f64 / num_simulations as f64,
        })
        .collect()
}
