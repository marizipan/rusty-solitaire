use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButton;
use crate::components::*;
use crate::utils::{is_red_suit, is_valid_stack_sequence};
use tracing::debug;



pub fn card_drag_system(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut selected_card: ResMut<SelectedCard>,
    mut transform_query: Query<&mut Transform, (With<Card>, With<Draggable>, Without<SkippedWasteCard>)>,
    card_data_query: Query<&CardData, (With<Card>, With<Draggable>, Without<SkippedWasteCard>)>,
    entity_query: Query<Entity, (With<Card>, With<Draggable>, Without<SkippedWasteCard>)>,
    window_query: Query<&Window>,
    mut foundation_piles: ResMut<FoundationPiles>,
    mut commands: Commands,
    mut last_click_time: Local<Option<std::time::Instant>>,
    mut clicked_entity: ResMut<ClickedEntity>,
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
            for entity in &entity_query {
                if let Ok(card_data) = card_data_query.get(entity) {
                    if !card_data.is_face_up {
                        continue;
                    }
                    
                    if let Ok(transform) = transform_query.get_mut(entity) {
                        let card_pos = transform.translation.truncate();
                        let card_bounds = Vec2::new(40.0, 60.0);
                        
                        if (cursor_world_pos - card_pos).abs().cmplt(card_bounds).all() {
                            // Check if this card is actually the top card of its stack
                            let mut is_top_card = true;
                            let current_pos = transform.translation;
                            
                            // Drop the mutable borrow before checking other entities
                            drop(transform);
                            
                            for other_entity in &entity_query {
                                if other_entity != entity {
                                    if let Ok(other_transform) = transform_query.get(other_entity) {
                                        let x_same = (other_transform.translation.x - current_pos.x).abs() < 15.0;
                                        // Use Z position for stacking detection instead of Y position
                                        // Visual stacking affects Y but Z represents the actual stacking order
                                        let z_higher = other_transform.translation.z > current_pos.z + 0.5;
                                        
                                        if x_same && z_higher {
                                            is_top_card = false;
                                            break;
                                        }
                                    }
                                }
                            }
                            
                            // Only allow dragging if this is the top card
                            if !is_top_card {
                                continue;
                            }
                            
                            // Check if this card is a stack leader (can drag entire stack)
                            let mut can_lead_stack = false;
                            
                            // A card can lead a stack if:
                            // 1. It's a single card (no cards above it), OR
                            // 2. It's part of a valid descending sequence with alternating suits
                            if let Ok(card_data) = card_data_query.get(entity) {
                                // First, collect all cards that are stacked above this card
                                let mut cards_above = Vec::new();
                                let mut stack_entities = Vec::new();
                                
                                for other_entity in &entity_query {
                                    if other_entity != entity {
                                        if let Ok(other_transform) = transform_query.get(other_entity) {
                                            let x_same = (other_transform.translation.x - current_pos.x).abs() < 15.0;
                                            // Use Z position for stacking detection instead of Y position
                                            // Visual stacking affects Y but Z represents the actual stacking order
                                            let z_higher = other_transform.translation.z > current_pos.z + 0.5;
                                            
                                            if x_same && z_higher {
                                                if let Ok(other_card_data) = card_data_query.get(other_entity) {
                                                    cards_above.push((other_card_data.suit, other_card_data.value));
                                                    stack_entities.push(other_entity);
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                // If no cards above, this card can lead a stack
                                if cards_above.is_empty() {
                                    can_lead_stack = true;
                                } else {
                                    // Check if this forms a valid descending sequence with alternating colors
                                    let mut all_cards = vec![(card_data.suit, card_data.value)];
                                    all_cards.extend(cards_above);
                                    
                                    // Sort by value in descending order (highest to lowest)
                                    all_cards.sort_by(|a, b| b.1.cmp(&a.1));
                                    
                                    // Check if the sequence is valid
                                    can_lead_stack = is_valid_stack_sequence(&all_cards);
                                }
                            }
                            
                            // Only allow dragging if this card can lead a stack
                            if !can_lead_stack {
                                continue;
                            }
                            
                            // All cards now require double-click to move to foundation
                            // Single-click only selects the card for dragging
                            
                                                                                    // Check if there are face-down cards underneath this card that need flipping
                                // This triggers the flip system even when cards aren't moved
                                let mut has_face_down_underneath = false;
                                for other_entity in &entity_query {
                                    if other_entity != entity {
                                        if let Ok(other_transform) = transform_query.get(other_entity) {
                                            let x_same = (other_transform.translation.x - current_pos.x).abs() < 15.0;
                                            // Use Z position for stacking detection instead of Y position
                                            // Visual stacking affects Y but Z represents the actual stacking order
                                            let z_lower = other_transform.translation.z < current_pos.z - 0.5;
                                            
                                            if x_same && z_lower {
                                                if let Ok(other_card_data) = card_data_query.get(other_entity) {
                                                    if !other_card_data.is_face_up {
                                                        has_face_down_underneath = true;
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                        
                        // If there are face-down cards underneath, spawn a flip trigger
                        // This triggers the flip system when the card is moved
                        if has_face_down_underneath {
                            commands.spawn(NeedsFlipUnderneath(current_pos));
                        }
                            
                            // Update double-click tracking
                            let now = std::time::Instant::now();
                            *last_click_time = Some(now);
                            clicked_entity.0 = Some(entity);
                            selected_card.0 = Some(entity);
                            debug!("Card selected for dragging - entity: {:?}, value: {}, suit: {:?}", 
                                   entity, card_data.value, card_data.suit);
                            break;
                        }
                    }
                }
            }
        }
    }

    if mouse_input.just_released(MouseButton::Left) {
        // Clear selected card on mouse release
        selected_card.0 = None;
    }
}
