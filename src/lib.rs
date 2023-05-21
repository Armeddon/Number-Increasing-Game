const BOARD_WIDTH: usize = 6;
const BOARD_HEIGHT: usize = 8;
const DISPLAY_GAP: u8 = 6;

const MAX_NUMBER_VALUE: u8 = 5;
const WIN_SCORE: u8 = 10;

#[derive(Copy, Clone)]
enum Player {
    FirstPlayer,
    SecondPlayer,
}

impl Player {
    fn other(&self) -> Player {
        match self {
            Player::FirstPlayer => Player::SecondPlayer,
            Player::SecondPlayer => Player::FirstPlayer,
        }
    }
    fn same(&self) -> Player {
        match self {
            Player::FirstPlayer => Player::FirstPlayer,
            Player::SecondPlayer => Player::SecondPlayer,
        }
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Player) -> bool {
        let fst =
            match self {
                Player::FirstPlayer => true,
                Player::SecondPlayer => false,
            };
        let snd =
            match other {
                Player::FirstPlayer => true,
                Player::SecondPlayer => false,
            };
        fst == snd
    }
}

impl Eq for Player {}

#[derive(Copy, Clone)]
struct Point {
    x: u8,
    y: u8,
}

#[derive(Copy, Clone)]
enum Action {
    Up,
    Right,
    Down,
    Left,
    Spawn,
    Square,
    Increase,
}

#[derive(Copy, Clone)]
struct Turn {
    point: Point,
    action: Action,
}
#[derive(Copy, Clone)]
struct Number {
    player: Player,
    value: u8,
}

#[derive(Copy, Clone)]
struct Tile {
    number: Option<Number>,
    square: Option<Player>,
}

#[derive(Copy, Clone)]
struct Board {
    tiles: [[Tile;BOARD_WIDTH];BOARD_HEIGHT],
}

impl Board {
    fn new() -> Self {
        let tile = Tile {
            number: None,
            square: None,
        };
        let special_tile = Tile {
            number: None,
            square: Some(Player::SecondPlayer),
        };
        let mut tiles = [[tile.clone();BOARD_WIDTH];BOARD_HEIGHT];
        tiles[2][BOARD_WIDTH/2] = special_tile;

        Board {
            tiles: tiles.clone(),
        }
    }
    fn draw(&self) {
        let mut numbers = [[None;BOARD_WIDTH];BOARD_HEIGHT];
        let mut squares = [[None;BOARD_WIDTH];BOARD_HEIGHT];
        for (i, line) in self.tiles.iter().enumerate() {
            for (j, tile) in line.iter().enumerate() {
                if let Some(Number {player, value}) = tile.number {
                    numbers[i][j] = Some(Number {player, value});
                }
                if let Some(square_owner) = tile.square {
                    squares[i][j] = Some(square_owner);
                }
            }
        }
        print!("  ");
        for i in 1..=BOARD_WIDTH {
            print!("{i} ");
        }
        for _ in 0..(DISPLAY_GAP+2) {
            print!(" ");
        }
        for i in 1..=BOARD_WIDTH {
            print!("{i} ");
        }
        print!("\n");
        for i in 0..BOARD_HEIGHT {
            print!("{}|",i+1);
            for j in 0..BOARD_WIDTH {
                print!("{}|", match numbers[i][j]{
                    Some(Number{player, value}) => {
                        if player == Player::FirstPlayer {
                            (0x30 + value) as char
                        } else {
                            match value {
                                0 => '\u{24EA}',
                                1 => '\u{2460}',
                                2 => '\u{2461}',
                                3 => '\u{2462}',
                                4 => '\u{2463}',
                                5 => '\u{2464}',
                                6 => '\u{2465}',
                                7 => '\u{2466}',
                                8 => '\u{2467}',
                                9 => '\u{2468}',
                                _ => ' ',
                            }
                        }

                    }
                    None => ' ',
                });
            }
            for _ in 0..DISPLAY_GAP {
                print!(" ");
            }
            print!("{}|",i+1);
            for j in 0..BOARD_WIDTH {
                print!("{}|", match squares[i][j] {
                    Some(player) => {
                        match player {
                            Player::FirstPlayer => '1',
                            Player::SecondPlayer => '2',
                        }
                    }
                    None => ' ',
                });
            }
            print!("\n");
        }
    }
}

pub struct Game {
    player: Player,
    board: Board,
    score: (u8, u8),
    squares: (u8, u8),
}

impl Game {
    pub fn new() -> Self {
        Game {
            player: Player::FirstPlayer,
            board: Board::new(),
            score: (0, 0),
            squares: (0, 1),
        }
    }
    fn draw(&self) {
        println!("Current score is {}:{}",
                 self.score.0,
                 self.score.1);
        self.board.draw();
    }
    pub fn start(&mut self) {
        loop {
            self.draw();
            turn::make_turn(self);
            if self.score.0 >= WIN_SCORE {
                println!("Player 1 wins!!!");
                break;
            }
            if self.score.1 >= WIN_SCORE {
                println!("Player 2 wins!!!");
                break;
            }
        }
    }
}

mod turn {
    use super::*;
    use std::io;
    use std::io::Read;

    pub fn make_turn(game: &mut Game) {
        loop {
            print!("(Player {}) ",
                match game.player {
                    Player::FirstPlayer => 1,
                    Player::SecondPlayer => 2,
                });
            match input_turn() {
                Ok(turn) => {
                    let x = turn.point.x as usize;
                    let y = turn.point.y as usize;
                    if x > BOARD_WIDTH || x < 1 || y > BOARD_HEIGHT || y < 1 {
                        println!("Wrong coordinate!");
                        continue;
                    }

                    if let Err(s) = match turn.action {
                        Action::Spawn => exec_spawn(turn.point, game),
                        Action::Increase => exec_increase(turn.point, game),
                        Action::Square => exec_square(turn.point, game),
                        move_action=> {
                            let pt1 = match move_action {
                                Action::Up => Point {y: turn.point.y-1, ..turn.point},
                                Action::Right => Point {x: turn.point.x+1, ..turn.point},
                                Action::Down => Point {y: turn.point.y+1, ..turn.point},
                                Action::Left => Point {x: turn.point.x-1, ..turn.point},
                                _ => Point {x:0, y:0},
                            };
                            exec_move(turn.point, pt1, game)
                        }
                    } {
                        println!("{}", s);
                        continue;
                    }
                    break;
                }
                Err(s) => println!("{}",s),
            }
        }
        game.player = game.player.other();
    }

    fn read_digit() -> Result<u8, &'static str> {
        let mut x = [0];
        if let Err(_) = io::stdin().read_exact(&mut x) {
            return Err("Error reading line!");
        }
        match (x[0] as char).to_string().as_str().parse() {
            Ok(num) => Ok(num),
            Err(_) => Err("Error parsing digit!"),
        }
    }

    fn input_turn() -> Result<Turn, &'static str>{
        println!("Enter your coordinate and command: ");

        let y = match read_digit() {
            Ok(num) => num,
            Err(e) => return Err(e),
        };
        let x = match read_digit() {
            Ok(num) => num,
            Err(e) => return Err(e),
        };

        let mut action = String::new();
        if let Err(_) = io::stdin().read_line(&mut action) {
            return Err("Error reading line!");
        }
        let action = match action.trim() {
            "" => Action::Spawn,
            "i" => Action::Increase,
            "u" => Action::Up,
            "r" => Action::Right,
            "d" => Action::Down,
            "l" => Action::Left,
            "s" => Action::Square,
            _ => return Err("Error parsing action!"),
        };

        Ok(Turn {
            point: Point {x, y},
            action,
        })
    }

    fn exec_spawn(point: Point, game: &mut Game) -> Result<(), &'static str>{
        let x = point.x as usize;
        let y = point.y as usize;

        if game.board.tiles[y-1][x-1].number.is_some() {
            return Err("Can't spawn there!");
        }
        let line = match &game.player {
            Player::FirstPlayer => 8,
            Player::SecondPlayer => 1,
        };
        if y != line {
            return Err("Can't spawn there!");
        }
        game.board.tiles[y-1][x-1].number = Some(Number {
            player: game.player.same(),
            value: 0,
        });
        Ok(())
    }

    fn exec_increase(point: Point, game: &mut Game) -> Result<(), &'static str> {
        let x = point.x as usize;
        let y = point.y as usize;

        match &game.board.tiles[y-1][x-1].number {
            Some(Number {player, value }) => {
                if *player == game.player {
                    if y < 3 || y + 3 > BOARD_HEIGHT {
                        return Err("Can't increase there!");
                    }
                    if *value >= MAX_NUMBER_VALUE {
                        return Err("Can't increase over limit!");
                    }
                    game.board.tiles[y-1][x-1].number = Some(Number {
                        player: player.same(),
                        value: value+1,
                    });
                    Ok(())
                } else {
                    Err("Can't increase that!")
                }
            }
            None => Err("Can't increase that!"),
        }
    }

    fn exec_square(point: Point, game: &mut Game) -> Result<(), &'static str> {
        let x = point.x as usize;
        let y = point.y as usize;

        match &game.board.tiles[y-1][x-1].number.clone() {
            Some(Number {player, value}) => {
                if *player != game.player {
                    return Err("Can't create square there!");
                }

                if y < 3 || y + 3 > BOARD_HEIGHT {
                    return Err("Can't create square there!");
                }

                let squares = match player {
                    Player::FirstPlayer => game.squares.0,
                    Player::SecondPlayer => game.squares.1,
                };

                if *value > squares {
                    game.board.tiles[y-1][x-1].number = None;
                    game.board.tiles[y-1][x-1].square = Some(player.same());
                    match player {
                        Player::FirstPlayer => game.squares.0 += 1,
                        Player::SecondPlayer => game.squares.1 += 1,
                    }

                    Ok(())
                } else {
                    Err("The number is too small!")
                }
            }
            None => Err("Can't create square there!"),
        }
    }

    fn exec_move(point: Point, dest: Point, game: &mut Game) -> Result<(), &'static str> {
        let x = point.x as usize;
        let y = point.y as usize;

        let x1 = dest.x as usize;
        let y1 = dest.y as usize;

        match &game.board.tiles[y-1][x-1].number.clone() {
            Some(Number {player, value}) => {
                if *player != game.player {
                    return Err("That's not your number!");
                }
                if x1 <1 || x1 >BOARD_WIDTH || y1 < 1 || y1 >BOARD_HEIGHT {
                    return Err("Can't move there!");
                }
                if let Some(number) = &game.board.tiles[y1-1][x1-1].number {
                    if number.player == *player {
                        return Err("Can't move there!");
                    }
                    let mut value = *value;
                    let squared =
                        if let Some(square_owner) = &game.board.tiles[y-1][x-1].square {
                            *square_owner == *player
                        } else {
                            false
                    };
                    let actual_value = if squared {value.pow(2)} else {value};
                    if actual_value > number.value {
                        value -= if squared {1} else {number.value};
                        game.board.tiles[y-1][x-1].number = Some(Number {
                            player: player.same(),
                            value,
                        });
                        game.board.tiles[y1-1][x1-1].number = Some(Number {
                            player: player.other(),
                            value: 0,
                        });
                        return Ok(());
                    }
                    return Err("Not enough power!");
                };
                let mut value = *value;
                game.board.tiles[y-1][x-1].number = None;
                if let Some(square_owner) = &game.board.tiles[y1-1][x1-1].square {
                    if *player != *square_owner {
                        if value == 0 {
                            game.board.tiles[y1-1][x1-1].square = None;
                            game.board.tiles[y1-1][x1-1].number = None;
                            match *player {
                                Player::FirstPlayer => {
                                    game.squares.0 -= 1;
                                    game.score.1 += 1;
                                }
                                Player::SecondPlayer => {
                                    game.squares.1 -= 1;
                                    game.score.0 += 1;
                                }
                            }
                        } else {
                            value -= 1;
                        }
                    }
                }
                game.board.tiles[y1-1][x1-1].number = Some(Number {
                    player: player.same(),
                    value,
                });

                Ok(())
            }
            None => Err("No number to move!"),
        }
    }
}