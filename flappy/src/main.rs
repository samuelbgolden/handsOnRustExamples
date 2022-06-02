#![warn(clippy::all, clippy::pedantic)]

use bracket_lib::prelude::*;

enum GameMode {
    Menu,
    Playing,
    End,
}

#[derive(PartialEq, Debug)]
enum PlayerState {
    Falling,
    Flapping,
    Diving,
}

// env config
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 80.0;

// falling consts
const FALLING_GRAVITY: f32 = 0.4;
const TERMINAL_FALLING_VELOCITY: f32 = 2.0;
const FALLING_CHAR: char = '~';

// flapping consts
const MAX_FLAPPING_VELOCITY: f32 = -2.0;
const FLAP_ACCELERATION: f32 = -0.8;
const FLAP_FRAME_DURATION: usize = 8;
const FLAPPING_ANIMATION_LENGTH: usize = 8;
const FLAPPING_CHARS: [char; FLAPPING_ANIMATION_LENGTH] = ['v', 'V', 'v', '_', '-', '^', 'A', '^'];

// diving consts
const DIVING_CHAR: char = 'v';
const DIVING_HOLD_LENGTH: usize = 5;
const DIVING_GRAVITY: f32 = 0.6;
const TERMINAL_DIVING_VELOCITY: f32 = 3.0;

#[derive(Debug)]
struct Player {
    x: i32,
    y: i32,
    velocity: f32,
    flap_frame: usize,
    state: PlayerState,
    dive_counter: usize,
}

struct Obstacle {
    x: i32,
    gap_y: i32,
    size: i32,
}

impl Player {
    fn new(x: i32, y: i32) -> Self {
        Player {
            x,
            y,
            velocity: 0.0,
            flap_frame: 0,
            state: PlayerState::Falling,
            dive_counter: 0,
        }
    }

    fn render(&mut self, ctx: &mut BTerm) {
        match self.state {
            PlayerState::Falling => ctx.set(0, self.y, YELLOW, BLACK, to_cp437(FALLING_CHAR)),
            PlayerState::Flapping => ctx.set(
                0,
                self.y,
                YELLOW,
                BLACK,
                to_cp437(
                    FLAPPING_CHARS[(self.flap_frame
                        / (FLAP_FRAME_DURATION / FLAPPING_ANIMATION_LENGTH))
                        as usize],
                ),
            ),
            PlayerState::Diving => ctx.set(0, self.y, YELLOW, BLACK, to_cp437(DIVING_CHAR)),
        }
    }

    fn gravity_and_move(&mut self) {
        match self.state {
            PlayerState::Falling => {
                if self.velocity < TERMINAL_FALLING_VELOCITY {
                    self.velocity += FALLING_GRAVITY;
                } else {
                    self.velocity = TERMINAL_FALLING_VELOCITY;
                }
            }
            PlayerState::Diving => {
                if self.velocity < TERMINAL_DIVING_VELOCITY {
                    self.velocity += DIVING_GRAVITY;
                } else {
                    self.velocity = TERMINAL_DIVING_VELOCITY;
                }
            }
            PlayerState::Flapping => {
                if self.velocity < TERMINAL_FALLING_VELOCITY {
                    self.velocity += FALLING_GRAVITY;
                }
                if self.velocity > MAX_FLAPPING_VELOCITY {
                    self.velocity += FLAP_ACCELERATION;
                }
            }
        }

        self.y += self.velocity as i32;
        self.x += 1;
        if self.y < 0 {
            self.y = 0;
        }
    }

    fn handle_flap(&mut self) {
        if self.state == PlayerState::Flapping {
            self.flap_frame += 1;
            if self.flap_frame == FLAP_FRAME_DURATION {
                self.flap_frame = 0;
                self.state = PlayerState::Falling;
            }
        }
    }

    fn inc_dive_counter(&mut self, ctx: &mut BTerm) {
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.dive_counter += 1;
        } else {
            self.dive_counter = 0;
        }
    }
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle {
            x,
            gap_y: random.range(10, 40),
            size: i32::max(2, 20 - score),
        }
    }

    fn render(&mut self, ctx: &mut BTerm, player_x: i32) {
        let screen_x = self.x - player_x;
        let half_size = self.size / 2;

        // top half of obstacle
        for y in 0..self.gap_y - half_size {
            ctx.set(screen_x, y, RED, BLACK, to_cp437('|'));
        }

        // bottom half of obstacle
        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.set(screen_x, y, RED, BLACK, to_cp437('|'));
        }
    }

    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let does_x_match = player.x == self.x;
        let player_above_gap = player.y < self.gap_y - half_size;
        let player_below_gap = player.y > self.gap_y + half_size;
        does_x_match && (player_above_gap || player_below_gap)
    }
}

struct State {
    player: Player,
    frame_time: f32,
    obstacle: Obstacle,
    mode: GameMode,
    score: i32,
}

impl State {
    fn new() -> Self {
        State {
            player: Player::new(5, 25),
            frame_time: 0.0,
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
            mode: GameMode::Menu,
            score: 0,
        }
    }

    fn render_debug_info(&self, ctx: &mut BTerm) {
        ctx.print(0, 0, "Press SPACE to flap");
        ctx.print(0, 1, &format!("Score: {}", self.score));
        ctx.print(60, 0, &format!("x={}", self.player.x));
        ctx.print(60, 1, &format!("y={}", self.player.y));
        ctx.print(60, 2, &format!("vel={}", self.player.velocity));
        ctx.print(60, 3, &format!("fidx={}", self.player.flap_frame));
        ctx.print(60, 4, &format!("state={:?}", self.player.state));
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);
        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.player.gravity_and_move();
            self.player.handle_flap();
            self.player.inc_dive_counter(ctx);
        }
        self.player.render(ctx);

        self.render_debug_info(ctx);

        self.obstacle.render(ctx, self.player.x);
        if self.player.x > self.obstacle.x {
            self.score += 1;
            self.obstacle = Obstacle::new(self.player.x + SCREEN_WIDTH, self.score);
        }

        if self.player.dive_counter >= DIVING_HOLD_LENGTH {
            self.player.state = PlayerState::Diving;
        } else if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.state = PlayerState::Flapping;
        } else {
            self.player.state = PlayerState::Falling;
        }

        if self.player.y >= SCREEN_HEIGHT || self.obstacle.hit_obstacle(&self.player) {
            self.mode = GameMode::End;
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "You are DEAD");
        ctx.print_centered(6, &format!("score: {}", self.score));
        ctx.print_centered(8, "(P) Play Again");
        ctx.print_centered(9, "(Q) Quit Game");
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn restart(&mut self) {
        self.player = Player::new(5, 25);
        self.frame_time = 0.0;
        self.obstacle = Obstacle::new(SCREEN_WIDTH, 0);
        self.mode = GameMode::Playing;
        self.score = 0;
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "Welcome to Flappy Dragon");
        ctx.print_centered(8, "(P) Play Game");
        ctx.print_centered(9, "(Q) Quit Game");
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::End => self.dead(ctx),
            GameMode::Playing => self.play(ctx),
        }
        println!("{:?}", self.player);
    }
}

fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("Flappy Dragon")
        .build()?;
    main_loop(context, State::new())
}
