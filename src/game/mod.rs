use crate::AppHandler;

mod gfx;

enum GameState {
    Menu,
    InGame
}

pub struct Game {
    state: GameState
}

impl Game {
    
}

impl AppHandler for Game {
    fn draw(&mut self) {
        
    }
}