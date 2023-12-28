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
    use sdl2::{EventPump, Sdl, TimerSubsystem, VideoSubsystem};
    use crate::random::get_random_int;

    use super::sprite::*;

    pub fn run () {
        let mut sdl_components = SdlComponents::init();

        let mut past = TimerSubsystem::ticks64(&sdl_components.timer_subsystem);
        let mut now;
        let mut past_fps = past;
        let mut fps = 0;
        let mut frames_skipped = 0;

        let mut key_downs = KeyDowns::new();

        let sprites: &mut Vec<Sprite> = &mut Vec::new();
        add_sprite(sprites, SpriteType::PLAYER, &mut sdl_components);
        add_sprite(sprites, SpriteType::ENEMY, &mut sdl_components);

        'running: loop {
            let mut time_elapsed = 0;

            for event in sdl_components.event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                    _ => {}
                }

                handle_key_events(event, &mut key_downs);
            }

            update_sprites(sprites, &mut key_downs, &mut sdl_components);

            sdl_components.canvas.set_draw_color(Color::RGB(255, 255, 255));
            sdl_components.canvas.clear();

            draw(&mut sdl_components.canvas, sprites);

            now = TimerSubsystem::ticks64(&sdl_components.timer_subsystem);
            time_elapsed = now - past;

            if time_elapsed >= (1000 / 60) {
                past = now;

                if frames_skipped + 1 >= 0 {
                    sdl_components.canvas.present();
                    fps += 1;
                    frames_skipped = 0;
                }
            }

            if now - past_fps >= 1000 {
                past_fps = now;
                SdlComponents::update_window_title(&mut sdl_components, fps.to_string().as_str());
                fps = 0;
            }

            std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }

    pub fn add_sprite (sprites: &mut Vec<Sprite>, sprite_type: SpriteType, sdl_components: &mut SdlComponents) {
        let (mut window_width,mut window_height) = sdl_components.canvas.output_size().unwrap();
        let window_width: i32 = window_width as i32;
        let window_height: i32 = window_height as i32;

        sprites.push(Sprite{
            x: get_random_int(0, window_width),
            y: get_random_int(0, window_height),
            sprite_type,
            red: get_random_int(0, 255) as u8,
            green: get_random_int(0, 255) as u8,
            blue: get_random_int(0, 255) as u8,
        });
    }

    pub fn draw (canvas: &mut WindowCanvas, sprites: &Vec<Sprite>) {
        let rects: &mut Vec<Rect> = &mut Vec::new();
        for sprite in sprites {
            let rect = Rect::new(sprite.x, sprite.y, 20, 20);

            rects.push(rect);

            canvas.set_draw_color(Color::RGB(sprite.red, sprite.green, sprite.blue));
            canvas.fill_rect(rect).expect("Failed to draw rectangle");
        }
    }

    pub fn update_sprites (sprites: &mut Vec<Sprite>, key_downs: &mut KeyDowns, sdl_components: &mut SdlComponents) {
        let (mut window_width,mut window_height) = sdl_components.canvas.output_size().unwrap();
        let window_width: i32 = window_width as i32;
        let window_height: i32 = window_height as i32;

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

    struct SdlComponents {
        sdl_context: Sdl,
        video_subsystem: VideoSubsystem,
        canvas: WindowCanvas,
        event_pump: EventPump,
        timer_subsystem: TimerSubsystem
    }

    impl SdlComponents {
        fn init () -> SdlComponents {
            let sdl_context = sdl2::init().unwrap();
            let video_subsystem = sdl_context.video().unwrap();
            let window = video_subsystem.window("hi", 800, 600)
                .position_centered()
                .build()
                .unwrap();
            let mut canvas = window.into_canvas().build().unwrap();
            let event_pump = sdl_context.event_pump().unwrap();
            let timer_subsystem: TimerSubsystem = sdl_context.timer().unwrap();

            SdlComponents{
                sdl_context,
                video_subsystem,
                canvas,
                event_pump,
                timer_subsystem
            }
        }

        fn update_window_title (&mut self, title: &str) {
            self.canvas.window_mut().set_title(title).expect("could not set title");
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
}