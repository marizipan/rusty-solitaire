use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButton;
use crate::components::*;
use crate::utils::{get_card_front_image, can_place_on_tableau};

pub fn stock_click_system(
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
                    let _card_entity = commands.spawn((
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

pub fn double_click_system(
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