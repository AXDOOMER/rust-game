use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::WindowCanvas;

pub struct Level {

	pub player: Player,

	pub level_width: i32,
	pub level_height: i32,

	// Entities
	pub spawn_spots: Vec::<(i32, i32)>,
	pub blocks: Vec::<(i32, i32)>,
	pub drones: Vec::<Drone>,	// Enemy AI
	pub bullets: Vec::<Bullet>,
}

pub struct Game {

	pub events: Events,

	pub rng: rand::rngs::ThreadRng,

	pub sdl_context: sdl2::Sdl,

	pub canvas: sdl2::render::Canvas<sdl2::video::Window>,
}

pub struct Events {

	pub event_pump: sdl2::EventPump,

	pub quit: bool,

	// Keys
	pub key_left: bool,
	pub key_right: bool,
	pub key_up: bool,

	pub key_attack: bool,
	pub key_fullscreen: bool,

	pub set_fullscreen: i32,
	pub set_shoot: i32,
}


pub trait monTrait{
    fn maFunc(self) -> i32;
}

pub struct Player {
	pub x: i32,
	pub y: i32,
	pub g: i32,
	pub in_air: bool,
	pub alive: bool,

	// Player's direction
	pub going_right: bool,
}

impl Player
{
	pub fn new() -> Self
	{
		return Player {x: 0, y:0, g: 0, in_air: true, alive: true, going_right: true}
	}
}

impl monTrait for Player
{
    fn maFunc(self) -> i32{
        return 69420;
    }
}

pub struct Drone {
	pub x: i32,
	pub y: i32,
	pub pursuit: i32,
//	shoot: i32,
	pub health: i32,

	pub going_right: bool,
	pub shoot_timer: i32,
}

pub struct Bullet {
	pub x: i32,
	pub y: i32,

	pub going_right: bool,

	// Bullet's source
	pub source_is_drone: bool,
}