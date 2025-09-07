use bevy::prelude::*;
use crate::components::*;
use crate::utils::get_card_back_image;


pub fn update_tableau_visual_stacking_system(
    mut tableau_cards: Query<(Entity, &mut Transform, &CardData), (With<TableauPile>, Or<(With<CardFront>, With<Draggable>)>, Without<CurrentlyDragging>)>,
    selected_card: Res<SelectedCard>,
    time: Res<Time>,
    mut last_update: Local<f64>,
) {
    // Only update visual stacking every 0.1 seconds to prevent excessive repositioning
    let current_time = time.elapsed_secs_f64();
    if current_time - *last_update < 0.1 {
        return;
    }
    *last_update = current_time;
    // Group cards by their X position to identify stacks (only X, not Y)
    let mut stacks: std::collections::HashMap<i32, Vec<(Entity, f32, f32)>> = std::collections::HashMap::new();
    
    // Collect all tableau cards that are either face-up or draggable
    // Skip cards that are currently being dragged
    for (entity, transform, _card_data) in tableau_cards.iter() {
        // Skip the currently selected card to avoid interfering with dragging
        if let Some(selected) = selected_card.0 {
            if entity == selected {
                continue;
            }
        }
        
        // Round to nearest 5 pixels to group cards that are "at the same X position"
        let x_key = (transform.translation.x / 5.0).round() as i32;
        let z_pos = transform.translation.z;
        let y_pos = transform.translation.y;
        
        stacks.entry(x_key).or_insert_with(Vec::new).push((entity, z_pos, y_pos));
    }
    
    // For each stack, sort by Z position and apply visual stacking
    for (_x_pos, mut cards) in stacks.iter_mut() {
        if cards.len() > 1 {
            // Sort by Z position (lowest Z = bottom of stack)
            cards.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            
            // Get the bottom card's Y position as the base for stacking
            let bottom_card = cards.first().unwrap();
            let base_y = if let Ok((_entity_id, transform, _card_data)) = tableau_cards.get(bottom_card.0) {
                transform.translation.y
            } else {
                continue; // Skip this stack if we can't get the bottom card
            };
            
            // Apply stacking offsets to all cards in the stack
            for (stack_index, (entity, _z_pos, _original_y)) in cards.iter().enumerate() {
                if let Ok((_entity_id, mut transform, _card_data)) = tableau_cards.get_mut(*entity) {
                    // Skip cards that are currently being dragged
                    if let Some(selected) = selected_card.0 {
                        if *entity == selected {
                            continue;
                        }
                    }
                    
                    // Apply stacking offset: each card above gets a 30-pixel Y offset
                    // This ensures each card shows enough of itself to remain clickable
                    let stacked_y = base_y - (stack_index as f32 * 30.0);
                    
                    // Only update if the position has changed significantly to avoid unnecessary updates
                    // Use a larger threshold to prevent cards from jumping around after placement
                    if (transform.translation.y - stacked_y).abs() > 15.0 {
                        transform.translation.y = stacked_y;
                    }
                }
            }
        }
    }
}
