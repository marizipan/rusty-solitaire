use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButton;
use crate::components::*;
use crate::card_validation::*;
use crate::card_placement::*;
use crate::card_double_click::*;
use tracing::debug;

/// Main drag and drop system for cards
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
