use macroquad::prelude::*;

pub fn draw_help_overlay() {
    let position_x = screen_width() - 310.0;
    let position_y = 15.0;
    let width = 295.0;
    let height = 250.0;

    draw_rectangle(
        position_x,
        position_y,
        width,
        height,
        Color::new(0.02, 0.02, 0.05, 0.85),
    );
    draw_rectangle_lines(
        position_x,
        position_y,
        width,
        height,
        2.0,
        Color::new(0.3, 0.5, 0.9, 0.8),
    );

    let font_size = 15.0;
    let line_height = 20.0;
    let mut current_y = position_y + 25.0;

    draw_text(
        "EDITOR CONTROLS HUD",
        position_x + 12.0,
        current_y,
        16.0,
        GOLD,
    );
    current_y += line_height + 5.0;

    let keybinds = [
        ("WASD / Arrows", "Pan Camera"),
        ("Mouse Scroll", "Zoom to Cursor"),
        ("Left Ctrl + Scroll", "Rotate Start Grid 5°"),
        ("Middle Drag", "Pan Canvas"),
        ("Key [ F ]", "Focus Track Center"),
        ("Left Click", "Draw / Place / Drag"),
        ("Right Click", "Quick Delete Node"),
        ("Ctrl + Z / Y", "Undo / Redo"),
    ];

    for (key, action) in keybinds {
        draw_text(key, position_x + 12.0, current_y, font_size, WHITE);
        draw_text(action, position_x + 145.0, current_y, font_size, LIGHTGRAY);
        current_y += line_height;
    }
}
