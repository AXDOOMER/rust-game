extern crate sdl2;
extern crate rand;

mod structs;
use structs::Bullet;
use structs::Drone;
use structs::Events;
use structs::Game;
use structs::Level;
use structs::Player;

mod utils;
use utils::aabb_test;
use utils::distance2d;
use utils::line2box;
use utils::line2line;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::WindowCanvas;

use rand::Rng;

use std::vec::Vec;
use std::cmp;
use std::{thread, time};
use std::fs::File;
use std::io::{BufRead, BufReader};

const BLOCK_SIZE: i32 = 40;
const BULLET_SIZE: i32 = 8;
const SCREEN_WIDTH: i32 = 640;
const SCREEN_HEIGHT: i32 = 480;

fn render(canvas: &mut WindowCanvas, player: &Player, drones: &Vec::<Drone>, bullets: &Vec::<Bullet>, blocks: &Vec::<(i32, i32)>, level_width: i32, level_height: i32) {
	// Camera's position so the player is centered on screen
	let mut camx = SCREEN_WIDTH / 2 - player.x - BLOCK_SIZE / 2;
	let mut camy = SCREEN_HEIGHT / 2 - player.y - BLOCK_SIZE / 2;

	// Limit the camera's position range
	camx = cmp::min(camx, 0);
	camx = cmp::max(camx, -(BLOCK_SIZE * level_width) + SCREEN_WIDTH);

	camy = cmp::min(camy, 0);
	camy = cmp::max(camy, -(BLOCK_SIZE * level_height) + SCREEN_HEIGHT);

	// Window background
	canvas.set_draw_color(Color::RGB(0, 0, 0));
	canvas.clear();

	// Game background
	canvas.set_draw_color(Color::RGB(0, 128, 0));
	let _ = canvas.fill_rect(Rect::new(0, 0, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32));

	// Draw player
	canvas.set_draw_color(Color::RGB(0, 0, 0));
	if player.alive {
		let _ = canvas.fill_rect(Rect::new(camx + player.x, camy + player.y, 40, 40));
	} else {
		let _ = canvas.fill_rect(Rect::new(camx + player.x, camy + player.y + 22, 40, 18));
	}

	canvas.set_draw_color(Color::RGB(255, 255, 255));
	if player.going_right {
		let _ = canvas.fill_rect(Rect::new(camx + player.x + 20, camy + player.y + 20, 18, 4));
	} else {
		let _ = canvas.fill_rect(Rect::new(camx + player.x + 2, camy + player.y + 20, 18, 4));
	}

	// Draw enemy drones
	canvas.set_draw_color(Color::RGB(192, 0, 0));
	for drone in drones {
		let _ = canvas.fill_rect(Rect::new(camx + drone.x, camy + drone.y, BLOCK_SIZE as u32, BLOCK_SIZE as u32));
	}

	// Draw bullets
	canvas.set_draw_color(Color::RGB(255, 255, 0));
	for bullet in bullets {
//		if bullet.x >= player.x - SCREEN_WIDTH && bullet.x <= player.x + SCREEN_WIDTH {
			let _ = canvas.fill_rect(Rect::new(camx + bullet.x, camy + bullet.y, BULLET_SIZE as u32, BULLET_SIZE as u32));
//		}
	}

	// Draw level
	canvas.set_draw_color(Color::RGB(80, 40, 13));
	for block in blocks {
		let _ = canvas.fill_rect(Rect::new(camx + block.0, camy + block.1, BLOCK_SIZE as u32, BLOCK_SIZE as u32));
	}

	// Display to screen
    canvas.present();
}


fn main() -> Result<(), String> {

	/****************************** INIT SDL2 ******************************/

	let sdl_context = sdl2::init()?;
	let video_subsystem = sdl_context.video()?;

	let window = video_subsystem.window("Rust Game", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
		.position_centered()
		.resizable()
		/*.opengl()*/
		.build()
		.expect("Error: could not create window.");

	let mut canvas = window.into_canvas()
		.build()
		.expect("Error: could not create canvas.");

	let _ = canvas.set_logical_size(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);

	sdl2::hint::set_video_minimize_on_focus_loss(false);

	/****************************** GAME STRUCTURES ******************************/

	let mut g: Game;
	let mut lvl: Level;

	lvl = Level {
		// Init player
		player: Player::new(),

		level_width: 0,
		level_height: 0,

		// Init entities
		spawn_spots: Vec::<(i32, i32)>::new(),
		blocks: Vec::<(i32, i32)>::new(),
		drones: Vec::<Drone>::new(),	// Enemy AI
		bullets: Vec::<Bullet>::new(),
	};

	/****************************** EVENTS AND ACTIONS ******************************/

	g = Game {

		events: Events {

			event_pump: sdl_context.event_pump()?,

			quit: false,

			// Keys
			key_left: false,
			key_right: false,
			key_up: false,

			key_attack: false,
			key_fullscreen: false,

			set_fullscreen: 0,
			set_shoot: 0,
		},

		rng: rand::thread_rng(),

		sdl_context: sdl_context,

		canvas: canvas,
	};


	/****************************** LEVEL LOADING ******************************/

	// Create level
	lvl.level_width = 0;
	lvl.level_height = 0;

	// Load a level from a file
	let f = File::open("res/level3.txt").expect("Error: Unable to open level.");
	let f = BufReader::new(f);

	for (i, line) in f.lines().enumerate() {
		let line = line.expect("Unable to read line.");

		lvl.level_height += 1;
		lvl.level_width = cmp::max(lvl.level_width, line.len() as i32);

		for (j, c) in line.chars().enumerate() {
			let i = i as i32;
			let j = j as i32;

			if c == '#' {
				lvl.blocks.push((j * BLOCK_SIZE, i * BLOCK_SIZE));
			} else if c == 'd' {
				let drone = Drone {
					x: j * BLOCK_SIZE,
					y: i * BLOCK_SIZE,
					pursuit: 0,
					going_right: true,
					health: 3,
					shoot_timer: 0,
				};
				lvl.drones.push(drone);
			} else if c == '$' {
				lvl.player.x = j * BLOCK_SIZE;
				lvl.player.y = i * BLOCK_SIZE;
				lvl.spawn_spots.push((j * BLOCK_SIZE, i * BLOCK_SIZE));
			}
			print!("{}", c);
		}
		print!("\n");
	}

	/****************************** GAME LOOP ******************************/

	while !g.events.quit {

		/****************************** EVENT LOOP ******************************/

		// Catch events
		for event in g.events.event_pump.poll_iter() {
			match event {
				// Game control
				Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => g.events.quit = true,

				// Player control
				Event::KeyDown { keycode, .. } => {
					match keycode {
						Some(Keycode::Left) => g.events.key_left = true,
						Some(Keycode::Right) => g.events.key_right = true,
						Some(Keycode::Up) => g.events.key_up = true,
//						Some(Keycode::Down) => key_down = true,
						Some(Keycode::Space) | Some(Keycode::LCtrl) => g.events.key_attack = true,

						Some(Keycode::F4) => g.events.key_fullscreen = true,

						_ => {}
					}
				}

				Event::KeyUp { keycode, .. } => {
					match keycode {
						Some(Keycode::Left) => g.events.key_left = false,
						Some(Keycode::Right) => g.events.key_right = false,
						Some(Keycode::Up) => g.events.key_up = false,
//						Some(Keycode::Down) => key_down = false,
						Some(Keycode::Space) | Some(Keycode::LCtrl) => g.events.key_attack = false,

						Some(Keycode::F4) => g.events.key_fullscreen = false,

						_ => {}
					}
				}

				_ => {}
			}
		}


		if g.events.key_fullscreen {
			g.events.set_fullscreen += 1
		} else {
			g.events.set_fullscreen = 0;
		}

		if g.events.set_fullscreen == 1 && g.canvas.window_mut().fullscreen_state() == sdl2::video::FullscreenType::Off {
			let _ = g.canvas.window_mut().set_fullscreen(sdl2::video::FullscreenType::Desktop);
			g.sdl_context.mouse().show_cursor(false);
		} else if g.events.set_fullscreen == 1 && g.canvas.window_mut().fullscreen_state() == sdl2::video::FullscreenType::Desktop {
			let _ = g.canvas.window_mut().set_fullscreen(sdl2::video::FullscreenType::Off);
			g.sdl_context.mouse().show_cursor(true);
		}

/*		if key_fullscreen {
			canvas.window_mut().maximize();
		}*/

		/****************************** MOVEMENT AND PHYSICS ******************************/

		// Test one axis at the time. This is so the player can properly slide against walls.
		// https://www.reddit.com/r/gamedev/comments/qijw6/collision_detection_why_does_notch_do_separate_x/
		// https://gist.github.com/mrspeaker/1978410

		if lvl.player.alive {
			if g.events.key_left {
				lvl.player.x -= 8;
				lvl.player.going_right = false;
			}

			if g.events.key_right {
				lvl.player.x += 8;
				lvl.player.going_right = true;
			}

			for block in &lvl.blocks {
				if aabb_test(lvl.player.x, lvl.player.y, BLOCK_SIZE, block.0, block.1, BLOCK_SIZE) {
					// Player hit obstacle on his left
					if block.0 < lvl.player.x {
						lvl.player.x = block.0 + BLOCK_SIZE;
					}

					// Player hit obstacle on his right
					if block.0 > lvl.player.x {
						lvl.player.x = block.0 - BLOCK_SIZE;
					}
				}
			}

			// Jump.
			// The player is allowed to jump in-air until he reaches a certain falling speed.
			// This short jump extension makes jump timing easier.
			if g.events.key_up && !lvl.player.in_air && lvl.player.g < 4 {
				lvl.player.g = -16;
				lvl.player.in_air = true;
			}
		}

		// Apply gravity
		lvl.player.y += lvl.player.g;

		// When player is falling, cap the maximum falling speed
		if lvl.player.g < 16 {
			lvl.player.g += 1;
		}

		for block in &lvl.blocks {
			if aabb_test(lvl.player.x, lvl.player.y, BLOCK_SIZE, block.0, block.1, BLOCK_SIZE) {
				lvl.player.g = 0;

				// Player hit obstacle with his head
				if block.1 < lvl.player.y {
					lvl.player.y = block.1 + BLOCK_SIZE;
				}

				// Player hit obstacle with his feet
				if block.1 > lvl.player.y {
					lvl.player.y = block.1 - BLOCK_SIZE;
					lvl.player.in_air = false;
				}
			}
		}

		// Touching an enemy kills
		if lvl.player.alive {
			for drone in &mut lvl.drones {
				if aabb_test(lvl.player.x, lvl.player.y, BLOCK_SIZE, drone.x, drone.y, BLOCK_SIZE) {
					lvl.player.alive = false;
				}
			}
		}

		/****************************** PLAYER FIRES BULLETS ******************************/

		if g.events.key_attack && lvl.player.alive {
			g.events.set_shoot += 1
		} else {
			g.events.set_shoot = 0;
		}

		if g.events.set_shoot % 10 == 1 {
			let bullet = Bullet {
				x: if lvl.player.going_right { lvl.player.x + 40 - 8 } else { lvl.player.x },
				y: lvl.player.y + 18,
				going_right: lvl.player.going_right,
				source_is_drone: false,
			};
			lvl.bullets.push(bullet);
		}

		/****************************** BULLETS MANAGEMENT ******************************/

		// Move bullets are different speed depending on who shoot them
		for bullet in &mut lvl.bullets {
			let mut bullet_speed = 20;
			if bullet.source_is_drone {
				bullet_speed = 10;
			}

			if bullet.going_right {
				bullet.x += bullet_speed;
			} else {
				bullet.x -= bullet_speed;
			}
		}

		// Bullets hits on a drone removes health
		for i in (0..lvl.drones.len()).rev() {
			for j in (0..lvl.bullets.len()).rev() {
				if aabb_test(lvl.bullets[j].x, lvl.bullets[j].y, BULLET_SIZE, lvl.drones[i].x, lvl.drones[i].y, BLOCK_SIZE) {
					if lvl.drones[i].health > 1 {
						lvl.drones[i].health -= 1;
					} else {
						lvl.drones.remove(i);
					}
					lvl.bullets.remove(j);
					break;
				}
			}
		}

		// Bullet hits on a player kills him
		for i in (0..lvl.bullets.len()).rev() {
			if aabb_test(lvl.bullets[i].x, lvl.bullets[i].y, BULLET_SIZE, lvl.player.x, lvl.player.y, BLOCK_SIZE) {
				if lvl.player.alive {
					lvl.bullets.remove(i);
					lvl.player.alive = false;
				}
			}
		}

		// Delete bullets that hit walls
		for block in &mut lvl.blocks {
			lvl.bullets.retain(|i| !aabb_test(i.x, i.y, BULLET_SIZE, block.0, block.1, BLOCK_SIZE));
		}


		/****************************** ENEMY AI ******************************/

		for drone in &mut lvl.drones {

			let mut is_close = distance2d(lvl.player.x, lvl.player.y, drone.x, drone.y) < 280;

			let mut line_of_sight = true;

			// Always increment shoot timer. It's set to 0 when the drone shoots.
			drone.shoot_timer += 1;

			for block in &lvl.blocks {
				let intersec =
					line2box(lvl.player.x + BLOCK_SIZE / 2, lvl.player.y + BLOCK_SIZE / 2,
						drone.x + BLOCK_SIZE / 2, drone.y + BLOCK_SIZE / 2,
						block.0, block.1, BLOCK_SIZE);

				if intersec {
					// Drone can's see the player
					is_close = false;
					line_of_sight = false;
					//break;
				}
			}

			// Decrease the pursuit counter when the player is out of range
			if !is_close && drone.pursuit > 0 {
				drone.pursuit -= 1;
			}

			if is_close || drone.pursuit > 30 {

				if is_close && drone.pursuit < 120 {
					drone.pursuit += 2;
				}

				let dx = (lvl.player.x - drone.x).abs();
				let dy = (lvl.player.y - drone.y).abs();

				// Check the direction to move

				// Drones try to lineup with the player on the Y axis first
				if dy > 16 {

					if drone.y < lvl.player.y {
						drone.y += 6;
					} else if drone.y > lvl.player.y {
						drone.y -= 6;
					}

					// Check collisions on the Y axis
					for block in &lvl.blocks {
						if aabb_test(drone.x, drone.y, BLOCK_SIZE, block.0, block.1, BLOCK_SIZE) {
							// Drone hit obstacle above
							if block.1 < drone.y {
								drone.y = block.1 + BLOCK_SIZE;
							}

							// Drone hit obstacle below
							if block.1 > drone.y {
								drone.y = block.1 - BLOCK_SIZE;
							}
						}
					}
				} else {
					// In the way of a shot
					let random_number = g.rng.gen::<u32>();

					// Shoot chance
					if random_number % 30 == 0 && drone.shoot_timer > 90 {
						let bullet = Bullet {
							x: if drone.x < lvl.player.x { drone.x + 40 } else { drone.x - 8 },
							y: drone.y + 18,
							going_right: drone.x < lvl.player.x,
							source_is_drone: true,
						};

						lvl.bullets.push(bullet);
						drone.shoot_timer = 0;
					}
				}

				if dx > 100 || !line_of_sight {

					if drone.x < lvl.player.x {
						drone.x += 4;
					} else if drone.x > lvl.player.x {
						drone.x -= 4;
					}

					// Check collisions on the X axis
					for block in &lvl.blocks {
						if aabb_test(drone.x, drone.y, BLOCK_SIZE, block.0, block.1, BLOCK_SIZE) {
							// Drone hit obstacle on his left
							if block.0 < drone.x {
								drone.x = block.0 + BLOCK_SIZE;
							}

							// Drone hit obstacle on his right
							if block.0 > drone.x {
								drone.x = block.0 - BLOCK_SIZE;
							}
						}
					}
				}


			}/* else {
				drone.pursuit = 0;
			}*/
		}

		/****************************** EPILOGUE ******************************/

		render(&mut g.canvas, &lvl.player, &lvl.drones, &lvl.bullets, &lvl.blocks, lvl.level_width, lvl.level_height);

		thread::sleep(time::Duration::from_millis(1000 / 60));
	}

	/****************************** EXIT ******************************/

	Ok(())
}
