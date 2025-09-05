use bevy::prelude::*;
use crate::components::*;
use crate::utils::get_card_back_image;
use crate::card_entity::create_card_entity;



pub fn setup_initial_tableau_and_stock(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut stock_cards: ResMut<StockCards>,
) {
    // Create a standard 52-card deck
    let mut deck = Vec::new();
    let suits = [CardSuit::Hearts, CardSuit::Diamonds, CardSuit::Clubs, CardSuit::Spades];
    let values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
    
    for suit in suits {
        for value in values {
            deck.push((suit, value));
        }
    }
    
    // Shuffle the deck using a simple but effective algorithm
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    // Use a seed based on current time for some randomness
    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut hasher);
    let seed = hasher.finish();
    
    // Simple shuffle: swap each card with a random position
    for i in 0..deck.len() {
        let j = (seed.wrapping_add(i as u64 * 17)) % deck.len() as u64;
        let j = j as usize;
        deck.swap(i, j);
    }
    
    // Deal exactly 28 cards to the tableau (7 piles with 1, 2, 3, 4, 5, 6, 7 cards)
    let tableau_start_x = -(6.0 * 100.0) / 2.0;
    let tableau_y = WINDOW_HEIGHT / 2.0 - 250.0; // Moved down to align with top row
    
    let mut card_index = 0;
    for pile in 0..7 {
        let pile_size = pile + 1; // Stack 1 has 1 card, Stack 2 has 2 cards, etc.
        let x_pos = tableau_start_x + (pile as f32 * 100.0);
        
        for card_in_pile in 0..pile_size {
            if card_index < deck.len() {
                let (suit, value) = deck[card_index];
                
                // Only the top card of each pile is face-up
                let is_face_up = card_in_pile == pile_size - 1;
            
                // Calculate vertical offset for stacked appearance
                // Each card is offset downward by 30 pixels for reasonable visual spacing
                let vertical_offset = if card_in_pile == 0 {
                    0.0 // Bottom card stays at base position
                } else {
                    card_in_pile as f32 * 30.0 // Each card above is offset by 30 pixels
                };
                
                let y_pos = tableau_y - vertical_offset;
                
                // Create card entity
                let _card_entity = if is_face_up {
                    // Only face-up cards get Draggable component
                    create_card_entity(
                        commands,
                        &asset_server,
                        Vec3::new(x_pos, y_pos, card_in_pile as f32),
                        suit,
                        value,
                        is_face_up,
                        (
                            Draggable,
                            TableauPile,
                            OriginalPosition(Vec3::new(x_pos, y_pos, card_in_pile as f32)),
                            CoveredCard(None), // Top card is not covered
                        ),
                    )
                } else {
                    // Face-down cards are not draggable and get CardBack component
                    create_card_entity(
                        commands,
                        &asset_server,
                        Vec3::new(x_pos, y_pos, card_in_pile as f32),
                        suit,
                        value,
                        is_face_up,
                        (
                            TableauPile,
                            OriginalPosition(Vec3::new(x_pos, y_pos, card_in_pile as f32)),
                            CoveredCard(None),
                            CardBack, // Ensure face-down cards have CardBack component
                        ),
                    )
                };
                
                // Sprite is now handled by the create_card_entity helper function
                
                card_index += 1;
            }
        }
    }
    
    // Store the remaining 24 cards in the stock pile
    let remaining_cards: Vec<(CardSuit, u8)> = deck.iter().cloned().skip(28).collect();
    stock_cards.0 = remaining_cards;
    
    // Create stock pile above Stack 7 (rightmost stack)
    let stock_x = -(6.0 * 100.0) / 2.0 + (6.0 * 100.0); // Above Stack 7 (x = 300)
    let stock_y = WINDOW_HEIGHT / 2.0 - 100.0; // Above the tableau stacks
    
    // Create stock pile visual representation (always shows card back initially)
    create_card_entity(
        commands,
        &asset_server,
        Vec3::new(stock_x, stock_y, 0.0),
        CardSuit::Hearts, // Dummy suit - not important for stock pile
        1, // Dummy value - not important for stock pile
        false, // Always face down
        (
            StockPile,
            CardBack, // Add the CardBack component
        ),
    );
    
    // Update tableau positions resource
    let mut tableau_positions = Vec::new();
    for pile in 0..7 {
        let x_pos = tableau_start_x + (pile as f32 * 100.0);
        tableau_positions.push(Vec3::new(x_pos, tableau_y, 0.0));
    }
    commands.insert_resource(TableauPositions(tableau_positions));
}
