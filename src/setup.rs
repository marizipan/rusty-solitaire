use bevy::prelude::*;
use crate::components::*;
use crate::utils::{create_deck, get_card_back_image, get_card_front_image};

pub fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    
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
        // Create stock card
        let stock_card_entity = commands.spawn((
            Sprite {
                image: asset_server.load(get_card_back_image(suit)),
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
        }
    commands.insert_resource(TableauPositions(tableau_positions));

    // Deal cards to tableau piles
    let mut card_index = 0;
    for pile in 0..7 {
        for card_in_pile in 0..=pile {
            if card_index < deck.len() {
                let (suit, value) = deck[card_index];
                let x_pos = tableau_start_x + (pile as f32 * 100.0);
                let y_pos = WINDOW_HEIGHT / 2.0 - 200.0;
                
                // Determine if this card should be face-up (only the bottom card of each pile)
                let is_face_up = card_in_pile == pile; // Only the last card in each pile is face-up
                
                // Add a small visual offset for face-down cards so they look stacked
                let visual_offset = if is_face_up { 0.0 } else { 2.0 };
                
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
                        Transform::from_xyz(x_pos, y_pos + visual_offset, card_in_pile as f32),
                        Card,
                        CardBack,
                        CardData {
                            suit,
                            value,
                            is_face_up,
                        },
                        // No Draggable component for face-down cards
                        TableauPile,
                        OriginalPosition(Vec3::new(x_pos, y_pos + visual_offset, card_in_pile as f32)),
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
