use bevy::prelude::*;
use bevy::ecs::query::Or;
use crate::components::*;
use crate::utils::{get_card_front_image, get_card_back_image, get_card_data_from_filename};

// Helper function to create a card entity with sprite
pub fn create_card_entity(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec3,
    suit: CardSuit,
    value: u8,
    is_face_up: bool,
    components: impl Bundle,
) -> Entity {
    let sprite_image = if is_face_up {
        get_card_front_image(suit, value)
    } else {
        get_card_back_image(suit).to_string()
    };
    
    // Use the reliable filename mapping to ensure consistency
    // This ensures the card data matches exactly what the image shows
    let (card_suit, card_value) = if is_face_up {
        if let Some((s, v)) = get_card_data_from_filename(&sprite_image) {
            (s, v)
        } else {
            // Fallback to passed parameters if filename parsing fails
            (suit, value)
        }
    } else {
        // For face-down cards, use passed parameters (they'll be face-up later)
        (suit, value)
    };
    
    let entity = commands.spawn((
        Sprite {
            image: asset_server.load(sprite_image),
            custom_size: Some(Vec2::new(80.0, 120.0)),
            ..default()
        },
        Transform::from_translation(position),
        Card,
        CardData {
            suit: card_suit,
            value: card_value,
            is_face_up,
        },
        components,
    )).id();

    entity
}

// Helper function removed - was not being used after refactoring









