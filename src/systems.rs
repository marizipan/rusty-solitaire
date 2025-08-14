use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButton;
use crate::components::*;
use crate::utils::{get_card_front_image, get_card_back_image, can_place_on_card};


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
                    let card_entity = commands.spawn((
                        Sprite {
                            image: asset_server.load(get_card_back_image(suit)),
                            custom_size: Some(Vec2::new(80.0, 120.0)),
                            ..default()
                        },
                        Transform::from_xyz(waste_x, waste_y, 10.0),
                        Card,
                        CardFront,
                        CardData {
                            suit,
                            value,
                            is_face_up: true,
                        },
                        Draggable, // Waste cards are draggable
                        WastePile,
                        OriginalPosition(Vec3::new(waste_x, waste_y, 10.0)),
                    )).id();
                                       
                    // Add card front image
                    commands.spawn((
                        Sprite {
                            image: asset_server.load(get_card_front_image(suit, value)),
                            custom_size: Some(Vec2::new(50.0, 70.0)), // Much smaller for clear centering
                            ..default()
                        },
                        Transform::from_xyz(0.0, -10.0, 1.0), // Positioned relative to card center
                    )).set_parent_in_place(card_entity);
                } else {
                    // Stock is empty, recycle waste pile
                    let mut waste_card_data = Vec::new();
                    
                    // Collect all waste cards and their data
                    for (entity, card_data) in waste_cards.iter() {
                        waste_card_data.push((card_data.suit, card_data.value));
                        commands.entity(entity).despawn();
                    }
                    
                    // Reverse the waste cards and add them back to stock
                    waste_card_data.reverse();
                    stock_cards.0 = waste_card_data;
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
            
            // Check tableau targets first
            for (target_pos, distance, target_value, target_suit) in potential_targets {
                // Check if this is a valid placement (card value must be one lower AND colors must alternate)
                if can_place_on_card(selected_value, target_value) {
                    // Additional check: colors must alternate (red on black, black on red)
                    let selected_is_red = matches!(selected_suit, CardSuit::Hearts | CardSuit::Diamonds);
                    let target_is_red = matches!(target_suit, CardSuit::Hearts | CardSuit::Diamonds);
                    
                    if selected_is_red != target_is_red { // Colors must be different
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
            
            // If we found a valid target, collect stacked cards info first
            if let Some((_, _, _)) = best_target {
                // Get the original position from the selected card
                if let Ok((_, transform, _, _)) = card_query.get(selected_entity) {
                    original_position = transform.translation;
                    
                    // Collect all cards that are stacked on top of the selected card
                    for (card_entity, card_transform, _, _) in card_query.iter() {
                        if card_entity != selected_entity {
                            // Check if this card is at the same X,Y position but higher Z (stacked on top)
                            let same_position = (card_transform.translation.x - original_position.x).abs() < 5.0 
                                && (card_transform.translation.y - original_position.y).abs() < 5.0;
                            let higher_z = card_transform.translation.z > original_position.z;
                            
                            if same_position && higher_z {
                                cards_to_move.push((card_entity, card_transform.translation.z - original_position.z));
                            }
                        }
                    }
                }
            }
            
            // Now get the selected card data and apply the movement
            if let Ok((_, mut transform, _, _)) = card_query.get_mut(selected_entity) {
                if let Some((target_pos, _, _)) = best_target {
                    // Position the selected card on top of the target card
                    let new_position = Vec3::new(target_pos.x, target_pos.y - 30.0, target_pos.z + 1.0);
                    transform.translation = new_position;
                    
                    // Update the original position for future reference
                    commands.entity(selected_entity).insert(OriginalPosition(new_position));
                    
                    // Move all stacked cards to maintain their relative positions
                    for (card_entity, z_offset) in cards_to_move {
                        let new_card_position = Vec3::new(
                            new_position.x, 
                            new_position.y - (z_offset * 30.0), // Stack cards vertically
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
                            // Empty tableau pile - only allow if the top card (being dragged) is a King
                            if selected_value == 13 {
                                target_tableau_pos = Some(*tableau_pos);
                                should_flip = true;
                            } else {
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
    // Find all cards at the original position (these are the ones that were covered)
    for (entity, transform, card_data, _) in card_query.iter() { 
        
        let distance = ((transform.translation.x - original_position.x).powi(2) + 
                       (transform.translation.y - original_position.y).powi(2)).sqrt();
                
        // Use a small distance threshold to account for visual stacking offset
        if distance < 5.0 && !card_data.is_face_up {
           
            // Update the card to be face-up
            commands.entity(entity).insert(CardData {
                suit: card_data.suit,
                value: card_data.value,
                is_face_up: true,
            });
            
            // Add the Draggable component so it can be moved
            commands.entity(entity).insert(Draggable);
            
            // Change the sprite from CardBack to CardFront
            let front_image_path = get_card_front_image(card_data.suit, card_data.value);
           
            // Remove the old sprite and add the new one
            commands.entity(entity).remove::<Sprite>();
            commands.entity(entity).insert(Sprite {
                image: asset_server.load(front_image_path),
                custom_size: Some(Vec2::new(80.0, 120.0)),
                ..default()
            });
            
            // Remove the CardBack component and add CardFront
            commands.entity(entity).remove::<CardBack>();
            commands.entity(entity).insert(CardFront);
            
        }
    }
}
