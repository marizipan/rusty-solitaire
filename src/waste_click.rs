use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButton;
use crate::components::*;
use crate::utils::{can_place_on_tableau, is_in_waste_or_stock_area};



pub fn waste_card_click_system(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window>,
    waste_cards: Query<(Entity, &Transform, &CardData), (With<WastePile>, Without<SkippedWasteCard>)>,
    tableau_cards: Query<(Entity, &Transform, &CardData), (With<TableauPile>, Without<WastePile>)>,
    tableau_positions: Res<TableauPositions>,
    mut foundation_piles: ResMut<FoundationPiles>,
    mut stock_cards: ResMut<StockCards>,
    mut last_click_time: Local<Option<std::time::Instant>>,
    clicked_entity: Res<ClickedEntity>,
) {
    // Handle double-click detection for waste cards
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }
    
    let now = std::time::Instant::now();
    
    // Check if this is a double-click on the same entity
    if let Some(last_time) = *last_click_time {
        if let Some(last_entity) = clicked_entity.0 {
            let time_diff = now.duration_since(last_time);
            
            // If double-click detected (within 500ms) and same entity
            if time_diff.as_millis() < 500 {
                // Check if the clicked entity is a waste card
                for (entity, _transform, _card_data) in waste_cards.iter() {
                    if entity == last_entity {
                        // This is a double-click on a waste card, proceed with the existing logic
                        break;
                    }
                }
            } else {
                // Not a double-click, update tracking and return
                *last_click_time = Some(now);
                return;
            }
        } else {
            // No previous entity, update tracking and return
            *last_click_time = Some(now);
            return;
        }
    } else {
        // No previous click time, update tracking and return
        *last_click_time = Some(now);
        return;
    }
    
    let Ok(window) = window_query.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    
    let cursor_world_pos = Vec2::new(
        cursor_pos.x - window.width() / 2.0,
        window.height() / 2.0 - cursor_pos.y,
    );
    
    
    // Find the top waste card (highest Z position)
    let mut top_waste_card: Option<(Entity, &Transform, &CardData)> = None;
    
    for (entity, transform, card_data) in waste_cards.iter() {
        if let Some((_entity, current_transform, _card_data)) = top_waste_card {
            if transform.translation.z > current_transform.translation.z {
                top_waste_card = Some((entity, transform, card_data));
            }
        } else {
            top_waste_card = Some((entity, transform, card_data));
        }
    }
    
    // Check if user clicked on the top waste card
    if let Some((waste_entity, waste_transform, waste_card_data)) = top_waste_card {
        let card_bounds = Vec2::new(40.0, 60.0);
        let card_pos = waste_transform.translation.truncate();
        
        if (cursor_world_pos - card_pos).abs().cmplt(card_bounds).all() {
            // Check Foundation Piles FIRST (higher priority than tableau)
            let foundation_start_x = -(6.0 * 100.0) / 2.0;
            let foundation_y = WINDOW_HEIGHT / 2.0 - 100.0;
            
            let mut best_target: Option<(Vec3, f32)> = None;
            
            // First check if it can be placed on foundation piles
            for i in 0..4 {
                let foundation_x = foundation_start_x + (i as f32 * 100.0);
                let foundation_pos = Vec3::new(foundation_x, foundation_y, 1.0);
                let foundation_pile = &foundation_piles.0[i];
                
                if foundation_pile.is_empty() {
                    // Empty foundation pile - only Aces can be placed
                    if waste_card_data.value == 1 {
                        best_target = Some((foundation_pos, 0.0));
                        break;
                    }
                } else {
                    // Foundation pile has cards - check if this card can be added
                    let (top_suit, top_value) = foundation_pile.last().unwrap();
                    if waste_card_data.suit == *top_suit && waste_card_data.value == top_value + 1 {
                        best_target = Some((foundation_pos, 0.0));
                        break;
                    }
                }
            }
            
            // If no valid foundation pile found, check if it can be placed on existing tableau cards
            if best_target.is_none() {
                for (_tableau_entity, tableau_transform, tableau_card_data) in tableau_cards.iter() {
                // Only consider face-up cards as valid targets
                if !tableau_card_data.is_face_up {
                    continue;
                }
                
                // CRITICAL: Basic validation - card value must be one lower than target
                if waste_card_data.value != tableau_card_data.value - 1 {
                    continue; // Skip invalid placements
                }
                
                // Check if this is a valid placement (card value must be one lower AND colors must alternate)
                // The value check was already done above, so we just need to check colors
                
                // CRITICAL: Check for duplicate values in the target stack FIRST
                let mut target_stack_values = Vec::new();
                for (other_entity, other_transform, other_card_data) in tableau_cards.iter() {
                    // Only check X position for stacking - Y position varies due to visual stacking
                    let x_same = (other_transform.translation.x - tableau_transform.translation.x).abs() < 5.0;
                    if x_same {
                        target_stack_values.push(other_card_data.value);
                        println!("WASTE STACK DEBUG: Found card {} at X position {:.1} (target X: {:.1})", 
                            other_card_data.value, other_transform.translation.x, tableau_transform.translation.x);
                    }
                }
                
                // Check if we're trying to place a card with a value that already exists in the stack
                if target_stack_values.contains(&waste_card_data.value) {
                    println!("VALIDATION FAILED: Card value {} already exists in target stack", waste_card_data.value);
                    continue; // Skip this placement
                }
                
                // Additional check: colors must alternate (red on black, black on red)
                let waste_is_red = matches!(waste_card_data.suit, CardSuit::Hearts | CardSuit::Diamonds);
                let tableau_is_red = matches!(tableau_card_data.suit, CardSuit::Hearts | CardSuit::Diamonds);
                
                if waste_is_red != tableau_is_red { // Colors must be different
                    // Additional check: make sure we're not placing on a card that's already covered
                    // Only place on the top card of each stack
                    let mut is_top_card = true;
                    for (other_entity, other_transform, _card_data) in tableau_cards.iter() {
                        if other_entity != _tableau_entity {
                            // Check if this other card is on top of our target
                            // Only check X position for stacking - Y position varies due to visual stacking
                            let x_same = (other_transform.translation.x - tableau_transform.translation.x).abs() < 5.0;
                            let z_higher = other_transform.translation.z > tableau_transform.translation.z;
                            
                            if x_same && z_higher {
                                is_top_card = false;
                                break;
                            }
                        }
                    }
                    
                    // Additional check: prevent placing waste card on its own stack
                    let mut is_valid_target = true;
                    let waste_x = waste_transform.translation.x;
                    let waste_y = waste_transform.translation.y;
                    let tableau_x = tableau_transform.translation.x;
                    let tableau_y = tableau_transform.translation.y;
                    
                    // Check if waste card and tableau card are at the same position (same stack)
                    if (waste_x - tableau_x).abs() < 5.0 && (waste_y - tableau_y).abs() < 5.0 {
                        is_valid_target = false;
                    }
                    
                    if is_top_card && is_valid_target {
                        let distance = (waste_transform.translation - tableau_transform.translation).length();
                        best_target = Some((tableau_transform.translation, distance));
                        break; // Found a valid tableau target, no need to continue
                    }
                }
                }
            }
            
            // If no valid foundation pile found, check if it can be placed on empty tableau piles
            if best_target.is_none() {
                // Only Kings can be placed on empty tableau piles
                if waste_card_data.value == 13 {
                    for tableau_pos in &tableau_positions.0 {
                        // Check if this tableau position is empty
                        let mut is_empty = true;
                        for (_entity, tableau_transform, _card_data) in tableau_cards.iter() {
                            if (tableau_transform.translation.x - tableau_pos.x).abs() < 5.0 
                                && (tableau_transform.translation.y - tableau_pos.y).abs() < 5.0 {
                                is_empty = false;
                                break;
                            }
                        }
                        
                        if is_empty {
                            best_target = Some((*tableau_pos, 0.0)); // Distance 0 for empty piles
                            break;
                        }
                    }
                }
            }
            
            // If we found a valid target, move the waste card there
            if let Some((target_pos, _target_distance)) = best_target {
                // Check if this is a Foundation Pile placement
                let foundation_start_x = -(6.0 * 100.0) / 2.0;
                let foundation_y = WINDOW_HEIGHT / 2.0 - 100.0;
                
                if (target_pos.x - foundation_start_x).abs() < 200.0 && (target_pos.y - foundation_y).abs() < 50.0 {
                    // This is a foundation pile placement
                    let foundation_index = ((target_pos.x - foundation_start_x) / 100.0) as usize;
                    
                    // Move the waste card
                    commands.entity(waste_entity)
                        .remove::<WastePile>()
                        .remove::<SkippedWasteCard>()
                        .insert(FoundationPile)
                        .insert(OriginalPosition(target_pos));
                    
                    // Update the transform to move to foundation pile
                    commands.entity(waste_entity).insert(Transform::from_translation(target_pos));
                    
                    // Update foundation pile
                    foundation_piles.0[foundation_index].push((waste_card_data.suit, waste_card_data.value));
                    
                    // Remove from stock_cards (waste pile)
                    stock_cards.0.retain(|card| card.0 != waste_card_data.suit || card.1 != waste_card_data.value);
                    
                    // Reset double-click tracking after successful move
                    *last_click_time = None;
                } else {
                    // This is a tableau placement
                    let new_position = Vec3::new(target_pos.x, target_pos.y, target_pos.z + 1.0);
                    
                    // Move the waste card
                    commands.entity(waste_entity)
                        .remove::<WastePile>()
                        .remove::<SkippedWasteCard>()
                        .insert(TableauPile)
                        .insert(OriginalPosition(new_position))
                        .insert(Draggable);
                    
                    // Update the transform to move to tableau position
                    commands.entity(waste_entity).insert(Transform::from_translation(new_position));
                    
                    // Remove from stock_cards (waste pile)
                    stock_cards.0.retain(|card| card.0 != waste_card_data.suit || card.1 != waste_card_data.value);
                    
                    // Reset double-click tracking after successful move
                    *last_click_time = None;
                }
                
                // Mark all other waste cards as skipped since the top one moved
                for (entity, _transform, _card_data) in waste_cards.iter() {
                    if entity != waste_entity {
                        commands.entity(entity).insert(SkippedWasteCard);
                    }
                }
            }
        }
    }
}

