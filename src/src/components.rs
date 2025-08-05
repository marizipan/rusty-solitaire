use bevy::prelude::*;

pub const WINDOW_WIDTH: f32 = 1280.0;
pub const WINDOW_HEIGHT: f32 = 720.0;

#[derive(States, Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Playing,
}

#[derive(Component)]
pub struct Score;

#[derive(Resource)]
pub struct GameScore(pub u32);

#[derive(Component)]
pub struct Card;

#[derive(Component)]
pub struct CardBack;

#[derive(Component)]
pub struct CardFront;

#[derive(Component)]
pub struct Draggable;

#[derive(Component)]
pub struct FoundationPile;

#[derive(Component)]
pub struct TableauPile;

#[derive(Component)]
pub struct StockPile;

#[derive(Component)]
pub struct WastePile;

#[derive(Component)]
pub struct CardData {
    pub suit: CardSuit,
    pub value: u8, // 1-13 (Ace=1, Jack=11, Queen=12, King=13)
    pub is_face_up: bool,
}

#[derive(Component, Clone, Copy, PartialEq)]
pub enum CardSuit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[derive(Resource)]
pub struct SelectedCard(pub Option<Entity>);

#[derive(Resource)]
pub struct StockCards(pub Vec<(CardSuit, u8)>);

#[derive(Component)]
pub struct MovingCard {
    pub target_position: Vec3,
    pub speed: f32,
}

#[derive(Component)]
pub struct CardNumber;

#[derive(Component)]
pub struct CardOutline;

#[derive(Component)]
pub struct OriginalPosition(pub Vec3);

#[derive(Component)]
pub struct CoveredCard(pub Option<Entity>); // Points to the card that is covering this one

#[derive(Resource)]
pub struct TableauPositions(pub Vec<Vec3>); 