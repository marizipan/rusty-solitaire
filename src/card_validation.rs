use bevy::prelude::*;
use crate::components::*;
use crate::utils::{can_place_on_foundation, can_place_on_tableau_card, find_best_tableau_target};
use tracing::debug;

/// Finds a valid drop target for the card with proper solitaire rules
pub fn find_valid_drop_target(
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
pub fn find_foundation_target(cursor_pos: Vec2, foundation_piles: &FoundationPiles, card_data: &CardData) -> Option<Vec3> {
    let foundation_y = WINDOW_HEIGHT / 2.0 - 100.0; // This is 260
    
    // Foundation piles are at x positions: -150, -50, 50, 150
    // Waste pile is at x=0, y=260
    // Only consider foundation area if NOT in waste pile area
    let waste_pile_x_range = cursor_pos.x >= -50.0 && cursor_pos.x <= 50.0;
    let foundation_y_range = cursor_pos.y >= foundation_y - 50.0 && cursor_pos.y <= foundation_y + 50.0;
    
    // Exclude waste pile area from foundation detection
    if waste_pile_x_range && foundation_y_range {
        return None;
    }
    
    // Only check foundation if cursor is in foundation area (not tableau area)
    if !foundation_y_range {
        return None;
    }

    for i in 0..4 {
        let foundation_x = -150.0 + (i as f32 * 100.0);
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
pub fn find_tableau_target(
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
            
            // Exclude waste pile area from tableau detection
            // Waste pile is at x=0, y=260, foundation piles are at x=-150, -50, 50, 150, y=260
            let waste_pile_x_range = target_pos.x >= -50.0 && target_pos.x <= 50.0;
            let foundation_y = WINDOW_HEIGHT / 2.0 - 100.0; // This is 260
            let foundation_y_range = target_pos.y >= foundation_y - 50.0 && target_pos.y <= foundation_y + 50.0;
            
            // Skip waste pile cards (they should only be targets for foundation moves, not tableau moves)
            if waste_pile_x_range && foundation_y_range {
                debug!("Skipping waste pile card at {:?}", target_pos);
                continue;
            }
            
            // Consider all other cards for tableau moves
            if distance < 80.0 && distance < best_distance {
                // Check if this card can be placed on the target card
                if let Ok(target_card_data) = card_data_query.get(entity) {
                    debug!("Target card: suit={:?}, value={}, face_up: {}", target_card_data.suit, target_card_data.value, target_card_data.is_face_up);
                    if can_place_on_tableau_card(selected_card_data, target_card_data) {
                        debug!("DRAG VALID: Card {:?} (value: {}, suit: {:?}) can be placed on {:?} (value: {}, suit: {:?})", 
                               selected_card_data.suit, selected_card_data.value, selected_card_data.suit,
                               target_card_data.suit, target_card_data.value, target_card_data.suit);
                        best_target = Some(target_pos);
                        best_distance = distance;
                    } else {
                        debug!("DRAG INVALID: Card {:?} (value: {}, suit: {:?}) cannot be placed on {:?} (value: {}, suit: {:?})", 
                               selected_card_data.suit, selected_card_data.value, selected_card_data.suit,
                               target_card_data.suit, target_card_data.value, target_card_data.suit);
                    }
                }
            }
        }
    }

    best_target
}

/// Finds empty tableau targets (only for Kings)
pub fn find_empty_tableau_target(cursor_pos: Vec2) -> Option<Vec3> {
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

/// Checks if a card can be dragged (is top card and can lead stack)
pub fn can_drag_card(
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
        
        return crate::utils::is_valid_stack_sequence(&all_cards);
    }
    
    false
}

/// Finds the card under the cursor
pub fn find_card_under_cursor(
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
