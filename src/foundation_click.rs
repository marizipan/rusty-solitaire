use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButton;
use bevy::input::keyboard::KeyCode;
use crate::components::*;
use crate::utils::get_card_back_image;
use tracing::{debug, info, warn};

fn try_move_to_foundation(
    entity: Entity,
    transform: &mut Transform,
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
    
    // Check if this card can be placed on the foundation
    let can_place = if foundation_pile.is_empty() {
        // Empty foundation pile - only Aces can be placed
        card_data.value == 1
    } else {
        // Non-empty foundation pile - check if this is the next card in sequence
        let (top_suit, top_value) = foundation_pile.last().unwrap();
        card_data.suit == *top_suit && card_data.value == top_value + 1
    };
    
    if can_place {
        // Calculate foundation pile position
        let foundation_start_x = -(6.0 * 100.0) / 2.0;
        let foundation_x = foundation_start_x + (foundation_index as f32 * 100.0);
        let foundation_y = 260.0; // WINDOW_HEIGHT / 2.0 - 100.0
        
        // Move the card to the foundation pile
        let foundation_pos = Vec3::new(foundation_x, foundation_y, foundation_pile.len() as f32 + 1.0);
        transform.translation = foundation_pos;
        
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
        
        true
    } else {
        false
    }
}


pub fn double_click_foundation_system(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut foundation_piles: ResMut<FoundationPiles>,
    mut draggable_cards: Query<(Entity, &mut Transform, &CardData), (With<Draggable>, Or<(With<TableauPile>, With<WastePile>)>, Without<SkippedWasteCard>)>,
    mut last_click_time: Local<Option<std::time::Instant>>,
    clicked_entity: Res<ClickedEntity>,
) {
    // Handle double-click detection and auto-move to foundation
    if mouse_input.just_pressed(MouseButton::Left) {
        let now = std::time::Instant::now();
        
        // Check if this is a double-click on the same entity
        if let Some(last_time) = *last_click_time {
            if let Some(last_entity) = clicked_entity.0 {
                let time_diff = now.duration_since(last_time);
                
                // If double-click detected (within 500ms) and same entity
                if time_diff.as_millis() < 500 {
                    // Check all draggable cards (includes both tableau and waste cards)
                    for (entity, mut transform, card_data) in draggable_cards.iter_mut() {
                        if entity == last_entity && card_data.is_face_up {
                            // First try foundation placement
                            if try_move_to_foundation(entity, &mut transform, card_data, &mut foundation_piles, &mut commands) {
                                // Reset double-click tracking
                                *last_click_time = None;
                                return;
                            }
                            
                            // If foundation placement failed, try tableau placement
                            // For now, skip tableau placement in double-click to avoid query conflicts
                            // Tableau placement can be done via drag and drop
                            debug!("Foundation placement failed, skipping tableau placement for double-click");
                            
                            break; // Found the entity, no need to continue iterating
                        }
                    }
                }
            }
        }
        
        // Update double-click tracking
        *last_click_time = Some(now);
        // clicked_entity.0 will be set by the drag system when a card is actually clicked
    }
}

// validate_card_draggability_system removed - was causing conflicts and not needed

// cleanup_flip_markers_system removed - AlreadyFlipped component was removed