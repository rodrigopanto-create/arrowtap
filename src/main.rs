use macroquad::prelude::*;

const MAX_SCORES: usize = 10;
const BACKGROUND: Color = Color::new(0.05, 0.05, 0.1, 1.0);

#[derive(Clone, Copy, PartialEq, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    const ALL: [Self; 4] = [Self::Up, Self::Down, Self::Left, Self::Right];

    fn offset(self) -> (i32, i32) {
        match self {
            Self::Up => (0, -1),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ArrowTile {
    id: usize,
    grid_x: i32,
    grid_y: i32,
    dir: Direction,
    is_escaping: bool,
    anim_progress: f32,
}

struct Board {
    size: i32,
    tiles: Vec<ArrowTile>,
}

impl Board {
    fn can_escape(&self, tile: &ArrowTile, current_tiles: &[ArrowTile]) -> bool {
        let (dx, dy) = tile.dir.offset();
        let (mut x, mut y) = (tile.grid_x + dx, tile.grid_y + dy);
        while x >= 0 && x < self.size && y >= 0 && y < self.size {
            if current_tiles.iter().any(|other| {
                other.id != tile.id && !other.is_escaping && other.grid_x == x && other.grid_y == y
            }) {
                return false;
            }
            x += dx;
            y += dy;
        }
        true
    }
}

fn level_settings(level: u32) -> (i32, usize) {
    assert!(level > 0, "level must start at 1");
    let size = 5 + ((level - 1) / 4).min(4) as i32;
    let density = 8_usize.saturating_add(((level - 1) as usize).saturating_mul(3));
    (size, density.min((size * size) as usize))
}

fn generate_level(size: i32, density: usize) -> Board {
    assert!((1..=9).contains(&size), "grid size must be between 1 and 9");
    assert!(
        density > 0 && density <= (size * size) as usize,
        "invalid density"
    );
    let mut positions: Vec<_> = (0..size)
        .flat_map(|y| (0..size).map(move |x| (x, y)))
        .collect();
    for i in 0..density {
        let selected = macroquad::rand::gen_range(i, positions.len());
        positions.swap(i, selected);
    }
    let mut remaining = Board {
        size,
        tiles: positions[..density]
            .iter()
            .enumerate()
            .map(|(id, &(grid_x, grid_y))| ArrowTile {
                id,
                grid_x,
                grid_y,
                dir: Direction::Up,
                is_escaping: false,
                anim_progress: 0.0,
            })
            .collect(),
    };
    let mut tiles = Vec::with_capacity(density);
    // Each chosen direction is clear after the previously chosen tiles are removed.
    while !remaining.tiles.is_empty() {
        let mut candidates = Vec::new();
        for (index, tile) in remaining.tiles.iter().enumerate() {
            for dir in Direction::ALL {
                let candidate = ArrowTile { dir, ..*tile };
                if remaining.can_escape(&candidate, &remaining.tiles) {
                    candidates.push((index, dir));
                }
            }
        }
        assert!(!candidates.is_empty(), "a boundary tile must have an exit");
        let (index, dir) = candidates[macroquad::rand::gen_range(0, candidates.len())];
        let mut tile = remaining.tiles.swap_remove(index);
        tile.dir = dir;
        tiles.push(tile);
    }
    Board { size, tiles }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ScoreEntry {
    score: u64,
    level: u32,
}

fn level_score(tile_count: usize, level: u32, elapsed: f32, mistakes: u32) -> u64 {
    let time_bonus = (120.0 - elapsed).max(0.0) as u64 * 5;
    let precision_bonus = 500_u64.saturating_sub(u64::from(mistakes) * 50);
    tile_count as u64 * 100 + u64::from(level) * 250 + time_bonus + precision_bonus
}

fn encode_scores(scores: &[ScoreEntry]) -> String {
    let mut data = String::from("TAPAWAY1\n");
    for entry in scores {
        data.push_str(&format!("{} {}\n", entry.score, entry.level));
    }
    data
}

fn decode_scores(data: &str) -> Result<Vec<ScoreEntry>, String> {
    let mut lines = data.lines();
    if lines.next() != Some("TAPAWAY1") {
        return Err("Unrecognized highscore format".into());
    }
    let mut scores = Vec::new();
    for line in lines {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() != 2 || scores.len() >= MAX_SCORES {
            return Err("Invalid highscore entry".into());
        }
        let score = fields[0].parse().map_err(|_| "Invalid score")?;
        let level = fields[1].parse().map_err(|_| "Invalid level")?;
        if level == 0 {
            return Err("Invalid level".into());
        }
        scores.push(ScoreEntry { score, level });
    }
    sort_scores(&mut scores);
    Ok(scores)
}

fn sort_scores(scores: &mut Vec<ScoreEntry>) {
    scores.sort_by(|a, b| b.score.cmp(&a.score).then(b.level.cmp(&a.level)));
    scores.truncate(MAX_SCORES);
}

#[cfg(not(target_arch = "wasm32"))]
fn read_scores() -> Result<Option<String>, String> {
    match std::fs::read_to_string("arrow-highscores.txt") {
        Ok(data) => Ok(Some(data)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("Cannot read highscores: {error}")),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn write_scores(data: &str) -> Result<(), String> {
    std::fs::write("arrow-highscores.txt", data)
        .map_err(|error| format!("Cannot save highscores: {error}"))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn arcade_scores_crate_version() -> u32 {
    1
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn arcade_scores_read(buffer: *mut u8, capacity: usize) -> i32;
    fn arcade_scores_write(data: *const u8, length: usize) -> i32;
}

#[cfg(target_arch = "wasm32")]
fn read_scores() -> Result<Option<String>, String> {
    let mut buffer = [0_u8; 512];
    // The browser bridge writes at most capacity bytes and returns the byte count.
    let length = unsafe { arcade_scores_read(buffer.as_mut_ptr(), buffer.len()) };
    match length {
        0 => Ok(None),
        1..=512 => String::from_utf8(buffer[..length as usize].to_vec())
            .map(Some)
            .map_err(|error| format!("Invalid highscore text: {error}")),
        _ => Err("Cannot read browser highscores; check browser storage permissions".into()),
    }
}

#[cfg(target_arch = "wasm32")]
fn write_scores(data: &str) -> Result<(), String> {
    // The browser bridge copies the string synchronously before returning.
    if unsafe { arcade_scores_write(data.as_ptr(), data.len()) } == 0 {
        Ok(())
    } else {
        Err("Cannot save browser highscores; check browser storage permissions".into())
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Start,
    Playing,
    Victory,
    Highscores,
}

struct Game {
    screen: Screen,
    scores_return: Screen,
    board: Board,
    level: u32,
    total_score: u64,
    last_score: u64,
    elapsed: f32,
    mistakes: u32,
    highscores: Vec<ScoreEntry>,
    storage_error: Option<String>,
}

impl Game {
    fn new() -> Self {
        let (highscores, storage_error) = match read_scores()
            .and_then(|data| data.map(|text| decode_scores(&text)).transpose())
        {
            Ok(scores) => (scores.unwrap_or_default(), None),
            Err(error) => {
                error!("{error}");
                (Vec::new(), Some(error))
            }
        };
        Self {
            screen: Screen::Start,
            scores_return: Screen::Start,
            board: Board {
                size: 5,
                tiles: Vec::new(),
            },
            level: 1,
            total_score: 0,
            last_score: 0,
            elapsed: 0.0,
            mistakes: 0,
            highscores,
            storage_error,
        }
    }

    fn start_level(&mut self) {
        let (size, density) = level_settings(self.level);
        self.board = generate_level(size, density);
        self.elapsed = 0.0;
        self.mistakes = 0;
        self.screen = Screen::Playing;
    }

    fn win(&mut self) {
        let (_, density) = level_settings(self.level);
        self.last_score = level_score(density, self.level, self.elapsed, self.mistakes);
        self.total_score = self.total_score.saturating_add(self.last_score);
        self.highscores.push(ScoreEntry {
            score: self.last_score,
            level: self.level,
        });
        sort_scores(&mut self.highscores);
        match write_scores(&encode_scores(&self.highscores)) {
            Ok(()) => self.storage_error = None,
            Err(error) => {
                error!("{error}");
                self.storage_error = Some(error);
            }
        }
        self.screen = Screen::Victory;
    }
}

fn centered_text(text: &str, y: f32, size: f32, color: Color) {
    let size = size.min(screen_width() / (text.len().max(1) as f32 * 0.62));
    let width = measure_text(text, None, size as u16, 1.0).width;
    draw_text(text, (screen_width() - width) * 0.5, y, size, color);
}

fn button(label: &str, y: f32, pointer: Option<Vec2>) -> bool {
    let width = 280.0_f32.min(screen_width() - 24.0);
    let rect = Rect::new((screen_width() - width) * 0.5, y, width, 44.0);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.1, 0.15, 0.25, 1.0),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, GREEN);
    centered_text(label, y + 29.0, 22.0, LIME);
    pointer.is_some_and(|point| rect.contains(point))
}

fn input_position() -> Option<Vec2> {
    let touches = touches();
    touches
        .iter()
        .find(|touch| touch.phase == TouchPhase::Started)
        .map(|touch| touch.position)
        .or_else(|| {
            if touches.is_empty() && is_mouse_button_pressed(MouseButton::Left) {
                let (x, y) = mouse_position();
                Some(vec2(x, y))
            } else {
                None
            }
        })
}

fn draw_tile(tile: &ArrowTile, start: Vec2, step: f32, tile_size: f32) {
    let (dx, dy) = tile.dir.offset();
    let distance = tile.anim_progress * (screen_width().max(screen_height()) + step);
    let tx = start.x + tile.grid_x as f32 * step + dx as f32 * distance;
    let ty = start.y + tile.grid_y as f32 * step + dy as f32 * distance;
    draw_rectangle(
        tx,
        ty,
        tile_size,
        tile_size,
        Color::new(0.1, 0.15, 0.25, 0.9),
    );
    draw_rectangle_lines(tx, ty, tile_size, tile_size, 2.0, GREEN);
    let center = vec2(tx + tile_size * 0.5, ty + tile_size * 0.5);
    let forward = vec2(dx as f32, dy as f32);
    let side = vec2(-forward.y, forward.x);
    let radius = tile_size * 0.23;
    draw_triangle(
        center + forward * radius,
        center - forward * radius + side * radius,
        center - forward * radius - side * radius,
        LIME,
    );
}

fn play_frame(game: &mut Game, pointer: Option<Vec2>) {
    let available = screen_width().min(screen_height() - 140.0).max(1.0) - 24.0;
    let step = (available.max(1.0) / game.board.size as f32).min(70.0);
    let tile_size = step * (60.0 / 70.0);
    let board_width = step * game.board.size as f32 - (step - tile_size);
    let start = vec2(
        (screen_width() - board_width) * 0.5,
        85.0 + ((screen_height() - 140.0 - board_width) * 0.5).max(0.0),
    );

    if let Some(point) = pointer {
        let selected = game.board.tiles.iter().position(|tile| {
            let x = start.x + tile.grid_x as f32 * step;
            let y = start.y + tile.grid_y as f32 * step;
            !tile.is_escaping && Rect::new(x, y, tile_size, tile_size).contains(point)
        });
        if let Some(index) = selected {
            if game
                .board
                .can_escape(&game.board.tiles[index], &game.board.tiles)
            {
                game.board.tiles[index].is_escaping = true;
            } else {
                game.mistakes = game.mistakes.saturating_add(1);
            }
        }
    }

    let dt = get_frame_time();
    game.elapsed += dt;
    for tile in &mut game.board.tiles {
        if tile.is_escaping {
            tile.anim_progress += dt * 2.5;
        }
    }
    game.board
        .tiles
        .retain(|tile| !tile.is_escaping || tile.anim_progress < 1.0);

    centered_text(
        &format!("LEVEL {:02} // SCORE {}", game.level, game.total_score),
        30.0,
        24.0,
        WHITE,
    );
    centered_text(
        &format!("TIME {:.0}s // MISSES {}", game.elapsed, game.mistakes),
        56.0,
        18.0,
        SKYBLUE,
    );
    draw_rectangle_lines(
        start.x - 8.0,
        start.y - 8.0,
        board_width + 16.0,
        board_width + 16.0,
        2.0,
        SKYBLUE,
    );
    for tile in &game.board.tiles {
        draw_tile(tile, start, step, tile_size);
    }
    if game.board.tiles.is_empty() {
        game.win();
    }
}

#[macroquad::main("TapAwayArcade")]
async fn main() {
    simulate_mouse_with_touch(false);
    let mut game = Game::new();
    loop {
        clear_background(BACKGROUND);
        let pointer = input_position();
        let confirm = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space);
        let base = (screen_height() * 0.5 - 150.0).max(4.0);

        match game.screen {
            Screen::Start => {
                centered_text("TAP AWAY", base + 30.0, 54.0, LIME);
                centered_text("RETRO ARCADE", base + 66.0, 24.0, SKYBLUE);
                if ((get_time() * 2.0) as u64).is_multiple_of(2) {
                    centered_text("PRESS START", base + 122.0, 28.0, WHITE);
                }
                let start = button("START / ENTER", base + 146.0, pointer);
                let scores = button("HIGHSCORES / H", base + 202.0, pointer);
                if start || confirm {
                    game.level = 1;
                    game.total_score = 0;
                    game.start_level();
                } else if scores || is_key_pressed(KeyCode::H) {
                    game.scores_return = Screen::Start;
                    game.screen = Screen::Highscores;
                }
            }
            Screen::Playing => play_frame(&mut game, pointer),
            Screen::Victory => {
                centered_text("VICTORY!", base + 20.0, 48.0, LIME);
                centered_text(
                    &format!("LEVEL {} COMPLETE", game.level),
                    base + 58.0,
                    24.0,
                    WHITE,
                );
                centered_text(
                    &format!("LEVEL SCORE: {}", game.last_score),
                    base + 91.0,
                    22.0,
                    SKYBLUE,
                );
                centered_text(
                    &format!("TOTAL: {}", game.total_score),
                    base + 120.0,
                    22.0,
                    WHITE,
                );
                let next = button("NEXT LEVEL / ENTER", base + 145.0, pointer);
                let scores = button("HIGHSCORES / H", base + 199.0, pointer);
                let menu = button("MAIN MENU / ESC", base + 253.0, pointer);
                if next || confirm {
                    game.level = game.level.saturating_add(1);
                    game.start_level();
                } else if scores || is_key_pressed(KeyCode::H) {
                    game.scores_return = Screen::Victory;
                    game.screen = Screen::Highscores;
                } else if menu || is_key_pressed(KeyCode::Escape) {
                    game.screen = Screen::Start;
                }
            }
            Screen::Highscores => {
                centered_text("HIGH SCORES", 42.0, 36.0, LIME);
                centered_text("BEST LEVEL SCORES", 70.0, 18.0, SKYBLUE);
                let spacing = ((screen_height() - 180.0) / MAX_SCORES as f32).clamp(12.0, 30.0);
                if game.highscores.is_empty() {
                    centered_text("CLEAR A LEVEL TO JOIN THE RANKS", 125.0, 20.0, WHITE);
                }
                for (rank, entry) in game.highscores.iter().enumerate() {
                    centered_text(
                        &format!(
                            "{:02}   {:08}   LV {:02}",
                            rank + 1,
                            entry.score,
                            entry.level
                        ),
                        100.0 + rank as f32 * spacing,
                        22.0_f32.min(spacing),
                        if rank == 0 { YELLOW } else { WHITE },
                    );
                }
                if button("BACK / ESC", screen_height() - 74.0, pointer)
                    || is_key_pressed(KeyCode::Escape)
                    || confirm
                {
                    game.screen = game.scores_return;
                }
            }
        }

        if game.storage_error.is_some() {
            centered_text(
                "SCORE STORAGE ERROR - CHECK LOG",
                screen_height() - 8.0,
                14.0,
                RED,
            );
        }
        next_frame().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn difficulty_is_bounded_and_non_decreasing() {
        let mut previous = (0, 0);
        for level in 1..=500 {
            let settings = level_settings(level);
            assert!(settings.0 >= previous.0 && settings.1 >= previous.1);
            assert!(settings.1 <= (settings.0 * settings.0) as usize);
            previous = settings;
        }
        assert_eq!(level_settings(1), (5, 8));
        assert_eq!(level_settings(500), (9, 81));
        assert_eq!(level_settings(u32::MAX), (9, 81));
    }

    #[test]
    fn generated_levels_are_unique_and_solvable_even_when_full() {
        for size in 1..=9 {
            for _ in 0..5 {
                let mut board = generate_level(size, (size * size) as usize);
                let mut cells = std::collections::HashSet::new();
                for tile in &board.tiles {
                    assert!(cells.insert((tile.grid_x, tile.grid_y)));
                }
                while !board.tiles.is_empty() {
                    let index = board
                        .tiles
                        .iter()
                        .position(|tile| board.can_escape(tile, &board.tiles))
                        .expect("generated board must be solvable");
                    board.tiles.remove(index);
                }
            }
        }
    }

    #[test]
    fn score_storage_round_trips_and_keeps_top_ten() {
        let mut scores: Vec<_> = (1..=15)
            .map(|level| ScoreEntry {
                score: u64::from(level) * 100,
                level,
            })
            .collect();
        sort_scores(&mut scores);
        assert_eq!(scores.len(), MAX_SCORES);
        assert_eq!(scores[0].score, 1500);
        assert_eq!(decode_scores(&encode_scores(&scores)).unwrap(), scores);
        assert!(decode_scores("bad data").is_err());
        assert!(decode_scores("TAPAWAY1\n100 0\n").is_err());
    }

    #[test]
    fn score_rewards_speed_precision_and_progress() {
        let best = level_score(8, 1, 10.0, 0);
        assert!(best > level_score(8, 1, 30.0, 0));
        assert!(best > level_score(8, 1, 10.0, 1));
        assert!(level_score(11, 2, 10.0, 0) > best);
        assert!(level_score(8, 1, 300.0, 100) > 0);
    }

    #[test]
    fn collisions_check_every_direction_and_ignore_departing_tiles() {
        for dir in Direction::ALL {
            let (dx, dy) = dir.offset();
            let tile = ArrowTile {
                id: 0,
                grid_x: 2,
                grid_y: 2,
                dir,
                is_escaping: false,
                anim_progress: 0.0,
            };
            let blocker = ArrowTile {
                id: 1,
                grid_x: 2 + dx,
                grid_y: 2 + dy,
                ..tile
            };
            let mut board = Board {
                size: 5,
                tiles: vec![tile, blocker],
            };
            assert!(!board.can_escape(&tile, &board.tiles));
            board.tiles[1].is_escaping = true;
            assert!(board.can_escape(&tile, &board.tiles));
        }
    }
}
