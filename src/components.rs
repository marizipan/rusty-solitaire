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
pub struct SkippedWasteCard; // Marks waste cards that have been skipped and are not clickable

#[derive(Component)]
pub struct UndoButton;

#[derive(Component)]
pub struct CardData {
    pub suit: CardSuit,
    pub value: u8, // 1-13 (Ace=1, Jack=11, Queen=12, King=13)
    pub is_face_up: bool,
}

#[derive(Component, Clone, Copy, PartialEq, Debug)]
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
pub struct CardNumber;

#[derive(Component)]
pub struct OriginalPosition(pub Vec3);

#[derive(Component)]
pub struct CoveredCard(pub Option<Entity>); // Points to the card that is covering this one

#[derive(Component)]
pub struct NeedsFlipUnderneath(pub Vec3); // Marks that a card underneath needs to be flipped

#[derive(Component)]
pub struct AlreadyFlipped; // Marks that a card has already been flipped in this movement



#[derive(Resource)]
pub struct TableauPositions(pub Vec<Vec3>);

#[derive(Resource)]
pub struct FoundationPiles(pub Vec<Vec<(CardSuit, u8)>>); // Tracks the entire stack of each foundation pile

#[derive(Resource)]
pub struct ClickedEntity(pub Option<Entity>); // Tracks the last clicked entity for double-click detection 

#[derive(Resource)]
pub struct UndoStack(pub Vec<UndoAction>); // Tracks undo actions

#[derive(Clone)]
pub struct UndoAction {
    pub card_entity: Entity,
    pub from_position: Vec3,
    pub to_position: Vec3,
    pub from_components: Vec<ComponentType>,
    pub to_components: Vec<ComponentType>,
    pub stack_cards: Vec<(Entity, Vec3)>, // For moving entire stacks
    pub original_face_up: bool, // Track the original face up/down state
}

#[derive(Clone, PartialEq)]
pub enum ComponentType {
    TableauPile,
    WastePile,
    FoundationPile,
    StockPile,
    Draggable,
    CardFront,
    CardBack,
}

 