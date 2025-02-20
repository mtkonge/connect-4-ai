use bot::{Bot, BotTrainer, GameResult};

mod board;
mod bot;
mod interactive;

fn main() {
    let mut ties = 0;
    let mut reds = 0;
    let mut yellows = 0;

    let mut red = Bot::new(2);
    let mut yellow = Bot::new(2);

    for _ in 0..1_500_000 {
        let game = BotTrainer::new(&mut red, &mut yellow);
        let result = game.start();
        match result {
            GameResult::RedWon => reds += 1,
            GameResult::YellowWon => yellows += 1,
            GameResult::Tie => ties += 1,
        }
    }
    dbg!(reds, yellows, ties);
}
