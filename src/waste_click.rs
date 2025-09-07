use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButton;
use crate::components::*;
use crate::utils::can_place_on_card;
use tracing::debug;

pub fn waste_card_click_system(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window>,
    waste_cards: Query<(Entity, &Transform, &CardData), (With<WastePile>, Without<SkippedWasteCard>)>,
    tableau_cards: Query<(Entity, &Transform, &CardData), (With<TableauPile>, Without<WastePile>)>,
    tableau_positions: Res<TableauPositions>,
    mut foundation_piles: ResMut<FoundationPiles>,
    time: Res<Time>,
    mut last_click_time: Local<Option<f64>>,
    mut last_clicked_entity: Local<Option<Entity>>,
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
            // Check for double-click
            let current_time = time.elapsed_secs_f64();
            let is_double_click = if let Some(last_time) = *last_click_time {
                if let Some(last_entity) = *last_clicked_entity {
                    // Double-click if same entity and within 0.5 seconds
                    last_entity == waste_entity && (current_time - last_time) < 0.5
                } else {
                    false
                }
            } else {
                false
            };
            
            // Update click tracking
            *last_click_time = Some(current_time);
            *last_clicked_entity = Some(waste_entity);
            
            // Only process if this is a double-click
            if !is_double_click {
                return;
            }
            
            tracing::debug!("DOUBLE-CLICK DETECTED on waste card: {:?} (value: {}, suit: {:?})", 
                          waste_entity, waste_card_data.value, waste_card_data.suit);
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
                    
                    // Check if this is a valid placement (card value must be one lower AND colors must alternate)
                    if can_place_on_card(waste_card_data.value, tableau_card_data.value) {
                        // Additional check: colors must alternate (red on black, black on red)
                        let waste_is_red = matches!(waste_card_data.suit, CardSuit::Hearts | CardSuit::Diamonds);
                        let tableau_is_red = matches!(tableau_card_data.suit, CardSuit::Hearts | CardSuit::Diamonds);
                        
                        if waste_is_red != tableau_is_red { // Colors must be different
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
                            
                            if is_top_card {
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
                        // Update the FoundationPiles resource
                        foundation_piles.0[foundation_index].push((waste_card_data.suit, waste_card_data.value));
                        
                        // Move the waste card
                        commands.entity(waste_entity)
                            .remove::<WastePile>()
                            .remove::<SkippedWasteCard>()
                            .insert(FoundationPile)
                            .insert(OriginalPosition(target_pos));
                        
                        // Update the transform to move to foundation pile
                        commands.entity(waste_entity).insert(Transform::from_translation(target_pos));
                    }
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
                        
                    }
                }
                
                // Mark all other waste cards as skipped since the top one moved
                for (entity, _transform, _card_data) in waste_cards.iter() {
                    if entity != waste_entity {
                        commands.entity(entity).insert(SkippedWasteCard);
                    }
                }
                
                // Reset double-click tracking after successful move
                *last_click_time = None;
                *last_clicked_entity = None;
            }
        }
    }
}
