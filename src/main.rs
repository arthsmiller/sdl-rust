pub fn main() {
    engine::run();
}

pub mod engine {
    use std::collections::HashMap;
    use std::time::Duration;

    use sdl2::event::Event;
    use sdl2::keyboard::Keycode;
    use sdl2::pixels::Color;
    use sdl2::rect::Rect;
    use sdl2::render::WindowCanvas;
    use sdl2::{EventPump, Sdl, TimerSubsystem, VideoSubsystem};
    use sdl2::mouse::MouseButton;
    use crate::osm::{Node, Relation, Way};

    use crate::random::random_int;
    use super::sprite::*;
    use super::api::*;

    pub fn run () {
        let mut sdl_components = SdlComponents::init();

        let mut past = sdl_components.timer_subsystem.ticks64();
        let mut now;
        let mut past_fps = past;
        let mut fps = 0;
        let mut frames_skipped = 0;

        let mut input = Input::new();

        let sprites: &mut Vec<Sprite> = &mut Vec::new();
        add_sprite(sprites, SpriteType::PLAYER, &mut sdl_components);
        let num_enemies = 10;
        for _ in 1..=num_enemies {
            add_sprite(sprites, SpriteType::ENEMY, &mut sdl_components);
        }

        // todo delete, debug
        let body = Relation::build_query(43.731, 7.418, 43.732, 7.419);
        post("https://overpass-api.de/api/interpreter", body);

        'running: loop {
            let mut time_elapsed = 0;

            for event in sdl_components.event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                    _ => {}
                }

                handle_key_events(event.clone(), &mut input);
                handle_mouse_events(event, &mut input);
            }

            update_sprites(sprites, &mut input, &mut sdl_components);

            sdl_components.canvas.set_draw_color(Color::RGB(255, 255, 255));
            sdl_components.canvas.clear();

            draw(&mut sdl_components.canvas, sprites);

            now = sdl_components.timer_subsystem.ticks64();
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
                sdl_components.update_window_title(["fps: ", fps.to_string().as_str()].join("").as_str());
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
            x: random_int(0, window_width),
            y: random_int(0, window_height),
            sprite_type: sprite_type,
            red: random_int(0, 255) as u8,
            green: random_int(0, 255) as u8,
            blue: random_int(0, 255) as u8,
            action_end_timestamp: 0,
            current_direction: Direction::STOP,
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

    pub fn update_sprites (sprites: &mut Vec<Sprite>, input: &mut Input, sdl_components: &mut SdlComponents) {
        let (mut window_width,mut window_height) = sdl_components.canvas.output_size().unwrap();
        let window_width: i32 = window_width as i32;
        let window_height: i32 = window_height as i32;

        let mut i = 0;
        while i < sprites.len() {
            let sprite = &mut sprites[i];
            sprite.return_sprite_to_canvas(window_width, window_height);
            sprite.auto_move(sdl_components);
            
            if sprite.sprite_type == SpriteType::PLAYER {
                if input.is_key_down(Keycode::Up) { sprite.y -= 10 }
                if input.is_key_down(Keycode::Down) { sprite.y += 10 }
                if input.is_key_down(Keycode::Left) { sprite.x -= 10 }
                if input.is_key_down(Keycode::Right) { sprite.x += 10 }
            }

            if input.is_mouse_btn_down(MouseButton::Left) &&
               input.is_mouse_over_sprite(sprite) == true &&
               sprite.sprite_type == SpriteType::ENEMY
            {
                Sprite::destroy_sprite(sprites, i);
            } else {
                i += 1;
            }
        }
    }

    pub struct SdlComponents {
        sdl_context: Sdl,
        video_subsystem: VideoSubsystem,
        canvas: WindowCanvas,
        event_pump: EventPump,
        pub timer_subsystem: TimerSubsystem
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

    struct Input {
        keys: HashMap<Keycode, bool>,
        mouse: HashMap<MouseButton, bool>,
        mouse_current_pos_x: i32,
        mouse_current_pos_y: i32,
        mouse_last_click_pos_x: i32,
        mouse_last_click_pos_y: i32,
    }

    impl Input {
        fn new() -> Self {
            Input {
                keys: HashMap::new(),
                mouse: HashMap::new(),
                mouse_current_pos_x: 0,
                mouse_current_pos_y: 0,
                mouse_last_click_pos_x: 0,
                mouse_last_click_pos_y: 0
            }
        }

        fn set_key(&mut self, key: Keycode, value: bool) {
            self.keys.insert(key, value);
        }

        fn set_mouse_btn(&mut self, mouse: MouseButton, value: bool) {
            self.mouse.insert(mouse, value);
        }

        fn set_current_mouse_pos(&mut self, x: i32, y: i32) {
            self.mouse_current_pos_x = x;
            self.mouse_current_pos_y = y;
        }

        fn set_last_click_mouse_pos(&mut self, x: i32, y: i32) {
            self.mouse_last_click_pos_x = x;
            self.mouse_last_click_pos_y = y;
        }

        fn is_key_down(&self, key: Keycode) -> bool {
            *self.keys.get(&key).unwrap_or(&false)
        }

        fn is_mouse_btn_down(&self, mouse_btn: MouseButton) -> bool {
            *self.mouse.get(&mouse_btn).unwrap_or(&false)
        }

        fn current_mouse_pos(&mut self) -> (i32, i32) {
            (self.mouse_current_pos_x, self.mouse_current_pos_y)
        }

        fn last_click_mouse_pos(&mut self) -> (i32, i32) {
            (self.mouse_last_click_pos_x, self.mouse_last_click_pos_y)
        }

        fn is_mouse_over_sprite (&mut self, sprite: &Sprite) -> bool {
            if self.mouse_current_pos_x >= sprite.x && self.mouse_current_pos_x <= sprite.x + 20 &&
               self.mouse_current_pos_y >= sprite.y && self.mouse_current_pos_y <= sprite.y + 20
            {
                return true;
            }

            false
        }
    }

    fn handle_key_events(event: Event, input: &mut Input) {
        match event {
            Event::KeyDown { keycode: Some(keycode), .. } => {
                input.set_key(keycode, true);
            },
            Event::KeyUp { keycode: Some(keycode), .. } => {
                input.set_key(keycode, false);
            },
            _ => {}
        }
    }

    fn handle_mouse_events(event: Event, input: &mut Input) {
        match event {
            Event::MouseButtonDown { mouse_btn,x, y, .. } => {
                input.set_mouse_btn(mouse_btn, true);
            },
            Event::MouseButtonUp { mouse_btn, x, y, .. } => {
                input.set_mouse_btn(mouse_btn, false);
                input.set_last_click_mouse_pos(x, y);
            },
            Event::MouseMotion { x, y, .. } => {
                input.mouse_current_pos_x = x;
                input.mouse_current_pos_y = y;
            }
            _ => {}
        }
    }
}

pub mod sprite {
    use crate::random::random_int;
    use super::engine::SdlComponents;

    #[derive(PartialEq)]
pub struct Sprite {
        pub x: i32,
        pub y: i32,
        pub red: u8,
        pub green: u8,
        pub blue: u8,
        pub sprite_type: SpriteType,
        pub current_direction: Direction,
        pub action_end_timestamp: i32,
    }

    #[derive(PartialEq)]
    pub enum SpriteType {
        PLAYER,
        ENEMY,
        DEFAULT,
    }

    #[derive(PartialEq, Eq)]
    pub enum Direction {
        STOP,
        UP,
        RIGHT,
        DOWN,
        LEFT,
        UPRIGHT,
        DOWNRIGHT,
        DOWNLEFT,
        UPLEFT,
    }

    impl Direction {
        pub fn from_int(index: i32) -> Option<Direction> {
            match index {
                0 => Some(Direction::STOP),
                1 => Some(Direction::UP),
                2 => Some(Direction::RIGHT),
                3 => Some(Direction::DOWN),
                4 => Some(Direction::LEFT),
                5 => Some(Direction::UPRIGHT),
                6 => Some(Direction::DOWNRIGHT),
                7 => Some(Direction::UPLEFT),
                8 => Some(Direction::DOWNLEFT),
                _ => None,
            }
        }
    }

    impl Sprite {
        pub fn auto_move (&mut self, sdl_components: &mut SdlComponents) {
            if self.sprite_type == SpriteType::PLAYER { return; }

            let now = sdl_components.timer_subsystem.ticks64();

            if self.action_end_timestamp == 0 || self.action_end_timestamp <= now as i32 {
                self.current_direction = Direction::from_int(random_int(0, 8)).unwrap_or(Direction::STOP);
                self.action_end_timestamp = random_int(0, 3000) + now as i32;
            }
            else {
                match self.current_direction {
                    Direction::STOP => { },
                    Direction::UP => {self.y += 10}
                    Direction::RIGHT => {self.x += 10}
                    Direction::DOWN => {self.y -= 10}
                    Direction::LEFT => {self.x -= 10}
                    Direction::UPRIGHT => {self.y += 10; self.x += 10}
                    Direction::DOWNRIGHT => {self.y -= 10; self.x += 10}
                    Direction::UPLEFT => {self.y -= 10; self.x -= 10}
                    Direction::DOWNLEFT => {self.y += 10; self.x -= 10}
                };
            }
        }

        pub fn return_sprite_to_canvas(&mut self, window_width: i32, window_height: i32) {
            if self.x > window_width {
                self.x = 0
            }
            else if self.x < 0 {
                self.x = window_width
            }
            else if self.y > window_height {
                self.y = 0
            }
            else if self.y < 0 {
                self.y = window_height
            }
        }

        pub fn destroy_sprite(sprites: &mut Vec<Sprite>, index: usize) {
            sprites.remove(index);
        }
    }
}

pub mod random {
    use rand::Rng;

    pub fn random_int(min: i32, max: i32) -> i32 {
        let mut rng = rand::thread_rng();
        rng.gen_range(min..max)
    }
}

pub mod api {
    use reqwest::Error;

    #[tokio::main]
    pub async fn post(uri: &str, body: String) -> Result<(), Error> {
        let client = reqwest::Client::new();

        // https://overpass-api.de/api/interpreter
        // node(43.731, 7.418, 43.732, 7.419); out body;

        let response = client
            .post(uri)
            .body(body)
            .send()
            .await?;

        if response.status().is_success() {
            let response_text = response.text().await?;
            println!("Received XML Response: {}", response_text);
        } else {
            println!("Failed to receive successful response");
        }

        Ok(())
    }
}

pub mod osm {
    pub struct Osm {
        nodes: Node,
        ways: Way,
        relations: Relation,
    }

    impl Osm {
        pub fn new(nodes: Node, ways: Way, relations: Relation) -> Self {
            Self { nodes, ways, relations }
        }
    }

    pub struct Node {
        id: i64,
        lat: f32,
        lon: f32,
        tags: Vec<(String, String)>,
    }

    impl Node {
        pub fn new(id: i64, lat: f32, lon: f32, tags: Vec<(String, String)>) -> Self {
            Self { id, lat, lon, tags }
        }

        pub fn build_query (min_lat: f32, min_lon: f32, max_lat: f32, max_lon: f32) -> String {
            format!("node({}, {}, {}, {}); out body;", min_lat, min_lon, max_lat, max_lon)
        }
    }

    pub struct Way {
        id: i32,
        refs: Vec<i64>,
        tags: Vec<(String, String)>,
    }

    impl Way {
        pub fn new(id: i32, refs: Vec<i64>, tags: Vec<(String, String)>) -> Self {
            Self { id, refs, tags }
        }

        pub fn build_query (min_lat: f32, min_lon: f32, max_lat: f32, max_lon: f32) -> String {
            format!("way({}, {}, {}, {}); out body;", min_lat, min_lon, max_lat, max_lon)
        }
    }

    pub struct Relation {
        id: i32,
        members: Vec<(i64, String)>,
        tags: Vec<(String, String)>,
    }

    impl Relation {
        pub fn new(id: i32, members: Vec<(i64, String)>, tags: Vec<(String, String)>) -> Self {
            Self { id, members, tags }
        }

        pub fn build_query (min_lat: f32, min_lon: f32, max_lat: f32, max_lon: f32) -> String {
            format!("relation({}, {}, {}, {}); out body;", min_lat, min_lon, max_lat, max_lon)
        }
    }
}