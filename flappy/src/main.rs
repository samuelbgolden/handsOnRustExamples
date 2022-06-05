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

// general game constants
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 60.0;
const BGR_COLOR: (u8, u8, u8) = NAVY;
const MIN_GAP_SIZE: i32 = 4;

// falling consts
const FALLING_GRAVITY: f32 = 0.4;
const TERMINAL_FALLING_VELOCITY: f32 = 1.8;
const FALLING_CHAR: char = '~';

// flapping consts
const MAX_FLAPPING_VELOCITY: f32 = -2.2;
const FLAP_MAX_ACCELERATION: f32 = -2.2;
const FLAP_INIT_ACCELERATION: f32 = 0.1;
const FLAP_DURATION: usize = 10; // in frames, must be multiple of FLAPPING_ANIMATION_LENGTH
const FLAPPING_ANIMATION_LENGTH: usize = 10;
const FLAPPING_CHARS: [char; FLAPPING_ANIMATION_LENGTH] =
    ['v', 'V', 'v', '_', '-', '^', 'A', '^', '^', '^'];
//const FLAPPING_CHARS: [char; FLAPPING_ANIMATION_LENGTH] = ['V', 'v', '-', '^', 'A'];

// diving consts
const DIVING_CHAR: char = 'v';
const DIVING_HOLD_LENGTH: usize = 2;
const DIVING_GRAVITY: f32 = 0.8;
const TERMINAL_DIVING_VELOCITY: f32 = 3.5;

// scoring animation
const SCORE_ANIMATION_LENGTH: usize = 11;
const SCORE_ANIMATION_TEXT_COLORS: [(u8, u8, u8); SCORE_ANIMATION_LENGTH] = [
    GREEN, GREEN, WHITE, WHITE, GREEN, GREEN, WHITE, WHITE, GREEN, GREEN, WHITE,
];
const SCORE_ANIMATION_PLAYER_COLORS: [(u8, u8, u8); SCORE_ANIMATION_LENGTH] = [
    GREEN, GREEN, GREEN, GREEN, GREEN, GREEN, GREEN, GREEN, GREEN, GREEN, YELLOW,
];

#[derive(Debug)]
struct Player {
    x: i32,
    y: i32,
    velocity: f32,
    flap_frame: usize,
    state: PlayerState,
    dive_counter: usize,
    color: (u8, u8, u8),
    score_animation_frame: usize,
}

struct Obstacle {
    x: i32,
    gap_y: i32,
    size: i32,
}

#[derive(Debug)]
struct Objective {
    x: i32,
    y: i32,
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
            color: YELLOW,
            score_animation_frame: SCORE_ANIMATION_LENGTH - 1,
        }
    }

    fn set_state(&mut self, state: PlayerState) {
        if self.state == state {
            return;
        }

        match state {
            PlayerState::Flapping => {
                self.dive_counter = 0;
            }
            PlayerState::Falling => {
                self.dive_counter = 0;
                self.flap_frame = 0;
            }
            PlayerState::Diving => {
                self.flap_frame = 0;
            }
        }
        self.state = state;
        println!("SET PLAYER {:?}", self.state);
    }

    fn render(&mut self, ctx: &mut BTerm) {
        match self.state {
            PlayerState::Falling => ctx.set(
                0,
                self.y,
                SCORE_ANIMATION_PLAYER_COLORS[self.score_animation_frame],
                BGR_COLOR,
                to_cp437(FALLING_CHAR),
            ),
            PlayerState::Flapping => ctx.set(
                0,
                self.y,
                SCORE_ANIMATION_PLAYER_COLORS[self.score_animation_frame],
                BGR_COLOR,
                to_cp437(
                    FLAPPING_CHARS
                        [(self.flap_frame / (FLAP_DURATION / FLAPPING_ANIMATION_LENGTH)) as usize],
                ),
            ),
            PlayerState::Diving => ctx.set(
                0,
                self.y,
                SCORE_ANIMATION_PLAYER_COLORS[self.score_animation_frame],
                BGR_COLOR,
                to_cp437(DIVING_CHAR),
            ),
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
                    // scale acceleration with the progress of the flap animation
                    let vel_tmp = FLAP_INIT_ACCELERATION
                        + ((FLAP_MAX_ACCELERATION - FLAP_INIT_ACCELERATION) / FLAP_DURATION as f32)
                            * self.flap_frame as f32;
                    print!("{:?}\t", vel_tmp);
                    self.velocity += vel_tmp;
                } else {
                    self.velocity = MAX_FLAPPING_VELOCITY;
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
            if self.flap_frame == FLAP_DURATION {
                self.set_state(PlayerState::Falling);
            }
        }
    }
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle {
            x,
            gap_y: random.range(10, 40),
            size: i32::max(MIN_GAP_SIZE, 20 - score),
        }
    }

    fn render(&self, ctx: &mut BTerm, player_x: i32) {
        let screen_x = self.x - player_x;
        let half_size = self.size / 2;

        // top half of obstacle
        for y in 0..self.gap_y - half_size {
            ctx.set(screen_x, y, RED, BGR_COLOR, to_cp437('|'));
        }

        // bottom half of obstacle
        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.set(screen_x, y, RED, BGR_COLOR, to_cp437('|'));
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

impl Objective {
    fn new(x: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Objective {
            x,
            y: random.range(2, SCREEN_HEIGHT - 1),
        }
    }

    fn render(&self, ctx: &mut BTerm, player_x: i32) {
        // rendering in this shape:
        // /=\
        // \=/

        let screen_x = self.x - player_x;
        ctx.set(screen_x, self.y, GOLD, BGR_COLOR, to_cp437('/'));
        ctx.set(screen_x + 1, self.y, GOLD, BGR_COLOR, to_cp437('='));
        ctx.set(screen_x + 2, self.y, GOLD, BGR_COLOR, to_cp437('\\'));

        ctx.set(screen_x, self.y + 1, GOLD, BGR_COLOR, to_cp437('\\'));
        ctx.set(screen_x + 1, self.y + 1, GOLD, BGR_COLOR, to_cp437('='));
        ctx.set(screen_x + 2, self.y + 1, GOLD, BGR_COLOR, to_cp437('/'));
    }

    fn hit_objective(&self, player: &Player) -> bool {
        let in_x_bounds = player.x >= self.x && player.x < (self.x + 3);
        let in_y_bounds = player.y == self.y || player.y == (self.y + 1);
        in_x_bounds && in_y_bounds
    }
}

struct State {
    player: Player,
    frame_time: f32,
    obstacle: Obstacle,
    objective: Objective,
    mode: GameMode,
    score: i32,
    space_pressed_this_frame: bool,
}

impl State {
    fn new() -> Self {
        State {
            player: Player::new(5, 25),
            frame_time: 0.0,
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
            objective: Objective::new(SCREEN_WIDTH + (SCREEN_WIDTH / 2)),
            mode: GameMode::Menu,
            score: 0,
            space_pressed_this_frame: false,
        }
    }

    fn render_screen_text(&self, ctx: &mut BTerm) {
        ctx.print(0, 0, "Press SPACE to flap, hold to dive");
        ctx.print_color(
            0,
            1,
            SCORE_ANIMATION_TEXT_COLORS[self.player.score_animation_frame],
            BGR_COLOR,
            &format!("Score: {}", self.score),
        );
        ctx.print(60, 0, &format!("x={}", self.player.x));
        ctx.print(60, 1, &format!("y={}", self.player.y));
        ctx.print(60, 2, &format!("vel={}", self.player.velocity));
        ctx.print(60, 3, &format!("fidx={}", self.player.flap_frame));
        ctx.print(60, 4, &format!("state={:?}", self.player.state));
        ctx.print(
            60,
            5,
            &format!("obj={},{}", self.objective.x, self.objective.y),
        );
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(BGR_COLOR);
        self.frame_time += ctx.frame_time_ms;

        //println!("{:?}", ctx.key);
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.space_pressed_this_frame = true;
        }

        // per frame
        if self.frame_time > FRAME_DURATION {
            self.player.gravity_and_move();
            self.player.handle_flap();
            if self.space_pressed_this_frame {
                self.player.dive_counter += 1;
            }
            if self.player.score_animation_frame != (SCORE_ANIMATION_LENGTH - 1) {
                self.player.score_animation_frame += 1;
            }
        }

        // handle space input
        if self.space_pressed_this_frame {
            if self.player.dive_counter >= DIVING_HOLD_LENGTH {
                self.player.set_state(PlayerState::Diving);
            } else {
                self.player.set_state(PlayerState::Flapping);
            }
        } else {
        }

        // render on-screen stuff
        self.player.render(ctx);
        self.render_screen_text(ctx);
        self.obstacle.render(ctx, self.player.x);
        self.objective.render(ctx, self.player.x);

        // detect passed obstacle
        if self.player.x > self.obstacle.x {
            self.inc_score();
            self.obstacle = Obstacle::new(self.player.x + SCREEN_WIDTH, self.score);
        }

        // detect obtained objective or passed objective
        if self.objective.hit_objective(&self.player) {
            self.inc_score();
            self.objective = Objective::new(self.player.x + SCREEN_WIDTH);
        } else if self.player.x > self.objective.x {
            self.objective = Objective::new(self.player.x + SCREEN_WIDTH);
        }

        // detect death
        if self.player.y >= SCREEN_HEIGHT || self.obstacle.hit_obstacle(&self.player) {
            self.mode = GameMode::End;
        }

        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            if self.space_pressed_this_frame {
                self.space_pressed_this_frame = false;
            } else if self.player.state != PlayerState::Flapping {
                self.player.set_state(PlayerState::Falling);
            }
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
        self.space_pressed_this_frame = false;
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

    fn inc_score(&mut self) {
        self.score += 1;
        self.player.score_animation_frame = 0;
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
