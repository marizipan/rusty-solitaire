use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButton;

const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 720.0;

#[derive(States, Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Playing,
}


#[derive(Component)]
struct Score;

#[derive(Resource)]
struct GameScore(u32);

#[derive(Component)]
struct Card;

#[derive(Component)]
struct CardBack;

#[derive(Component)]
struct CardFront;

#[derive(Component)]
struct Draggable;

#[derive(Component)]
struct FoundationPile;

#[derive(Component)]
struct TableauPile;

#[derive(Component)]
struct StockPile;

#[derive(Component)]
struct WastePile;

#[derive(Component)]
struct CardData {
    suit: CardSuit,
    value: u8, // 1-13 (Ace=1, Jack=11, Queen=12, King=13)
    is_face_up: bool,
}

#[derive(Component, Clone, Copy, PartialEq)]
enum CardSuit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[derive(Resource)]
struct SelectedCard(Option<Entity>);

#[derive(Resource)]
struct StockCards(Vec<(CardSuit, u8)>);

#[derive(Component)]
struct MovingCard {
    target_position: Vec3,
    speed: f32,
}

#[derive(Component)]
struct CardNumber;

#[derive(Component)]
struct CardOutline;

#[derive(Component)]
struct OriginalPosition(Vec3);

#[derive(Component)]
struct CoveredCard(Option<Entity>); // Points to the card that is covering this one

#[derive(Resource)]
struct TableauPositions(Vec<Vec3>);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.1, 0.4, 0.1))) // Green background for solitaire
        .insert_resource(GameScore(0))
        .insert_resource(SelectedCard(None))
        .insert_resource(StockCards(Vec::new()))
        .add_plugins(DefaultPlugins)      
        .add_systems(Startup, setup_game)
        .add_systems(
            Update,
            (
                card_drag_system,
                card_drop_system,
                stock_click_system,
                double_click_system,
                card_movement_system,
            ),
        )
        .run();
}


fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    
    // Spawn a 2D camera
    commands.spawn(Camera2d::default());
    
    // Create a standard 52-card deck
    let deck = create_deck();
    
    // Store remaining cards in stock
    let mut stock_cards = StockCards(deck.clone());
    
    // Create stock pile (top left) - just show the top card
    let stock_x = -WINDOW_WIDTH / 2.0 + 100.0;
    let stock_y = -WINDOW_HEIGHT / 2.0 + 100.0;
    
    if let Some((suit, value)) = stock_cards.0.pop() {
        // Create stock card with white background and black border
        let stock_card_entity = commands.spawn((
            Sprite {
                color: Color::srgb(1.0, 1.0, 1.0), // White background
                custom_size: Some(Vec2::new(80.0, 120.0)),
                ..default()
            },
            Transform::from_xyz(stock_x, stock_y, 0.0),
            Card,
            CardBack,
            CardData {
                suit,
                value,
                is_face_up: false,
            },
            StockPile,
        )).id();
        
        // Add black border around the white rectangle
        commands.spawn((
            Sprite {
                color: Color::srgb(0.0, 0.0, 0.0), // Black border
                custom_size: Some(Vec2::new(88.0, 128.0)), // Much larger for visible border
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, -1.0), // Behind the white rectangle
        )).set_parent_in_place(stock_card_entity);
        
        // Add sprite image just below center on the white rectangle
        commands.spawn((
            Sprite {
                image: asset_server.load(get_card_back_image(suit)),
                custom_size: Some(Vec2::new(50.0, 70.0)), // Much smaller for clear centering
                ..default()
            },
            Transform::from_xyz(0.0, -10.0, 1.0), // Just below center on the white rectangle
        )).set_parent_in_place(stock_card_entity);
    }

    // Create waste pile (next to stock)
    let waste_x = stock_x + 100.0;
    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(80.0, 120.0)),
            ..default()
        },
        Transform::from_xyz(waste_x, stock_y, 0.0),
        WastePile,
    ));

    // Create foundation piles (top right)
    let foundation_start_x = WINDOW_WIDTH / 2.0 - 400.0;
    for i in 0..4 {
        let x_pos = foundation_start_x + (i as f32 * 100.0);
        commands.spawn((
            Sprite {
                color: Color::srgb(0.2, 0.2, 0.2),
                custom_size: Some(Vec2::new(80.0, 120.0)),
                ..default()
            },
            Transform::from_xyz(x_pos, WINDOW_HEIGHT / 2.0 - 50.0, 0.0),
            FoundationPile,
        ));
    }

    // Create tableau piles (middle area)
    let tableau_start_x = -(6 as f32 * 100.0) / 2.0;
    let mut tableau_positions = Vec::new();
    for pile in 0..7 {
        let x_pos = tableau_start_x + (pile as f32 * 100.0);
        let y_pos = WINDOW_HEIGHT / 2.0 - 200.0;
        tableau_positions.push(Vec3::new(x_pos, y_pos, 0.0));
        
        // Create empty tableau pile
        commands.spawn((
            Sprite {
                color: Color::srgb(0.2, 0.2, 0.2),
                custom_size: Some(Vec2::new(80.0, 120.0)),
                ..default()
            },
            Transform::from_xyz(x_pos, y_pos, 0.0),
            TableauPile,
        ));
    }
    commands.insert_resource(TableauPositions(tableau_positions));

    // Deal cards to tableau piles
    let mut card_index = 0;
    for pile in 0..7 {
        for card_in_pile in 0..=pile {
            if card_index < deck.len() {
                let (suit, value) = deck[card_index];
                let x_pos = tableau_start_x + (pile as f32 * 100.0);
                let y_pos = WINDOW_HEIGHT / 2.0 - 200.0 - (card_in_pile as f32 * 30.0);
                
                // Determine if this card should be face-up (only the bottom card of each pile)
                let is_face_up = card_in_pile == pile; // Only the last card in each pile is face-up
                
                // Create card entity with just the card image
                let card_entity = if is_face_up {
                    commands.spawn((
                        Transform::from_xyz(x_pos, y_pos, card_in_pile as f32),
                        Card,
                        CardFront,
                        CardData {
                            suit,
                            value,
                            is_face_up,
                        },
                        Draggable, // Only face-up cards are draggable
                        TableauPile,
                        OriginalPosition(Vec3::new(x_pos, y_pos, card_in_pile as f32)),
                        CoveredCard(None), // Top card is not covered
                    )).id()
                } else {
                    commands.spawn((
                        Transform::from_xyz(x_pos, y_pos, card_in_pile as f32),
                        Card,
                        CardBack,
                        CardData {
                            suit,
                            value,
                            is_face_up,
                        },
                        // No Draggable component for face-down cards
                        TableauPile,
                        OriginalPosition(Vec3::new(x_pos, y_pos, card_in_pile as f32)),
                        CoveredCard(None), // Will be updated below
                    )).id()
                };
                
                // Add card image directly to the card entity
                if is_face_up {
                    // Add front image for face-up cards
                    let front_image_path = get_card_front_image(suit, value);
                    commands.entity(card_entity).insert((
                        Sprite {
                            image: asset_server.load(front_image_path),
                            custom_size: Some(Vec2::new(80.0, 120.0)),
                            ..default()
                        },
                    ));
                } else {
                    // Add back image for face-down cards
                    commands.entity(card_entity).insert((
                        Sprite {
                            image: asset_server.load(get_card_back_image(suit)),
                            custom_size: Some(Vec2::new(80.0, 120.0)),
                            ..default()
                        },
                    ));
                }
                
                card_index += 1;
            }
        }
    }
    


    // Score display
    commands.spawn((
        Text::new("Score: 0"),
        Transform::from_xyz(-WINDOW_WIDTH / 2.0 + 100.0, WINDOW_HEIGHT / 2.0 - 50.0, 2.0),
        Score,
    ));

}

fn get_card_back_image(_suit: CardSuit) -> &'static str {
    "sprites/cards/CardBack.png"
}

fn get_card_front_image(suit: CardSuit, value: u8) -> String {
    let (suit_name, use_card_prefix) = match suit {
        CardSuit::Hearts => ("King", true),
        CardSuit::Diamonds => ("EvilFerris", false), 
        CardSuit::Clubs => ("Stabby", true),
        CardSuit::Spades => ("Corro", true),
    };
    
    let value_name = match value {
        1 => "A",
        11 => "J", 
        12 => "Q",
        13 => "K",
        _ => &value.to_string(),
    };
    
    if use_card_prefix {
        format!("sprites/cards/{}/{}Card{}.png", suit_name, suit_name, value_name)
    } else {
        format!("sprites/cards/{}/{}{}.png", suit_name, suit_name, value_name)
    }
}

fn get_card_display_value(value: u8) -> String {
    match value {
        1 => "A".to_string(),
        11 => "J".to_string(),
        12 => "Q".to_string(),
        13 => "K".to_string(),
        _ => value.to_string(),
    }
}

fn create_deck() -> Vec<(CardSuit, u8)> {
    let mut deck = Vec::new();
    let suits = [CardSuit::Hearts, CardSuit::Diamonds, CardSuit::Clubs, CardSuit::Spades];
    
    for suit in suits {
        for value in 1..=13 {
            deck.push((suit, value));
        }
    }
    
    // Shuffle the deck (simplified - just reverse for now)
    deck.reverse();
    deck
}

// Solitaire rules helper functions
fn can_place_on_foundation(card_value: u8, _foundation_suit: Option<CardSuit>) -> bool {
    // Foundation starts with Ace (1), then builds up in same suit
    if card_value == 1 {
        return true; // Ace can always start a foundation
    }
    // For now, allow any card on foundation (simplified)
    true
}

fn can_place_on_tableau(card_value: u8, card_suit: CardSuit, tableau_card_value: u8, tableau_card_suit: CardSuit) -> bool {
    // Tableau builds down in alternating colors
    let is_red = matches!(card_suit, CardSuit::Hearts | CardSuit::Diamonds);
    let tableau_is_red = matches!(tableau_card_suit, CardSuit::Hearts | CardSuit::Diamonds);
    
    // Must be different color and one value lower
    // In solitaire: King (13) can be placed on empty tableau piles
    // Other cards must be placed on cards with value one higher
    if card_value == 13 {
        // Kings can be placed on empty tableau piles (we'll handle this separately)
        return false; // For now, don't allow kings on other cards
    } else {
        // Other cards must be placed on cards with value one higher
        is_red != tableau_is_red && card_value == tableau_card_value - 1
    }
}

fn stock_click_system(
    mouse_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window>,
    mut stock_cards: ResMut<StockCards>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    _stock_query: Query<Entity, With<StockPile>>,
) {
    let Ok(window) = window_query.single() else { return };
    
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = window.cursor_position() {
            let cursor_world_pos = Vec2::new(
                cursor_pos.x - window.width() / 2.0,
                window.height() / 2.0 - cursor_pos.y,
            );

            // Check if stock pile was clicked
            let stock_x = -WINDOW_WIDTH / 2.0 + 100.0;
            let stock_y = -WINDOW_HEIGHT / 2.0 + 100.0;
            let stock_bounds = Vec2::new(40.0, 60.0);
            
            if (cursor_world_pos - Vec2::new(stock_x, stock_y)).abs().cmplt(stock_bounds).all() {
                // Deal a card from stock to waste
                if let Some((suit, value)) = stock_cards.0.pop() {
                    let waste_x = stock_x + 100.0;
                    let waste_y = stock_y;
                    
                    // Create moving card animation with white background and black border
                    let card_entity = commands.spawn((
                        Sprite {
                            color: Color::srgb(1.0, 1.0, 1.0), // White background
                            custom_size: Some(Vec2::new(80.0, 120.0)),
                            ..default()
                        },
                        Transform::from_xyz(stock_x, stock_y, 10.0),
                        Card,
                        CardFront,
                        CardData {
                            suit,
                            value,
                            is_face_up: true,
                        },
                        // No Draggable component for stock cards
                        MovingCard {
                            target_position: Vec3::new(waste_x, waste_y, 0.0),
                            speed: 200.0,
                        },
                    )).id();
             
                }
            }
        }
    }
}

fn double_click_system(
    mouse_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window>,
    mut commands: Commands,
    card_query: Query<(Entity, &Transform, &CardData), (With<Card>, With<Draggable>)>,
) {
    let Ok(window) = window_query.single() else { return };
    
    // Simple double-click detection (in real implementation, you'd want proper timing)
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = window.cursor_position() {
            let cursor_world_pos = Vec2::new(
                cursor_pos.x - window.width() / 2.0,
                window.height() / 2.0 - cursor_pos.y,
            );

            // Find the closest card to the cursor
            let mut closest_card: Option<(Entity, f32)> = None;
            
            for (entity, transform, _card_data) in &card_query {
                let card_pos = transform.translation.truncate();
                let card_bounds = Vec2::new(40.0, 60.0);
                
                if (cursor_world_pos - card_pos).abs().cmplt(card_bounds).all() {
                    let distance = (cursor_world_pos - card_pos).length();
                    
                    if let Some((_, current_distance)) = closest_card {
                        if distance < current_distance {
                            closest_card = Some((entity, distance));
                        }
                    } else {
                        closest_card = Some((entity, distance));
                    }
                }
            }
            
            // Auto-move the closest card if it's an Ace
            if let Some((entity, _)) = closest_card {
                if let Ok((_, _, card_data)) = card_query.get(entity) {
                    if card_data.value == 1 {
                        let foundation_x = WINDOW_WIDTH / 2.0 - 400.0;
                        let foundation_y = WINDOW_HEIGHT / 2.0 - 50.0;
                        
                        commands.entity(entity).insert(MovingCard {
                            target_position: Vec3::new(foundation_x, foundation_y, 0.0),
                            speed: 300.0,
                        });
                    }
                }
            }
        }
    }
}

fn card_movement_system(
    mut commands: Commands,
    mut moving_cards: Query<(Entity, &mut Transform, &MovingCard)>,
    time: Res<Time>,
) {
    for (entity, mut transform, moving_card) in &mut moving_cards {
        let direction = moving_card.target_position - transform.translation;
        let distance = direction.length();
        
        if distance < 5.0 {
            // Arrived at destination
            transform.translation = moving_card.target_position;
            commands.entity(entity).remove::<MovingCard>();
        } else {
            // Move towards target
            let movement = direction.normalize() * moving_card.speed * time.delta_secs();
            transform.translation += movement;
        }
    }
}

fn card_drag_system(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut selected_card: ResMut<SelectedCard>,
    mut card_query: Query<(Entity, &mut Transform, &CardData), (With<Card>, With<Draggable>)>,
    window_query: Query<&Window>,
) {
    let Ok(window) = window_query.single() else { return };
    
    if mouse_input.just_pressed(MouseButton::Left) {
        // Get mouse position
        if let Some(cursor_pos) = window.cursor_position() {
            let cursor_world_pos = Vec2::new(
                cursor_pos.x - window.width() / 2.0,
                window.height() / 2.0 - cursor_pos.y,
            );

            // Check if any face-up card was clicked
            for (entity, transform, card_data) in &mut card_query {
                // Only allow dragging face-up cards
                if !card_data.is_face_up {
                    continue;
                }
                
                let card_pos = transform.translation.truncate();
                let card_bounds = Vec2::new(40.0, 60.0);
                
                if (cursor_world_pos - card_pos).abs().cmplt(card_bounds).all() {
                    selected_card.0 = Some(entity);
                    break;
                }
            }
        }
    }

    if mouse_input.just_released(MouseButton::Left) {
        selected_card.0 = None;
    }
}

fn card_drop_system(
    selected_card: ResMut<SelectedCard>,
    mut card_query: Query<(Entity, &mut Transform, &CardData, &OriginalPosition), With<Card>>,
    window_query: Query<&Window>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tableau_positions: Res<TableauPositions>,
) {
    if let Some(selected_entity) = selected_card.0 {
        let Ok(window) = window_query.single() else { return };
        
        if let Some(cursor_pos) = window.cursor_position() {
            let cursor_world_pos = Vec2::new(
                cursor_pos.x - window.width() / 2.0,
                window.height() / 2.0 - cursor_pos.y,
            );
            
            // First, collect all potential targets and tableau pile information
            let mut potential_targets = Vec::new();
            let mut tableau_pile_info = Vec::new();
            
            for (entity, target_transform, target_card_data, _) in &card_query {
                if entity == selected_entity {
                    continue; // Skip the card being dragged
                }
                
                // Only consider face-up cards as valid targets
                if !target_card_data.is_face_up {
                    continue;
                }
                
                let target_pos = target_transform.translation;
                let distance = (cursor_world_pos - target_pos.truncate()).length();
                potential_targets.push((target_pos, distance, target_card_data.value, target_card_data.suit));
            }
            
            // Check tableau pile positions
            for tableau_pos in &tableau_positions.0 {
                let distance = (cursor_world_pos - tableau_pos.truncate()).length();
                if distance < 50.0 {
                    let mut pile_has_cards = false;
                    for (_, card_transform, _, _) in card_query.iter() {
                        if (card_transform.translation.x - tableau_pos.x).abs() < 5.0 
                            && (card_transform.translation.y - tableau_pos.y).abs() < 5.0 {
                            pile_has_cards = true;
                            break;
                        }
                    }
                    tableau_pile_info.push((*tableau_pos, !pile_has_cards));
                    break;
                }
            }
            
            // Now get the selected card data and find the best target
            if let Ok((_, mut transform, selected_card_data, original_pos)) = card_query.get_mut(selected_entity) {
                let mut best_target: Option<(Vec3, f32)> = None;
                
                for (target_pos, distance, target_value, target_suit) in potential_targets {
                    // Check if this is a valid placement (card value must be one higher)
                    if can_place_on_tableau(
                        selected_card_data.value, 
                        selected_card_data.suit, 
                        target_value, 
                        target_suit
                    ) {
                        if let Some((_, current_distance)) = best_target {
                            if distance < current_distance {
                                best_target = Some((target_pos, distance));
                            }
                        } else {
                            best_target = Some((target_pos, distance));
                        }
                    }
                }
                
                // If we found a valid target, snap to it
                if let Some((target_pos, _)) = best_target {
                    // Position the card on top of the target card
                    let new_position = Vec3::new(target_pos.x, target_pos.y - 30.0, target_pos.z + 1.0);
                    transform.translation = new_position;
                    
                    // Update the original position for future reference
                    commands.entity(selected_entity).insert(OriginalPosition(new_position));
                    
                    // Find and flip the card that was underneath the moved card
                    flip_card_underneath(selected_entity, &mut commands, &mut card_query, &asset_server);
                } else {
                    // Check if dropped on an empty tableau pile
                    let mut should_flip = false;
                    let mut target_tableau_pos = None;
                    
                    // Use the collected tableau pile information
                    for (tableau_pos, is_empty) in &tableau_pile_info {
                        if *is_empty {
                            // Empty tableau pile - only allow if the top card (being dragged) is a King
                            if selected_card_data.value == 13 {
                                target_tableau_pos = Some(*tableau_pos);
                                should_flip = true;
                            }
                        }
                    }
                    
                    // Now apply the movement if we found a valid target
                    if let Some(tableau_pos) = target_tableau_pos {
                        transform.translation = tableau_pos;
                        commands.entity(selected_entity).insert(OriginalPosition(tableau_pos));
                    } else if !tableau_pile_info.is_empty() {
                        // Not a valid move, snap back to original position
                        transform.translation = original_pos.0;
                    } else {
                        // If not dropped on any valid target or empty pile, snap back to original position
                        transform.translation = original_pos.0;
                    }
                    
                    // Flip card underneath if needed (after dropping the mutable borrow)
                    if should_flip {
                        flip_card_underneath(selected_entity, &mut commands, &mut card_query, &asset_server);
                    }
                }
            }
        }
    }
}

fn flip_card_underneath(
    moved_card_entity: Entity,
    commands: &mut Commands,
    card_query: &mut Query<(Entity, &mut Transform, &CardData, &OriginalPosition), With<Card>>,
    asset_server: &Res<AssetServer>,
) {
    // Get the original position of the moved card
    let original_pos = if let Ok((_, _, _, original_pos)) = card_query.get(moved_card_entity) {
        original_pos.0
    } else {
        return;
    };

    // Find the card that was directly underneath the moved card
    // It should be at the same x position but 30 units higher (y + 30)
    let card_underneath_pos = Vec3::new(original_pos.x, original_pos.y + 30.0, original_pos.z);
    
    // Now iterate to find the card at that position
    for (entity, transform, card_data, _) in card_query.iter_mut() {
        if entity == moved_card_entity {
            continue;
        }
        
        // Check if this card is at the position directly underneath the moved card
        if !card_data.is_face_up 
            && (transform.translation.x - card_underneath_pos.x).abs() < 5.0
            && (transform.translation.y - card_underneath_pos.y).abs() < 5.0
        {
            // This is the card that was underneath, flip it face-up
            commands.entity(entity).remove::<Sprite>();
            commands.entity(entity).remove::<CardBack>();
            commands.entity(entity).insert((
                CardFront,
                Draggable,
                Sprite {
                    image: asset_server.load(get_card_front_image(card_data.suit, card_data.value)),
                    custom_size: Some(Vec2::new(80.0, 120.0)),
                    ..default()
                },
            ));
            commands.entity(entity).insert(CardData {
                suit: card_data.suit,
                value: card_data.value,
                is_face_up: true,
            });
            break;
        }
    }
}


