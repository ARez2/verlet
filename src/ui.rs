use macroquad::{prelude::*, ui::{hash, Id, Ui}};


pub fn color_picker_texture(w: usize, h: usize) -> (Texture2D, Image) {
    let ratio = 1.0 / h as f32;

    let mut image = Image::gen_image_color(w as u16, h as u16, WHITE);
    let image_data = image.get_image_data_mut();

    for j in 0..h {
        for i in 0..w {
            let lightness = 1.0 - i as f32 * ratio;
            let hue = j as f32 * ratio;

            image_data[i + j * w] = macroquad::color::hsl_to_rgb(hue, 1.0, lightness).into();
        }
    }

    (Texture2D::from_image(&image), image)
}

fn color_picker(ui: &mut Ui, id: Id, data: &mut Color, color_picker_texture: Texture2D) -> bool {
    let is_mouse_captured = ui.is_mouse_captured();

    let mut canvas = ui.canvas();
    let cursor = canvas.request_space(Vec2::new(200., 220.));
    let mouse = mouse_position();

    let x = mouse.0 as i32 - cursor.x as i32;
    let y = mouse.1 as i32 - (cursor.y as i32 + 20);

    if x > 0 && x < 200 && y > 0 && y < 200 {
        let ratio = 1.0 / 200.0 as f32;
        let lightness = 1.0 - x as f32 * ratio;
        let hue = y as f32 * ratio;

        if is_mouse_button_down(MouseButton::Left) && is_mouse_captured == false {
            *data = macroquad::color::hsl_to_rgb(hue, 1.0, lightness).into();
        }
    }

    canvas.rect(
        Rect::new(cursor.x - 5.0, cursor.y - 5.0, 210.0, 395.0),
        Color::new(0.7, 0.7, 0.7, 1.0),
        Color::new(0.9, 0.9, 0.9, 1.0),
    );

    canvas.rect(
        Rect::new(cursor.x, cursor.y, 200.0, 18.0),
        Color::new(0.0, 0.0, 0.0, 1.0),
        Color::new(data.r, data.g, data.b, 1.0),
    );
    canvas.image(
        Rect::new(cursor.x, cursor.y + 20.0, 200.0, 200.0),
        &color_picker_texture,
    );

    let (h, _, l) = macroquad::color::rgb_to_hsl(*data);

    canvas.rect(
        Rect::new(
            cursor.x + (1.0 - l) * 200.0 - 3.5,
            cursor.y + h * 200. + 20.0 - 3.5,
            7.0,
            7.0,
        ),
        Color::new(0.3, 0.3, 0.3, 1.0),
        Color::new(1.0, 1.0, 1.0, 1.0),
    );

    ui.slider(hash!(id, "alpha"), "Alpha", 0.0..1.0, &mut data.a);
    ui.separator();
    ui.slider(hash!(id, "red"), "Red", 0.0..1.0, &mut data.r);
    ui.slider(hash!(id, "green"), "Green", 0.0..1.0, &mut data.g);
    ui.slider(hash!(id, "blue"), "Blue", 0.0..1.0, &mut data.b);
    ui.separator();
    let (mut h, mut s, mut l) = macroquad::color::rgb_to_hsl(*data);
    ui.slider(hash!(id, "hue"), "Hue", 0.0..1.0, &mut h);
    ui.slider(hash!(id, "saturation"), "Saturation", 0.0..1.0, &mut s);
    ui.slider(hash!(id, "lightess"), "Lightness", 0.0..1.0, &mut l);
    let Color { r, g, b, .. } = macroquad::color::hsl_to_rgb(h, s, l);
    data.r = r;
    data.g = g;
    data.b = b;

    ui.separator();
    if ui.button(None, "    ok    ")
        || is_key_down(KeyCode::Escape)
        || is_key_down(KeyCode::Enter)
        || (is_mouse_button_pressed(MouseButton::Left)
            && Rect::new(cursor.x - 10., cursor.y - 10.0, 230., 420.)
                .contains(vec2(mouse.0, mouse.1))
                == false)
    {
        return true;
    }

    false
}

pub fn colorbox(ui: &mut Ui, id: Id, label: &str, data: &mut Color, color_picker_texture: Texture2D) {
    ui.label(None, label);
    let mut canvas = ui.canvas();
    let cursor = canvas.cursor();

    canvas.rect(
        Rect::new(cursor.x + 20.0, cursor.y, 50.0, 18.0),
        Color::new(0.2, 0.2, 0.2, 1.0),
        Color::new(data.r, data.g, data.b, 1.0),
    );
    if ui.last_item_clicked() {
        *ui.get_bool(hash!(id, "color picker opened")) ^= true;
    }
    if *ui.get_bool(hash!(id, "color picker opened")) {
        ui.popup(hash!(id, "color popup"), Vec2::new(200., 400.), |ui| {
            if color_picker(ui, id, data, color_picker_texture) {
                *ui.get_bool(hash!(id, "color picker opened")) = false;
            }
        });
    }
}