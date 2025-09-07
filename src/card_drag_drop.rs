use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButton;
use crate::components::*;
use crate::utils::{is_valid_stack_sequence, can_place_on_foundation, can_place_on_tableau_card, find_best_tableau_target};
use tracing::debug;

/// Unified drag and drop system for cards
pub fn card_drag_drop_system(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut selected_card: ResMut<SelectedCard>,
    mut transform_query: Query<&mut Transform, (With<Card>, With<Draggable>)>,
    card_data_query: Query<&CardData>,
    entity_query: Query<Entity, (With<Card>, With<Draggable>)>,
    mut foundation_piles: ResMut<FoundationPiles>,
    tableau_positions: Res<TableauPositions>,
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut last_click_time: Local<Option<std::time::Instant>>,
    mut clicked_entity: ResMut<ClickedEntity>,
    mut original_positions: Local<std::collections::HashMap<Entity, Vec3>>,
    tableau_cards_query: Query<(Entity, &CardData), (With<TableauPile>, Without<WastePile>)>,
) {
    let Ok(window) = window_query.single() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    
    // Collect tableau cards data for validation
    let tableau_cards: Vec<(Entity, Vec3, CardData)> = tableau_cards_query
        .iter()
        .filter_map(|(entity, card_data)| {
            if let Ok(transform) = transform_query.get(entity) {
                Some((entity, transform.translation, card_data.clone()))
            } else {
                None
            }
        })
        .collect();

    // Handle mouse press - check for double-click and update tracking
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = window.cursor_position() {
            let Ok(cursor_world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };

            // Find the card under the cursor
            if let Some(entity) = find_card_under_cursor(cursor_world_pos, &entity_query, &transform_query, &card_data_query) {
                // Check if this card can be dragged
                if can_drag_card(entity, &entity_query, &transform_query, &card_data_query) {
                    let now = std::time::Instant::now();
                    
                    // Check for double-click
                    if let Some(last_time) = *last_click_time {
                        if let Some(last_clicked_entity) = clicked_entity.0 {
                            let time_diff = now.duration_since(last_time);
                            
                            // If double-click detected (within 500ms) and same entity
                            if time_diff.as_millis() < 500 && last_clicked_entity == entity {
                                debug!("DOUBLE-CLICK DETECTED on entity: {:?}", entity);
                                
                                // Try to move to foundation pile
                                if let Ok(card_data) = card_data_query.get(entity) {
                                    if card_data.is_face_up {
                                        debug!("DOUBLE-CLICK: Attempting foundation move for card {:?} (value: {}, suit: {:?})", 
                                               card_data.suit, card_data.value, card_data.suit);
                                        
                                        // Try foundation move first
                                        if try_foundation_move_simple(entity, &mut transform_query, card_data, &mut foundation_piles, &mut commands) {
                                            debug!("DOUBLE-CLICK: Successfully moved card to foundation");
                                            // Reset double-click tracking
                                            *last_click_time = None;
                                            clicked_entity.0 = None;
                                            return;
                                        } else {
                                            debug!("DOUBLE-CLICK: Foundation move failed, trying tableau move");
                                            // Try tableau move if foundation failed
                                            if try_tableau_move_simple(entity, &mut transform_query, card_data, &tableau_cards, &tableau_positions.0, &mut commands) {
                                                debug!("DOUBLE-CLICK: Successfully moved card to tableau");
                                                // Reset double-click tracking
                                                *last_click_time = None;
                                                clicked_entity.0 = None;
                                                return;
                                            } else {
                                                debug!("DOUBLE-CLICK: Both foundation and tableau moves failed");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Update double-click tracking
                    *last_click_time = Some(now);
                    clicked_entity.0 = Some(entity);
                    
                    debug!("Card clicked - entity: {:?}, waiting for potential double-click", entity);
                }
            }
        }
    }

    // Handle mouse hold - start dragging after a short delay to allow for double-click detection
    if mouse_input.pressed(MouseButton::Left) {
        if let Some(clicked_entity_id) = clicked_entity.0 {
            if selected_card.0.is_none() {
                // Check if enough time has passed since the click to start dragging
                if let Some(last_time) = *last_click_time {
                    let now = std::time::Instant::now();
                    let time_since_click = now.duration_since(last_time);
                    
                    // Start dragging after 200ms to allow double-click detection
                    if time_since_click.as_millis() > 200 {
                        selected_card.0 = Some(clicked_entity_id);
                        commands.entity(clicked_entity_id).insert(CurrentlyDragging);
                        
                        // Store the original position for snap-back
                        if let Ok(transform) = transform_query.get(clicked_entity_id) {
                            original_positions.insert(clicked_entity_id, transform.translation);
                            debug!("Started dragging entity {:?}: {:?}", clicked_entity_id, transform.translation);
                        }
                        
                        // Clear the clicked entity to prevent double-click conflicts
                        clicked_entity.0 = None;
                    }
                }
            }
        }
    }

    // Handle mouse release - drop or snap back
    if mouse_input.just_released(MouseButton::Left) {
        if let Some(selected_entity) = selected_card.0 {
            // Get cursor position for drop validation
            if let Some(cursor_pos) = window.cursor_position() {
                let Ok(cursor_world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };
                
                debug!("Attempting to drop card at position: {:?}", cursor_world_pos);
                
                // Try to place the card with proper validation
                if let Some(target_pos) = find_valid_drop_target(cursor_world_pos, selected_entity, &foundation_piles, &entity_query, &transform_query, &card_data_query) {
                    debug!("Valid drop target found at: {:?}", target_pos);
                    place_card(&mut commands, &mut foundation_piles, selected_entity, target_pos, &card_data_query);
                    // Clean up the stored position since placement was successful
                    original_positions.remove(&selected_entity);
                } else {
                    debug!("No valid drop target found - snapping back");
                    // No valid target - snap back to original position
                    snap_back_card(&mut commands, selected_entity, &transform_query, &mut original_positions);
                }
            } else {
                // No cursor position - snap back
                snap_back_card(&mut commands, selected_entity, &transform_query, &mut original_positions);
            }
            
            // Remove CurrentlyDragging component and clear selection
            commands.entity(selected_entity).remove::<CurrentlyDragging>();
            selected_card.0 = None;
        }
    }

    // Handle dragging - update card position
    if let Some(selected_entity) = selected_card.0 {
        if let Some(cursor_pos) = window.cursor_position() {
            let Ok(cursor_world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };
            
            if let Ok(mut transform) = transform_query.get_mut(selected_entity) {
                transform.translation = Vec3::new(cursor_world_pos.x, cursor_world_pos.y, 10.0);
            }
        }
    }
}

/// Finds the card under the cursor
fn find_card_under_cursor(
    cursor_pos: Vec2,
    entity_query: &Query<Entity, (With<Card>, With<Draggable>)>,
    transform_query: &Query<&mut Transform, (With<Card>, With<Draggable>)>,
    card_data_query: &Query<&CardData>,
) -> Option<Entity> {
    let mut best_entity = None;
    let mut best_distance = f32::INFINITY;

    for entity in entity_query.iter() {
        if let Ok(card_data) = card_data_query.get(entity) {
            // Only allow face-up cards
            if !card_data.is_face_up {
                continue;
            }
            
            if let Ok(transform) = transform_query.get(entity) {
                let card_pos = transform.translation.truncate();
                let card_bounds = Vec2::new(40.0, 60.0);
                
                if (cursor_pos - card_pos).abs().cmplt(card_bounds).all() {
                    let distance = (cursor_pos - card_pos).length();
                    if distance < best_distance {
                        best_entity = Some(entity);
                        best_distance = distance;
                    }
                }
            }
        }
    }

    best_entity
}

/// Checks if a card can be dragged (is top card and can lead stack)
fn can_drag_card(
    entity: Entity,
    entity_query: &Query<Entity, (With<Card>, With<Draggable>)>,
    transform_query: &Query<&mut Transform, (With<Card>, With<Draggable>)>,
    card_data_query: &Query<&CardData>,
) -> bool {
    let Ok(transform) = transform_query.get(entity) else { return false; };
    let current_pos = transform.translation;
    
    // Check if this card is the top card of its stack
    for other_entity in entity_query.iter() {
        if other_entity != entity {
            if let Ok(other_transform) = transform_query.get(other_entity) {
                let x_same = (other_transform.translation.x - current_pos.x).abs() < 15.0;
                let z_higher = other_transform.translation.z > current_pos.z + 0.5;
                
                if x_same && z_higher {
                    return false; // Not the top card
                }
            }
        }
    }
    
    // Check if this card can lead a stack
    if let Ok(card_data) = card_data_query.get(entity) {
        // Collect all cards above this one
        let mut cards_above = Vec::new();
        
        for other_entity in entity_query.iter() {
            if other_entity != entity {
                if let Ok(other_transform) = transform_query.get(other_entity) {
                    let x_same = (other_transform.translation.x - current_pos.x).abs() < 15.0;
                    let z_higher = other_transform.translation.z > current_pos.z + 0.5;
                    
                    if x_same && z_higher {
                        if let Ok(other_card_data) = card_data_query.get(other_entity) {
                            cards_above.push((other_card_data.suit, other_card_data.value));
                        }
                    }
                }
            }
        }
        
        // If no cards above, this card can lead
        if cards_above.is_empty() {
            return true;
        }
        
        // Check if this forms a valid descending sequence
        let mut all_cards = vec![(card_data.suit, card_data.value)];
        all_cards.extend(cards_above);
        all_cards.sort_by(|a, b| b.1.cmp(&a.1));
        
        return is_valid_stack_sequence(&all_cards);
    }
    
    false
}

/// Finds a valid drop target for the card with proper solitaire rules
fn find_valid_drop_target(
    cursor_pos: Vec2,
    selected_entity: Entity,
    foundation_piles: &FoundationPiles,
    entity_query: &Query<Entity, (With<Card>, With<Draggable>)>,
    transform_query: &Query<&mut Transform, (With<Card>, With<Draggable>)>,
    card_data_query: &Query<&CardData>,
) -> Option<Vec3> {
    let Ok(selected_card_data) = card_data_query.get(selected_entity) else { return None; };
    
    // Check foundation piles first
    if let Some(target_pos) = find_foundation_target(cursor_pos, foundation_piles, selected_card_data) {
        return Some(target_pos);
    }
    
    // Check tableau targets (only for tableau cards, not waste pile cards)
    if let Some(target_pos) = find_tableau_target(cursor_pos, selected_card_data, selected_entity, entity_query, transform_query, card_data_query) {
        return Some(target_pos);
    }
    
    // Check empty tableau positions (only for Kings)
    if selected_card_data.value == 13 { // King
        if let Some(target_pos) = find_empty_tableau_target(cursor_pos) {
            return Some(target_pos);
        }
    }
    
    None
}

/// Finds foundation pile targets with proper validation
fn find_foundation_target(cursor_pos: Vec2, foundation_piles: &FoundationPiles, card_data: &CardData) -> Option<Vec3> {
    let foundation_start_x = -(6.0 * 100.0) / 2.0;
    let foundation_y = WINDOW_HEIGHT / 2.0 - 100.0;
    
    // Check if cursor is near foundation area (more lenient)
    if (cursor_pos.y - foundation_y).abs() > 100.0 {
        return None;
    }

    for i in 0..4 {
        let foundation_x = foundation_start_x + (i as f32 * 100.0);
        let foundation_distance = (cursor_pos - Vec2::new(foundation_x, foundation_y)).length();
        
        if foundation_distance < 80.0 {
            // Check if this card can be placed on this foundation pile
            if can_place_on_foundation(card_data, &foundation_piles.0[i]) {
                let target_pos = Vec3::new(foundation_x, foundation_y, 1.0);
                return Some(target_pos);
            }
        }
    }

    None
}

/// Finds tableau card targets with proper validation
fn find_tableau_target(
    cursor_pos: Vec2,
    selected_card_data: &CardData,
    selected_entity: Entity,
    entity_query: &Query<Entity, (With<Card>, With<Draggable>)>,
    transform_query: &Query<&mut Transform, (With<Card>, With<Draggable>)>,
    card_data_query: &Query<&CardData>,
) -> Option<Vec3> {
    let mut best_target = None;
    let mut best_distance = f32::INFINITY;

    debug!("Looking for tableau target at cursor: {:?}", cursor_pos);

    for entity in entity_query.iter() {
        // Skip the currently selected card
        if entity == selected_entity {
            continue;
        }
        
        if let Ok(transform) = transform_query.get(entity) {
            let target_pos = transform.translation;
            let distance = (cursor_pos - target_pos.truncate()).length();
            
            debug!("Checking entity at {:?}, distance: {:.2}, y: {:.2}", target_pos, distance, target_pos.y);
            
            // Consider all cards for tableau moves (including waste pile cards)
            if distance < 80.0 && distance < best_distance {
                // Check if this card can be placed on the target card
                if let Ok(target_card_data) = card_data_query.get(entity) {
                    debug!("Target card: suit={:?}, value={}, face_up: {}", target_card_data.suit, target_card_data.value, target_card_data.is_face_up);
                    if can_place_on_tableau_card(selected_card_data, target_card_data) {
                        debug!("Valid tableau target found!");
                        best_target = Some(target_pos);
                        best_distance = distance;
                    } else {
                        debug!("Invalid tableau placement");
                    }
                }
            }
        }
    }

    best_target
}

/// Finds empty tableau targets (only for Kings)
fn find_empty_tableau_target(cursor_pos: Vec2) -> Option<Vec3> {
    // Only allow empty tableau placement in the tableau area (more lenient)
    if cursor_pos.y > 250.0 {
        return None;
    }
    
    for i in 0..7 {
        let tableau_x = -300.0 + (i as f32 * 100.0);
        let tableau_y = 110.0;
        let tableau_pos = Vec3::new(tableau_x, tableau_y, 0.0);
        let distance = (cursor_pos - tableau_pos.truncate()).length();
        
        if distance < 80.0 {
            return Some(tableau_pos);
        }
    }

    None
}


/// Places a card at the target position
fn place_card(
    commands: &mut Commands,
    foundation_piles: &mut FoundationPiles,
    selected_entity: Entity,
    target_pos: Vec3,
    card_data_query: &Query<&CardData>,
) {
    let Ok(card_data) = card_data_query.get(selected_entity) else { return; };
    
    // Determine if this is a foundation or tableau placement
    let foundation_y = WINDOW_HEIGHT / 2.0 - 100.0;
    let is_foundation = (target_pos.y - foundation_y).abs() < 100.0;
    
    if is_foundation {
        place_on_foundation(commands, foundation_piles, selected_entity, target_pos, card_data);
    } else {
        place_on_tableau(commands, selected_entity, target_pos);
    }
}

/// Places a card on a foundation pile
fn place_on_foundation(
    commands: &mut Commands,
    foundation_piles: &mut FoundationPiles,
    selected_entity: Entity,
    target_pos: Vec3,
    card_data: &CardData,
) {
    let foundation_start_x = -(6.0 * 100.0) / 2.0;
    let foundation_index = ((target_pos.x - foundation_start_x) / 100.0) as usize;
    
    // Bounds check to prevent index out of bounds
    if foundation_index >= 4 {
        debug!("Foundation index out of bounds: {} (target_pos.x: {})", foundation_index, target_pos.x);
        return;
    }
    
    // Update foundation pile
    foundation_piles.0[foundation_index].push((card_data.suit, card_data.value));
    
    // Position the card
    let new_position = Vec3::new(
        target_pos.x,
        target_pos.y,
        foundation_piles.0[foundation_index].len() as f32 * 0.1
    );
    
    commands.entity(selected_entity).insert(Transform::from_translation(new_position));
    commands.entity(selected_entity).insert(OriginalPosition(new_position));
    
    // Update components for foundation
    commands.entity(selected_entity).insert(FoundationPile);
    commands.entity(selected_entity).remove::<TableauPile>();
    commands.entity(selected_entity).remove::<WastePile>();
}

/// Places a card on a tableau
fn place_on_tableau(
    commands: &mut Commands,
    selected_entity: Entity,
    target_pos: Vec3,
) {
    // Position the card
    let new_position = Vec3::new(target_pos.x, target_pos.y, target_pos.z + 0.1);
    
    commands.entity(selected_entity).insert(Transform::from_translation(new_position));
    commands.entity(selected_entity).insert(OriginalPosition(new_position));
    
    // Update components for tableau
    commands.entity(selected_entity).insert(TableauPile);
    commands.entity(selected_entity).remove::<FoundationPile>();
    commands.entity(selected_entity).remove::<WastePile>();
}

/// Snaps a card back to its original position
fn snap_back_card(
    commands: &mut Commands,
    selected_entity: Entity,
    transform_query: &Query<&mut Transform, (With<Card>, With<Draggable>)>,
    original_positions: &mut std::collections::HashMap<Entity, Vec3>,
) {
    debug!("Card snapped back - entity: {:?}", selected_entity);
    
    // Get the original position from our local storage
    if let Some(original_pos) = original_positions.get(&selected_entity) {
        // Restore the original position directly
        commands.entity(selected_entity).insert(Transform::from_translation(*original_pos));
        debug!("Restored card to original position: {:?}", original_pos);
        
        // Clean up the stored position
        original_positions.remove(&selected_entity);
    } else {
        debug!("No original position found for card");
    }
    
    // Remove the CurrentlyDragging component
    commands.entity(selected_entity).remove::<CurrentlyDragging>();
}

/// Simple foundation move function that reuses existing validation logic
fn try_foundation_move_simple(
    entity: Entity,
    transform_query: &mut Query<&mut Transform, (With<Card>, With<Draggable>)>,
    card_data: &CardData,
    foundation_piles: &mut ResMut<FoundationPiles>,
    commands: &mut Commands,
) -> bool {
    // Find the appropriate foundation pile for this card
    let foundation_index = match card_data.suit {
        CardSuit::Hearts => 0,
        CardSuit::Diamonds => 1,
        CardSuit::Clubs => 2,
        CardSuit::Spades => 3,
    };
    
    let foundation_pile = &foundation_piles.0[foundation_index];
    
    // Use existing validation logic from utils.rs
    if can_place_on_foundation(card_data, foundation_pile) {
        debug!("FOUNDATION PLACEMENT: Card {:?} (value: {}, suit: {:?}) can be placed on foundation pile {}", 
               card_data.suit, card_data.value, card_data.suit, foundation_index);
        
        // Calculate foundation pile position
        let foundation_start_x = -(6.0 * 100.0) / 2.0;
        let foundation_x = foundation_start_x + (foundation_index as f32 * 100.0);
        let foundation_y = WINDOW_HEIGHT / 2.0 - 100.0;
        
        // Store original position for flip trigger
        let original_position = if let Ok(transform) = transform_query.get(entity) {
            transform.translation
        } else {
            return false;
        };
        
        // Move the card to the foundation pile
        let foundation_pos = Vec3::new(foundation_x, foundation_y, foundation_pile.len() as f32 + 1.0);
        if let Ok(mut transform) = transform_query.get_mut(entity) {
            transform.translation = foundation_pos;
        }
        
        // Update the FoundationPiles resource
        foundation_piles.0[foundation_index].push((card_data.suit, card_data.value));
        
        // Remove tableau/waste components and add foundation component
        commands.entity(entity)
            .remove::<TableauPile>()
            .remove::<WastePile>()
            .remove::<SkippedWasteCard>()
            .remove::<StockPile>()
            .remove::<Draggable>() // Foundation cards cannot be moved
            .insert(FoundationPile)
            .insert(OriginalPosition(foundation_pos));
        
        // Trigger card flipping for face-down cards underneath
        commands.spawn(NeedsFlipUnderneath(original_position));
        
        true
    } else {
        debug!("FOUNDATION REJECTED: Card {:?} (value: {}, suit: {:?}) cannot be placed on foundation pile {} (empty: {}, top: {:?})", 
               card_data.suit, card_data.value, card_data.suit, foundation_index, foundation_pile.is_empty(), 
               foundation_pile.last());
        false
    }
}

/// Simple tableau move function that reuses existing validation logic
fn try_tableau_move_simple(
    entity: Entity,
    transform_query: &mut Query<&mut Transform, (With<Card>, With<Draggable>)>,
    card_data: &CardData,
    tableau_cards: &[(Entity, Vec3, CardData)],
    tableau_positions: &[Vec3],
    commands: &mut Commands,
) -> bool {
    // Use existing validation logic from utils.rs
    if let Some(target_pos) = find_best_tableau_target(card_data, transform_query.get(entity).unwrap().translation, tableau_cards, tableau_positions, Some(entity)) {
        debug!("TABLEAU PLACEMENT: Card {:?} (value: {}, suit: {:?}) can be placed on tableau at {:?}", 
               card_data.suit, card_data.value, card_data.suit, target_pos);
        
        // Store original position for flip trigger
        let original_position = if let Ok(transform) = transform_query.get(entity) {
            transform.translation
        } else {
            return false;
        };
        
        // Check if it's on an existing card or empty pile
        let mut is_on_existing_card = false;
        let mut highest_z = target_pos.z;
        
        // Check if there are existing cards at this position
        for (_entity, card_transform, _card_data) in tableau_cards.iter() {
            let x_same = (card_transform.x - target_pos.x).abs() < 5.0;
            let y_same = (card_transform.y - target_pos.y).abs() < 5.0;
            if x_same && y_same {
                is_on_existing_card = true;
                if card_transform.z > highest_z {
                    highest_z = card_transform.z;
                }
            }
        }
        
        let new_position = if is_on_existing_card {
            Vec3::new(target_pos.x, target_pos.y, highest_z + 1.0)
        } else {
            target_pos
        };
        
        // Move the card to the tableau
        if let Ok(mut transform) = transform_query.get_mut(entity) {
            transform.translation = new_position;
        }
        
        // Update components
        commands.entity(entity)
            .remove::<WastePile>()
            .remove::<SkippedWasteCard>()
            .remove::<StockPile>()
            .insert(TableauPile)
            .insert(OriginalPosition(new_position))
            .insert(Draggable);
        
        // Trigger card flipping for face-down cards underneath
        commands.spawn(NeedsFlipUnderneath(original_position));
        
        true
    } else {
        debug!("TABLEAU REJECTED: Card {:?} (value: {}, suit: {:?}) cannot be placed on tableau", 
               card_data.suit, card_data.value, card_data.suit);
        false
    }
}