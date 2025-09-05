use bevy::prelude::*;
use crate::components::*;
use crate::utils::{get_card_back_image, get_card_front_image};



pub fn flip_cards_system(
    mut commands: Commands,
    needs_flip_query: Query<(Entity, &NeedsFlipUnderneath)>,
    all_transform_query: Query<&Transform, With<Card>>,
    all_card_data_query: Query<&CardData, With<Card>>,
    all_entity_query: Query<Entity, With<Card>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, needs_flip) in needs_flip_query.iter() {
        let original_position = needs_flip.0;
        
        // Remove the entity immediately to prevent duplicate processing
        commands.entity(entity).despawn();
        
        // Find face-down cards at the original position that need to be flipped
        let mut cards_at_position = Vec::new();
        
        // Collect all cards at the original X,Y position with precise detection
        for card_entity in all_entity_query.iter() { 
            if let Ok(transform) = all_transform_query.get(card_entity) {
                if let Ok(card_data) = all_card_data_query.get(card_entity) {
                    // Skip cards that have already been flipped
                    if card_data.is_face_up {
                        continue;
                    }
                    
                    let x_distance = (transform.translation.x - original_position.x).abs();
                    let y_distance = (transform.translation.y - original_position.y).abs();
                    
                    // Use precise position matching - cards should be at the exact same X,Y position
                    // Only allow small tolerance for floating point precision
                    if x_distance < 5.0 && y_distance < 5.0 {
                        cards_at_position.push((card_entity, transform.translation.z, card_data));
                    }
                }
            }
        }
        
        // Sort by Z position to find the card that was directly underneath
        cards_at_position.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        // Find and flip ONLY the topmost face-down card (highest Z position = closest to camera)
        // Only flip ONE card per movement to prevent multiple flips
        if let Some((card_entity, _z_pos, card_data)) = cards_at_position.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()) {
            
            // Update the card to be face-up
            commands.entity(*card_entity).insert(CardData {
                suit: card_data.suit,
                value: card_data.value,
                is_face_up: true,
            });
            
            // Add the Draggable component so it can be moved
            commands.entity(*card_entity).insert(Draggable);
            
            // Change the sprite from CardBack to CardFront
            let front_image_path = get_card_front_image(card_data.suit, card_data.value);
           
            // Remove the old sprite and add the new one
            commands.entity(*card_entity).remove::<Sprite>();
            commands.entity(*card_entity).insert(Sprite {
                image: asset_server.load(front_image_path),
                custom_size: Some(Vec2::new(80.0, 120.0)),
                ..default()
            });
        
            // Remove the CardBack component and add CardFront
            commands.entity(*card_entity).insert(CardFront);
        }
    }
}

