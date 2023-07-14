const UPPER: [char; 26] =
['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'];
const LOWER: [char; 26] =
['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'];
const MINE: char = '*';
const FLAG: char = '!';
const NUMBERS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

struct Minefield {
    squares: Vec<Square>,
    width: usize,
    height: usize,
    target_to_open: usize,
    flags: usize,
}

impl Minefield {
    fn new(width: usize, height: usize) -> Self {
        let total = width * height; // max size = 26*26 = 676 squares
        let mut mines = Minefield::randomise_mines(total);
        mines.sort_unstable(); // sort in order to ensure placement via for loop below
        let mines_total = mines.len();
        let mut mines_index = 0;
        let mut squares = Vec::with_capacity(total);
        let mut col = 1;
        for each in 1..=total {
            let mine =
                if mines_total == mines_index {Mine::Count(0)} // prevent panic from index below when all mines placed
                else if (each-1) == mines[mines_index] {mines_index += 1; Mine::Explode}
                else {Mine::Count(0)};
            let edge =
                if each == 1 {Edge::TopLeft}
                else if each == width {Edge::TopRight}
                else if each < width {Edge::Top}
                else if each == total {Edge::BottomRight}
                else if each == (total - (width - 1)) {Edge::BottomLeft}
                else if each > width && col == width {Edge::Right}
                else if each > width && col == 1 {Edge::Left}
                else if each > (total - (width - 1)) {Edge::Bottom}
                else {Edge::Middle};
            squares.push(Square::new(mine, edge));
            col += 1;
            if each % width == 0 {col = 1;}
        }
        let mut minefield = Self {squares, width, height, target_to_open: total-mines_total, flags: mines_total};
        minefield.populate_numbers(); // now that mines are placed update the neighbouring square counts
        minefield
    }
    fn randomise_mines(total: usize) -> Vec<usize> {
        let limit = if total/5 == 0 {1} else {total/5}; // Fifth of squares are mines
        let mut numbers = Vec::new();
        let mut count = 0;
        while count != limit {
            for num in Minefield::seed(3) { // size of 3 ensures we get digits of 1, 2 and 3 lengths
                if !numbers.contains(&num) && count < limit && num < total {
                    numbers.push(num);
                    count += 1;
                }
            }
        }
        numbers
    }
    fn seed(size: usize) -> Vec<usize> {
        let mut seed: Vec<char> = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("Check system clock!").as_nanos().to_string().chars().collect(); // nanoseconds are u128
        seed.reverse(); // the end digits are the most different so reverse the chars
        let mut numbers = Vec::new();
        for characters in seed.windows(size) { // leading zeros in a window will parse to smaller digits
            let mut num_string = characters[0].to_string(); // create string from first char of window
            num_string.push(characters[1]);
            num_string.push(characters[2]); // seed() is only ever called with size 3 so indexing never panics
            numbers.push(num_string.parse::<usize>().unwrap()); // unwrap() guaranteed to work here (started with u128)
        }
        numbers
    }
    fn populate_numbers(&mut self) {
        let total = self.width * self.height;
        for index in 0..total {
            if let Mine::Explode = self.squares[index].mine {
                for i in self.squares[index].edge.get_index_offsets(index, self.width) {
                    self.squares[i].mine.increment_count();
                }
            }
        }
    }
    fn is_solved(&self) -> bool {
        self.squares.iter().filter(|square| square.opened).count() == self.target_to_open && self.flags == 0
    }
    fn update(&mut self, action: PlayerAction) -> End {
        match action {
            PlayerAction::ToggleFlag(position) => {
                let index = self.get_square_index(position);
                if self.squares[index].opened {
                    println!("Cannot set flag! Square already opened!");
                    return End::Unchanged;
                }
                if self.flags == 0 && !self.squares[index].flagged {
                    println!("Not enough flags! Are you sure you marked the right squares and opened the rest?");
                    return End::Unchanged;
                }
                if self.squares[index].toggle_flag() {self.flags -= 1;} else {self.flags += 1;}
            },
            PlayerAction::OpenSquare(position) => {
                let index = self.get_square_index(position);
                self.squares[index].opened = true;
                match self.squares[index].mine {
                    Mine::Explode => return End::Death,
                    Mine::Count(count) => {
                        if count == 0 {
                            let mut next_squares = std::collections::VecDeque::new();
                            next_squares.push_front(index);
                            while !next_squares.is_empty() {
                                let i = next_squares.pop_front().unwrap();
                                let surrounding_indicies = self.squares[i].edge.get_index_offsets(i, self.width);
                                for square in surrounding_indicies {
                                    if !self.squares[square].opened {
                                        self.squares[square].opened = true;
                                        if self.squares[square].mine.get_count() == 0 {
                                            next_squares.push_back(square);
                                        }
                                    }
                                }
                            }
                        }
                    },
                }
            },
            PlayerAction::Quit => return End::Quit,
        }
        if self.is_solved() {End::Victory} else {End::Unfinished}
    }
    fn get_square_index(&self, position: Position) -> usize {
        let row = usize::from(position.lower);
        let col = usize::from(position.upper);
        let offset = self.width - col;
        ((self.width * row) - offset) - 1 // indexing starts at 0
    }
}

impl std::fmt::Display for Minefield {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut top_row = String::from("  ");
        UPPER.iter().take(self.width).for_each(|c| {top_row.push(*c); top_row.push(' ');});
        top_row.push('\n');
        let mut dashes = String::from(" +");
        for _ in 0..self.width {dashes.push_str("-+");}
        dashes.push('\n');
        let mut rows = String::new();
        let mut row = 1;
        for square in &self.squares {
            match square.edge {
                Edge::TopLeft | Edge::Left | Edge::BottomLeft => {
                    if let Edge::TopLeft = square.edge {rows.push_str(&dashes);}
                    rows.push(LOWER[row - 1]);
                    rows.push('|');
                    rows.push_str(&square.to_string());
                    rows.push('|');
                },
                Edge::TopRight | Edge::Right | Edge::BottomRight => {
                    rows.push_str(&square.to_string());
                    rows.push('|');
                    rows.push(LOWER[row - 1]);
                    rows.push('\n');
                    rows.push_str(&dashes);
                    row += 1;
                },
                _ => {
                    rows.push_str(&square.to_string());
                    rows.push('|');
                },
            }
        }
        write!(f, "{top_row}{rows}{top_row}Flags remaining: {}", self.flags)
    }
}

struct Square {
    mine: Mine,
    opened: bool,
    flagged: bool,
    edge: Edge,
}

impl Square {
    fn new(mine: Mine, edge: Edge) -> Self {
        Self {mine, opened: false, flagged: false, edge}
    }
    fn toggle_flag(&mut self) -> bool {
        self.flagged = !self.flagged;
        self.flagged
    }
}

impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let char =
            if self.flagged {FLAG}
            else if self.opened {match self.mine {Mine::Explode => MINE, Mine::Count(num) => NUMBERS[num]}}
            else {' '};
        write!(f, "{char}")
    }
}

enum Mine {
    Explode,
    Count(usize),
}

impl Mine {
    fn increment_count(&mut self) {
        if let Self::Count(num) = self {*num += 1;}
    }
    fn get_count(&self) -> usize {
        if let Self::Count(count) = self {*count} else {unreachable!()}
    }
}

struct Position {
    lower: Lower,
    upper: Upper,
}

impl Position {
    fn new(lower: Lower, upper: Upper) -> Self {
        Self {lower, upper}
    }
}

#[allow(non_camel_case_types)]
enum Lower {
    a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z,
}

impl From<usize> for Lower {
    fn from(num: usize) -> Self {
        match num {
            1 => Self::a,
            2 => Self::b,
            3 => Self::c,
            4 => Self::d,
            5 => Self::e,
            6 => Self::f,
            7 => Self::g,
            8 => Self::h,
            9 => Self::i,
            10 => Self::j,
            11 => Self::k,
            12 => Self::l,
            13 => Self::m,
            14 => Self::n,
            15 => Self::o,
            16 => Self::p,
            17 => Self::q,
            18 => Self::r,
            19 => Self::s,
            20 => Self::t,
            21 => Self::u,
            22 => Self::v,
            23 => Self::w,
            24 => Self::x,
            25 => Self::y,
            26 => Self::z,
            _ => unreachable!(),
        }
    }
}

impl From<char> for Lower {
    fn from(letter: char) -> Self {
        match letter {
            'a' => Self::a,
            'b' => Self::b,
            'c' => Self::c,
            'd' => Self::d,
            'e' => Self::e,
            'f' => Self::f,
            'g' => Self::g,
            'h' => Self::h,
            'i' => Self::i,
            'j' => Self::j,
            'k' => Self::k,
            'l' => Self::l,
            'm' => Self::m,
            'n' => Self::n,
            'o' => Self::o,
            'p' => Self::p,
            'q' => Self::q,
            'r' => Self::r,
            's' => Self::s,
            't' => Self::t,
            'u' => Self::u,
            'v' => Self::v,
            'w' => Self::w,
            'x' => Self::x,
            'y' => Self::y,
            'z' => Self::z,
            _ => unreachable!(),
        }
    }
}

impl From<Lower> for usize {
    fn from(lower: Lower) -> usize {
        match lower {
            Lower::a => 1,
            Lower::b => 2,
            Lower::c => 3,
            Lower::d => 4,
            Lower::e => 5,
            Lower::f => 6,
            Lower::g => 7,
            Lower::h => 8,
            Lower::i => 9,
            Lower::j => 10,
            Lower::k => 11,
            Lower::l => 12,
            Lower::m => 13,
            Lower::n => 14,
            Lower::o => 15,
            Lower::p => 16,
            Lower::q => 17,
            Lower::r => 18,
            Lower::s => 19,
            Lower::t => 20,
            Lower::u => 21,
            Lower::v => 22,
            Lower::w => 23,
            Lower::x => 24,
            Lower::y => 25,
            Lower::z => 26,
        }
    }
}

enum Upper {
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
}

impl From<usize> for Upper {
    fn from(num: usize) -> Self {
        match num {
            1 => Self::A,
            2 => Self::B,
            3 => Self::C,
            4 => Self::D,
            5 => Self::E,
            6 => Self::F,
            7 => Self::G,
            8 => Self::H,
            9 => Self::I,
            10 => Self::J,
            11 => Self::K,
            12 => Self::L,
            13 => Self::M,
            14 => Self::N,
            15 => Self::O,
            16 => Self::P,
            17 => Self::Q,
            18 => Self::R,
            19 => Self::S,
            20 => Self::T,
            21 => Self::U,
            22 => Self::V,
            23 => Self::W,
            24 => Self::X,
            25 => Self::Y,
            26 => Self::Z,
            _ => unreachable!(),
        }
    }
}

impl From<char> for Upper {
    fn from(letter: char) -> Self {
        match letter {
            'A' => Self::A,
            'B' => Self::B,
            'C' => Self::C,
            'D' => Self::D,
            'E' => Self::E,
            'F' => Self::F,
            'G' => Self::G,
            'H' => Self::H,
            'I' => Self::I,
            'J' => Self::J,
            'K' => Self::K,
            'L' => Self::L,
            'M' => Self::M,
            'N' => Self::N,
            'O' => Self::O,
            'P' => Self::P,
            'Q' => Self::Q,
            'R' => Self::R,
            'S' => Self::S,
            'T' => Self::T,
            'U' => Self::U,
            'V' => Self::V,
            'W' => Self::W,
            'X' => Self::X,
            'Y' => Self::Y,
            'Z' => Self::Z,
            _ => unreachable!(),
        }
    }
}

impl From<Upper> for usize {
    fn from(upper: Upper) -> usize {
        match upper {
            Upper::A => 1,
            Upper::B => 2,
            Upper::C => 3,
            Upper::D => 4,
            Upper::E => 5,
            Upper::F => 6,
            Upper::G => 7,
            Upper::H => 8,
            Upper::I => 9,
            Upper::J => 10,
            Upper::K => 11,
            Upper::L => 12,
            Upper::M => 13,
            Upper::N => 14,
            Upper::O => 15,
            Upper::P => 16,
            Upper::Q => 17,
            Upper::R => 18,
            Upper::S => 19,
            Upper::T => 20,
            Upper::U => 21,
            Upper::V => 22,
            Upper::W => 23,
            Upper::X => 24,
            Upper::Y => 25,
            Upper::Z => 26,
        }
    }
}

enum Edge {
    TopLeft,
    Top,
    TopRight,
    Left,
    Middle,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl Edge {
    fn get_index_offsets(&self, index: usize, offset: usize) -> Vec<usize> {
        let right = index+1;
        let down_right = (index+offset)+1;
        let down = index+offset;
        let down_left = (index+offset)-1;
        match self {
            Self::TopLeft => vec![right, down_right, down],
            Self::Top => vec![index-1, right, down_right, down, down_left],
            Self::TopRight => vec![down, down_left, index-1],
            Self::Left => vec![(index-offset), ((index-offset)+1), right, down_right, down],
            Self::Middle => vec![(index-offset), ((index-offset)+1), right, down_right, down, down_left, index-1,
                ((index-offset)-1)],
            Self::Right => vec![(index-offset), down, down_left, index-1, ((index-offset)-1)],
            Self::BottomLeft => vec![(index-offset), ((index-offset)+1), right],
            Self::Bottom => vec![(index-offset), ((index-offset)+1), right, index-1, ((index-offset)-1)],
            Self::BottomRight => vec![(index-offset), index-1, ((index-offset)-1)],
        }
    }
}

enum End {
    Unfinished,
    Unchanged,
    Death,
    Victory,
    Quit,
}

enum PlayerAction {
    Quit,
    ToggleFlag(Position),
    OpenSquare(Position),
}

fn main() {
    let width = user_input_usize("Input minefield width: ");
    let height = user_input_usize("Input minefield height: ");
    let mut minefield = Minefield::new(width, height);
    println!("{minefield}");
    while !minefield.is_solved() {
        match minefield.update(user_input_text(width, height)) {
            End::Quit => break,
            End::Death => {println!("{minefield}\nYou stepped on a mine! You lose!"); break;},
            End::Victory => {println!("{minefield}\nYou flagged all the mines! You win!"); break;},
            End::Unfinished => println!("{minefield}"),
            End::Unchanged => {},
        }
    }
}

fn user_input_usize(text: &str) -> usize {
    loop {
        let mut input = String::new();
        print!("{text}");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        std::io::stdin().read_line(&mut input).expect("Failed to read line.");
        let answer: usize = if let Ok(num) = input.trim().parse() {num} else {
            println!("Input a number!");
            continue;};
        if (2..=26).contains(&answer) {return answer;}
        println!("Number must be between 2 and 26 inclusive!");
    }
}

fn user_input_text(width: usize, height: usize) -> PlayerAction {
    loop {
        let mut input = String::new();
        println!("Open square with: \"o aA\", Toggle flag with: \"f aA\", Quit game with \"q\"");
        print!("Input: ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        std::io::stdin().read_line(&mut input).expect("Failed to read line.");
        let input: Vec<char> = input.trim().chars().collect();
        match input.len() {
            0 => println!("Empty input is invalid!"),
            1 =>
                if input[0] == 'q' {return PlayerAction::Quit;}
                else {println!("First character is invalid!");},
            2 | 3 => println!("Incomplete input!"),
            4 => {
                if input[0] != 'o' && input[0] != 'f' {println!("First character is invalid!"); continue;}
                if input[1] != ' ' {println!("Second character should be a space!"); continue;}
                let lower =
                    if LOWER.contains(&input[2]) {
                        if usize::from(Lower::from(input[2])) > height {println!("Invalid row!"); continue;}
                        Lower::from(input[2])
                    } else {println!("Third character should be a lowercase letter!"); continue;};
                let upper =
                    if UPPER.contains(&input[3]) {
                        if usize::from(Upper::from(input[3])) > width {println!("Invalid column!"); continue;}
                        Upper::from(input[3])
                    } else {println!("Fourth character should be an uppercase letter!"); continue;};
                match input.first() {
                    Some('o') => return PlayerAction::OpenSquare(Position::new(lower, upper)),
                    Some('f') => return PlayerAction::ToggleFlag(Position::new(lower, upper)),
                    _ => unreachable!(),
                }
            },
            _ => println!("Too much input!"),
        }
    }
}
