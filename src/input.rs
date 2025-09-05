use bevy::prelude::*;
use crate::components::*;
use crate::undo::create_undo_action;
use crate::utils::{get_card_front_image, get_card_front_handle};

pub fn stock_click_system(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window>,
    mut stock_cards: ResMut<StockCards>,
    waste_cards: Query<(Entity, &Transform, &CardData, Option<&SkippedWasteCard>), With<WastePile>>,
    _stock_entities: Query<Entity, (With<StockPile>, With<Card>)>,
    asset_server: Res<AssetServer>,
) {
    let Ok(window) = window_query.single() else { return };
    
    if mouse_input.just_pressed(MouseButton::Left) {
        let Some(cursor_pos) = window.cursor_position() else { return };
        
        let cursor_world_pos = Vec2::new(
            cursor_pos.x - window.width() / 2.0,
            window.height() / 2.0 - cursor_pos.y,
        );
        
        // Check if clicking on stock pile (left side of screen)
        if cursor_world_pos.x < -200.0 && cursor_world_pos.y > -100.0 && cursor_world_pos.y < 100.0 {
            if !stock_cards.0.is_empty() {
                // Take the top card from stock
                let card_data = stock_cards.0.pop().unwrap();
                
                // Find the highest Z position among waste cards
                let mut highest_z = 0.0;
                for (_entity, waste_transform, _card_data, _skipped) in waste_cards.iter() {
                    if waste_transform.translation.z > highest_z {
                        highest_z = waste_transform.translation.z;
                    }
                }
                
                // Create the waste card at the waste pile position
                let waste_x = -(6.0 * 100.0) / 2.0 + (5.0 * 100.0); // Above Stack 6 (x = 200)
                let waste_y = WINDOW_HEIGHT / 2.0 - 100.0; // Aligned with Stock Pile and Foundation Piles
                
                commands.spawn((
                    Sprite {
                        image: get_card_front_handle(card_data.0, card_data.1, &asset_server),
                        custom_size: Some(Vec2::new(80.0, 120.0)),
                        ..default()
                    },
                    Transform::from_xyz(waste_x, waste_y, highest_z + 1.0),
                    Card,
                    CardData {
                        suit: card_data.0,
                        value: card_data.1,
                        is_face_up: true, // Face up in waste pile
                    },
                    WastePile,
                    CardFront,
                    Draggable, // Make it draggable
                ));
            }
        }
    }
}

// REMOVED: Duplicate card_drag_system - using the one from card_drag_sys.rs instead

// REMOVED: Duplicate card_drop_system - using the one from card_drop_sys.rs instead

// Duplicate double-click system removed - using the one from foundation_click.rs instead
