//! A 2D toy game written in Rust, using the ggez library.
#![deny(missing_docs)]

extern crate geometry;
extern crate geometry_derive;

mod controllers;
mod game_state;
mod models;
mod util;
mod view;

use ggez::event;
use ggez::input::keyboard::KeyInput;
use ggez::{Context, GameResult};
use rand::prelude::ThreadRng;
use structopt::StructOpt;

use crate::{
    controllers::{CollisionsController, Event, InputController, TimeController},
    game_state::GameState,
    geometry::Size,
    view::Resources,
};

/// This struct contains the application's state
pub struct ApplicationState {
    // Keep track of window focus to play/pause the game
    has_focus: bool,
    // Resources holds our loaded font, images and sounds
    resources: Resources,
    // The game state contains all information needed to run the game
    game_state: GameState,
    // The time controller modifies the game state as time passes
    time_controller: TimeController,
    // The input controller keeps track of the actions that are triggered by the player
    input_controller: InputController,
    // The event buffer keeps track of events that trigger sounds, so we can separate
    // sound playing from the game logic
    event_buffer: Vec<Event>,
    // A source of randomness
    rng: ThreadRng,
}

impl ApplicationState {
    /// Simply creates a new application state
    fn new(ctx: &mut Context, game_size: Size) -> GameResult<ApplicationState> {
        let mut rng = rand::thread_rng();
        let app_state = ApplicationState {
            has_focus: true,
            resources: Resources::new(ctx),
            game_state: GameState::new(game_size, &mut rng),
            time_controller: TimeController::new(),
            input_controller: InputController::new(),
            event_buffer: Vec::new(),
            rng,
        };
        Ok(app_state)
    }

    /// This will be called when the game needs to be reset
    fn reset(&mut self) {
        // Reset time controller
        self.time_controller.reset();

        // Reset game state
        self.game_state.reset(&mut self.rng);

        self.event_buffer.push(Event::GameStart);
    }
}

// We implement `ggez::event::EventHandler` trait on our application state - this is where we can
// listen for input and other events
impl event::EventHandler for ApplicationState {
    // This is called each time the game loop updates so we can update the game state
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Pause the game if the window has no focus
        if !self.has_focus {
            return Ok(());
        }

        // Update game state, and check for collisions
        let duration = ctx.time.delta();
        self.time_controller.update_seconds(
            duration,
            self.input_controller.actions(),
            &mut self.game_state,
            &mut self.event_buffer,
            &mut self.rng,
        );

        CollisionsController::handle_collisions(
            &mut self.game_state,
            &mut self.time_controller,
            &mut self.event_buffer,
        );

        Ok(())
    }

    // This is called when ggez wants us to draw our game
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        view::play_sounds(ctx, &mut self.event_buffer, &mut self.resources)?;
        view::render_game(self, ctx)
    }

    // Listen for keyboard events
    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        input: KeyInput,
        _repeated: bool,
    ) -> GameResult {
        // If we're displaying a message (waiting for user input) then hide it and reset the game
        if let Some(_) = self.game_state.message {
            self.reset();
        }
        self.input_controller.key_press(input);
        Ok(())
    }

    fn key_up_event(&mut self, _ctx: &mut Context, input: KeyInput) -> GameResult {
        self.input_controller.key_release(input);
        Ok(())
    }

    // Listen for window focus to pause the game's execution
    fn focus_event(&mut self, _ctx: &mut Context, has_focus: bool) -> GameResult {
        self.has_focus = has_focus;
        Ok(())
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Rocket")]
struct Opt {
    /// Window width
    #[structopt(long = "width", default_value = "1024")]
    width: usize,

    /// Window height
    #[structopt(long = "height", default_value = "576")]
    height: usize,
}

fn main() {
    let opt = Opt::from_args();
    let game_size = Size::new(opt.width as f32, opt.height as f32);

    // Create the rendering context and set the background color to black
    let (mut ctx, event_loop) = view::init_rendering_ctx(game_size).unwrap();

    // Load the application state and start the event loop
    let state = ApplicationState::new(&mut ctx, game_size).unwrap();
    event::run(ctx, event_loop, state);
}
