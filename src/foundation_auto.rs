use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButton;
use bevy::input::keyboard::KeyCode;
use crate::components::*;
use crate::utils::get_card_back_image;


pub fn auto_move_to_foundation_system(
    mut commands: Commands,
    mut foundation_piles: ResMut<FoundationPiles>,
    tableau_cards: Query<(Entity, &Transform, &CardData), (With<TableauPile>, With<CardFront>, Without<StockPile>)>,
    waste_cards: Query<(Entity, &Transform, &CardData), (With<WastePile>, With<CardFront>, Without<StockPile>)>,
) {
    // TEMPORARILY DISABLED: Auto-move system is causing cards to move immediately
    // This should be re-enabled with proper delay logic later
    return;
    
    // Check tableau cards for auto-move to foundation (excluding stock pile cards)
    for (entity, _transform, card_data) in tableau_cards.iter() {
        if card_data.is_face_up {
            let mut should_auto_move = false;
            let mut target_foundation_index = None;
            
            // Check all foundation piles for potential auto-move
            for (i, foundation_pile) in foundation_piles.0.iter().enumerate() {
                if foundation_pile.is_empty() {
                    // Empty foundation pile - only Aces can be placed
                    if card_data.value == 1 {
                        should_auto_move = true;
                        target_foundation_index = Some(i);
                        break;
                    }
                } else {
                    // Non-empty foundation pile - check if this card can be added
                    if let Some((top_suit, top_value)) = foundation_pile.last() {
                        if card_data.suit == *top_suit && card_data.value == top_value + 1 {
                            should_auto_move = true;
                            target_foundation_index = Some(i);
                            break;
                        }
                    }
                }
            }
            
            if should_auto_move {
                if let Some(foundation_index) = target_foundation_index {
                    // Calculate foundation pile position
                    let foundation_start_x = -(6.0 * 100.0) / 2.0;
                    let foundation_x = foundation_start_x + (foundation_index as f32 * 100.0);
                    let foundation_y = WINDOW_HEIGHT / 2.0 - 100.0;
                    
                    // Since we already validated the move above, we can proceed directly
                    // Update the FoundationPiles resource
                    foundation_piles.0[foundation_index].push((card_data.suit, card_data.value));
                    
                    // Move the card to the foundation pile
                    commands.entity(entity).insert(Transform::from_translation(
                        Vec3::new(foundation_x, foundation_y, 1.0)
                    ));
                    
                    // Remove tableau/waste components and add foundation component
                    commands.entity(entity)
                        .remove::<TableauPile>()
                        .remove::<WastePile>()
                        .remove::<SkippedWasteCard>()
                        .remove::<StockPile>()
                        .insert(FoundationPile)
                        .insert(OriginalPosition(Vec3::new(foundation_x, foundation_y, 1.0)));
                    
                    // Auto-move completed
                }
            }
        }
    }
    
    // Check waste cards for auto-move to foundation (excluding stock pile cards)
    for (entity, _transform, card_data) in waste_cards.iter() {
        if card_data.is_face_up {
            let mut should_auto_move = false;
            let mut target_foundation_index = None;
            
            // Check all foundation piles for potential auto-move
            for (i, foundation_pile) in foundation_piles.0.iter().enumerate() {
                if foundation_pile.is_empty() {
                    // Empty foundation pile - only Aces can be placed
                    if card_data.value == 1 {
                        should_auto_move = true;
                        target_foundation_index = Some(i);
                        break;
                    }
                } else {
                    // Non-empty foundation pile - check if this card can be added
                    if let Some((top_suit, top_value)) = foundation_pile.last() {
                        if card_data.suit == *top_suit && card_data.value == top_value + 1 {
                            should_auto_move = true;
                            target_foundation_index = Some(i);
                            break;
                        }
                    }
                }
            }
            
            if should_auto_move {
                if let Some(foundation_index) = target_foundation_index {
                    // Calculate foundation pile position
                    let foundation_start_x = -(6.0 * 100.0) / 2.0;
                    let foundation_x = foundation_start_x + (foundation_index as f32 * 100.0);
                    let foundation_y = WINDOW_HEIGHT / 2.0 - 100.0;
                    
                    // Move the card to the foundation pile
                    commands.entity(entity).insert(Transform::from_translation(
                        Vec3::new(foundation_x, foundation_y, 1.0)
                    ));
                    
                    // Since we already validated the move above, we can proceed directly
                    // Update the FoundationPiles resource
                    foundation_piles.0[foundation_index].push((card_data.suit, card_data.value));
                    
                    // Remove tableau/waste components and add foundation component
                    commands.entity(entity)
                        .remove::<TableauPile>()
                        .remove::<WastePile>()
                        .remove::<SkippedWasteCard>()
                        .remove::<StockPile>()
                        .insert(FoundationPile)
                        .insert(OriginalPosition(Vec3::new(foundation_x, foundation_y, 1.0)));
                    
                    // Auto-move completed
                }
            }
        }
    }
}

// validate_tableau_stacks_system removed - was causing conflicts and not needed

