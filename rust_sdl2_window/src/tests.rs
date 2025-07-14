use super::*;
use sdl2::rect::Rect;
use sdl2::pixels::Color;

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
    let tetromino = spawn_tetromino(10);
    assert!(tetromino.x >= 0 && tetromino.x < 10);
    assert!(tetromino.y >= 0);
}

#[test]
fn test_check_collision() {
    let mut playfield = vec![vec![0; 10]; 20];
    let tetromino = spawn_tetromino(10);
    assert_eq!(check_collision(&tetromino.blocks, &playfield, tetromino.x, tetromino.y), false);
    playfield[10][5] = 1;
    assert_eq!(check_collision(&tetromino.blocks, &playfield, 5, 10), true);
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
