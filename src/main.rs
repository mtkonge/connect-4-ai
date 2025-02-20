use std::time::Instant;

use bot::{Bot, BotTrainer, GameResult};
use interactive::InteractiveGame;

mod board;
mod bot;
mod interactive;

fn main() {
    let mut ties = 0;
    let mut red_wins = 0;
    let mut yellow_wins = 0;

    let mut red = Bot::new(50);
    let mut yellow = Bot::new(50);
    let iterations = 10_000_000;
    let mut percentage = 0;
    let print_per_percentage = 10;
    let mut last_iteration_started_at = Instant::now();
    let mut total_seconds = 0;

    for iteration in 1..=iterations {
        if iteration % (iterations / print_per_percentage) == 0 {
            percentage += print_per_percentage;
            let time_when_reached = Instant::now() - last_iteration_started_at;
            last_iteration_started_at = Instant::now();
            let seconds = time_when_reached.as_secs();
            total_seconds += seconds;
            println!("reached {iteration} iterations ({percentage}%) after {seconds} seconds ({total_seconds} seconds total)",);
        }
        let trainer = BotTrainer::new(&mut red, &mut yellow);
        let result = trainer.start();
        match result {
            GameResult::RedWon => red_wins += 1,
            GameResult::YellowWon => yellow_wins += 1,
            GameResult::Tie => ties += 1,
        }
        std::mem::swap(&mut red, &mut yellow);
        std::mem::swap(&mut red_wins, &mut yellow_wins);
    }
    println!("Trained bot! Red won {red_wins} times, yellow won {yellow_wins} times, and the bots tied {ties} times.");
    dbg!(red_wins, yellow_wins, ties);
    let game = InteractiveGame::new();
    red.exploration = 5;
    game.start_against_bot(red);
}
