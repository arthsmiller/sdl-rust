pub fn main() {
    engine::run();
}

mod engine {
    use std::collections::HashMap;
    use std::time::Duration;

    use sdl2::event::Event;
    use sdl2::keyboard::Keycode;
    use sdl2::pixels::Color;
    use sdl2::rect::Rect;
    use sdl2::render::WindowCanvas;
    use sdl2::TimerSubsystem;
    use crate::random::{get_random_color, get_random_position};

    use super::sprite::*;

    pub fn run () {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let mut window = video_subsystem.window("hi", 800, 600)
            .position_centered()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();
        let timer_subsystem: TimerSubsystem = sdl_context.timer().unwrap();

        let mut past = TimerSubsystem::ticks64(&timer_subsystem);
        let mut now;
        let mut past_fps = past;
        let mut fps = 0;
        let mut frames_skipped = 0;

        let mut key_downs = KeyDowns::new();

        let (mut window_width,mut window_height) = canvas.output_size().unwrap();
        let window_width: i32 = window_width as i32;
        let window_height: i32 = window_height as i32;

        let sprites: &mut Vec<Sprite> = &mut Vec::new();
        add_sprite(sprites, SpriteType::PLAYER, Some(window_width), Some(window_height));
        add_sprite(sprites, SpriteType::ENEMY, Some(window_width), Some(window_height));

        'running: loop {
            let mut time_elapsed = 0;

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                    _ => {}
                }

                handle_key_events(event, &mut key_downs);
            }

            let (mut window_width,mut window_height) = canvas.output_size().unwrap();
            let window_width: i32 = window_width as i32;
            let window_height: i32 = window_height as i32;

            update_sprites(sprites, &mut key_downs, window_width, window_height);

            canvas.set_draw_color(Color::RGB(255, 255, 255));
            canvas.clear();

            draw(&mut canvas, sprites);

            now = TimerSubsystem::ticks64(&timer_subsystem);
            time_elapsed = now - past;

            if now - past_fps >= 1000 {
                past_fps = now;
                // todo fps counter
                fps = 0;
            }

            if time_elapsed >= (1000 / 60) {
                past = now;

                if frames_skipped +1 >= 0 {
                    canvas.present();

                    fps += 1;
                    frames_skipped = 0;
                }
            }

            std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }

    pub fn add_sprite (
        sprites: &mut Vec<Sprite>,
        sprite_type: SpriteType,
        x: Option<i32>,
        y: Option<i32>,
        red: Option<u8>,
        green: Option<u8>,
        blue: Option<u8>,
    ) {
        let (x_pos, y_pos) = (x.unwrap_or())

        if x > 0 || y > 0  {
            let (x_pos, y_pos) = get_random_position(x, y);
        }
        else {
            let x_pos = x;
            let y_pos = y;
        }

        let color_red = if red == 0 {  }

        sprites.push(Sprite{
            x: x_pos,
            y: y_pos,
            sprite_type,
            red: get_random_color(),
            green: get_random_color(),
            blue: get_random_color()
        });
    }

    pub fn draw (canvas: &mut WindowCanvas, sprites: &Vec<Sprite>) {
        let rects: &mut Vec<Rect> = &mut Vec::new();
        for sprite in sprites {
            let mut rect = Rect::new(0, 0, 20, 20);
            rect.x = sprite.x;
            rect.y = sprite.y;

            rects.push(rect);

            canvas.set_draw_color(Color::RGB(sprite.red, sprite.green, sprite.blue));
            canvas.fill_rect(rect).expect("Failed to draw rectangle");
        }
    }

    pub fn update_sprites (
        sprites: &mut Vec<Sprite>,
        key_downs: &mut KeyDowns,
        window_width: i32,
        window_height: i32,
    ){
        for sprite in sprites {
            return_sprite_to_canvas(sprite, window_width, window_height);

            if sprite.sprite_type == SpriteType::PLAYER {
                if key_downs.is_key_down(Keycode::Up) { sprite.y -= 10 }
                if key_downs.is_key_down(Keycode::Down) { sprite.y += 10 }
                if key_downs.is_key_down(Keycode::Left) { sprite.x -= 10 }
                if key_downs.is_key_down(Keycode::Right) { sprite.x += 10 }
            }
        }
    }

    pub fn return_sprite_to_canvas(sprite: &mut Sprite, window_width: i32, window_height: i32) {
        if sprite.x > window_width {
            sprite.x = 0
        }
        else if sprite.x < 0 {
            sprite.x = window_width
        }
        else if sprite.y > window_height {
            sprite.y = 0
        }
        else if sprite.y < 0 {
            sprite.y = window_height
        }
    }

    struct KeyDowns {
        keys: HashMap<Keycode, bool>,
    }

    impl KeyDowns {
        fn new() -> Self {
            KeyDowns { keys: HashMap::new() }
        }

        fn set_key(&mut self, key: Keycode, value: bool) {
            self.keys.insert(key, value);
        }

        fn is_key_down(&self, key: Keycode) -> bool {
            *self.keys.get(&key).unwrap_or(&false)
        }
    }

    fn handle_key_events(event: Event, key_downs: &mut KeyDowns) {
        match event {
            Event::KeyDown { keycode: Some(keycode), .. } => {
                key_downs.set_key(keycode, true);
            },
            Event::KeyUp { keycode: Some(keycode), .. } => {
                key_downs.set_key(keycode, false);
            },
            _ => {}
        }
    }
}

pub mod sprite {
    pub struct Sprite {
        pub x: i32,
        pub y: i32,
        pub red: u8,
        pub green: u8,
        pub blue: u8,
        pub sprite_type: SpriteType,
    }

    #[derive(PartialEq)]
    pub enum SpriteType {
        PLAYER,
        ENEMY,
        DEFAULT,
    }
}

pub mod random {
    use rand::Rng;

    pub fn get_random_int (min: i32, max: i32) -> i32 {
        let mut rng = rand::thread_rng();
        rng.gen_range(min..max)
    }

    pub fn get_random_position(x: i32, y: i32) -> (i32, i32) {
        let mut rng = rand::thread_rng();
        (rng.gen_range(0..x), rng.gen_range(0..y))
    }

    pub fn get_random_color() -> u8 {
        let mut rng = rand::thread_rng();
        rng.gen_range(0..255)
    }
}