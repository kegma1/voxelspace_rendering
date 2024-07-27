use macroquad::prelude::*;

#[macroquad::main("test")]
async fn main() {
    loop {
        clear_background(BLACK);
        
        next_frame().await;
    }
}
