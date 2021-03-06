extern crate ggez;
extern crate rand;

mod gui;

use gui::GUI;

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
}

#[derive(Debug)]
struct Actor {
    tag: ActorType,
    pos: Point2,
    velocity: Vector2,
    life: i32,
}

struct Enemy {
    pos: Point2,
    direction: Vector2,
    speed: f32,
    movement_mod: Option<MovementMod>,
    life: i32,
    max_life: i32,
    worth: i32
}

enum MovementMod {
    Accelerating(f32), // acceleration
    Spiral(f32) // Spinniness
}

impl Enemy {
    pub fn new(pos: Point2, direction: Vector2, speed: f32, movement_mod: Option<MovementMod>, life: i32, worth: i32) -> Self {
        Enemy {
            pos: pos,
            direction: direction,
            speed: speed,
            movement_mod: movement_mod,
            life: life,
            max_life: life,
            worth: worth
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        if let Some(ref modifier) = self.movement_mod {
            match modifier {
                MovementMod::Accelerating(acceleration) => {
                    self.speed += acceleration;
                },
                MovementMod::Spiral(spiral) => {
                    let to_base = Vector2::new(-self.pos.x, -self.pos.y).normalize();
                    let perp = Vector2::new(to_base.y, -to_base.x);

                    let new_direction = Vector2::new(
                        (1.0 - spiral) * to_base.x + spiral * perp.x,
                        (1.0 - spiral)* to_base.y + spiral * perp.y
                    );
                    self.direction = new_direction.normalize();
                }
            }
        }

        self.pos += self.direction * self.speed * delta_time;
    }
}

struct Player {
    life: i32,
    num_lasers: i32,
    laser_speed: f32,
    laser_fire_rate: f32
}

impl Player {
    fn new() -> Self {
        Player {
            life: 1000,
            num_lasers: 1,
            laser_speed: 40.0,
            laser_fire_rate: 0.5
        }
    }
}

impl Actor {
    pub fn update(&mut self, delta_time: f32) {
        self.pos += self.velocity * delta_time;

        let distance_from_origin = ((self.pos.x).powf(2.0) + (self.pos.y).powf(2.0)).powf(0.5);
        if distance_from_origin > 800.0 {
            self.life = -1;
        }
    }
}

// All game objects n stuff
struct MainState {
    lasers: Vec<Actor>,
    enemies: Vec<Enemy>,
    player: Player,
    firing: bool,
    next_shot_timeout: f32,
    next_enemy_timeout: f32,
    mouse_position: (i32, i32),
    gui: GUI,
    score: i32,
    money: i32,
    spaceout_charge: i32,
    spaceout_time: f32
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let s = MainState {
            player: Player::new(),
            enemies: Vec::new(),
            lasers: Vec::new(),
            firing: false,
            next_shot_timeout: 0.0,
            next_enemy_timeout: 0.0,
            mouse_position: (0, 0),
            gui: GUI::new(ctx)?,
            score: 0,
            money: 100,
            spaceout_charge: 0,
            spaceout_time: 0.0
        };
        Ok(s)
    }

    fn collision(line_end: &Point2, circle_center: &Point2, circle_radius: f32) -> bool {
        let sq_distance = (line_end.x - circle_center.x).powf(2.0) + (line_end.y - circle_center.y).powf(2.0);
        sq_distance <= circle_radius.powf(2.0)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_FPS: u32 = 60;

        while timer::check_update_time(ctx, DESIRED_FPS) {
            let seconds = 1.0 / (DESIRED_FPS as f32);
            self.next_shot_timeout -= seconds;
            self.next_enemy_timeout -= seconds;
            if self.spaceout_time > 0.0 {
                self.spaceout_time -= seconds;
            }

            if self.firing && self.next_shot_timeout <= 0.0 {
                let vec_to_mouse = Vector2::new(
                    self.mouse_position.0 as f32 - 640.0,
                    self.mouse_position.1 as f32 - 360.0
                );
                let laser = Actor {
                    tag: ActorType::Laser,
                    pos: Point2::new(0.0, 0.0),
                    velocity: vec_to_mouse.normalize() * 300.0,
                    life: 5,
                };
                self.lasers.push(laser);
                self.next_shot_timeout = self.player.laser_fire_rate;
            }

            if self.next_enemy_timeout <= 0.0 && self.spaceout_time <= 0.0 {
                let mut rng = thread_rng();
                let random_angle: f32 = rng.gen_range(0.0, 360.0);
                let enemy_pos = Point2::new(
                    670.0 * random_angle.cos(),
                    670.0 * random_angle.sin()
                );

                let vec_to_base = Vector2::new(-enemy_pos.x, -enemy_pos.y).normalize();
                let movement_mod = MovementMod::Spiral(0.8);
                let enemy = Enemy::new(enemy_pos, vec_to_base, 60.0, Some(movement_mod), 15, 5);
                self.enemies.push(enemy);
                self.next_enemy_timeout = 1.0;
            }

            // Move entities
            for laser in &mut self.lasers {
                laser.update(seconds);
            }

            if self.spaceout_time <= 0.0 {
                for enemy in &mut self.enemies {
                    enemy.update(seconds);
                }
            }

            // Check collisions (this might be fun)
            for laser in &mut self.lasers {
                for enemy in &mut self.enemies {
                    let colliding = MainState::collision(&laser.pos, &enemy.pos, 16.0);
                    if colliding {
                        enemy.life -= 5;
                        laser.life -= 5;
                        continue;                        
                    }
                }
            }
        }

        let num_enemies = self.enemies.len();
        self.lasers.retain(|ref laser| laser.life > 0);
        self.enemies.retain(|ref enemy| enemy.life > 0);
        let num_kills = (num_enemies - self.enemies.len()) as i32;
        self.score += num_kills * 5;
        self.money += num_kills * 5;
        self.spaceout_charge += num_kills * 10;
        if self.spaceout_charge > 100 {
            self.spaceout_charge = 100;
        }

        // Update GUI
        self.gui.score = self.score;
        self.gui.money = self.money;
        self.gui.spaceout_charge = self.spaceout_charge;

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

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        if keycode == Keycode::Space {
            if self.spaceout_charge == 100 {
                self.spaceout_time = 5.0;
                self.spaceout_charge = 0;
            }
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        let center_offset = Vector2::new(640.0, 360.0);

        graphics::circle(
            ctx,
            DrawMode::Fill,
            Point2::new(center_offset.x, center_offset.y),
            32.0,
            0.5,
        )?;

        // Draw lasers
        for laser in &self.lasers {
            let start_point = laser.pos;
            let end_point = start_point + laser.velocity.normalize() * 16.0;

            let line_points: [Point2; 2] = [
                start_point + center_offset, 
                end_point + center_offset
            ]; 

            graphics::line(ctx, &line_points[..], 2.0)?;
        }

        // Draw enemies and their healthbars
        for enemy in &self.enemies {
            let enemy_pos = enemy.pos + center_offset;
            graphics::set_color(ctx, graphics::WHITE)?;
            graphics::circle(
                ctx,
                DrawMode::Fill,
                enemy_pos,
                10.0,
                1.0
            )?;

            graphics::set_color(ctx, graphics::Color::new(0.25, 0.25, 0.25, 1.0))?;
            let healthbar_background = graphics::Rect {
                x: enemy_pos.x - 24.0,
                y: enemy_pos.y - 32.0,
                w: 48.0,
                h: 10.0
            };

            graphics::rectangle(
                ctx, 
                DrawMode::Fill,
                healthbar_background
            )?;

            graphics::set_color(ctx, graphics::Color::new(0.7, 1.0, 0.65, 1.0))?;

            let bar_width = (enemy.life as f32 / enemy.max_life as f32) * 44.0;
            let healthbar = graphics::Rect::new(enemy_pos.x - 22.0, enemy_pos.y - 30.0, bar_width, 6.0);
            graphics::rectangle(ctx, DrawMode::Fill, healthbar)?;
        }

        // GUI
        graphics::set_color(ctx, graphics::WHITE)?;
        self.gui.draw(ctx)?;

        graphics::present(ctx);
        Ok(())
    }
}

fn main() {
    let mut cb = ContextBuilder::new("spaced_out", "ggez")
        .window_setup(conf::WindowSetup::default().title("Spaced Out"))
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
