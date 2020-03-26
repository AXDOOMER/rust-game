extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::WindowCanvas;

use std::vec::Vec;
use std::cmp;
use std::{thread, time};
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
const LEVEL_SIZE: usize = 20;
const BLOCK_SIZE: i32 = 40;
const SCREEN_WIDTH: i32 = 640;
const SCREEN_HEIGHT: i32 = 480;

fn render(canvas: &mut WindowCanvas, player: &Player, lvl : [[bool; LEVEL_SIZE]; LEVEL_SIZE]) {
	// Player position on screen
	let mut camx = SCREEN_WIDTH / 2 - player.x - BLOCK_SIZE / 2;
	let mut camy = SCREEN_HEIGHT / 2 - player.y - BLOCK_SIZE / 2;

	// Limit the camera's position range
	camx = cmp::min(camx, 0);
	camx = cmp::max(camx, -(BLOCK_SIZE * LEVEL_SIZE as i32) + SCREEN_WIDTH);

	camy = cmp::min(camy, 0);
	camy = cmp::max(camy, -(BLOCK_SIZE * LEVEL_SIZE as i32) + SCREEN_HEIGHT);

	// Background
    canvas.set_draw_color(Color::RGB(0, 128, 0));
    canvas.clear();

	// Draw player
	canvas.set_draw_color(Color::RGB(0, 0, 0));
	let _ = canvas.fill_rect(Rect::new(camx + player.x, camy + player.y, 40, 40));

	// Draw level
	canvas.set_draw_color(Color::RGB(80, 40, 13));
	for i in 0..lvl.len() {
		for j in 0..lvl[0].len() {
			if lvl[i][j] {
				let _ = canvas.fill_rect(Rect::new(camx + j as i32 * BLOCK_SIZE,
					camy + i as i32 * BLOCK_SIZE, BLOCK_SIZE as u32, BLOCK_SIZE as u32));
			}
		}
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
		x: 128, y: 64, g: 0
	};

	/****************************** LEVEL LOADING ******************************/

	// Create level
	let mut level: [[bool; LEVEL_SIZE]; LEVEL_SIZE] = [[false; LEVEL_SIZE]; LEVEL_SIZE];

	// spawn spots
	let mut spawn_spots = Vec::<(i32, i32)>::new();

	let mut blocks = Vec::<(i32, i32)>::new();

	// load from this for now, load from a file later
	let lvlstrs: [String; LEVEL_SIZE] = 
	[
		String::from("####################"),
		String::from("#....$.............#"),
		String::from("#..................#"),
		String::from("#........####....###"),
		String::from("#.#######..........#"),

		String::from("#..................#"),
		String::from("#................###"),
		String::from("#.......#######....#"),
		String::from("#..................#"),
		String::from("#.................##"),

		String::from("#...########.......#"),
		String::from("#..................#"),
		String::from("#........######....#"),
		String::from("#...#.............##"),
		String::from("#..................#"),

		String::from("########...........#"),
		String::from("#...........###....#"),
		String::from("#..................#"),
		String::from("#..................#"),
		String::from("####################"),
	];	

	for (i, line) in lvlstrs.iter().enumerate() {
		for (j, c) in line.chars().enumerate() {
			if c == '#' {
				level[i][j] = true;
				blocks.push((j as i32 * BLOCK_SIZE, i as i32 * BLOCK_SIZE));
			} else if c == '$' {
				player.x = j as i32 * BLOCK_SIZE;
				player.y = i as i32 * BLOCK_SIZE;
				spawn_spots.push((j as i32 * BLOCK_SIZE, i as i32 * BLOCK_SIZE));
			}
			print!("{}", c);
		}
		print!("\n");
	}

	/****************************** INIT SDL2 ******************************/

	let sdl_context = sdl2::init()?;
	let video_subsystem = sdl_context.video()?;

	let window = video_subsystem.window("Window", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
		.position_centered()
		.build()
		.expect("Error: could not create window.");

	let mut canvas = window.into_canvas()
		.build()
		.expect("Error: could not create canvas.");

	/****************************** EVENTS AND ACTIONS ******************************/

	let mut event_pump = sdl_context.event_pump()?;

	let mut quit = false;

	let mut in_air = true;

	// Keys
	let mut key_left = false;
	let mut key_right = false;
	let mut key_up = false;
//	let mut key_down = false;
//	let mut key_space = false;

	/****************************** MAIN LOOP ******************************/

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
		if key_up && !in_air && player.g < 4 {
			player.g = -16;
			in_air = true;
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
					in_air = false;
				}
			}
		}

		/****************************** EPILOGUE ******************************/

		render(&mut canvas, &player, level);

		thread::sleep(time::Duration::from_millis(1000 / 60));
	}

	Ok(())
}
