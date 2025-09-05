use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButton;
use bevy::input::keyboard::KeyCode;
use crate::components::*;
use crate::utils::get_card_back_image;


pub fn update_tableau_visual_stacking_system(
    mut tableau_cards: Query<(Entity, &mut Transform, &CardData), (With<TableauPile>, Or<(With<CardFront>, With<Draggable>)>)>,
) {
    // Group cards by their X,Y position to identify stacks
    let mut stacks: std::collections::HashMap<(i32, i32), Vec<(Entity, f32, usize)>> = std::collections::HashMap::new();
    
    // Collect all tableau cards that are either face-up or draggable
                for (entity, transform, _card_data) in tableau_cards.iter() {
        // Round to nearest 5 pixels to group cards that are "at the same position"
        let x_key = (transform.translation.x / 5.0).round() as i32;
        let y_key = (transform.translation.y / 5.0).round() as i32;
        let z_pos = transform.translation.z;
        
        stacks.entry((x_key, y_key)).or_insert_with(Vec::new).push((entity, z_pos, 0));
    }
    
    // Visual stacking system processing stacks
    
    // For each stack, sort by Z position and apply visual stacking
            for (_pos, mut cards) in stacks.iter_mut() {
        if cards.len() > 1 {
            // Sort by Z position (lowest Z = bottom of stack)
            cards.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            
            // Processing stack
            
            // Get the bottom card's Y position as the base for stacking
            let bottom_card = cards.first().unwrap();
            let base_y = if let Ok((_entity_id, transform, _card_data)) = tableau_cards.get(bottom_card.0) {
                transform.translation.y
            } else {
                continue; // Skip this stack if we can't get the bottom card
            };
            
            // Apply stacking offsets to all cards in the stack
            for (stack_index, (entity, _z_pos, _card_data)) in cards.iter().enumerate() {
                if let Ok((_entity_id, mut transform, _card_data)) = tableau_cards.get_mut(*entity) {
                    // Apply stacking offset: each card above gets a 30-pixel Y offset
                    // This ensures each card shows enough of itself to remain clickable
                    let stacked_y = base_y - (stack_index as f32 * 30.0);
                    
                    // Update the transform to show proper visual stacking
                    transform.translation.y = stacked_y;
                }
            }
        }
    }
}
