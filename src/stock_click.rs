use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButton;
use bevy::input::keyboard::KeyCode;
use crate::components::*;
use crate::utils::get_card_back_image;
use crate::card_entity::create_card_entity;
use tracing::debug;

pub fn stock_click_system(
    mouse_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window>,
    mut stock_cards: ResMut<StockCards>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    waste_cards: Query<(Entity, &Transform, &CardData, Option<&SkippedWasteCard>), With<WastePile>>,
    _stock_entities: Query<Entity, (With<StockPile>, With<Card>)>,
) {
    let Ok(window) = window_query.single() else { return };
    
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = window.cursor_position() {
            let cursor_world_pos = Vec2::new(
                cursor_pos.x - window.width() / 2.0,
                window.height() / 2.0 - cursor_pos.y,
            );

            // Check if stock pile was clicked
            let stock_x = -(6.0 * 100.0) / 2.0 + (6.0 * 100.0); // Above Stack 7 (x = 300)
            let stock_y = WINDOW_HEIGHT / 2.0 - 100.0; // Above the tableau stacks
            let stock_bounds = Vec2::new(40.0, 60.0);
            
            if (cursor_world_pos - Vec2::new(stock_x, stock_y)).abs().cmplt(stock_bounds).all() {
                // If stock has cards, deal the top card to waste pile
                if !stock_cards.0.is_empty() {
                    // Get and remove the top card from stock
                    if let Some((suit, value)) = stock_cards.0.pop() {
                        debug!("Dealing card from stock - value: {}, suit: {:?}", value, suit);

                        
                        // Create the waste card at the waste pile position
                        let waste_x = -(6.0 * 100.0) / 2.0 + (5.0 * 100.0); // Above Stack 6 (x = 200)
                        let waste_y = WINDOW_HEIGHT / 2.0 - 100.0;
                        
                        // Find highest Z in waste pile for stacking
                        let mut highest_z = 0.0;
                        for (_entity, waste_transform, _card_data, _skipped) in waste_cards.iter() {
                            if waste_transform.translation.z > highest_z {
                                highest_z = waste_transform.translation.z;
                            }
                        }
                        
                        // Create waste card entity
                        create_card_entity(
                            &mut commands,
                            &asset_server,
                            Vec3::new(waste_x, waste_y, highest_z + 1.0),
                            suit,
                            value,
                            true, // Face up in waste pile
                            (
                                WastePile,
                                CardFront,
                                Draggable, // Make it draggable

                            ),
                        );
                    }
                } else {
                    // Stock is empty - recycle waste cards back to stock
                    debug!("Stock is empty, recycling waste cards back to stock");
                    
                    // Safety check: only recycle if there are actually waste cards
                    if waste_cards.is_empty() {
                        debug!("No waste cards to recycle, skipping");
                        return;
                    }
                    
                    // Collect waste card data in the order they were dealt (oldest first)
                    let mut waste_cards_info: Vec<(Entity, CardSuit, u8, f32)> = waste_cards
                        .iter()
                        .map(|(entity, transform, card_data, _)| (entity, card_data.suit, card_data.value, transform.translation.z))
                        .collect();
                    
                    debug!("Found {} waste cards to recycle", waste_cards_info.len());
                    
                    // Sort by Z position to ensure correct order (lowest Z = oldest = dealt first)
                    waste_cards_info.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap());
                    
                    // Put all waste cards back into stock (oldest first, so they'll be dealt last)
                    let waste_card_data: Vec<(CardSuit, u8)> = waste_cards_info
                        .iter()
                        .map(|(_entity, suit, value, _z_pos)| (*suit, *value))
                        .collect();
                    
                    debug!("Recycling {} cards back to stock", waste_card_data.len());
                    stock_cards.0 = waste_card_data;
                    
                    // Now despawn all the waste entities
                    for (entity, _suit, _value, _z_pos) in &waste_cards_info {
                        commands.entity(*entity).despawn();
                    }
                }
            }
        }
    }
}
