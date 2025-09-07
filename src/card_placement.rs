use bevy::prelude::*;
use crate::components::*;
use tracing::debug;

/// Places a card at the target position
pub fn place_card(
    commands: &mut Commands,
    foundation_piles: &mut FoundationPiles,
    selected_entity: Entity,
    target_pos: Vec3,
    original_position: Vec3,
    card_data_query: &Query<&CardData>,
) {
    let Ok(card_data) = card_data_query.get(selected_entity) else { return; };
    
    // Determine if this is a foundation or tableau placement
    let foundation_y = WINDOW_HEIGHT / 2.0 - 100.0; // This is 260
    // Foundation piles are at y=260, but we need to distinguish between foundation and waste pile
    // Foundation piles are at x positions: -150, -50, 50, 150
    // Waste pile is at x=0, y=260
    let foundation_x_range = target_pos.x >= -200.0 && target_pos.x <= 200.0;
    let foundation_y_range = target_pos.y >= foundation_y - 50.0 && target_pos.y <= foundation_y + 50.0;
    let is_waste_pile = target_pos.x >= -50.0 && target_pos.x <= 50.0 && target_pos.y >= foundation_y - 50.0 && target_pos.y <= foundation_y + 50.0;
    let is_foundation = foundation_x_range && foundation_y_range && !is_waste_pile;
    
    if is_foundation {
        place_on_foundation(commands, foundation_piles, selected_entity, target_pos, card_data, original_position);
    } else {
        place_on_tableau(commands, selected_entity, target_pos, original_position);
    }
}

/// Places a card on a foundation pile
pub fn place_on_foundation(
    commands: &mut Commands,
    foundation_piles: &mut FoundationPiles,
    selected_entity: Entity,
    target_pos: Vec3,
    card_data: &CardData,
    original_position: Vec3,
) {
    // Foundation piles are at x positions: -150, -50, 50, 150
    // Map x position to foundation index (0-3)
    let foundation_index = if target_pos.x < -100.0 {
        0  // -150
    } else if target_pos.x < 0.0 {
        1  // -50
    } else if target_pos.x < 100.0 {
        2  // 50
    } else {
        3  // 150
    };
    
    // Bounds check to prevent index out of bounds
    if foundation_index >= 4 {
        debug!("Foundation index out of bounds: {} (target_pos.x: {})", foundation_index, target_pos.x);
        return;
    }
    
    // Validate foundation placement using existing logic
    if !crate::utils::can_place_on_foundation(card_data, &foundation_piles.0[foundation_index]) {
        debug!("FOUNDATION REJECTED: Card {:?} (value: {}, suit: {:?}) cannot be placed on foundation pile {} (empty: {}, top: {:?})", 
               card_data.suit, card_data.value, card_data.suit, foundation_index, 
               foundation_piles.0[foundation_index].is_empty(), 
               foundation_piles.0[foundation_index].last());
        return;
    }
    
    // Update foundation pile
    foundation_piles.0[foundation_index].push((card_data.suit, card_data.value));
    
    // Position the card
    let new_position = Vec3::new(
        target_pos.x,
        target_pos.y,
        foundation_piles.0[foundation_index].len() as f32 + 1.0  // Use proper Z positioning
    );
    
    commands.entity(selected_entity).insert(Transform::from_translation(new_position));
    commands.entity(selected_entity).insert(OriginalPosition(new_position));
    
    // Update components for foundation
    commands.entity(selected_entity).insert(FoundationPile);
    commands.entity(selected_entity).remove::<TableauPile>();
    commands.entity(selected_entity).remove::<WastePile>();
    
    // Trigger card flipping for face-down cards underneath
    debug!("Spawning flip trigger at position: {:?}", original_position);
    commands.spawn(NeedsFlipUnderneath(original_position));
}

/// Places a card on a tableau
pub fn place_on_tableau(
    commands: &mut Commands,
    selected_entity: Entity,
    target_pos: Vec3,
    original_position: Vec3,
) {
    // Position the card
    let new_position = Vec3::new(target_pos.x, target_pos.y, target_pos.z + 0.1);
    
    commands.entity(selected_entity).insert(Transform::from_translation(new_position));
    commands.entity(selected_entity).insert(OriginalPosition(new_position));
    
    // Update components for tableau
    commands.entity(selected_entity).insert(TableauPile);
    commands.entity(selected_entity).remove::<FoundationPile>();
    commands.entity(selected_entity).remove::<WastePile>();
    
    // Trigger card flipping for face-down cards underneath
    debug!("Spawning flip trigger at position: {:?}", original_position);
    commands.spawn(NeedsFlipUnderneath(original_position));
}

/// Snaps a card back to its original position
pub fn snap_back_card(
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
