use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButton;
use bevy::input::keyboard::KeyCode;
use crate::components::*;
use crate::utils::get_card_back_image;

/// Handles undo button clicks and executes undo actions
pub fn undo_button_system(
    mut commands: Commands,
    mut undo_stack: ResMut<UndoStack>,
    mut stock_cards: ResMut<StockCards>,
    asset_server: Res<AssetServer>,
    mut transform_query: Query<&mut Transform, With<Card>>,
    mut card_data_query: Query<&mut CardData, With<Card>>,
    undo_button_query: Query<&Transform, (With<UndoButton>, Without<Card>)>,
    window_query: Query<&Window>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        // Get cursor position from window
        let Ok(window) = window_query.single() else { return };
        let Some(cursor_pos) = window.cursor_position() else { return };
        
        // Convert cursor position to world coordinates
        // The undo button is at (WINDOW_WIDTH/2 - 100, WINDOW_HEIGHT/2 - 50) = (540, 310)
        // We need to convert screen coordinates to world coordinates
        let cursor_world_pos = Vec2::new(
            cursor_pos.x - window.width() / 2.0, // Center at window center
            window.height() / 2.0 - cursor_pos.y  // Center at window center, flip Y
        );
        
        
        // Check if undo button was clicked
        for undo_transform in undo_button_query.iter() {
            let button_pos = undo_transform.translation;
            let distance = (cursor_world_pos - button_pos.truncate()).length();
            if distance < 60.0 { // Within button bounds (increased for better usability)
                // Proper undo: restore card to previous state
                if let Some(undo_action) = undo_stack.0.pop() {
                    
                    // CRITICAL FIX: Check if the entity still exists before trying to modify it
                    // This prevents crashes when entities have been despawned and recreated
                    if !commands.get_entity(undo_action.card_entity).is_ok() {
                        continue; // Skip this undo action
                    }
                    
                    // First, remove all current components from the card
                    commands.entity(undo_action.card_entity)
                        .remove::<TableauPile>()
                        .remove::<WastePile>()
                        .remove::<FoundationPile>()
                        .remove::<StockPile>()
                        .remove::<Draggable>()
                        .remove::<CardFront>()
                        .remove::<CardBack>();
                    
                    // Add back the original components
                    let mut entity_commands = commands.entity(undo_action.card_entity);
                    for component_type in &undo_action.from_components {
                        match component_type {
                            ComponentType::TableauPile => entity_commands.insert(TableauPile),
                            ComponentType::WastePile => entity_commands.insert(WastePile),
                            ComponentType::FoundationPile => entity_commands.insert(FoundationPile),
                            ComponentType::StockPile => entity_commands.insert(StockPile),
                            ComponentType::Draggable => entity_commands.insert(Draggable),
                            ComponentType::CardFront => entity_commands.insert(CardFront),
                            ComponentType::CardBack => entity_commands.insert(CardBack),
                        };
                    }
                    
                    // Restore the original face up/down state
                    if let Ok(mut card_data) = card_data_query.get_mut(undo_action.card_entity) {
                        card_data.is_face_up = undo_action.original_face_up;
                    }
                    
                    // Move the main card back to its original position
                    if let Ok(mut transform) = transform_query.get_mut(undo_action.card_entity) {
                        transform.translation = undo_action.from_position;
                    }
                    
                    // CRITICAL FIX: If we're restoring a card to the stock pile, we need to add it back to the stock data
                    // This prevents the "unusable card" bug where cards appear in stock but aren't actually part of the stock pile
                    if undo_action.from_components.contains(&ComponentType::StockPile) {
                        // This card is being restored to the stock pile
                        // We need to add it back to the stock_cards resource
                        if let Ok(card_data) = card_data_query.get(undo_action.card_entity) {
                            // Add the card back to the stock pile data structure
                            stock_cards.0.push((card_data.suit, card_data.value));
                            
                            // CRITICAL FIX: Also restore the card back sprite since stock cards are face down
                            commands.entity(undo_action.card_entity).insert(Sprite {
                                image: asset_server.load(get_card_back_image(card_data.suit)),
                                custom_size: Some(Vec2::new(80.0, 120.0)),
                                ..default()
                            });
                        }
                    }
                    
                    // Move stacked cards back to their original positions
                    for (card_entity, original_pos) in undo_action.stack_cards {
                        if let Ok(mut transform) = transform_query.get_mut(card_entity) {
                            transform.translation = original_pos;
                        }
                    }
                    
                } else {
                }
            }
        }
    }
}

/// Handles keyboard-based undo functionality (Ctrl+Z or Ctrl+U)
pub fn undo_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut undo_stack: ResMut<UndoStack>,
    mut stock_cards: ResMut<StockCards>,
    asset_server: Res<AssetServer>,
    mut transform_query: Query<&mut Transform, With<Card>>,
    mut card_data_query: Query<&mut CardData, With<Card>>,
    mut commands: Commands,
) {
    // Undo on Ctrl+Z or Ctrl+U
    if (keyboard_input.pressed(KeyCode::ControlLeft) || keyboard_input.pressed(KeyCode::ControlRight)) 
        && (keyboard_input.just_pressed(KeyCode::KeyZ) || keyboard_input.just_pressed(KeyCode::KeyU)) {
        
        if let Some(undo_action) = undo_stack.0.pop() {
            
            // CRITICAL FIX: Check if the entity still exists before trying to modify it
            // This prevents crashes when entities have been despawned and recreated
            if !commands.get_entity(undo_action.card_entity).is_ok() {
                return; // Skip this undo action
            }
            
            // First, remove all current components from the card
            commands.entity(undo_action.card_entity)
                .remove::<TableauPile>()
                .remove::<WastePile>()
                .remove::<FoundationPile>()
                .remove::<StockPile>()
                .remove::<Draggable>()
                .remove::<CardFront>()
                .remove::<CardBack>();
            
            // Add back the original components
            let mut entity_commands = commands.entity(undo_action.card_entity);
            for component_type in &undo_action.from_components {
                match component_type {
                    ComponentType::TableauPile => entity_commands.insert(TableauPile),
                    ComponentType::WastePile => entity_commands.insert(WastePile),
                    ComponentType::FoundationPile => entity_commands.insert(FoundationPile),
                    ComponentType::StockPile => entity_commands.insert(StockPile),
                    ComponentType::Draggable => entity_commands.insert(Draggable),
                    ComponentType::CardFront => entity_commands.insert(CardFront),
                    ComponentType::CardBack => entity_commands.insert(CardBack),
                };
            }
            
            // Restore the original face up/down state
            if let Ok(mut card_data) = card_data_query.get_mut(undo_action.card_entity) {
                card_data.is_face_up = undo_action.original_face_up;
            }
            
            // Move the main card back to its original position
            if let Ok(mut transform) = transform_query.get_mut(undo_action.card_entity) {
                transform.translation = undo_action.from_position;
            }
            
            // CRITICAL FIX: If we're restoring a card to the stock pile, we need to add it back to the stock data
            // This prevents the "unusable card" bug where cards appear in stock but aren't actually part of the stock pile
            if undo_action.from_components.contains(&ComponentType::StockPile) {
                // This card is being restored to the stock pile
                // We need to add it back to the stock_cards resource
                if let Ok(card_data) = card_data_query.get(undo_action.card_entity) {
                    // Add the card back to the stock pile data structure
                    stock_cards.0.push((card_data.suit, card_data.value));
                    
                    // CRITICAL FIX: Also restore the card back sprite since stock cards are face down
                    commands.entity(undo_action.card_entity).insert(Sprite {
                        image: asset_server.load(get_card_back_image(card_data.suit)),
                        custom_size: Some(Vec2::new(80.0, 120.0)),
                        ..default()
                    });
                }
            }
            
            // Move stacked cards back to their original positions
            for (card_entity, original_pos) in undo_action.stack_cards {
                if let Ok(mut transform) = transform_query.get_mut(card_entity) {
                    transform.translation = original_pos;
                }
            }
            
        } else {
        }
    }
}
