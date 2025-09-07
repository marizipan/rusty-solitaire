use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButton;
use crate::components::*;
use crate::utils::{can_place_on_tableau_card, find_best_tableau_target};

/// Helper function to move a card to foundation with proper validation
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
        tracing::debug!("FOUNDATION PLACEMENT: Card {:?} (value: {}, suit: {:?}) can be placed on foundation pile {}", 
                       card_data.suit, card_data.value, card_data.suit, foundation_index);
        
        // Calculate foundation pile position
        let foundation_start_x = -(6.0 * 100.0) / 2.0;
        let foundation_x = foundation_start_x + (foundation_index as f32 * 100.0);
        let foundation_y = WINDOW_HEIGHT / 2.0 - 100.0;
        
        // Store original position for flip trigger
        let original_position = transform.translation;
        
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
        
        // Trigger card flipping for face-down cards underneath
        commands.spawn(NeedsFlipUnderneath(original_position));
        
        true
    } else {
        tracing::debug!("FOUNDATION REJECTED: Card {:?} (value: {}, suit: {:?}) cannot be placed on foundation pile {} (empty: {}, top: {:?})", 
                       card_data.suit, card_data.value, card_data.suit, foundation_index, foundation_pile.is_empty(), 
                       foundation_pile.last());
        false
    }
}

/// Helper function to move a card to tableau with proper validation
/// For now, tableau moves are handled by the drag-and-drop system
fn try_move_to_tableau(
    _entity: Entity,
    _transform: &mut Transform,
    _card_data: &CardData,
    _commands: &mut Commands,
    _tableau_cards: &Query<(Entity, &Transform, &CardData), (With<TableauPile>, Without<WastePile>)>,
    _tableau_positions: &Res<TableauPositions>,
) -> bool {
    // Tableau moves are handled by the drag-and-drop system
    // Double-click should primarily handle foundation moves
    tracing::debug!("DOUBLE-CLICK: Tableau moves handled by drag-and-drop system");
    false
}

/// Double-click system for moving cards to foundation piles
pub fn double_click_foundation_system(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut foundation_piles: ResMut<FoundationPiles>,
    mut draggable_cards: Query<(Entity, &mut Transform, &CardData), (With<Draggable>, Or<(With<TableauPile>, With<WastePile>)>, Without<SkippedWasteCard>)>,
    mut last_click_time: Local<Option<std::time::Instant>>,
    clicked_entity: Res<ClickedEntity>,
) {
    // Handle double-click detection and move to foundation
    if mouse_input.just_pressed(MouseButton::Left) {
        let now = std::time::Instant::now();
        
        // Check if this is a double-click on the same entity
        if let Some(last_time) = *last_click_time {
            if let Some(last_entity) = clicked_entity.0 {
                let time_diff = now.duration_since(last_time);
                
                // If double-click detected (within 500ms) and same entity
                if time_diff.as_millis() < 500 {
                    tracing::debug!("DOUBLE-CLICK DETECTED on entity: {:?}", last_entity);
                    
                    // Check all draggable cards (includes both tableau and waste cards)
                    for (entity, mut transform, card_data) in draggable_cards.iter_mut() {
                        if entity == last_entity && card_data.is_face_up {
                            tracing::debug!("DOUBLE-CLICK: Attempting to move card {:?} (value: {}, suit: {:?}) to foundation", 
                                           card_data.suit, card_data.value, card_data.suit);
                            
                            // Try foundation placement first
                            if try_move_to_foundation(entity, &mut transform, card_data, &mut foundation_piles, &mut commands) {
                                tracing::debug!("DOUBLE-CLICK: Successfully moved card to foundation");
                                // Reset double-click tracking
                                *last_click_time = None;
                                return;
                            } else {
                                tracing::debug!("DOUBLE-CLICK: Foundation move failed - use drag-and-drop for tableau moves");
                            }
                            
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

/// Comprehensive foundation validation system (disabled - no auto-move)
pub fn foundation_validation_system(
    _commands: Commands,
    _foundation_piles: ResMut<FoundationPiles>,
    _tableau_cards: Query<(Entity, &Transform, &CardData), (With<TableauPile>, With<CardFront>, Without<StockPile>)>,
    _waste_cards: Query<(Entity, &Transform, &CardData), (With<WastePile>, With<CardFront>, Without<StockPile>)>,
) {
    // Foundation validation system is disabled - cards should never move automatically without user input
    // This maintains proper solitaire gameplay where all moves are user-initiated
    // The validation logic is available in the try_move_to_foundation function above
}
