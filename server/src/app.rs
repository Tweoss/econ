//* included by main (uses server)

use std::sync::Mutex;
use actix::Addr;

//* From Root
//* Includes game ID (#) and player ID's (# and cookie) and game states
//* App - Game - {Player, Viewer, Director)


//* App State can receive messages about a new Game, the end of a Game, getting a Game (to operate on)
struct AppState {
	games: Vec<Game>,
	// games: Mutex<i32>, // <- Mutex is necessary to mutate safely across threads
}

//* Game can receive messages about a Player joining, admin Messages, other things?
struct Game {
	//* will never be modified - read multiple times
	id: usize, //* 6 digits? 
	//* will be modified
	players: Vec<Player>, 
	//* will never be modified
	app_addr: Addr<AppState>
}

//* players can make actions to be sent to the game to manage
struct Player {
	//* will never be modified
	id: usize,
	//* will never be modified
	addr: Addr<server::>,
	//* will never be modified
	game_addr: Addr<Game>,
}


