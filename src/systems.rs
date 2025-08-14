use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButton;
use crate::components::*;
use crate::utils::{get_card_front_image, get_card_back_image, can_place_on_card, has_complete_stack};

// Helper function to create a card entity with sprite
fn create_card_entity(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec3,
    suit: CardSuit,
    value: u8,
    is_face_up: bool,
    components: impl Bundle,
) -> Entity {
    let sprite_image = if is_face_up {
        get_card_front_image(suit, value)
    } else {
        get_card_back_image(suit).to_string()
    };
    
    commands.spawn((
        Sprite {
            image: asset_server.load(sprite_image),
            custom_size: Some(Vec2::new(80.0, 120.0)),
            ..default()
        },
        Transform::from_translation(position),
        Card,
        CardData {
            suit,
            value,
            is_face_up,
        },
        components,
    )).id()
}


pub fn stock_click_system(
    mouse_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window>,
    mut stock_cards: ResMut<StockCards>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    waste_cards: Query<(Entity, &CardData), With<WastePile>>,
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
                    
                    // Create card in waste pile with proper components
                    create_card_entity(
                        &mut commands,
                        &asset_server,
                        Vec3::new(waste_x, waste_y, 10.0),
                        suit,
                        value,
                        true,
                        (
                            Draggable, // Waste cards are draggable
                            WastePile,
                            OriginalPosition(Vec3::new(waste_x, waste_y, 10.0)),
                        ),
                    );
                    
                    // Mark all existing waste cards as skipped (they're no longer the top card)
                    for (entity, _) in waste_cards.iter() {
                        commands.entity(entity).insert(SkippedWasteCard);
                        // Don't remove Draggable - we'll use SkippedWasteCard to prevent dragging instead
                    }
                } else {
                    // If stock is empty, do nothing - stock remains empty
                    // No recycling from waste pile in standard Solitaire
                }
            }
        }
    }
}

pub fn double_click_system(
    mouse_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window>,
    _commands: Commands,
    card_query: Query<(Entity, &mut Transform, &CardData, &OriginalPosition), (With<Card>, With<Draggable>)>,
    _asset_server: Res<AssetServer>,
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
            
            for (entity, transform, _card_data, _) in &card_query {
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
            
            // Foundation logic removed - cards can only go to foundation when a complete suit is collected
        }
    }
}


pub fn card_movement_system(
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

pub fn card_drag_system(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut selected_card: ResMut<SelectedCard>,
    mut card_query: Query<(Entity, &mut Transform, &CardData, Option<&SkippedWasteCard>), (With<Card>, With<Draggable>)>,
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
            for (entity, transform, card_data, skipped) in &mut card_query {
                // Only allow dragging face-up cards that aren't skipped
                if !card_data.is_face_up || skipped.is_some() {
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

pub fn card_drop_system(
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
            
            // First, get the selected card data and store what we need
            let (selected_value, selected_suit) = if let Ok((_, _, card_data, _)) = card_query.get(selected_entity) {
                (card_data.value, card_data.suit)
            } else {
                return; // Can't get selected card data, exit early
            };
            
            // Now collect all the information we need before any mutable operations
            let mut best_target: Option<(Vec3, f32, &'static str)> = None;
            let mut original_position = Vec3::ZERO;
            let mut cards_to_move = Vec::new();
            let mut stack_cards = Vec::new();
            
            // Get the original position from the selected card first
            if let Ok((_, transform, _, _)) = card_query.get(selected_entity) {
                original_position = transform.translation;
                
                // Collect all cards that are stacked on top of the selected card
                for (card_entity, card_transform, card_data, _) in card_query.iter() {
                    if card_entity != selected_entity {
                        // Check if this card is at the same X,Y position but higher Z (stacked on top)
                        let same_position = (card_transform.translation.x - original_position.x).abs() < 5.0 
                            && (card_transform.translation.y - original_position.y).abs() < 5.0;
                        let higher_z = card_transform.translation.z > original_position.z;
                        
                        if same_position && higher_z {
                            cards_to_move.push((card_entity, card_transform.translation.z - original_position.z));
                            stack_cards.push((card_data.suit, card_data.value));
                        }
                    }
                }
                
                // Add the selected card to the stack
                stack_cards.push((selected_suit, selected_value));
            }
            
            // Check if trying to drop on stock pile or waste pile (prevent this)
            let stock_x = -WINDOW_WIDTH / 2.0 + 100.0;
            let stock_y = -WINDOW_HEIGHT / 2.0 + 100.0;
            let waste_x = stock_x + 100.0;
            let waste_y = stock_y;
            
            // Check distance to stock pile center
            let stock_distance = (cursor_world_pos - Vec2::new(stock_x, stock_y)).length();
            // Check distance to waste pile center  
            let waste_distance = (cursor_world_pos - Vec2::new(waste_x, waste_y)).length();
            
            if stock_distance < 50.0 || waste_distance < 50.0 {
                // Don't allow dropping on stock pile or waste pile - snap back to original position
                if let Ok((_, mut transform, _, _)) = card_query.get_mut(selected_entity) {
                    transform.translation = original_position;
                }
                return;
            }
            
            // Check tableau targets first
            for (target_pos, distance, target_value, target_suit) in potential_targets {
                // Check if this is a valid placement (card value must be one lower AND colors must alternate)
                if can_place_on_card(selected_value, target_value) {
                    // Additional check: colors must alternate (red on black, black on red)
                    let selected_is_red = matches!(selected_suit, CardSuit::Hearts | CardSuit::Diamonds);
                    let target_is_red = matches!(target_suit, CardSuit::Hearts | CardSuit::Diamonds);
                    
                    if selected_is_red != target_is_red { // Colors must be different
                        // Allow placement on top of cards (we'll handle Z positioning when actually moving)
                        if let Some((_, current_distance, _)) = best_target {
                            if distance < current_distance {
                                best_target = Some((target_pos, distance, "tableau"));
                            }
                        } else {
                            best_target = Some((target_pos, distance, "tableau"));
                        }
                    }
                }
            }
            
            // Now get the selected card data and apply the movement
            if let Ok((_, mut transform, _, _)) = card_query.get_mut(selected_entity) {
                if let Some((target_pos, _, _)) = best_target {
                    // Position the selected card on top of the target card
                    let new_y = (target_pos.y - 30.0).max(-WINDOW_HEIGHT / 2.0 + 100.0); // Prevent going off bottom of screen
                    let new_position = Vec3::new(target_pos.x, new_y, target_pos.z + 1.0);
                    transform.translation = new_position;
                    
                    // Update the original position for future reference
                    commands.entity(selected_entity).insert(OriginalPosition(new_position));
                    
                    // Move all stacked cards to maintain their relative positions
                    for (card_entity, z_offset) in cards_to_move {
                        let stacked_y = (new_position.y - (z_offset * 30.0)).max(-WINDOW_HEIGHT / 2.0 + 100.0); // Prevent going off bottom of screen
                        let new_card_position = Vec3::new(
                            new_position.x, 
                            stacked_y, // Stack cards vertically with bounds checking
                            new_position.z + z_offset
                        );
                        commands.entity(card_entity).insert(Transform::from_translation(new_card_position));
                        commands.entity(card_entity).insert(OriginalPosition(new_card_position));
                    }
                    
                    // Find and flip the card that was underneath the moved card
                    flip_card_underneath(original_position, &mut commands, &asset_server, &mut card_query);
                } else {
                    // Check if dropped on an empty tableau pile
                    let mut should_flip = false;
                    let mut target_tableau_pos = None;
                    
                    // Use the collected tableau pile information
                    for (tableau_pos, is_empty) in &tableau_pile_info {
                        if *is_empty {
                            // Empty tableau pile - allow if the top card (being dragged) is a King
                            if selected_value == 13 {
                                // For single cards or incomplete stacks, just check if it's a King
                                if stack_cards.len() == 1 {
                                    // Single King card - allow placement
                                    target_tableau_pos = Some(*tableau_pos);
                                    should_flip = false; // No card underneath to flip
                                } else {
                                    // Multiple cards - check if it's a complete stack
                                    let mut sorted_stack = stack_cards.clone();
                                    sorted_stack.sort_by(|a, b| b.1.cmp(&a.1));
                                    
                                    if has_complete_stack(&sorted_stack) {
                                        target_tableau_pos = Some(*tableau_pos);
                                        should_flip = true;
                                    }
                                }
                            }
                        }
                    }
                    
                    // Now apply the movement if we found a valid target
                    if let Some(tableau_pos) = target_tableau_pos {
                        transform.translation = tableau_pos;
                        commands.entity(selected_entity).insert(OriginalPosition(tableau_pos));
                    } else {
                        // If not dropped on any valid target or empty pile, snap back to original position
                        transform.translation = original_position;
                    }
                    
                    // Flip card underneath if needed (after dropping the mutable borrow)
                    if should_flip {
                        flip_card_underneath(original_position, &mut commands, &asset_server, &mut card_query);
                    }
                }
            }
        }
    }
}



fn flip_card_underneath(
    original_position: Vec3,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    card_query: &mut Query<(Entity, &mut Transform, &CardData, &OriginalPosition), With<Card>>,
) {
    // Find the card that was directly underneath the moved card
    // We need to be more precise - only flip the card that was at the exact same X,Y position
    // but with a Z value that's just below the moved card's Z
    
    let mut cards_at_position = Vec::new();
    
    // First, collect all cards at the exact X,Y position
    for (entity, transform, card_data, _) in card_query.iter() { 
        let x_distance = (transform.translation.x - original_position.x).abs();
        let y_distance = (transform.translation.y - original_position.y).abs();
        
        // Use a very small threshold for exact positioning
        if x_distance < 1.0 && y_distance < 1.0 {
            cards_at_position.push((entity, transform.translation.z, card_data));
        }
    }
    
    // Sort by Z position to find the card that was directly underneath
    cards_at_position.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    
    // Only flip the card that was at the highest Z position (closest to the moved card)
    // and only if it's face down
    if let Some((entity, _, card_data)) = cards_at_position.last() {
        if !card_data.is_face_up {
            // Update the card to be face-up
            commands.entity(*entity).insert(CardData {
                suit: card_data.suit,
                value: card_data.value,
                is_face_up: true,
            });
            
            // Add the Draggable component so it can be moved
            commands.entity(*entity).insert(Draggable);
            
            // Change the sprite from CardBack to CardFront
            let front_image_path = get_card_front_image(card_data.suit, card_data.value);
           
            // Remove the old sprite and add the new one
            commands.entity(*entity).remove::<Sprite>();
            commands.entity(*entity).insert(Sprite {
                image: asset_server.load(front_image_path),
                custom_size: Some(Vec2::new(80.0, 120.0)),
                ..default()
            });
            
            // Remove the CardBack component and add CardFront
            commands.entity(*entity).remove::<CardBack>();
            commands.entity(*entity).insert(CardFront);
        }
    }
}

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
    
    // Shuffle the deck (simplified - just reverse for now)
    deck.reverse();
    
    // Deal exactly 28 cards to the tableau (7 piles with 1, 2, 3, 4, 5, 6, 7 cards)
    let tableau_start_x = -(6.0 * 100.0) / 2.0;
    let tableau_y = WINDOW_HEIGHT / 2.0 - 200.0;
    
    let mut card_index = 0;
    for pile in 0..7 {
        let pile_size = pile + 1; // Stack 1 has 1 card, Stack 2 has 2 cards, etc.
        let x_pos = tableau_start_x + (pile as f32 * 100.0);
        
        for card_in_pile in 0..pile_size {
            if card_index < deck.len() {
                let (suit, value) = deck[card_index];
                
                // Only the top card of each pile is face-up
                let is_face_up = card_in_pile == pile_size - 1;
                
                // Create card entity
                let _card_entity = create_card_entity(
                    commands,
                    &asset_server,
                    Vec3::new(x_pos, tableau_y, card_in_pile as f32),
                    suit,
                    value,
                    is_face_up,
                    (
                        Draggable, // Only face-up cards are draggable
                        TableauPile,
                        OriginalPosition(Vec3::new(x_pos, tableau_y, card_in_pile as f32)),
                        CoveredCard(None), // Top card is not covered
                    ),
                );
                
                // Sprite is now handled by the create_card_entity helper function
                
                card_index += 1;
            }
        }
    }
    
    // Store the remaining 24 cards in the stock pile
    let remaining_cards: Vec<(CardSuit, u8)> = deck.into_iter().skip(28).collect();
    stock_cards.0 = remaining_cards;
    
    // Create stock pile (top left) - show the top card
    let stock_x = -WINDOW_WIDTH / 2.0 + 100.0;
    let stock_y = -WINDOW_HEIGHT / 2.0 + 100.0;
    
    // Create stock pile visual representation (always shows card back)
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
