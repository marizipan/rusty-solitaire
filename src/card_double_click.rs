use bevy::prelude::*;
use crate::components::*;
use crate::utils::{can_place_on_foundation, find_best_tableau_target};
use tracing::debug;

/// Simple foundation move function that reuses existing validation logic
pub fn try_foundation_move_simple(
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
        
        // Store original position before moving
        let original_position = if let Ok(transform) = transform_query.get(entity) {
            transform.translation
        } else {
            return false;
        };
        
        // Calculate foundation pile position
        let foundation_start_x = -(6.0 * 100.0) / 2.0;
        let foundation_x = foundation_start_x + (foundation_index as f32 * 100.0);
        let foundation_y = WINDOW_HEIGHT / 2.0 - 100.0;
        
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
        
        // Trigger card flipping for face-down cards underneath (use original position)
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
pub fn try_tableau_move_simple(
    entity: Entity,
    transform_query: &mut Query<&mut Transform, (With<Card>, With<Draggable>)>,
    card_data: &CardData,
    tableau_cards: &[(Entity, Vec3, CardData)],
    tableau_positions: &[Vec3],
    commands: &mut Commands,
) -> bool {
    // Store original position before moving
    let original_position = if let Ok(transform) = transform_query.get(entity) {
        transform.translation
    } else {
        return false;
    };
    
    // Use existing validation logic from utils.rs
    if let Some(target_pos) = find_best_tableau_target(card_data, original_position, tableau_cards, tableau_positions, Some(entity)) {
        debug!("TABLEAU PLACEMENT: Card {:?} (value: {}, suit: {:?}) can be placed on tableau at {:?}", 
               card_data.suit, card_data.value, card_data.suit, target_pos);
        
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
        
        // Trigger card flipping for face-down cards underneath (use original position)
        commands.spawn(NeedsFlipUnderneath(original_position));
        
        true
    } else {
        debug!("TABLEAU REJECTED: Card {:?} (value: {}, suit: {:?}) cannot be placed on tableau", 
               card_data.suit, card_data.value, card_data.suit);
        false
    }
}
