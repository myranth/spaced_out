extern crate ggez;
extern crate rand;

use std::env;
use std::path;

use ggez::{GameResult, Context, ContextBuilder};
use ggez::graphics::{DrawMode, Point2, Vector2};
use ggez::graphics;
use ggez::conf;
use ggez::timer;
use ggez::event::{self, Keycode, Mod, MouseButton, MouseState};

use rand::{thread_rng, Rng};

#[derive(Debug)]
enum ActorType {
    Player,
    Laser,
    Enemy
}

#[derive(Debug)]
struct Actor {
    tag: ActorType,
    pos: Point2,
    velocity: Vector2,
    dead: bool
}

impl Actor {
    pub fn update(&mut self, delta_time: f32) {
        self.pos += self.velocity * delta_time;

        if self.pos.x < -20.0 {
            self.dead = true;
        }
        if self.pos.x > 1300.0 {
            self.dead = true;
        }
        if self.pos.y < -20.0 {
            self.dead = true;
        }
        if self.pos.y > 740.0 {
            self.dead = true;
        }
    }
}

fn create_player() -> Actor {
    Actor {
        tag: ActorType::Player,
        pos: Point2::new(0.0, 0.0),
        velocity: Vector2::new(0.0, 0.0),
        dead: false
    }
}

// All game objects n stuff
struct MainState {
    lasers: Vec<Actor>,
    enemies: Vec<Actor>,
    player: Actor,
    firing: bool,
    next_shot_timeout: f32,
    next_enemy_timeout: f32,
    mouse_position: (i32, i32)
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let s = MainState {
            player: create_player(),
            enemies: Vec::new(),
            lasers: Vec::new(),
            firing: false,
            next_shot_timeout: 0.0,
            next_enemy_timeout: 0.0,
            mouse_position: (0, 0),
        };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_FPS: u32 = 60;

        while timer::check_update_time(ctx, DESIRED_FPS) {
            let seconds = 1.0 / (DESIRED_FPS as f32);
            self.next_shot_timeout -= seconds;
            self.next_enemy_timeout -= seconds;

            if self.firing && self.next_shot_timeout <= 0.0 {
                let vec_to_mouse = Vector2::new(
                    self.mouse_position.0 as f32 - 640.0,
                    self.mouse_position.1 as f32 - 320.0
                );
                let laser = Actor {
                    tag: ActorType::Laser,
                    pos: Point2::new(640.0, 360.0),
                    velocity: vec_to_mouse.normalize() * 300.0,
                    dead: false,
                };
                self.lasers.push(laser);
                self.next_shot_timeout = 0.3;
            }

            if self.next_enemy_timeout <= 0.0 {
                let mut rng = thread_rng();
                let random_angle: f32 = rng.gen_range(0.0, 360.0);
                let enemy_pos = Point2::new(
                    640.0 + 670.0 * random_angle.cos(),
                    360.0 + 670.0 * random_angle.sin()
                );

                let enemy = Actor {
                    tag: ActorType::Enemy,
                    pos: enemy_pos,
                    velocity: Vector2::new(-enemy_pos.x, -enemy_pos.y),
                    dead: false
                };
                self.enemies.push(enemy);
                self.next_enemy_timeout = 1.0;
            }

            for laser in &mut self.lasers {
                laser.update(seconds);
            }

            for enemy in &mut self.enemies {
                enemy.update(seconds);
            }
        }

        self.lasers.retain(|ref laser| !laser.dead);

        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, _x: i32, _y: i32) {
        if button == MouseButton::Left {
            self.firing = true;
        }
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, _x: i32, _y: i32) {
        if button == MouseButton::Left {
            self.firing = false;
        }
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, _state: MouseState, x: i32,y: i32, _xrel: i32, _yrel: i32) {
        self.mouse_position = (x, y);
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: Keycode, keymod: Mod, repeat: bool) {
        println!(
            "Key pressed: {:?}, modifier {:?}, repeat: {}",
            keycode, keymod, repeat
        );
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        graphics::circle(
            ctx,
            DrawMode::Fill,
            self.player.pos,
            32.0,
            0.5,
        )?;

        // Draw lasers
        for laser in &self.lasers {
            let start_point = laser.pos;
            let end_point = start_point + laser.velocity.normalize() * 16.0;

            let line_points: [Point2; 2] = [
                start_point, end_point
            ]; 

            graphics::line(ctx, &line_points[..], 2.0)?;
        }

        // Draw enemies
        for enemy in &self.enemies {
            graphics::circle(
                ctx,
                DrawMode::Fill,
                enemy.pos,
                10.0,
                1.0
            )?;
        }

        graphics::present(ctx);
        Ok(())
    }
}

fn main() {
    let mut cb = ContextBuilder::new("ez_game_ez_life", "ggez")
        .window_setup(conf::WindowSetup::default().title("EZ Game EZ Life"))
        .window_mode(conf::WindowMode::default().dimensions(1280, 720));

    // Add top level resources directory to path
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        cb = cb.add_resource_path(path);
    } else {
        eprintln!("CARGO_MANIFEST_DIR not found");
    }

    let ctx = &mut cb.build().unwrap();
    
    match MainState::new(ctx) {
        Err(e) => {
            println!("Could not load game!");
            println!("Error: {}", e);
        },
        Ok(ref mut game) => {
            let result = event::run(ctx, game);
            if let Err(e) = result {
                println!("Error encountered running game: {}", e);
            } else {
                println!("Game exited cleanly.");
            }
        }
    }
}
