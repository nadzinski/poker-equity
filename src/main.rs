mod cards;
use cards::Card;
mod equity;
use equity::EquityResult;
mod game;
use game::GameSpec;
mod hands;

fn main() {
    let mut board = Vec::new();
    board.push(Card::from_str("Qs"));
    board.push(Card::from_str("Kd"));
    board.push(Card::from_str("Jc"));
    board.push(Card::from_str("Tc"));
    let mut hole_cards = Vec::new();
    hole_cards.push((Card::from_str("Qh"), Card::from_str("Qd")));
    hole_cards.push((Card::from_str("Ac"), Card::from_str("As")));
    let game_spec = GameSpec { board, hole_cards };

    let results = equity::simulate_equity_from_game_spec(game_spec, Some(100000));
    for (
        player,
        EquityResult {
            equity,
            win_percentage,
            draw_percentage,
        },
    ) in results.iter().enumerate()
    {
        println!(
            "Equity for player {}: {} ({}% hands were wins, {}% draws)",
            player, equity, win_percentage, draw_percentage
        );
    }
}
