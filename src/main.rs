use macroquad::prelude::*;

const SCREEN_WIDTH: u16 = 800;
const SCREEN_HEIGHT: u16 = 600;

struct Player {
    pos: Vec3,
    phi: f64,
    horizon: f64
}

impl Player {
    fn new() -> Self {
        Player {
            pos: Vec3::new((1024 / 2) as f32, (1024 / 2) as f32, 50.0),
            phi: 0.0,
            horizon: 120.0,
        }
    }

    fn move_in_dir(&mut self, distance: f32) {
        let delta_x: f32 = distance * self.phi.sin() as f32; 
        let delta_y: f32 = distance * self.phi.cos() as f32; 
        
        self.pos.x += delta_x;
        self.pos.y += delta_y;
    }
}

#[macroquad::main("test")]
async fn main() {
    let Ok(color_map) = Image::from_file_with_format(
        include_bytes!("../assets/testMap/C1W.png"), 
        Some(ImageFormat::Png)) else {
            eprintln!("ERROR: Cant find color map");
            std::process::exit(1);
        };
    
    let Ok(height_map) = Image::from_file_with_format(
        include_bytes!("../assets/testMap/D1.png"), 
        Some(ImageFormat::Png)) else {
            eprintln!("ERROR: Cant find height map");
            std::process::exit(1);
        };

    let mut screen = Image::gen_image_color(SCREEN_WIDTH, SCREEN_HEIGHT, BLUE);

    let mut p = Player::new();

    loop {
        clear_background(BLUE);

        render(&mut screen, &mut p, 300.0, 500.0, &color_map, &height_map).await;

        draw_text(&format!("{} FPS", get_fps()), 10.0, 20.0, 30.0, BLACK);
        
        for v in get_keys_down() {
            match v {
                KeyCode::A => p.phi += 0.1,
                KeyCode::D => p.phi -= 0.1,

                KeyCode::Q => p.horizon += 10.0,
                KeyCode::E => p.horizon -= 10.0,

                KeyCode::S => p.move_in_dir(10.0),
                KeyCode::W => p.move_in_dir(-10.0),

                KeyCode::Space => p.pos.z += 5.0,
                KeyCode::LeftShift => p.pos.z -= 5.0,
                _ => ()
            }
        }
        next_frame().await;
    }
}

async fn render(
    _screen: &mut Image, 
    p: &mut Player, 
    scale_height: f64, 
    distance: f64, 
    color_map: &Image, 
    height_map: &Image, 
) {
    let sinphi = p.phi.sin();
    let cosphi = p.phi.cos();

    let mut ybuffer = [SCREEN_HEIGHT as f64; SCREEN_WIDTH as usize];

    let mut dz = 1.0;
    let mut z = 1.0;

    while z < distance {
        let mut p_left_x = (-cosphi*z - sinphi*z) + p.pos.x as f64;
        let mut p_left_y = ( sinphi*z - cosphi*z) + p.pos.y as f64;

        let p_right_x = ( cosphi*z - sinphi*z) + p.pos.x as f64;
        let p_right_y = (-sinphi*z - cosphi*z) + p.pos.y as f64;

        let dx = (p_right_x - p_left_x) / SCREEN_WIDTH as f64;
        let dy = (p_right_y - p_left_y) / SCREEN_WIDTH as f64;

        for i in 0..SCREEN_WIDTH {
            let wrapped_x = p_left_x.abs() as u32 % 1024;
            let wrapped_y = p_left_y.abs() as u32 % 1024;

            let height_value = height_map.get_pixel(wrapped_x, wrapped_y).b * 255.0;

            let height_on_screen: f64 = (p.pos.z - height_value) as f64 / z * scale_height + p.horizon;

            if height_on_screen < ybuffer[i as usize] {
                let color = color_map.get_pixel(wrapped_x, wrapped_y);
                draw_line(
                    i.into(), height_on_screen as f32, 
                    i.into(), ybuffer[i as usize] as f32, 
                    1.0,
                    color
                );
                ybuffer[i as usize] = height_on_screen;
            }
            
            p_left_x += dx;
            p_left_y += dy;
            
        }

        z += dz;
        dz += 0.02;
       
    }
}

