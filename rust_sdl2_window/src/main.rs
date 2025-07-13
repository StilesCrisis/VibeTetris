extern crate sdl2;
extern crate rand;
#[macro_use]
extern crate lazy_static;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::{Duration, SystemTime};
use rand::Rng;
mod font;

#[cfg(test)]
mod tests;

// Game constants
const PLAYFIELD_WIDTH: usize = 10;
const PLAYFIELD_HEIGHT: usize = 20; // Visible area
const BLOCK_SIZE: u32 = 30; // For rendering
const SCORE_TEXT_AREA_WIDTH: u32 = 200; // Width for score display

// Basic game structures
type Playfield = Vec<Vec<u8>>; // 0: empty, 1-7: Tetromino type/color

#[derive(Debug, Clone, Copy, PartialEq)]
enum TetrominoType {
    I = 1, O, T, S, Z, J, L,
}

lazy_static! {
    static ref TETROMINO_SHAPES: Vec<Vec<Vec<(i32, i32)>>> = vec![
        // I
        vec![
            vec![(0,0), (-1,0), (1,0), (2,0)],
            vec![(0,0), (0,-1), (0,1), (0,2)],
            vec![(0,0), (-1,0), (1,0), (2,0)],
            vec![(0,0), (0,-1), (0,1), (0,2)]
        ],
        // O
        vec![
            vec![(0,0), (1,0), (0,1), (1,1)],
            vec![(0,0), (1,0), (0,1), (1,1)],
            vec![(0,0), (1,0), (0,1), (1,1)],
            vec![(0,0), (1,0), (0,1), (1,1)]
        ],
        // T
        vec![
            vec![(0,0), (-1,0), (1,0), (0,-1)],
            vec![(0,0), (0,-1), (0,1), (1,0)],
            vec![(0,0), (-1,0), (1,0), (0,1)],
            vec![(0,0), (0,-1), (0,1), (-1,0)]
        ],
        // S
        vec![
            vec![(0,0), (-1,0), (0,-1), (1,-1)],
            vec![(0,0), (0,1), (1,0), (1,-1)],
            vec![(0,0), (-1,0), (0,-1), (1,-1)],
            vec![(0,0), (0,1), (1,0), (1,-1)]
        ],
        // Z
        vec![
            vec![(0,0), (1,0), (0,-1), (-1,-1)],
            vec![(0,0), (0,-1), (1,0), (1,1)],
            vec![(0,0), (1,0), (0,-1), (-1,-1)],
            vec![(0,0), (0,-1), (1,0), (1,1)]
        ],
        // J
        vec![
            vec![(0,0), (-1,0), (1,0), (1,-1)],
            vec![(0,0), (0,-1), (0,1), (1,1)],
            vec![(0,0), (-1,0), (1,0), (-1,1)],
            vec![(0,0), (0,-1), (0,1), (-1,-1)]
        ],
        // L
        vec![
            vec![(0,0), (-1,0), (1,0), (-1,-1)],
            vec![(0,0), (0,-1), (0,1), (1,-1)],
            vec![(0,0), (-1,0), (1,0), (1,1)],
            vec![(0,0), (0,-1), (0,1), (-1,1)]
        ],
    ];
}

fn get_rotated_shape(tetromino_type: TetrominoType, rotation_index: usize) -> Vec<(i32, i32)> {
    TETROMINO_SHAPES[tetromino_type as usize - 1][rotation_index % 4].clone()
}

fn get_tetromino_color(tetromino_type: TetrominoType) -> Color {
    match tetromino_type {
        TetrominoType::I => Color::RGB(255, 215, 0),   // Gold
        TetrominoType::O => Color::RGB(0, 100, 0),     // Dark Green
        TetrominoType::T => Color::RGB(138, 43, 226),  // BlueViolet (Deep Purple)
        TetrominoType::S => Color::RGB(178, 34, 34),   // Firebrick Red
        TetrominoType::Z => Color::RGB(80, 80, 80),    // Medium-Dark Gray/Charcoal
        TetrominoType::J => Color::RGB(72, 61, 139),   // DarkSlateBlue (Indigo)
        TetrominoType::L => Color::RGB(205, 92, 92),   // IndianRed (Brownish/Muted Red)
    }
}

fn get_locked_block_color(value: u8) -> Color {
    match value {
        1 => get_tetromino_color(TetrominoType::I),
        2 => get_tetromino_color(TetrominoType::O),
        3 => get_tetromino_color(TetrominoType::T),
        4 => get_tetromino_color(TetrominoType::S),
        5 => get_tetromino_color(TetrominoType::Z),
        6 => get_tetromino_color(TetrominoType::J),
        7 => get_tetromino_color(TetrominoType::L),
        _ => Color::RGB(70, 70, 70),
    }
}

#[derive(Debug)]
struct Tetromino {
    tetromino_type: TetrominoType,
    rotation_index: usize,
    blocks: Vec<(i32, i32)>,
    x: i32,
    y: i32,
}

fn spawn_tetromino(playfield_width: usize) -> Tetromino {
    let mut rng = rand::thread_rng();
    let random_type_idx = rng.gen_range(1..=7);
    let random_type = match random_type_idx {
        1 => TetrominoType::I, 2 => TetrominoType::O, 3 => TetrominoType::T,
        4 => TetrominoType::S, 5 => TetrominoType::Z, 6 => TetrominoType::J,
        _ => TetrominoType::L,
    };
    let initial_rotation_index = 0;
    let initial_blocks = get_rotated_shape(random_type, initial_rotation_index).clone();
    let initial_x = playfield_width as i32 / 2 -1;
    let initial_y = match random_type {
        TetrominoType::T | TetrominoType::S | TetrominoType::Z | TetrominoType::I => 1, // Adjusted I as well
        _ => 0,
    };
    Tetromino {
        tetromino_type: random_type, rotation_index: initial_rotation_index,
        blocks: initial_blocks, x: initial_x, y: initial_y,
    }
}

fn check_collision(blocks: &[(i32, i32)], playfield: &Playfield, new_x: i32, new_y: i32) -> bool {
    for &(block_dx, block_dy) in blocks {
        let block_abs_x = new_x + block_dx;
        let block_abs_y = new_y + block_dy;
        if block_abs_x < 0 || block_abs_x >= PLAYFIELD_WIDTH as i32 { return true; }
        if block_abs_y < 0 { continue; }
        if block_abs_y >= PLAYFIELD_HEIGHT as i32 { return true; }
        if playfield[block_abs_y as usize][block_abs_x as usize] != 0 { return true; }
    }
    false
}

fn check_and_clear_lines(playfield: &mut Playfield, current_level: u32) -> (u32, u32) {
    let mut lines_cleared_this_turn = 0;
    let mut y = PLAYFIELD_HEIGHT -1; // Start from bottom

    // Iterate upwards
    while y > 0 { // No need to check row 0 as it cannot be "cleared" further up
        let is_full = playfield[y].iter().all(|&cell| cell != 0);
        if is_full {
            lines_cleared_this_turn += 1;
            // Move all rows above this one down
            for row_idx in (1..=y).rev() { // from y down to 1
                playfield[row_idx] = playfield[row_idx -1].clone();
            }
            // Clear top row
            playfield[0] = vec![0; PLAYFIELD_WIDTH];
            // Don't decrement y, check the "new" current row y again
        } else {
            y -= 1; // Move to check row above
        }
    }

    let score_gained = match lines_cleared_this_turn {
        1 => 100 * current_level,
        2 => 300 * current_level,
        3 => 500 * current_level,
        4 => 800 * current_level, // Tetris
        _ => 0,
    };

    (lines_cleared_this_turn, score_gained)
}


fn lock_piece(tetromino: &Tetromino, playfield: &mut Playfield, current_level: u32, score: &mut u32, total_lines_cleared: &mut u32, game_level: &mut u32) {
    let piece_value = tetromino.tetromino_type as u8;
    for &(block_dx, block_dy) in &tetromino.blocks {
        let block_abs_x = tetromino.x + block_dx;
        let block_abs_y = tetromino.y + block_dy;
        if block_abs_x >= 0 && block_abs_x < PLAYFIELD_WIDTH as i32 &&
           block_abs_y >= 0 && block_abs_y < PLAYFIELD_HEIGHT as i32 {
            playfield[block_abs_y as usize][block_abs_x as usize] = piece_value;
        }
    }
    // After locking, check for line clears
    let (lines_cleared, score_gained_from_lines) = check_and_clear_lines(playfield, current_level);
    *score += score_gained_from_lines;
    *total_lines_cleared += lines_cleared;
    *game_level = *total_lines_cleared / 10 + 1;
}

lazy_static! {
    static ref JLSTZ_KICKS: Vec<Vec<Vec<(i32, i32)>>> = vec![
        // Clockwise
        vec![
            vec![(0,0), (-1,0), (-1,-1), (0,2), (-1,2)], // 0 -> 1
            vec![(0,0), (1,0), (1,1), (0,-2), (1,-2)],   // 1 -> 2
            vec![(0,0), (1,0), (1,-1), (0,2), (1,2)],    // 2 -> 3
            vec![(0,0), (-1,0), (-1,1), (0,-2), (-1,-2)],// 3 -> 0
        ],
        // Counter-Clockwise
        vec![
            vec![(0,0), (1,0), (1,-1), (0,2), (1,2)],    // 0 -> 3
            vec![(0,0), (1,0), (1,1), (0,-2), (1,-2)],   // 1 -> 0
            vec![(0,0), (-1,0), (-1,-1), (0,2), (-1,2)], // Corrected: 2->1
            vec![(0,0), (-1,0), (-1,1), (0,-2), (-1,-2)], // Corrected: 3->2
        ]
    ];

    static ref I_KICKS: Vec<Vec<Vec<(i32, i32)>>> = vec![
        // Clockwise
        vec![
            vec![(0,0), (-2,0), (1,0), (-2,1), (1,-2)], // 0 -> 1
            vec![(0,0), (-1,0), (2,0), (-1,-2), (2,1)], // 1 -> 2
            vec![(0,0), (2,0), (-1,0), (2,-1), (-1,2)], // 2 -> 3
            vec![(0,0), (1,0), (-2,0), (1,2), (-2,-1)], // 3 -> 0
        ],
        // Counter-Clockwise
        vec![
            vec![(0,0), (-1,0), (2,0), (-1,-2), (2,1)], // 0 -> 3
            vec![(0,0), (2,0), (-1,0), (2,-1), (-1,2)], // 1 -> 0
            vec![(0,0), (1,0), (-2,0), (1,2), (-2,-1)], // 2 -> 1
            vec![(0,0), (-2,0), (1,0), (-2,1), (1,-2)], // 3 -> 2
        ]
    ];
}

fn attempt_rotation(
    tetromino: &Tetromino, playfield: &Playfield, clockwise: bool,
) -> Option<(Vec<(i32, i32)>, i32, i32, usize)> {
    if tetromino.tetromino_type == TetrominoType::O { return None; }
    let current_rotation_index = tetromino.rotation_index;
    let target_rotation_index = if clockwise { (current_rotation_index + 1) % 4 } else { (current_rotation_index + 3) % 4 };
    let target_shape_blocks = get_rotated_shape(tetromino.tetromino_type, target_rotation_index);

    let kick_set_index = if clockwise { 0 } else { 1 };
    // Kick table source rotation index is the current_rotation_index for JLSTZ and I.
    // E.g., for 0->1 (CW), use JLSTZ_KICKS[0][0] or I_KICKS[0][0].
    // For 1->0 (CCW), use JLSTZ_KICKS[1][1] or I_KICKS[1][1]. (This was a source of confusion, the provided tables look like source_rotation_index for both CW/CCW cases)
    // The kick tables are usually indexed by [CW/CCW][SourceRotationState]
    // So if current_rotation_index is 1 and we are rotating CCW (to state 0)
    // we'd use KICKS[CCW_idx (1)][current_rotation_index (1)]
    let kicks_for_rotation = current_rotation_index;

    let relevant_kicks = match tetromino.tetromino_type {
        TetrominoType::I => &I_KICKS[kick_set_index][kicks_for_rotation],
        _ => &JLSTZ_KICKS[kick_set_index][kicks_for_rotation],
    };

    // Test 0: No Kick (already done by SRS standard, but we can start with it)
    // Actually, the standard SRS tests [(0,0)] as the first kick in the list.
    // So, iterate through all kicks including (0,0) as the first.
    for kick_test_idx in 0..relevant_kicks.len() {
        let kick = relevant_kicks[kick_test_idx];
        let new_x = tetromino.x + kick.0;
        let new_y = tetromino.y + kick.1;
        if !check_collision(&target_shape_blocks, playfield, new_x, new_y) {
            return Some((target_shape_blocks, new_x, new_y, target_rotation_index));
        }
    }
    None
}

fn render_text(
    canvas: &mut Canvas<Window>, text: &str, x: i32, y: i32, color: Color,
) -> Result<(), String> {
    let char_width = 5 * 2; // Each pixel is 2x2
    canvas.set_draw_color(color);
    for (i, c) in text.chars().enumerate() {
        if let Some(char_data) = font::get_char_data(c) {
            for (row_idx, &row_data) in char_data.iter().enumerate() {
                for col_idx in 0..5 {
                    if (row_data >> (4 - col_idx)) & 1 == 1 {
                        let px = x + (i as i32 * char_width) + (col_idx * 2);
                        let py = y + (row_idx as i32 * 2);
                        canvas.fill_rect(Rect::new(px, py, 2, 2))?;
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let playfield_render_width = PLAYFIELD_WIDTH as u32 * BLOCK_SIZE;
    let window_width = playfield_render_width + SCORE_TEXT_AREA_WIDTH;
    let window_height = PLAYFIELD_HEIGHT as u32 * BLOCK_SIZE;

    let window = video_subsystem
        .window("Rust Tetris", window_width, window_height)
        .position_centered().build().map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut playfield: Playfield = vec![vec![0; PLAYFIELD_WIDTH]; PLAYFIELD_HEIGHT];
    
    // Initial spawn for current and next tetromino
    let mut current_tetromino = spawn_tetromino(PLAYFIELD_WIDTH);
    // Adjust initial position for the first current_tetromino
    current_tetromino.x = PLAYFIELD_WIDTH as i32 / 2 -1; // Standard spawn X
    current_tetromino.y = match current_tetromino.tetromino_type { // Standard spawn Y
        TetrominoType::T | TetrominoType::S | TetrominoType::Z | TetrominoType::I => 1,
        _ => 0,
    };
    let mut next_tetromino = spawn_tetromino(PLAYFIELD_WIDTH);


    let mut score: u32 = 0;
    let mut total_lines_cleared: u32 = 0;
    let mut current_level: u32 = 1;
    let mut game_over: bool = false;

    let mut lock_delay_timer: Option<SystemTime> = None;
    const LOCK_DELAY_DURATION: Duration = Duration::from_millis(500);

    let mut event_pump = sdl_context.event_pump()?;
    let mut gravity_timer = SystemTime::now();
    let base_gravity_interval_ms = 1000; // Gravity interval for level 1 (milliseconds)

    'running: loop {
        let gravity_interval = Duration::from_millis(
            ((base_gravity_interval_ms as f32 / (current_level as f32 * 0.5 + 0.5)) as u64) // Faster with level
            .max(100) // Minimum interval
        );


        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    match keycode {
                        Keycode::Escape => break 'running,
                        Keycode::Left => {
                            if !game_over {
                                let new_x = current_tetromino.x - 1;
                                if !check_collision(&current_tetromino.blocks, &playfield, new_x, current_tetromino.y) {
                                    current_tetromino.x = new_x;
                                    lock_delay_timer = None; // Reset timer on successful move
                                }
                            }
                        }
                        Keycode::Right => {
                            if !game_over {
                                let new_x = current_tetromino.x + 1;
                                if !check_collision(&current_tetromino.blocks, &playfield, new_x, current_tetromino.y) {
                                    current_tetromino.x = new_x;
                                    lock_delay_timer = None; // Reset timer on successful move
                                }
                            }
                        }
                        Keycode::Down => { // Soft Drop
                            if !game_over {
                                let new_y = current_tetromino.y + 1;
                                if !check_collision(&current_tetromino.blocks, &playfield, current_tetromino.x, new_y) {
                                    current_tetromino.y = new_y;
                                    score += 1; 
                                    gravity_timer = SystemTime::now(); // Reset gravity timer, but not lock delay
                                } else {
                                    // Landed due to soft drop
                                    if lock_delay_timer.is_none() {
                                        lock_delay_timer = Some(SystemTime::now());
                                    }
                                }
                            }
                        }
                        Keycode::Up => { // Clockwise Rotation
                            if !game_over {
                                if let Some((new_blocks, new_x, new_y, new_rotation_index)) =
                                    attempt_rotation(&current_tetromino, &playfield, true) {
                                    current_tetromino.blocks = new_blocks; current_tetromino.x = new_x;
                                    current_tetromino.y = new_y; current_tetromino.rotation_index = new_rotation_index;
                                    lock_delay_timer = None; // Reset timer on successful rotation
                                }
                            }
                        }
                        Keycode::Z | Keycode::LCtrl => { // Counter-Clockwise Rotation
                            if !game_over {
                                 if let Some((new_blocks, new_x, new_y, new_rotation_index)) =
                                    attempt_rotation(&current_tetromino, &playfield, false) {
                                    current_tetromino.blocks = new_blocks; current_tetromino.x = new_x;
                                    current_tetromino.y = new_y; current_tetromino.rotation_index = new_rotation_index;
                                    lock_delay_timer = None; // Reset timer on successful rotation
                                }
                            }
                        }
                        Keycode::Space => { // Hard Drop
                            if !game_over {
                                let mut final_y = current_tetromino.y;
                                while !check_collision(&current_tetromino.blocks, &playfield, current_tetromino.x, final_y + 1) {
                                    final_y += 1;
                                }
                                let cells_dropped = final_y - current_tetromino.y;
                                score += (cells_dropped * 2) as u32; 

                                current_tetromino.y = final_y;
                                lock_piece(&current_tetromino, &mut playfield, current_level, &mut score, &mut total_lines_cleared, &mut current_level);
                                
                                current_tetromino = next_tetromino;
                                current_tetromino.x = PLAYFIELD_WIDTH as i32 / 2 -1; // Standard spawn X
                                current_tetromino.y = match current_tetromino.tetromino_type { // Standard spawn Y
                                    TetrominoType::T | TetrominoType::S | TetrominoType::Z | TetrominoType::I => 1,
                                    _ => 0,
                                };
                                next_tetromino = spawn_tetromino(PLAYFIELD_WIDTH);

                                if check_collision(&current_tetromino.blocks, &playfield, current_tetromino.x, current_tetromino.y) {
                                    game_over = true;
                                    println!("Game Over! Final Score: {}, Lines: {}, Level: {}", score, total_lines_cleared, current_level);
                                }
                                lock_delay_timer = None; // Reset lock timer
                                gravity_timer = SystemTime::now(); 
                            }
                        }
                        Keycode::R => {
                            if game_over {
                                playfield = vec![vec![0; PLAYFIELD_WIDTH]; PLAYFIELD_HEIGHT];
                                score = 0; total_lines_cleared = 0; current_level = 1;
                                
                                current_tetromino = spawn_tetromino(PLAYFIELD_WIDTH);
                                current_tetromino.x = PLAYFIELD_WIDTH as i32 / 2 -1;
                                current_tetromino.y = match current_tetromino.tetromino_type {
                                     TetrominoType::T | TetrominoType::S | TetrominoType::Z | TetrominoType::I => 1, _ => 0,
                                };
                                next_tetromino = spawn_tetromino(PLAYFIELD_WIDTH);

                                game_over = false;
                                lock_delay_timer = None;
                                gravity_timer = SystemTime::now();
                                if check_collision(&current_tetromino.blocks, &playfield, current_tetromino.x, current_tetromino.y) {
                                   game_over = true; 
                                   println!("Immediate Game Over on restart - check spawn logic or playfield clearing.");
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        if !game_over {
            // Lock delay timer logic
            if let Some(timer_started_at) = lock_delay_timer {
                let still_landed = check_collision(&current_tetromino.blocks, &playfield, current_tetromino.x, current_tetromino.y + 1);
                if still_landed && timer_started_at.elapsed().unwrap_or_default() >= LOCK_DELAY_DURATION {
                    lock_piece(&current_tetromino, &mut playfield, current_level, &mut score, &mut total_lines_cleared, &mut current_level);
                    
                    current_tetromino = next_tetromino;
                    current_tetromino.x = PLAYFIELD_WIDTH as i32 / 2 -1; // Standard spawn X
                    current_tetromino.y = match current_tetromino.tetromino_type { // Standard spawn Y
                         TetrominoType::T | TetrominoType::S | TetrominoType::Z | TetrominoType::I => 1,
                         _ => 0,
                    };
                    next_tetromino = spawn_tetromino(PLAYFIELD_WIDTH);

                    if check_collision(&current_tetromino.blocks, &playfield, current_tetromino.x, current_tetromino.y) {
                        game_over = true;
                        println!("Game Over! Final Score: {}, Lines: {}, Level: {}", score, total_lines_cleared, current_level);
                    }
                    lock_delay_timer = None;
                    gravity_timer = SystemTime::now();
                } else if !still_landed {
                    lock_delay_timer = None; // Piece moved out of landed state, cancel timer
                }
            }

            // Gravity
            if gravity_timer.elapsed().unwrap_or_default() >= gravity_interval {
                let new_y = current_tetromino.y + 1;
                if !check_collision(&current_tetromino.blocks, &playfield, current_tetromino.x, new_y) {
                    current_tetromino.y = new_y;
                    // Piece moved down due to gravity, if it was about to lock, this doesn't cancel the lock timer
                    // unless it's no longer in a "landed" state (handled by the !still_landed check above)
                } else {
                    // Landed due to gravity
                    if lock_delay_timer.is_none() {
                        lock_delay_timer = Some(SystemTime::now());
                    }
                }
                gravity_timer = SystemTime::now();
            }
        }

        canvas.set_draw_color(Color::RGB(20, 20, 30)); // Main window background
        canvas.clear();

        let playfield_bg_rect = Rect::new(0,0, playfield_render_width, window_height);
        canvas.set_draw_color(Color::RGB(40, 40, 50)); // Playfield background
        canvas.fill_rect(playfield_bg_rect)?;

        canvas.set_draw_color(Color::RGB(80, 80, 90)); // Grid lines color
        for x_grid in 0..=PLAYFIELD_WIDTH {
            canvas.draw_line( (x_grid as i32 * BLOCK_SIZE as i32, 0), (x_grid as i32 * BLOCK_SIZE as i32, window_height as i32))?;
        }
        for y_grid in 0..=PLAYFIELD_HEIGHT {
            canvas.draw_line( (0, y_grid as i32 * BLOCK_SIZE as i32), (playfield_render_width as i32, y_grid as i32 * BLOCK_SIZE as i32))?;
        }

        for (r, row) in playfield.iter().enumerate() {
            for (c, &cell_value) in row.iter().enumerate() {
                if cell_value != 0 {
                    let main_color = get_locked_block_color(cell_value);
                    draw_block_with_border(
                        &mut canvas,
                        (c as u32 * BLOCK_SIZE) as i32,
                        (r as u32 * BLOCK_SIZE) as i32,
                        BLOCK_SIZE,
                        main_color
                    )?;
                }
            }
        }
        
        if !game_over {
            let main_color = get_tetromino_color(current_tetromino.tetromino_type);
            for &(block_dx, block_dy) in &current_tetromino.blocks {
                let px = current_tetromino.x + block_dx;
                let py = current_tetromino.y + block_dy;
                if px >= 0 && px < PLAYFIELD_WIDTH as i32 && py >= 0 && py < PLAYFIELD_HEIGHT as i32 {
                    draw_block_with_border(
                        &mut canvas,
                        px * BLOCK_SIZE as i32,
                        py * BLOCK_SIZE as i32,
                        BLOCK_SIZE,
                        main_color
                    )?;
                }
            }
        }

        // --- Render Score, Lines, Level, Next Piece ---
        let text_x = playfield_render_width as i32 + 20;
        let mut current_text_y = 30; // Starting Y for the text elements
        let line_spacing = 30;
        let ui_element_spacing = 40; // Space between major UI elements like "Level" and "Next"

        const PALE_GOLD_COLOR: Color = Color::RGB(230, 210, 160);

        render_text(&mut canvas, &format!("Score: {}", score), text_x, current_text_y, PALE_GOLD_COLOR)?;
        current_text_y += line_spacing;
        render_text(&mut canvas, &format!("Lines: {}", total_lines_cleared), text_x, current_text_y, PALE_GOLD_COLOR)?;
        current_text_y += line_spacing;
        render_text(&mut canvas, &format!("Level: {}", current_level), text_x, current_text_y, PALE_GOLD_COLOR)?;
        
        current_text_y += ui_element_spacing; // Add space before "Next" piece display

        // "Next" Piece Display
        render_text(&mut canvas, "Next:", text_x, current_text_y, PALE_GOLD_COLOR)?;
        current_text_y += line_spacing; // Space for the label itself

        let next_piece_box_x = text_x;
        let next_piece_box_y = current_text_y;
        let next_piece_box_width = 4 * BLOCK_SIZE; // Approx size for a 4-block wide piece
        let next_piece_box_height = 4 * BLOCK_SIZE;

        // Draw a background box for the next piece (optional)
        canvas.set_draw_color(Color::RGB(30, 30, 40)); // Next piece box background
        canvas.fill_rect(Rect::new(next_piece_box_x, next_piece_box_y, next_piece_box_width, next_piece_box_height))?;
        
        // Render the next_tetromino
        // Centering logic: Try to center the piece within the 4x4 block box.
        // This requires knowing the piece's general shape/bounds.
        // A simple heuristic: find min/max x/y of the piece's blocks.
        let mut min_bx = 2; let mut max_bx = -2;
        let mut min_by = 2; let mut max_by = -2;
        for &(bx, by) in &next_tetromino.blocks {
            if bx < min_bx { min_bx = bx; } if bx > max_bx { max_bx = bx; }
            if by < min_by { min_by = by; } if by > max_by { max_by = by; }
        }
        let piece_width_blocks = max_bx - min_bx + 1;
        let piece_height_blocks = max_by - min_by + 1;

        // Calculate offset to center the piece in the display box (BLOCK_SIZE units)
        // The display box is conceptually 4x4 blocks.
        let offset_x = ( (4 - piece_width_blocks) as f32 / 2.0 - min_bx as f32 ) * BLOCK_SIZE as f32;
        let offset_y = ( (4 - piece_height_blocks) as f32 / 2.0 - min_by as f32) * BLOCK_SIZE as f32;


        let main_color = get_tetromino_color(next_tetromino.tetromino_type);
        for &(block_dx, block_dy) in &next_tetromino.blocks {
            let render_x = next_piece_box_x + (block_dx * BLOCK_SIZE as i32) + offset_x as i32;
            let render_y = next_piece_box_y + (block_dy * BLOCK_SIZE as i32) + offset_y as i32;
            
            // Ensure blocks are drawn within the box for tidiness, though centering should help
            if render_x >= next_piece_box_x && render_x < next_piece_box_x + next_piece_box_width as i32 &&
               render_y >= next_piece_box_y && render_y < next_piece_box_y + next_piece_box_height as i32 {
                draw_block_with_border(
                    &mut canvas,
                    render_x,
                    render_y,
                    BLOCK_SIZE,
                    main_color
                )?;
            }
        }


        if game_over {
        let game_over_x = playfield_render_width as i32 / 2 - 100;
        let game_over_y = window_height as i32 / 2 - 60;
            let overlay_rect = Rect::new(game_over_x - 20, game_over_y - 20, 240, 140); // Adjusted width for potentially longer score text
            canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
            canvas.set_draw_color(Color::RGBA(10, 10, 20, 180)); // Game Over overlay
            canvas.fill_rect(overlay_rect)?;
            render_text(&mut canvas, "GAME OVER", game_over_x, game_over_y, Color::RED)?; // GAME OVER title remains Red
            render_text(&mut canvas, &format!("Final Score: {}", score), game_over_x, game_over_y + line_spacing, PALE_GOLD_COLOR)?;
            render_text(&mut canvas, "Press 'R' to Restart", game_over_x, game_over_y + 2 * line_spacing, PALE_GOLD_COLOR)?;
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}

// Helper function to draw a single block with a border
fn draw_block_with_border(
    canvas: &mut Canvas<Window>,
    x: i32,
    y: i32,
    block_size: u32,
    main_color: Color,
) -> Result<(), String> {
    let border_thickness = 2; // Adjust for desired thickness

    // Calculate darker shade for the border
    let border_color = Color::RGB(
        main_color.r.saturating_sub(50), // Increased difference for more contrast
        main_color.g.saturating_sub(50),
        main_color.b.saturating_sub(50),
    );

    // Draw the outer rectangle (the border)
    canvas.set_draw_color(border_color);
    canvas.fill_rect(Rect::new(x, y, block_size, block_size))?;

    // Draw the inner, slightly smaller rectangle with the main color
    // Ensure block_size is greater than 2 * border_thickness to avoid underflow
    if block_size > 2 * border_thickness {
        canvas.set_draw_color(main_color);
        canvas.fill_rect(Rect::new(
            x + border_thickness as i32,
            y + border_thickness as i32,
            block_size - (2 * border_thickness),
            block_size - (2 * border_thickness),
        ))?;
    } else {
        // If block_size is too small, just draw the main color block without a border or a very thin one
        canvas.set_draw_color(main_color);
        canvas.fill_rect(Rect::new(x, y, block_size, block_size))?;
    }

    Ok(())
}
