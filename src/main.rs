use std::default;

use macroquad::ui::root_ui;
use macroquad::prelude::*;

struct Player {
    pos: Vec3,
    phi: f64,
    horizon: f64,
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

struct Entity {
    pos: Vec3,
    sprite: Image,
    size: f64,
}

impl Entity {
    async fn new(sprite_path: &str, pos: Vec3, size: f64) -> Result<Self, macroquad::Error> {
        let sprite = load_image(sprite_path).await?;

        Ok(Entity { pos, sprite, size })
    }
}

struct Enviorment {
    color_map: Image,
    height_map: Image,
    fog_color: Color,
    sky_color: Color,
    horizon_color: Color,
}

impl Enviorment {
    async fn new(
        color_path: &str,
        height_path: &str,
        fog_color: Color,
        sky_color: Color,
        horizon_color: Color,
    ) -> Result<Self, macroquad::Error> {
        let color_map = load_image(color_path).await?;
        let height_map = load_image(height_path).await?;

        Ok(Enviorment {
            color_map,
            height_map,
            fog_color,
            sky_color,
            horizon_color,
        })
    }
}

#[derive(Debug)]
struct DepthBuffer {
    ybuffer: Vec<f64>,
    dbuffer: Vec<Vec<f64>>,
}

impl DepthBuffer {
    fn new(default: f64, size_x: usize, size_y: usize) -> Self {
        DepthBuffer {
            ybuffer: vec![default; size_x],
            dbuffer: vec![vec![f64::INFINITY; size_y]; size_x],
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "test".to_owned(),
        window_width: 1920,
        window_height: 1080,
        // window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let env = match Enviorment::new(
        "./assets/testMap/C1W.png",
        "./assets/testMap/D1.png",
        Color::from_rgba(50, 50, 127, 255),
        BLUE,
        GREEN,
    )
    .await
    {
        Ok(env) => env,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };

    let mut p = Player::new();

    let test_sprite: Entity = match Entity::new(
        "./assets/sprites/coin.png",
        Vec3::new(512.0, 180.0, 80.0),
        // 1.0,
        0.05,
    )
    .await
    {
        Ok(e) => e,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };

    let entitis = vec![test_sprite];

    loop {
        let scale_height = (screen_height() / 2.0) as f64;

        clear_background(env.sky_color);

        render(
            &mut p,
            scale_height,
            500.0,
            &env,
            screen_width() as f64,
            screen_height() as f64,
            &entitis,
        );

        draw_text(&format!("{} FPS", get_fps()), 10.0, 20.0, 30.0, BLACK);
        draw_text(
            &format!(
                "xyz: {}, {}, {}",
                p.pos.x.round(),
                p.pos.y.round(),
                p.pos.z.round()
            ),
            100.0,
            20.0,
            30.0,
            BLACK,
        );

        for v in get_keys_down() {
            match v {
                KeyCode::A => p.phi += (5.0 * get_frame_time()) as f64,
                KeyCode::D => p.phi -= (5.0 * get_frame_time()) as f64,

                KeyCode::Q => p.horizon += (500.0 * get_frame_time()) as f64,
                KeyCode::E => p.horizon -= (500.0 * get_frame_time()) as f64,

                KeyCode::S => p.move_in_dir(500.0 * get_frame_time()),
                KeyCode::W => p.move_in_dir(-500.0 * get_frame_time()),

                KeyCode::Space => p.pos.z += 500.0 * get_frame_time(),
                KeyCode::LeftShift => p.pos.z -= 500.0 * get_frame_time(),
                _ => (),
            }
        }

        // break;
        next_frame().await;
    }
}

fn render(
    p: &mut Player,
    scale_height: f64,
    distance: f64,
    env: &Enviorment,
    screen_width: f64,
    screen_height: f64,
    entities: &Vec<Entity>,
) {
    let sinphi = p.phi.sin();
    let cosphi = p.phi.cos();

    let mut depth_buf = DepthBuffer::new(screen_height, screen_width as usize, screen_height as usize);

    let mut dz = 1.0;
    let mut z = 1.0;

    while z < distance {
        let mut p_left_x = (-cosphi * z - sinphi * z) + p.pos.x as f64;
        let mut p_left_y = (sinphi * z - cosphi * z) + p.pos.y as f64;

        let p_right_x = (cosphi * z - sinphi * z) + p.pos.x as f64;
        let p_right_y = (-sinphi * z - cosphi * z) + p.pos.y as f64;

        let dx = (p_right_x - p_left_x) / screen_width;
        let dy = (p_right_y - p_left_y) / screen_width;

        for i in 0..screen_width as usize {
            let wrapped_x = p_left_x.abs() as u32 % 1024;
            let wrapped_y = p_left_y.abs() as u32 % 1024;

            let height_value = env.height_map.get_pixel(wrapped_x, wrapped_y).b * 255.0;

            let height_on_screen: f64 =
                (p.pos.z - height_value) as f64 / z * scale_height + p.horizon;

            if height_on_screen < depth_buf.ybuffer[i as usize] {
                let source_color = env.color_map.get_pixel(wrapped_x, wrapped_y);

                let t = (z / distance) as f32;

                let color: Color = lerp_color(source_color, env.fog_color, t);

                draw_line(
                    i as f32,
                    height_on_screen as f32,
                    i as f32,
                    depth_buf.ybuffer[i as usize] as f32,
                    1.0,
                    color,
                );
                
                for ix in height_on_screen as usize..depth_buf.ybuffer[i] as usize {
                    depth_buf.dbuffer[i][ix] = z;
                }

                depth_buf.ybuffer[i as usize] = height_on_screen;
            }

            p_left_x += dx;
            p_left_y += dy;
        }

        z += dz;
        dz += 0.02;
    }

    // let mut temp = Image::gen_image_color(screen_width as u16, screen_height as u16, WHITE);
    
    // for x in 0..depth_buf.dbuffer.len() {
    //     for y in 0..depth_buf.dbuffer[x].len() {
    //         let c = depth_buf.dbuffer[x][y] as u8;
    //         if depth_buf.dbuffer[x][y] != f64::INFINITY {
    //             temp.set_pixel(x as u32, y as u32, Color::from_rgba(c, c, c, 255))
    //         }
    //     }
    // }
    // let text = Texture2D::from_image(&temp);
    // draw_texture(&text, 0.0, 0.0, WHITE);

    render_sprites(
        p,
        entities,
        &mut depth_buf,
        scale_height,
        distance,
        env,
        screen_width,
        screen_height,
    );
}

fn render_sprites(
    p: &mut Player,
    entities: &Vec<Entity>,
    depth_buf: &mut DepthBuffer,
    scale_height: f64,
    distance: f64,
    env: &Enviorment,
    screen_width: f64,
    screen_height: f64,
) {
    let sinphi = p.phi.sin();
    let cosphi = p.phi.cos();

    for s in entities {
        let dx: f64 = (s.pos.x - p.pos.x) as f64;
        let dy: f64 = (s.pos.y - p.pos.y) as f64;
        let dz: f64 = (s.pos.z - p.pos.z) as f64;

        let tx = cosphi * dx - sinphi * dy;
        let ty: f64 = -sinphi * dx - cosphi * dy;

        if ty > 0.0 {
            let adjusted_horizon = p.horizon - dz * scale_height / ty;

            let sprite_screen_x = (screen_width / 2.0) * (1.0 + tx / ty);
            let sprite_height = (screen_height / ty) * s.size * scale_height;
            let sprite_height = sprite_height.min(screen_height);

            let draw_start_y = (adjusted_horizon - (sprite_height / 2.0)).max(0.0) as usize;
            let draw_end_y = (adjusted_horizon + (sprite_height / 2.0)).min(screen_height) as usize;

            let sprite_width = sprite_height;

            let draw_start_x = (sprite_screen_x - (sprite_width / 2.0)).max(0.0) as usize;
            let draw_end_x = (sprite_screen_x + (sprite_width / 2.0)).min(screen_width) as usize;

            for stripe in draw_start_x..draw_end_x {
                if ty < distance {
                    let tex_x = (((stripe as f64 - draw_start_x as f64) * s.sprite.width as f64)
                        / sprite_width) as u32;

                    for y in draw_start_y..draw_end_y {
                        let tex_y = (((y as f64 - draw_start_y as f64) * s.sprite.height as f64)
                            / sprite_height) as u32;

                        if ty < depth_buf.dbuffer[stripe][y] {
                            let source_color = s.sprite.get_pixel(tex_x, tex_y);

                            let t = (ty / distance) as f32;

                            let color: Color = lerp_color(source_color, env.fog_color, t);
                            draw_rectangle(stripe as f32, y as f32, 1.0, 1.0, color);
                        }
                    }
                }
            }
        }
    }
}

fn lerp_color(color1: Color, color2: Color, t: f32) -> Color {
    Color {
        r: color1.r + t * (color2.r - color1.r),
        g: color1.g + t * (color2.g - color1.g),
        b: color1.b + t * (color2.b - color1.b),
        a: color1.a,
    }
}
