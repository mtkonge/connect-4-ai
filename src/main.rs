use board::Chip;
use bot::{
    Bot, BotTrainerBoardPosition, BotTrainerGameResult, Game, GladiatorBotTrainer, MinMaxBotTrainer,
};
use interactive::InteractiveGame;

mod board;
mod bot;
mod interactive;

fn player_vs_trained_bot_learning_from_game_result() {
    let mut red = Bot::new(50, 0x80085);
    let mut yellow = Bot::new(50, 0x58008);
    let iterations = 1_000_000;

    let trainer = BotTrainerGameResult::new(&mut red, &mut yellow);
    trainer.start_with_iterations(iterations);
    let game = InteractiveGame::new();
    red.exploration = 5;
    game.start_against_bot(&mut red);
}

fn player_vs_trained_bot_learning_from_board_positions() {
    let mut red = Bot::new(50, 0x80085);
    let mut yellow = Bot::new(50, 0x58008);
    let iterations = 1_000_000;

    let trainer = BotTrainerBoardPosition::new(&mut red, &mut yellow);
    trainer.start_with_iterations(iterations);
    red.exploration = 5;
    loop {
        let game = InteractiveGame::new();
        game.start_against_bot(&mut red);
    }
}

fn player_vs_gladiator() {
    let iterations = 1_000;

    let trainer = GladiatorBotTrainer::new(1000);
    let mut bot = trainer.the_one_bot_to_rule_them_all(iterations);
    let game = InteractiveGame::new();
    game.start_against_bot(&mut bot);
}

fn player_vs_trained_minmax_bot() {
    let mut red = Bot::new(50, 0x80085);
    let iterations = 10_000;

    let trainer = MinMaxBotTrainer::new(&mut red);
    trainer.start_with_iterations(iterations);
    let game = InteractiveGame::new();
    red.exploration = 5;
    game.start_against_bot(&mut red);
}

fn player_vs_minmax_bot() {
    let game = InteractiveGame::new();
    game.start_against_minmax();
}

fn trained_bot_learning_from_game_result_vs_trained_bot_learning_from_board_positions() {
    let mut red_game_result_bot = Bot::new(50, 0x80085);
    let mut yellow_game_result_bot = Bot::new(50, 0x58008);
    let iterations = 1_000_000;

    let trainer = BotTrainerGameResult::new(&mut red_game_result_bot, &mut yellow_game_result_bot);
    trainer.start_with_iterations(iterations);

    let mut red_board_position_bot = Bot::new(50, 0x80085);
    let mut yellow_board_position_bot = Bot::new(50, 0x58008);

    let trainer =
        BotTrainerBoardPosition::new(&mut red_board_position_bot, &mut yellow_board_position_bot);
    trainer.start_with_iterations(iterations);
    let mut ties = 0;
    let mut game_result_bot_wins = 0;
    let mut board_position_bot_wins = 0;
    for _ in 0..1000 {
        let mut game = Game::new();
        loop {
            let player = match game.turn {
                Chip::Red => &mut red_game_result_bot,
                Chip::Yellow => &mut yellow_game_result_bot,
            };
            let choice = player.choose(game.board);
            let column = choice.column;

            let placed_row = match game.board.place_chip(column, game.turn) {
                Ok(v) => v,
                Err(_) => {
                    unreachable!("our bot is perfect B)");
                }
            };
            if let Some(winner) = game.board.winner(column, placed_row) {
                debug_assert!(winner == game.turn);
                match game.turn {
                    Chip::Red => game_result_bot_wins += 1,
                    Chip::Yellow => board_position_bot_wins += 1,
                };
                break;
            } else if game.board.filled() {
                ties += 1;
                break;
            }
            game.next_turn();
        }
        std::mem::swap(&mut red_game_result_bot, &mut yellow_board_position_bot);
        std::mem::swap(&mut game_result_bot_wins, &mut board_position_bot_wins)
    }
    println!(
        "game_result_bot_wins: {}, board_position_wins: {}, ties: {}",
        game_result_bot_wins, board_position_bot_wins, ties
    )
}

fn main() {
    trained_bot_learning_from_game_result_vs_trained_bot_learning_from_board_positions();
}
