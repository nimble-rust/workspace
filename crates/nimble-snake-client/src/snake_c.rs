use flood_rs::{Deserialize, ReadOctetStream, Serialize, WriteOctetStream};
use std::fmt::{Debug, Display, Formatter};
use std::mem;
use std::os::raw::c_int;

// Custom CBool Type to Represent C's bool
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CBool(u8);

impl CBool {
    /// Converts a Rust `bool` to `CBool`
    pub fn from_bool(value: bool) -> Self {
        CBool(value as u8)
    }

    /// Converts `CBool` to Rust `bool`
    pub fn to_bool(self) -> bool {
        self.0 != 0
    }
}

impl Default for CBool {
    fn default() -> Self {
        CBool(0)
    }
}

// Constants
pub const NIMBLE_EXAMPLE_SNAKE_MAX_LENGTH: usize = 20;
pub const EXAMPLE_ILLEGAL_INDEX: u8 = 0xff;

pub const EXAMPLE_GAME_MAX_PLAYERS: usize = 4;
pub const EXAMPLE_GAME_MAX_AVATARS: usize = 4;
pub const EXAMPLE_GAME_MAX_PARTICIPANTS: usize = 16;

pub const EXAMPLE_PLAYER_INPUT_INTENTIONAL_PADDING_SIZE: usize = 32;

// Enum Definitions
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ExampleDirection {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

// Struct Definitions
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ExamplePosition {
    pub x: c_int,
    pub y: c_int,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ExampleGameArea {
    pub width: usize,
    pub height: usize,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ExampleSnake {
    pub body: [ExamplePosition; NIMBLE_EXAMPLE_SNAKE_MAX_LENGTH],
    pub length: c_int,
    pub movementDirection: ExampleDirection,
    pub controlledByPlayerIndex: u8,
    pub isFrozen: CBool,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ExampleFood {
    pub position: ExamplePosition,
}

/// First step to send for a joining participant
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ExamplePlayerSelectTeam {
    pub preferredTeamToJoin: u8,
}

/// Used to control the avatar (snake)
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ExamplePlayerInGameInput {
    pub horizontalAxis: i8,
    pub verticalAxis: i8,
    pub abilityButton: CBool,
}
impl Default for ExamplePlayerInGameInput {
    fn default() -> Self {
        ExamplePlayerInGameInput {
            horizontalAxis: 0,               // Neutral horizontal position
            verticalAxis: 0,                 // Neutral vertical position
            abilityButton: CBool::default(), // Ability button not pressed
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ExamplePlayerInputType {
    Empty = 0,
    InGame = 1,
    SelectTeam = 2,
}

/// Example Player Input Union
#[repr(C)]
#[derive(Copy, Clone)]
pub union ExamplePlayerInputUnion {
    pub inGameInput: ExamplePlayerInGameInput,
    pub selectTeam: ExamplePlayerSelectTeam,
}

impl Debug for ExamplePlayerInputUnion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExamplePlayerInputUnion")
    }
}

/// Example Player Input Struct containing a union
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ExamplePlayerInput {
    pub inputType: ExamplePlayerInputType,
    pub input: ExamplePlayerInputUnion,
    pub intentionalPadding: [u8; EXAMPLE_PLAYER_INPUT_INTENTIONAL_PADDING_SIZE],
}

impl Serialize for ExamplePlayerInput {
    fn serialize(&self, stream: &mut impl WriteOctetStream) -> std::io::Result<()> {
        todo!()
    }
}

impl Deserialize for ExamplePlayerInput {
    fn deserialize(stream: &mut impl ReadOctetStream) -> std::io::Result<Self> {
        todo!()
    }
}

impl PartialEq<Self> for ExamplePlayerInput {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl Eq for ExamplePlayerInput {}

impl Display for ExamplePlayerInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for ExamplePlayerInput {
    fn default() -> Self {
        ExamplePlayerInput {
            inputType: ExamplePlayerInputType::Empty, // Initialize to a default variant
            input: ExamplePlayerInputUnion {
                inGameInput: ExamplePlayerInGameInput::default(), // Initialize one variant
            },
            intentionalPadding: [0; EXAMPLE_PLAYER_INPUT_INTENTIONAL_PADDING_SIZE], // Zero out padding
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ExamplePlayer {
    pub snakeIndex: u8,
    pub assignedToParticipantIndex: u8,
    pub playerIndex: u8,
    pub playerInput: ExamplePlayerInput,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ExamplePlayers {
    pub players: [ExamplePlayer; EXAMPLE_GAME_MAX_PLAYERS],
    pub playerCount: u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ExampleParticipant {
    pub participantId: u8,
    pub playerIndex: u8,
    pub isUsed: CBool,
    pub internalMarked: CBool,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ExampleSnakes {
    pub snakes: [ExampleSnake; EXAMPLE_GAME_MAX_AVATARS],
    pub snakeCount: u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ExampleGame {
    pub area: ExampleGameArea,
    pub players: ExamplePlayers,
    pub snakes: ExampleSnakes,
    pub food: ExampleFood,
    pub pseudoRandom: u32,
    pub gameIsOver: CBool,
    pub ticksBetweenMoves: u32,

    pub lastParticipantLookupCount: u8,
    pub participantLookup: [ExampleParticipant; EXAMPLE_GAME_MAX_PARTICIPANTS],
}

// Implement TryFrom for ExampleGame
impl TryFrom<&[u8]> for ExampleGame {
    type Error = &'static str;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        let size = size_of::<ExampleGame>();

        // Ensure the slice length matches the size of the struct
        if slice.len() == size {
            // SAFETY: We assume the slice is valid and correctly aligned for ExampleGame
            let ptr = slice.as_ptr() as *const ExampleGame;
            unsafe { Ok(*ptr) } // Dereference the pointer to get the struct
        } else {
            Err("Invalid slice length")
        }
    }
}

impl Default for ExamplePlayer {
    fn default() -> Self {
        ExamplePlayer {
            snakeIndex: 0,
            assignedToParticipantIndex: 0,
            playerIndex: 0,
            playerInput: ExamplePlayerInput::default(),
        }
    }
}

impl Default for ExampleParticipant {
    fn default() -> Self {
        ExampleParticipant {
            participantId: 0,
            playerIndex: 0,
            isUsed: CBool::default(),
            internalMarked: CBool::default(),
        }
    }
}

impl Default for ExampleSnake {
    fn default() -> Self {
        ExampleSnake {
            body: [ExamplePosition { x: 0, y: 0 }; NIMBLE_EXAMPLE_SNAKE_MAX_LENGTH],
            length: 0,
            movementDirection: ExampleDirection::Up,
            controlledByPlayerIndex: 0,
            isFrozen: CBool::default(),
        }
    }
}

impl Default for ExampleFood {
    fn default() -> Self {
        ExampleFood {
            position: ExamplePosition { x: 0, y: 0 },
        }
    }
}

impl Default for ExamplePlayers {
    fn default() -> Self {
        ExamplePlayers {
            players: [ExamplePlayer::default(); EXAMPLE_GAME_MAX_PLAYERS],
            playerCount: 0,
        }
    }
}

impl Default for ExampleSnakes {
    fn default() -> Self {
        ExampleSnakes {
            snakes: [ExampleSnake::default(); EXAMPLE_GAME_MAX_AVATARS],
            snakeCount: 0,
        }
    }
}

impl Default for ExampleGame {
    fn default() -> Self {
        ExampleGame {
            area: ExampleGameArea {
                width: 800,
                height: 600,
            },
            players: ExamplePlayers::default(),
            snakes: ExampleSnakes::default(),
            food: ExampleFood::default(),
            pseudoRandom: 0,
            gameIsOver: CBool::default(),
            ticksBetweenMoves: 0,
            lastParticipantLookupCount: 0,
            participantLookup: [ExampleParticipant::default(); EXAMPLE_GAME_MAX_PARTICIPANTS],
        }
    }
}
