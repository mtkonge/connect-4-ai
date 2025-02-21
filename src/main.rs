use bot::{Bot, BotTrainer, GladiatorBotTrainer, MinMaxBotTrainer};
use interactive::InteractiveGame;

mod board;
mod bot;
mod interactive;

fn player_vs_trained_bot() {
    let mut red = Bot::new(50, 0x80085);
    let mut yellow = Bot::new(50, 0x58008);
    let iterations = 1_000_000;

    let trainer = BotTrainer::new(&mut red, &mut yellow);
    trainer.start_with_iterations(iterations);
    let game = InteractiveGame::new();
    red.exploration = 5;
    game.start_against_bot(red);
}

fn player_vs_gladiator() {
    let iterations = 1_000;

    let trainer = GladiatorBotTrainer::new(1000);
    let bot = trainer.the_one_bot_to_rule_them_all(iterations);
    let game = InteractiveGame::new();
    game.start_against_bot(bot);
}

fn player_vs_trained_minmax_bot() {
    let mut red = Bot::new(50, 0x80085);
    let iterations = 10_000;

    let trainer = MinMaxBotTrainer::new(&mut red);
    trainer.start_with_iterations(iterations);
    let game = InteractiveGame::new();
    red.exploration = 5;
    game.start_against_bot(red);
}

fn player_vs_minmax_bot() {
    let game = InteractiveGame::new();
    game.start_against_minmax();
}

fn main() {
    player_vs_gladiator()
}
