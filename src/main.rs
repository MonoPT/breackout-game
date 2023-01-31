use macroquad::prelude::*;

const PLAYER_SIZE: Vec2 = Vec2::from_array([150f32, 40f32]);
const PLAYER_SPEED: f32 = 700f32;

const BLOCK_SIZE: Vec2 = Vec2::from_array([90f32, 20f32]);

const BALL_SIZE: f32 = 50f32;
const BALL_SPEED: f32 = 400f32;

pub struct Player {
    rect: Rect
}

impl Player {
    pub fn new() -> Self {
        Self {
            rect: Rect::new(
                screen_width() * 0.5f32 - PLAYER_SIZE.x / 2.0, 
                screen_height() - 100f32, 
                PLAYER_SIZE.x, 
                PLAYER_SIZE.y
            )
        }
    }

    pub fn update(&mut self, dt: f32) {
        let x_move = match(is_key_down(KeyCode::Left), is_key_down(KeyCode::Right)) { //Handle x input
            (true, false) => -1f32,
            (false, true) => 1f32,
            _ => 0f32
        };

        self.rect.x += x_move * dt * PLAYER_SPEED;

        if self.rect.x < 0.32f32 {
            self.rect.x =   0f32;
        }

        else if self.rect.x > screen_width() - self.rect.w {
            self.rect.x = screen_width() - self.rect.w;
        }
    } 

    pub fn draw(&self) {
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, BLUE);
    }
}

pub struct Block {
    rect: Rect,
    lives: i32,
    block_type: BlockType
}

impl Block {
    pub fn new(pos: Vec2, block_type: BlockType) -> Self {
        Self { 
            rect: Rect { 
                x: pos.x, 
                y: pos.y, 
                w: BLOCK_SIZE.x, 
                h: BLOCK_SIZE.y 
            },
            lives: 2,
            block_type
        }
    }

    pub fn draw(&self) {
        let color = match self.block_type {
            BlockType::Regular => {
                match self.lives {
                    2 => RED,
                    _ => ORANGE
                }
            },
            BlockType::SpawnBallOnDeath => GREEN,
        };


        draw_rectangle(self.rect.x, self.rect.y, BLOCK_SIZE.x, BLOCK_SIZE.y, color)
    }
}

pub struct Ball {
    rect: Rect,
    vel: Vec2
}

impl Ball {
    pub fn new(pos: Vec2) -> Self {
        Self { 
            rect: Rect::new(pos.x, pos.y, BALL_SIZE, BALL_SIZE), 
            vel: vec2(rand::gen_range(-1f32, 1f32), 1f32).normalize(),
        }
    }

    pub fn draw(&self) {
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, GREEN)
    }

    pub fn update(&mut self, dt:f32) {
        self.rect.x += self.vel.x * dt * BALL_SPEED;
        self.rect.y += self.vel.y * dt * BALL_SPEED;

        if self.rect.x < 0f32 {
            self.vel.x = 1f32;
        } else if self.rect.x > screen_width() - self.rect.w {
            self.vel.x = -1f32;
        }

        if self.rect.y < 0f32 {
            self.vel.y = 1f32;
        }
    }
}


pub enum GameState {
    Menu,
    Game,
    LevelCompleted,
    Dead
}

#[derive(PartialEq)]
pub enum BlockType {
    Regular,
    SpawnBallOnDeath
}


#[macroquad::main("breakout_game")]
async fn main() {
    let mut player_lives = 3;
    let mut game_state = GameState::Menu;

    let font = load_ttf_font("res/FiraSansCondensed-Regular.ttf").await.unwrap();

    let mut score = 0;

    let mut player = Player::new();
    let mut blocks = Vec::new();
    let mut balls = Vec::new();

    init_blocks(&mut blocks);

    

    balls.push(Ball::new(
        vec2(screen_width() * 0.5f32, screen_height() * 0.5f32)
    ));



    loop {
        clear_background(WHITE);

        match game_state {
            GameState::Menu => {
                draw_title_text("Press SPACE to start", font);

                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Game;
                }

            },
            GameState::Game => {
                player.update(get_frame_time());
        
                for ball in balls.iter_mut() {
                    ball.update(get_frame_time());
                }
                
                let mut spawn_later = vec![];
                for ball in balls.iter_mut() {
                    resolve_collision_a_b(&mut ball.rect, &mut ball.vel, &player.rect);
        
                    for block in blocks.iter_mut() {
                        if resolve_collision_a_b(&mut ball.rect, &mut ball.vel, &block.rect) {
                            block.lives -= 1;
                            
                            if block.lives <= 0 {
                                score += 10;

                                if block.block_type == BlockType::SpawnBallOnDeath {
                                    spawn_later.push(
                                        Ball::new(ball.rect.point())
                                    )
                                }
                            }
        
                        }
                    }
                }

                for ball in spawn_later.into_iter() {
                    balls.push(ball);
                }
                
                let balls_len = balls.len();
                balls.retain(|ball| ball.rect.y < screen_height() - 100f32);
        
                let removed_balls = balls.len() < balls_len;
        
                if  removed_balls && balls.is_empty() {
                    player_lives -= 1;

                    balls.push(Ball::new(
                        player.rect.point() + vec2(0f32, -50f32)
                    ));


                    if player_lives < 1 {
                        game_state = GameState::Dead;
                    }
        
                }
                
        
        
                blocks.retain(|block| block.lives > 0);

                if blocks.len() < 1 {
                    game_state = GameState::LevelCompleted;
                }
        
        
                let score_text = &format!("Score: {}", score);
                let score_text_dim = measure_text(&score_text, Some(font), 30u16, 1.0);


                draw_text_ex(
                    score_text, 
                    screen_width() * 0.5f32 - score_text_dim.width * 0.5f32, 
                    40.0,
                    TextParams { font, font_size: 30u16, color: BLACK, ..Default::default() }
                );


                draw_text_ex(
                    &format!("Lives: {}", player_lives), 
                    40.0, 
                    40.0,
                    TextParams { font, font_size: 30u16, color: BLACK, ..Default::default() }
                );
            },
            GameState::LevelCompleted => {
                draw_title_text(&format!("You WIN! {} score", score), font);

                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Menu;
                    reset_game(&mut score, &mut player_lives, &mut blocks, &mut balls, &mut player);
                }
            },
            GameState::Dead => {
                draw_title_text(&format!("You DIED! {} score", score), font);

                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Menu;
                    reset_game(&mut score, &mut player_lives, &mut blocks, &mut balls, &mut player);
                }
            }
        }



        
        
        player.draw();

        for block in blocks.iter() {
            block.draw();
        }

        for ball in balls.iter() {
            ball.draw();
        }

        next_frame().await;
    }
}

fn resolve_collision_a_b(a: &mut Rect, vel: &mut Vec2, b: &Rect) -> bool {

    let intersection = match a.intersect(*b) {
        Some(intersection) => intersection,
        None => return  false
    };




    
    let a_center = a.center();
    let b_center = b.center();
    let to = b_center - a_center;
    let to_signum = to.signum();

    match intersection.w > intersection.h {
        true => {
            a.y -= to_signum.y * intersection.h;
            vel.y = -to.signum().y * vel.y.abs();
        }
        false => {
            a.x -= to_signum.x * intersection.w;
            vel.x = -to.signum().x * vel.x.abs();
        }
    }
    
    true
}

pub fn draw_title_text(text: &str, font: Font) {
    let text = text;
    let dims = measure_text(text, Some(font), 30u16, 1.0);

    draw_text_ex(
        text, 
        screen_width() * 0.5f32 - dims.width * 0.5f32, 
        screen_height() * 0.5f32 - dims.height * 0.5f32,
        TextParams { font, font_size: 30u16, color: BLACK, ..Default::default() }
    );
}

pub fn reset_game(
    score: &mut i32,
    player_lives: &mut usize,
    blocks: &mut Vec<Block>,
    balls: &mut Vec<Ball>,
    player: &mut Player
) {
    *player = Player::new();
    *score = 0;
    *player_lives = 3;
    balls.clear();
    init_blocks(blocks);
    balls.push(Ball::new(
        vec2(screen_width() * 0.5f32, screen_height() * 0.5f32)
    ));
}

pub fn init_blocks(blocks: &mut Vec<Block>) {
    let (width, height) = (6,6);
    let padding = 5f32;
    let total_size = BLOCK_SIZE + vec2(padding, padding);

    let board_start_pos = vec2((screen_width() - (total_size.x * width as f32))* 0.5f32, 50f32);

    for i in 0..width * height {
        let block_x = (i % width) as f32 * total_size.x;
        let block_y = (i / height) as f32 * total_size.y;
        blocks.push(Block::new(board_start_pos + vec2(block_x, block_y), BlockType::Regular));
    }

    for _ in 0..3 {
        let rand_index = rand::gen_range(0, blocks.len());
        blocks[rand_index].block_type = BlockType::SpawnBallOnDeath;
    }

}