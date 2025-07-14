use super::*;

struct MockCanvas {
    rects: Vec<Rect>,
}

impl CanvasTrait for MockCanvas {
    fn set_draw_color(&mut self, _color: Color) {}

    fn fill_rect(&mut self, rect: Rect) -> Result<(), String> {
        self.rects.push(rect);
        Ok(())
    }
}

#[test]
fn test_spawn_tetromino() {
    let tetromino = spawn_tetromino(PLAYFIELD_WIDTH);

    // Verify that the tetromino is spawned within the playfield boundaries
    assert!(tetromino.x >= 0 && tetromino.x < PLAYFIELD_WIDTH as i32);
    assert!(tetromino.y >= 0 && tetromino.y < PLAYFIELD_HEIGHT as i32);

    // Verify that the tetromino has a valid type
    assert!(tetromino.tetromino_type as u8 >= 1 && tetromino.tetromino_type as u8 <= 7);

    // Verify that the tetromino has the correct initial rotation
    assert_eq!(tetromino.rotation_index, 0);
}

#[test]
fn test_collision_detection() {
    let mut playfield = vec![vec![0; PLAYFIELD_WIDTH]; PLAYFIELD_HEIGHT];
    let tetromino = spawn_tetromino(PLAYFIELD_WIDTH);

    // Test collision with the left boundary
    assert!(check_collision(&tetromino.blocks, &playfield, -1, tetromino.y));

    // Test collision with the right boundary
    assert!(check_collision(&tetromino.blocks, &playfield, PLAYFIELD_WIDTH as i32, tetromino.y));

    // Test collision with the bottom boundary
    assert!(check_collision(&tetromino.blocks, &playfield, tetromino.x, PLAYFIELD_HEIGHT as i32));

    // Test collision with another tetromino
    playfield[5][5] = 1;
    let mut colliding_tetromino = spawn_tetromino(PLAYFIELD_WIDTH);
    colliding_tetromino.x = 5;
    colliding_tetromino.y = 5;
    assert!(check_collision(&colliding_tetromino.blocks, &playfield, colliding_tetromino.x, colliding_tetromino.y));
}

#[test]
fn test_line_clearing_and_scoring() {
    let mut playfield = vec![vec![0; PLAYFIELD_WIDTH]; PLAYFIELD_HEIGHT];
    let mut score = 0;
    let mut total_lines_cleared = 0;
    let mut game_level = 1;

    // Fill one line completely, except for one block
    for i in 0..PLAYFIELD_WIDTH - 1 {
        playfield[PLAYFIELD_HEIGHT - 1][i] = 1;
    }

    let tetromino = Tetromino {
        tetromino_type: TetrominoType::I,
        rotation_index: 0,
        blocks: vec![(0, 0)],
        x: (PLAYFIELD_WIDTH - 1) as i32,
        y: (PLAYFIELD_HEIGHT - 1) as i32,
    };

    lock_piece(&tetromino, &mut playfield, 1, &mut score, &mut total_lines_cleared, &mut game_level);

    // After locking the piece, one line should be cleared
    assert_eq!(total_lines_cleared, 1);
    assert_eq!(score, 100);

    // Verify that the line has been cleared
    for i in 0..PLAYFIELD_WIDTH {
        assert_eq!(playfield[PLAYFIELD_HEIGHT - 1][i], 0);
    }
}


struct MockCanvas {
    draw_color: Color,
    rects: Vec<Rect>,
}

impl MockCanvas {
    fn new() -> Self {
        MockCanvas {
            draw_color: Color::RGB(0, 0, 0),
            rects: Vec::new(),
        }
    }
}

impl CanvasTrait for MockCanvas {
    fn set_draw_color(&mut self, color: Color) {
        self.draw_color = color;
    }

    fn fill_rect(&mut self, rect: Rect) -> Result<(), String> {
        self.rects.push(rect);
        Ok(())
    }
}

#[test]
fn test_render_text_with_mock_canvas() {
    let mut mock_canvas = MockCanvas::new();
    let text_to_render = "A";
    let text_color = Color::RGB(255, 255, 255);

    let result = render_text(&mut mock_canvas, text_to_render, 0, 0, text_color);
    assert!(result.is_ok());

    assert_eq!(mock_canvas.rects.len(), 17);
}

#[test]
fn test_render_multi_character_text_with_mock_canvas() {
    let mut mock_canvas = MockCanvas::new();
    let text_to_render = "MWmwpyqg_^";
    let text_color = Color::RGB(255, 255, 255);

    let result = render_text(&mut mock_canvas, text_to_render, 0, 0, text_color);
    assert!(result.is_ok());

    let mut expected_rects = Vec::new();
    let char_width = 6 * 2;
    for (i, c) in text_to_render.chars().enumerate() {
        if let Some(char_data) = font::get_char_data(c) {
            for (row_idx, &row_data) in char_data.iter().enumerate() {
                for col_idx in 0..5 {
                    if (row_data >> (4 - col_idx)) & 1 == 1 {
                        let px = (i as i32 * char_width) + (col_idx as i32 * 2);
                        let py = row_idx as i32 * 2;
                        expected_rects.push(Rect::new(px, py, 2, 2));
                    }
                }
            }
        }
    }

    assert_eq!(mock_canvas.rects.len(), expected_rects.len());
    for rect in &expected_rects {
        assert!(mock_canvas.rects.contains(rect));
    }
}

#[test]
fn test_rotation_and_wall_kick() {
    let playfield = vec![vec![0; PLAYFIELD_WIDTH]; PLAYFIELD_HEIGHT];
    let mut tetromino = spawn_tetromino(PLAYFIELD_WIDTH);
    tetromino.tetromino_type = TetrominoType::I;
    tetromino.x = 0;
    tetromino.y = 0;

    // Test simple rotation
    let rotation_result = attempt_rotation(&tetromino, &playfield, true);
    assert!(rotation_result.is_some());
    let (_new_blocks, _new_x, _new_y, new_rotation_index) = rotation_result.unwrap();
    assert_eq!(new_rotation_index, 1);

    // Test wall kick
    tetromino.x = -1;
    let wall_kick_result = attempt_rotation(&tetromino, &playfield, true);
    assert!(wall_kick_result.is_some());
}

#[cfg(test)]
mod rotation_tests {
    use super::*;

    fn assert_shape(actual: &[(i32, i32)], expected: &[(i32, i32)]) {
        let mut actual_sorted = actual.to_vec();
        actual_sorted.sort();
        let mut expected_sorted = expected.to_vec();
        expected_sorted.sort();
        assert_eq!(actual_sorted, expected_sorted);
    }

    #[test]
    fn test_i_rotations() {
        let piece_type = TetrominoType::I;
        assert_shape(&get_rotated_shape(piece_type, 0), &vec![(0,0), (-1,0), (1,0), (2,0)]);
        assert_shape(&get_rotated_shape(piece_type, 1), &vec![(0,0), (0,-1), (0,1), (0,2)]);
        assert_shape(&get_rotated_shape(piece_type, 2), &vec![(0,0), (-1,0), (1,0), (2,0)]);
        assert_shape(&get_rotated_shape(piece_type, 3), &vec![(0,0), (0,-1), (0,1), (0,2)]);
    }

    #[test]
    fn test_o_rotations() {
        let piece_type = TetrominoType::O;
        assert_shape(&get_rotated_shape(piece_type, 0), &vec![(0,0), (1,0), (0,1), (1,1)]);
        assert_shape(&get_rotated_shape(piece_type, 1), &vec![(0,0), (1,0), (0,1), (1,1)]);
        assert_shape(&get_rotated_shape(piece_type, 2), &vec![(0,0), (1,0), (0,1), (1,1)]);
        assert_shape(&get_rotated_shape(piece_type, 3), &vec![(0,0), (1,0), (0,1), (1,1)]);
    }

    #[test]
    fn test_t_rotations() {
        let piece_type = TetrominoType::T;
        assert_shape(&get_rotated_shape(piece_type, 0), &vec![(0,0), (-1,0), (1,0), (0,-1)]);
        assert_shape(&get_rotated_shape(piece_type, 1), &vec![(0,0), (0,-1), (0,1), (1,0)]);
        assert_shape(&get_rotated_shape(piece_type, 2), &vec![(0,0), (-1,0), (1,0), (0,1)]);
        assert_shape(&get_rotated_shape(piece_type, 3), &vec![(0,0), (0,-1), (0,1), (-1,0)]);
    }

    #[test]
    fn test_s_rotations() {
        let piece_type = TetrominoType::S;
        assert_shape(&get_rotated_shape(piece_type, 0), &vec![(0,0), (-1,0), (0,1), (1,1)]);
        assert_shape(&get_rotated_shape(piece_type, 1), &vec![(0,0), (0,-1), (1,0), (1,1)]);
        assert_shape(&get_rotated_shape(piece_type, 2), &vec![(0,0), (-1,0), (0,1), (1,1)]);
        assert_shape(&get_rotated_shape(piece_type, 3), &vec![(0,0), (0,-1), (1,0), (1,1)]);
    }

    #[test]
    fn test_z_rotations() {
        let piece_type = TetrominoType::Z;
        assert_shape(&get_rotated_shape(piece_type, 0), &vec![(0,0), (1,0), (0,1), (-1,1)]);
        assert_shape(&get_rotated_shape(piece_type, 1), &vec![(0,0), (0,1), (1,0), (1,-1)]);
        assert_shape(&get_rotated_shape(piece_type, 2), &vec![(0,0), (1,0), (0,1), (-1,1)]);
        assert_shape(&get_rotated_shape(piece_type, 3), &vec![(0,0), (0,1), (1,0), (1,-1)]);
    }

    #[test]
    fn test_j_rotations() {
        let piece_type = TetrominoType::J;
        assert_shape(&get_rotated_shape(piece_type, 0), &vec![(0,0), (-1,0), (1,0), (1,-1)]);
        assert_shape(&get_rotated_shape(piece_type, 1), &vec![(0,0), (0,-1), (0,1), (1,1)]);
        assert_shape(&get_rotated_shape(piece_type, 2), &vec![(0,0), (-1,0), (1,0), (-1,1)]);
        assert_shape(&get_rotated_shape(piece_type, 3), &vec![(0,0), (0,-1), (0,1), (-1,-1)]);
    }

    #[test]
    fn test_l_rotations() {
        let piece_type = TetrominoType::L;
        assert_shape(&get_rotated_shape(piece_type, 0), &vec![(0,0), (-1,0), (1,0), (-1,-1)]);
        assert_shape(&get_rotated_shape(piece_type, 1), &vec![(0,0), (0,-1), (0,1), (1,-1)]);
        assert_shape(&get_rotated_shape(piece_type, 2), &vec![(0,0), (-1,0), (1,0), (1,1)]);
        assert_shape(&get_rotated_shape(piece_type, 3), &vec![(0,0), (0,-1), (0,1), (-1,1)]);
    }
}

#[test]
fn test_render_text() {
    let mut mock_canvas = MockCanvas { rects: vec![] };
    render_text(&mut mock_canvas, "A", 0, 0, Color::RGB(255, 255, 255), 1).unwrap();

    let expected_rects = vec![
        Rect::new(1, 0, 1, 1), Rect::new(2, 0, 1, 1), Rect::new(3, 0, 1, 1),
        Rect::new(0, 1, 1, 1), Rect::new(4, 1, 1, 1),
        Rect::new(0, 2, 1, 1), Rect::new(4, 2, 1, 1),
        Rect::new(0, 3, 1, 1), Rect::new(1, 3, 1, 1), Rect::new(2, 3, 1, 1), Rect::new(3, 3, 1, 1), Rect::new(4, 3, 1, 1),
        Rect::new(0, 4, 1, 1), Rect::new(4, 4, 1, 1),
        Rect::new(0, 5, 1, 1), Rect::new(4, 5, 1, 1),
        Rect::new(0, 6, 1, 1), Rect::new(4, 6, 1, 1),
    ];

    assert_eq!(mock_canvas.rects.len(), expected_rects.len());
    for rect in &expected_rects {
        assert!(mock_canvas.rects.contains(rect));
    }
}
