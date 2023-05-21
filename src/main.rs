mod lib;

use lib::Game;

fn main() {
    let mut game = Game::new();
    game.start();
}