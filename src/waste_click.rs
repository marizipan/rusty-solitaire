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
) {
    // Only process when user clicks
    if !mouse_input.just_pressed(MouseButton::Left) {
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
            for (_tableau_entity, tableau_transform, tableau_card_data) in tableau_cards.iter() {
                // Only consider face-up cards as valid targets
                if !tableau_card_data.is_face_up {
                    continue;
                }
                
                // CRITICAL: Prevent placing cards on waste or stock pile areas
                if is_in_waste_or_stock_area(tableau_transform.translation.truncate()) {
                    continue;
                }
                
                // Check if this is a valid placement (card value must be one lower AND colors must alternate)
                if can_place_on_tableau(waste_card_data.value, waste_card_data.suit, tableau_card_data.value, tableau_card_data.suit) {
                    // Additional check: make sure we're not placing on a card that's already covered
                    // Only place on the top card of each stack
                    let mut is_top_card = true;
                    for (other_entity, other_transform, _card_data) in tableau_cards.iter() {
                        if other_entity != _tableau_entity {
                            // Check if this other card is on top of our target
                            let x_same = (other_transform.translation.x - tableau_transform.translation.x).abs() < 5.0;
                            let y_same = (other_transform.translation.y - tableau_transform.translation.y).abs() < 5.0;
                            let z_higher = other_transform.translation.z > tableau_transform.translation.z;
                            
                            if x_same && y_same && z_higher {
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
                    
                    if (waste_x - tableau_x).abs() < 5.0 && (waste_y - tableau_y).abs() < 5.0 {
                        // This would place the waste card on its own stack - not allowed
                        is_valid_target = false;
                    }
                    
                    // Additional check: prevent placing cards with duplicate values in the same stack
                    let mut has_duplicate_value = false;
                    let mut is_already_in_stack = false;
                    let mut stack_values = Vec::new();
                    
                    // First, collect all values already in the target stack
                    for (other_entity, other_transform, other_card_data) in tableau_cards.iter() {
                        if other_entity != _tableau_entity {
                            let x_same = (other_transform.translation.x - tableau_x).abs() < 5.0;
                            let y_same = (other_transform.translation.y - tableau_y).abs() < 5.0;
                            if x_same && y_same {
                                stack_values.push(other_card_data.value);
                                
                                // Check if this card has the same value as the waste card
                                if other_card_data.value == waste_card_data.value {
                                    has_duplicate_value = true;
                                    break;
                                }
                                
                                // Check if this is the same card (same suit and value)
                                if other_card_data.suit == waste_card_data.suit && 
                                   other_card_data.value == waste_card_data.value {
                                    is_already_in_stack = true;
                                    break;
                                }
                            }
                        }
                    }
                    
                    // Also check if the waste card's value already exists in the target stack
                    if stack_values.contains(&waste_card_data.value) {
                        has_duplicate_value = true;
                    }
                    
                    if is_top_card && is_valid_target && !has_duplicate_value && !is_already_in_stack {
                        let distance = (waste_transform.translation - tableau_transform.translation).length();
                        
                        if let Some((_target_pos, current_distance)) = best_target {
                            if distance < current_distance {
                                best_target = Some((tableau_transform.translation, distance));
                            }
                        } else {
                            best_target = Some((tableau_transform.translation, distance));
                        }
                    }
                }
            }
        
        // If no valid foundation pile found, check if it can be placed on empty tableau piles
        if best_target.is_none() {
            // Only Kings can be placed on empty tableau piles
            if waste_card_data.value == 13 {
                for tableau_pos in &tableau_positions.0 {
                    // CRITICAL: Prevent placing on waste or stock pile areas
                    if is_in_waste_or_stock_area(tableau_pos.truncate()) {
                        continue;
                    }
                    
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
                let is_foundation_placement = (target_pos.y - foundation_y).abs() < 5.0 
                    && (target_pos.x - foundation_start_x).abs() < 200.0; // Within foundation pile X range
                
                if is_foundation_placement {
                    // Check if this card can legally be placed on a foundation pile
                    let foundation_index = ((target_pos.x - foundation_start_x) / 100.0) as usize;
                    let foundation_pile = &foundation_piles.0[foundation_index];
                    
                    let can_place_on_foundation = if foundation_pile.is_empty() {
                        // Empty foundation pile - only Aces can be placed
                        waste_card_data.value == 1
                    } else {
                        // Non-empty foundation pile - check if this card can be added
                        if let Some((top_suit, top_value)) = foundation_pile.last() {
                            waste_card_data.suit == *top_suit && waste_card_data.value == top_value + 1
                        } else {
                            false
                        }
                    };
                    
                    if can_place_on_foundation {
                        // Placing on Foundation Pile
                        // Calculate foundation pile position
                        let foundation_start_x = -(6.0 * 100.0) / 2.0;
                        let foundation_x = foundation_start_x + (foundation_index as f32 * 100.0);
                        let foundation_y = WINDOW_HEIGHT / 2.0 - 100.0;
                        let new_position = Vec3::new(foundation_x, foundation_y, 1.0);
                        
                        // Update the FoundationPiles resource
                        foundation_piles.0[foundation_index].push((waste_card_data.suit, waste_card_data.value));
                        
                        // Move the waste card
                        commands.entity(waste_entity)
                            .remove::<WastePile>()
                            .remove::<SkippedWasteCard>()
                            .insert(FoundationPile)
                            .insert(OriginalPosition(new_position));
                        
                        // Update the transform to move to foundation pile
                        commands.entity(waste_entity).insert(Transform::from_translation(new_position));
                    }
                    // If can_place_on_foundation is false, the card will not be moved
                    // and the system will continue to check other placement options
                } else {
                    // Placing on Tableau - check if it's on an existing card or empty pile
                    let mut is_on_existing_card = false;
                    let mut highest_z = target_pos.z;
                    
                    // Check if there are existing cards at this position
                    for (_entity, card_transform, _card_data) in tableau_cards.iter() {
                        let x_same = (card_transform.translation.x - target_pos.x).abs() < 5.0;
                        let y_same = (card_transform.translation.y - target_pos.y).abs() < 5.0;
                        if x_same && y_same {
                            is_on_existing_card = true;
                            if card_transform.translation.z > highest_z {
                                highest_z = card_transform.translation.z;
                            }
                        }
                    }
                    
                    if is_on_existing_card {
                        // Placing on existing tableau card
                        let new_position = Vec3::new(target_pos.x, target_pos.y, highest_z + 1.0);
                        
                        // Move the waste card
                        commands.entity(waste_entity)
                            .remove::<WastePile>()
                            .remove::<SkippedWasteCard>()
                            .insert(TableauPile)
                            .insert(OriginalPosition(new_position))
                            .insert(Draggable);
                        
                        // Update the transform
                        commands.entity(waste_entity).insert(Transform::from_translation(new_position));
                        
                        // Check if there's a face-down card underneath that needs flipping
                        let target_pos_3d = Vec3::new(target_pos.x, target_pos.y, 0.0);
                        // Note: We can't use the helper function here due to different query types,
                        // but this is a simpler case with just position checking
                        for (_card_entity, card_transform, card_data) in tableau_cards.iter() {
                            if !card_data.is_face_up {
                                let card_x = card_transform.translation.x;
                                let card_y = card_transform.translation.y;
                                
                                if (card_x - target_pos.x).abs() < 5.0 && (card_y - target_pos.y).abs() < 5.0 {
                                    commands.entity(waste_entity).insert(NeedsFlipUnderneath(target_pos_3d));
                                    break;
                                }
                            }
                        }
                    } else {
                        // Placing on empty tableau pile
                        let new_position = Vec3::new(target_pos.x, target_pos.y, target_pos.z);
                        
                        // Move the waste card
                        commands.entity(waste_entity)
                            .remove::<WastePile>()
                            .remove::<SkippedWasteCard>()
                            .insert(TableauPile)
                            .insert(OriginalPosition(new_position))
                            .insert(Draggable);
                        
                        // Update the transform
                        commands.entity(waste_entity).insert(Transform::from_translation(new_position));
                        
                        // Check if there are face-down cards underneath that need flipping
                        let target_pos_3d = Vec3::new(target_pos.x, target_pos.y, 0.0);
                        // Note: We can't use the helper function here due to different query types,
                        // but this is a simpler case with just position checking
                        for (card_entity, card_transform, card_data) in tableau_cards.iter() {
                            if !card_data.is_face_up {
                                let card_x = card_transform.translation.x;
                                let card_y = card_transform.translation.y;
                                
                                if (card_x - target_pos.x).abs() < 5.0 && (card_y - target_pos.y).abs() < 5.0 {
                                    commands.entity(card_entity).insert(NeedsFlipUnderneath(target_pos_3d));
                                    break;
                                }
                            }
                        }
                    }
                }
                
                // Mark all other waste cards as skipped since the top one moved
                for (entity, _transform, _card_data) in waste_cards.iter() {
                    if entity != waste_entity {
                        commands.entity(entity).insert(SkippedWasteCard);
                    }
                }
            } // Close the if let Some((target_pos, _)) = best_target block
        }
    }
}
