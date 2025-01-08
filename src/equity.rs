use crate::game::Game;
use crate::game::GameSpec;

const DEFAULT_NUM_SIMULATIONS: u64 = 100000;
const NUM_THREADS: usize = 16;

pub struct EquityResult {
    pub equity: f64,
    pub win_percentage: f64,
    pub draw_percentage: f64,
}

pub fn simulate_equity_from_game_spec(
    game_spec: GameSpec,
    num_simulations: Option<u64>,
) -> Vec<EquityResult> {
    let num_players = game_spec.hole_cards.len();
    let num_simulations = num_simulations.unwrap_or(DEFAULT_NUM_SIMULATIONS);
    let num_simulations_per_thread = (num_simulations as u32).div_ceil(NUM_THREADS as u32);

    let results = std::thread::scope(|scope| {
        let mut thread_scopes: Vec<std::thread::ScopedJoinHandle<Vec<EquityResult>>> = Vec::new();
        for _ in 0..NUM_THREADS {
            let join_handler = scope.spawn(|| {
                threaded_simulate_equity_from_game_spec(
                    &game_spec,
                    num_simulations_per_thread as u64,
                )
            });
            thread_scopes.push(join_handler);
        }
        let mut results: Vec<Vec<EquityResult>> = Vec::new();
        for join_handler in thread_scopes {
            let result = join_handler.join().unwrap();
            results.push(result);
        }
        results
    });

    let mut total_results: Vec<EquityResult> = Vec::new();
    for player in 0..num_players {
        let mut sum_equity: f64 = 0.;
        let mut sum_win_percentage: f64 = 0.;
        let mut sum_draw_percentage: f64 = 0.;
        for thread in 0..NUM_THREADS {
            sum_equity += results[thread][player].equity;
            sum_win_percentage += results[thread][player].win_percentage;
            sum_draw_percentage += results[thread][player].draw_percentage;
        }

        let total_result = EquityResult {
            equity: sum_equity / NUM_THREADS as f64,
            win_percentage: sum_win_percentage / NUM_THREADS as f64,
            draw_percentage: sum_draw_percentage / NUM_THREADS as f64,
        };
        total_results.push(total_result);
    }
    total_results
}

fn threaded_simulate_equity_from_game_spec(
    game_spec: &GameSpec,
    num_simulations: u64,
) -> Vec<EquityResult> {
    let num_players = game_spec.hole_cards.len();
    let mut player_win_counts: Vec<u64> = vec![0; num_players];
    let mut player_draw_counts: Vec<u64> = vec![0; num_players];
    let mut player_equity_sums: Vec<f64> = vec![0.; num_players];

    for _ in 0..num_simulations {
        let mut game = Game::from_spec(game_spec);
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
