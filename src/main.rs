extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::WindowCanvas;

use std::vec::Vec;
use std::cmp;
use std::{thread, time};
use std::fs::File;
use std::io::{BufRead, BufReader};
/*
#[derive(PartialEq)]
enum Actions {
	Up,
	Down,
	Left,
	Right,
	Jump,
	Shoot,
}
*/
struct Player {
	x: i32,
	y: i32,
	g: i32,
	in_air: bool,
}
/*
impl Player {
	pub fn get_x(&self) -> i32 {
		self.x
	}

	pub fn get_y(&self) -> i32 {
		self.y
	}
}
*/
const BLOCK_SIZE: i32 = 40;
const SCREEN_WIDTH: i32 = 640;
const SCREEN_HEIGHT: i32 = 480;

fn render(canvas: &mut WindowCanvas, player: &Player, blocks: &Vec::<(i32, i32)>, level_width: i32, level_height: i32) {
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
	let _ = canvas.fill_rect(Rect::new(camx + player.x, camy + player.y, 40, 40));

	// Draw level
	canvas.set_draw_color(Color::RGB(80, 40, 13));
	for block in blocks {
		let _ = canvas.fill_rect(Rect::new(camx + block.0, camy + block.1, BLOCK_SIZE as u32, BLOCK_SIZE as u32));
	}

	// Display to screen
    canvas.present();
}

// AABB collision test: https://tutorialedge.net/gamedev/aabb-collision-detection-tutorial/
fn aabb_test(e1x: i32, e1y: i32, e1s: i32, e2x: i32, e2y: i32, e2s: i32) -> bool {
	if e1x < e2x + e2s &&
		e1x + e1s > e2x &&
		e1y < e2y + e2s &&
		e1y + e1s > e2y {
		return true;
	}
	return false;
}

fn main() -> Result<(), String> {
	// Init player
	let mut player = Player {
		x: 128, y: 64, g: 0, in_air: true
	};

	/****************************** LEVEL LOADING ******************************/

	// Create level
	let mut spawn_spots = Vec::<(i32, i32)>::new();
	let mut blocks = Vec::<(i32, i32)>::new();
	let mut level_width = 0;
	let mut level_height = 0;

	// Load a level from a file
	let f = File::open("res/level1.txt").expect("Error: Unable to open level.");
	let f = BufReader::new(f);

	for (i, line) in f.lines().enumerate() {
		let line = line.expect("Unable to read line.");

		level_height += 1;
		level_width = cmp::max(level_width, line.len() as i32);

		for (j, c) in line.chars().enumerate() {
			let i = i as i32;
			let j = j as i32;

			if c == '#' {
				blocks.push((j * BLOCK_SIZE, i * BLOCK_SIZE));
			} else if c == '$' {
				player.x = j * BLOCK_SIZE;
				player.y = i * BLOCK_SIZE;
				spawn_spots.push((j * BLOCK_SIZE, i * BLOCK_SIZE));
			}
			print!("{}", c);
		}
		print!("\n");
	}


	/****************************** INIT SDL2 ******************************/

	let sdl_context = sdl2::init()?;
	let video_subsystem = sdl_context.video()?;

	let window = video_subsystem.window("Rust Game", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
		.position_centered()
		.resizable()
		.build()
		.expect("Error: could not create window.");

	let mut canvas = window.into_canvas()
		.build()
		.expect("Error: could not create canvas.");

	let _ = canvas.set_logical_size(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);

	/****************************** EVENTS AND ACTIONS ******************************/

	let mut event_pump = sdl_context.event_pump()?;

	let mut quit = false;

	// Keys
	let mut key_left = false;
	let mut key_right = false;
	let mut key_up = false;
//	let mut key_down = false;
//	let mut key_space = false;

	/****************************** GAME LOOP ******************************/

	while !quit {

		/****************************** EVENT LOOP ******************************/

		// Catch events
		for event in event_pump.poll_iter() {
			match event {
				// Game control
				Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => quit = true,

				// Player control
				Event::KeyDown { keycode, .. } => {
					match keycode {
						Some(Keycode::Left) => key_left = true,
						Some(Keycode::Right) => key_right = true,
						Some(Keycode::Up) => key_up = true,
//						Some(Keycode::Down) => key_down = true,
//						Some(Keycode::Space) => key_space = true,

						_ => {}
					}
				}

				Event::KeyUp { keycode, .. } => {
					match keycode {
						Some(Keycode::Left) => key_left = false,
						Some(Keycode::Right) => key_right = false,
						Some(Keycode::Up) => key_up = false,
//						Some(Keycode::Down) => key_down = false,
//						Some(Keycode::Space) => key_space = false,

						_ => {}
					}
				}

				_ => {}
			}
		}

		/****************************** MOVEMENT AND PHYSICS ******************************/

		// Test one axis at the time. This is so the player can properly slide against walls.
		// https://www.reddit.com/r/gamedev/comments/qijw6/collision_detection_why_does_notch_do_separate_x/
		// https://gist.github.com/mrspeaker/1978410

		if key_left { player.x -= 8; }
		if key_right { player.x += 8; }

		for block in &blocks {
			if aabb_test(player.x, player.y, BLOCK_SIZE, block.0, block.1, BLOCK_SIZE) {
				// Player hit obstacle on his left
				if block.0 < player.x {
					player.x = block.0 + BLOCK_SIZE;
				}

				// Player hit obstacle on his right
				if block.0 > player.x {
					player.x = block.0 - BLOCK_SIZE;
				}
			}
		}

		// Jump.
		// The player is allowed to jump in-air shortly after falling to make jumps easier.
		if key_up && !player.in_air && player.g < 4 {
			player.g = -16;
			player.in_air = true;
		}

		// Apply gravity
		player.y += player.g;

		// When player is falling, cap the maximum falling speed
		if player.g < 16 {
			player.g += 1;
		}

		for block in &blocks {
			if aabb_test(player.x, player.y, BLOCK_SIZE, block.0, block.1, BLOCK_SIZE) {
				player.g = 0;

				// Player hit obstacle with his head
				if block.1 < player.y {
					player.y = block.1 + BLOCK_SIZE;
				}

				// Player hit obstacle with his feet
				if block.1 > player.y {
					player.y = block.1 - BLOCK_SIZE;
					player.in_air = false;
				}
			}
		}

		/****************************** EPILOGUE ******************************/

		render(&mut canvas, &player, &blocks, level_width, level_height);

		thread::sleep(time::Duration::from_millis(1000 / 60));
	}

	/****************************** EXIT ******************************/

	Ok(())
}
