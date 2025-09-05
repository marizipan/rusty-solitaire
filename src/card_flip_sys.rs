use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButton;
use bevy::input::keyboard::KeyCode;
use crate::components::*;
use crate::utils::{get_card_back_image, get_card_front_image};



pub fn flip_cards_system(
    mut commands: Commands,
    needs_flip_query: Query<(Entity, &NeedsFlipUnderneath)>,
    all_transform_query: Query<&Transform, With<Card>>,
    all_card_data_query: Query<&CardData, With<Card>>,
    all_entity_query: Query<Entity, With<Card>>,
    asset_server: Res<AssetServer>,
    mut foundation_piles: ResMut<FoundationPiles>,
) {
    println!("FLIP SYSTEM DEBUG: Running flip system, found {} flip requests", needs_flip_query.iter().count());
    for (entity, needs_flip) in needs_flip_query.iter() {
        let original_position = needs_flip.0;
        
        // Remove the entity immediately to prevent duplicate processing
        commands.entity(entity).despawn();
        

        

        
        println!("FLIP DEBUG: Processing flip request for entity at position ({:.1}, {:.1}, {:.1})", original_position.x, original_position.y, original_position.z);
        
        // Find face-down cards at the original position that need to be flipped
        let mut cards_at_position = Vec::new();
        
        println!("FLIP DEBUG: Looking for face-down cards at original position ({:.1}, {:.1}, {:.1})", original_position.x, original_position.y, original_position.z);
        println!("FLIP DEBUG: Will check for cards within x_distance < 15.0 and y_distance < 50.0 and Z < original Z");
        
        // Collect all cards at the original X,Y position with more precise detection
        for card_entity in all_entity_query.iter() { 
            if let Ok(transform) = all_transform_query.get(card_entity) {
                if let Ok(card_data) = all_card_data_query.get(card_entity) {
                    // Skip cards that have already been flipped
                    if card_data.is_face_up {
                        continue;
                    }
                    
                    let x_distance = (transform.translation.x - original_position.x).abs();
                    let y_distance = (transform.translation.y - original_position.y).abs();
                    
                    println!("FLIP DEBUG: Checking card at ({:.1}, {:.1}, {:.1}) - x_dist: {:.1}, y_dist: {:.1}", 
                        transform.translation.x, transform.translation.y, transform.translation.z, x_distance, y_distance);
                    
                    // Position check for tableau stacking - cards are stacked with 30px Y offsets
                    // Use more reasonable tolerances to find cards in the same stack
                    // We want to find face-down cards at the same X,Y position
                    // The Z comparison doesn't matter here since we're just looking for cards at the same position
                    if x_distance < 25.0 && y_distance < 80.0 {
                        println!("FLIP DEBUG: Found face-down card at position, adding to flip list");
                        cards_at_position.push((card_entity, transform.translation.z, card_data));
                    }
                }
            }
        }
        
        // Sort by Z position to find the card that was directly underneath
        cards_at_position.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        println!("FLIP DEBUG: Found {} cards at position, {} are face-down", 
            cards_at_position.len(), 
            cards_at_position.iter().filter(|(entity, _z_pos, card_data)| !card_data.is_face_up).count());
        
        // Find and flip ONLY the topmost face-down card (highest Z position = closest to camera)
        // Only flip ONE card per movement to prevent multiple flips
        if let Some((card_entity, _z_pos, card_data)) = cards_at_position.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()) {
            println!("FLIP DEBUG: Flipping face-down card {} of {} at position ({:.1}, {:.1})", 
                card_data.value, format!("{:?}", card_data.suit), original_position.x, original_position.y);
            
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
            
            // Note: We're not using AlreadyFlipped component to avoid complexity
            // The system processes each flip request only once per frame
            
            println!("FLIP DEBUG: Successfully flipped card {} of {}", card_data.value, format!("{:?}", card_data.suit));
        } else {
            println!("FLIP DEBUG: No face-down cards found to flip at position ({:.1}, {:.1})", 
                original_position.x, original_position.y);
        }
    }
}

