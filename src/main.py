import pygame
import pygame.gfxdraw
import math
import numpy as np

screen_vector = pygame.Vector2(800, 600)

class Player:
    def __init__(self) -> None:
        self.pos = pygame.Vector3(screen_vector.x/2, screen_vector.y/2, 50)
        self.phi = 0
        self.horizon = 120
    
    def move(self, distance: float) -> None:
        delta_x = distance * math.cos(self.phi)
        delta_y = distance * math.sin(self.phi)

        self.pos.x += delta_x
        self.pos.y += delta_y
        

def render(screen, p, scale_height, distance, color_map, height_map, debug_rendering = False):
    screen_width, screen_height = screen.get_size()
    map_width, map_height = height_map.get_size()

    sinphi = math.sin(p.phi)
    cosphi = math.cos(p.phi)

    ybuffer = np.full(screen_width, screen_height) 
    

    dz = 1.0
    z = 1.0
      

    while z < distance:
        pleft = pygame.Vector2(
            (-cosphi*z - sinphi*z) - p.pos.y,
            ( sinphi*z - cosphi*z) - p.pos.x
        )
        pright = pygame.Vector2(
            ( cosphi*z - sinphi*z) - p.pos.y,
            (-sinphi*z - cosphi*z) - p.pos.x
        )

        dx = (pright.x - pleft.x) / screen_width
        dy = (pright.y - pleft.y) / screen_width

        for i in range(0, screen_width):

            wrapped_x = int(pleft.x) % map_width
            wrapped_y = int(pleft.y) % map_height

            height_value = height_map.get_at((wrapped_x, wrapped_y)).hsla[2]

            height_value = (height_value / 255.0) * scale_height

            height_on_screen = (p.pos.z - height_value) / z * scale_height + p.horizon


            
            if height_on_screen < ybuffer[i]:
                color = color_map.get_at((wrapped_x, wrapped_y))


                pygame.gfxdraw.vline(screen, i, int(height_on_screen), int(ybuffer[i]), color)
                if debug_rendering:
                    pygame.display.flip()
                    
                ybuffer[i] = height_on_screen

            pleft.x += dx  
            pleft.y += dy       

        z += dz
        dz += 0.02

def main():
    pygame.init()
    screen = pygame.display.set_mode((screen_vector.x, screen_vector.y))
    clock = pygame.time.Clock()
    running = True
    debug_rendering = False

    color_map_path = "assets/testMap/C1W.png"
    height_map_path = "assets/testMap/D1.png"

    color_map = pygame.image.load(color_map_path)
    height_map = pygame.image.load(height_map_path)

    font = pygame.font.SysFont('Arial', 24)

    player = Player()

    

    while running:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False
            elif event.type == pygame.KEYDOWN:
                if event.key == pygame.K_h:
                    debug_rendering = not debug_rendering
                
        keys = pygame.key.get_pressed()
        if keys[pygame.K_LEFT]: player.phi += 0.1
        if keys[pygame.K_RIGHT]: player.phi -= 0.1
        if keys[pygame.K_UP]: player.horizon += 10
        if keys[pygame.K_DOWN]: player.horizon -= 10
        if keys[pygame.K_w]: player.move(10)
        if keys[pygame.K_s]: player.move(-10)
        
        screen.fill((0, 0, 0))
        render(screen, player, 300, 200, color_map, height_map, debug_rendering)

        fps = int(clock.get_fps())
        fps_text = font.render(f"FPS: {fps}", True, pygame.Color('white'))
        screen.blit(fps_text, (10, 10))

        pygame.display.flip()
        clock.tick(60)
    pygame.quit()


if __name__ == "__main__":
    
    main()


