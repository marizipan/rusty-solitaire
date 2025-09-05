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

pub fn card_drag_system(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut selected_card: ResMut<SelectedCard>,
    mut transform_query: Query<&mut Transform, (With<Card>, With<Draggable>)>,
    card_data_query: Query<&CardData, (With<Card>, With<Draggable>)>,
    entity_query: Query<Entity, (With<Card>, With<Draggable>)>,
    window_query: Query<&Window>,
    mut foundation_piles: ResMut<FoundationPiles>,
    mut commands: Commands,
    mut last_click_time: Local<Option<std::time::Instant>>,
    mut clicked_entity: ResMut<ClickedEntity>,
) {
    let Ok(window) = window_query.single() else { return };
    
    if mouse_input.just_pressed(MouseButton::Left) {
        // Get mouse position
        let Some(cursor_pos) = window.cursor_position() else { return };
        
        let cursor_world_pos = Vec2::new(
            cursor_pos.x - window.width() / 2.0,
            window.height() / 2.0 - cursor_pos.y,
        );
        
        // Check if any face-up card was clicked
        for entity in &entity_query {
            if let Ok(card_data) = card_data_query.get(entity) {
                if !card_data.is_face_up {
                    continue;
                }
                
                if let Ok(transform) = transform_query.get_mut(entity) {
                    let card_pos = transform.translation.truncate();
                    let card_bounds = Vec2::new(40.0, 60.0);
                    
                    if (cursor_world_pos - card_pos).abs().cmplt(card_bounds).all() {
                        // Check if this card is actually the top card of its stack
                        let mut is_top_card = true;
                        let current_pos = transform.translation;
                        
                        // Drop the mutable borrow before checking other entities
                        drop(transform);
                        
                        for other_entity in &entity_query {
                            if other_entity != entity {
                                if let Ok(other_transform) = transform_query.get_mut(other_entity) {
                                    let x_same = (other_transform.translation.x - current_pos.x).abs() < 15.0;
                                    let y_same = (other_transform.translation.y - current_pos.y).abs() < 35.0;
                                    let z_higher = other_transform.translation.z > current_pos.z + 0.5;
                                    
                                    if x_same && y_same && z_higher {
                                        is_top_card = false;
                                        break;
                                    }
                                }
                            }
                        }
                        
                        // Only allow dragging if this is the top card
                        if !is_top_card {
                            continue;
                        }
                        
                        // Update double-click tracking
                        let now = std::time::Instant::now();
                        *last_click_time = Some(now);
                        clicked_entity.0 = Some(entity);
                        selected_card.entity = Some(entity);
                        break;
                    }
                }
            }
        }
    }

    if mouse_input.just_released(MouseButton::Left) {
        // Clear selected card on mouse release
        selected_card.entity = None;
        
    }
}

pub fn card_drop_system(
    mut commands: Commands,
    mut selected_card: ResMut<SelectedCard>,
    mut undo_stack: ResMut<UndoStack>,
    mut foundation_piles: ResMut<FoundationPiles>,
    tableau_positions: Res<TableauPositions>,
    mut stock_cards: ResMut<StockCards>,
    mut draggable_transform_query: Query<&mut Transform, (With<Card>, With<Draggable>)>,
    card_data_query: Query<&CardData, With<Card>>,
    draggable_entity_query: Query<Entity, (With<Card>, With<Draggable>)>,
) {
    if let Some(selected_entity) = selected_card.entity {
        // Get the selected card data first
        let selected_card_data = card_data_query.get(selected_entity).ok();
        let selected_transform = draggable_transform_query.get(selected_entity).ok();
        
        if let (Some(selected_card_data), Some(selected_transform)) = (selected_card_data, selected_transform) {
            let selected_pos = selected_transform.translation;
            let selected_value = selected_card_data.value;
            let selected_suit = selected_card_data.suit;
            
            
            // Check if trying to drop on a Foundation Pile
            let foundation_start_x = -(6.0 * 100.0) / 2.0;
            let foundation_y = WINDOW_HEIGHT / 2.0 - 100.0;
            
            for i in 0..4 {
                let foundation_x = foundation_start_x + (i as f32 * 100.0);
                let foundation_pos = Vec3::new(foundation_x, foundation_y, 1.0);
                let distance = (selected_pos - foundation_pos).length();
                
                if distance < 50.0 {
                    let foundation_pile = &foundation_piles.0[i];
                    
                    if foundation_pile.is_empty() {
                        // Empty foundation pile - only Aces can be placed
                        if selected_value == 1 {
                            // Move to foundation pile
                            commands.entity(selected_entity)
                                .remove::<TableauPile>()
                                .remove::<WastePile>()
                                .remove::<SkippedWasteCard>()
                                .remove::<StockPile>()
                                .remove::<Draggable>() // Remove Draggable since foundation cards can't be moved
                                .insert(FoundationPile)
                                .insert(OriginalPosition(foundation_pos));
                            
                            // Update foundation pile
                            foundation_piles.0[i].push((selected_suit, selected_value));
                            
                            // Create undo action
                            create_undo_action(
                                selected_entity,
                                selected_card.original_position,
                                foundation_pos,
                                selected_card.original_components.clone(),
                                vec![ComponentType::FoundationPile],
                                Vec::new(),
                                selected_card_data.is_face_up,
                                &mut undo_stack,
                            );
                            
                            // Clear selection
                            selected_card.entity = None;
                            selected_card.original_position = Vec3::ZERO;
                            selected_card.original_components.clear();
                            return;
                        }
                    } else {
                        // Foundation pile has cards - check if this card can be added
                        let (top_suit, top_value) = foundation_pile.last().unwrap();
                        if selected_suit == *top_suit && selected_value == top_value + 1 {
                            // Move to foundation pile
                            commands.entity(selected_entity)
                                .remove::<TableauPile>()
                                .remove::<WastePile>()
                                .remove::<SkippedWasteCard>()
                                .remove::<StockPile>()
                                .remove::<Draggable>() // Foundation cards cannot be moved
                                .insert(FoundationPile);
                            
                            // Update foundation pile
                            foundation_piles.0[i].push((selected_suit, selected_value));
                            
                            // Create undo action
                            create_undo_action(
                                selected_entity,
                                selected_card.original_position,
                                foundation_pos,
                                selected_card.original_components.clone(),
                                vec![ComponentType::FoundationPile],
                                Vec::new(),
                                selected_card_data.is_face_up,
                                &mut undo_stack,
                            );
                            
                            // Clear selection
                            selected_card.entity = None;
                            selected_card.original_position = Vec3::ZERO;
                            selected_card.original_components.clear();
                            return;
                        }
                    }
                }
            }
            // Check if trying to drop on an empty tableau pile
            for (_i, tableau_pos) in tableau_positions.0.iter().enumerate() {
                let distance = (selected_pos - *tableau_pos).length();
                
                if distance < 50.0 {
                    // Check if this tableau position is empty
                    let mut is_empty = true;
                    for entity in draggable_entity_query.iter() {
                        if entity == selected_entity {
                            continue; // Skip the card being dragged
                        }
                        
                        if let Ok(transform) = draggable_transform_query.get(entity) {
                            if (transform.translation.x - tableau_pos.x).abs() < 5.0 
                                && (transform.translation.y - tableau_pos.y).abs() < 5.0 {
                                is_empty = false;
                                break;
                            }
                        }
                    }
                    
                    if is_empty {
                        // Only Kings can be placed on empty tableau piles
                        if selected_value == 13 {
                            // Move to empty tableau pile
                            commands.entity(selected_entity)
                                .remove::<WastePile>()
                                .remove::<SkippedWasteCard>()
                                .insert(TableauPile);
                            
                            // Create undo action
                            create_undo_action(
                                selected_entity,
                                selected_card.original_position,
                                *tableau_pos,
                                selected_card.original_components.clone(),
                                vec![ComponentType::TableauPile, ComponentType::Draggable],
                                Vec::new(),
                                selected_card_data.is_face_up,
                                &mut undo_stack,
                            );
                            
                            // Clear selection
                            selected_card.entity = None;
                            selected_card.original_position = Vec3::ZERO;
                            selected_card.original_components.clear();
                            return;
                        }
                    }
                }
            }
            
            // First, collect all potential targets and tableau pile information
            let mut potential_targets = Vec::new();
            
            // Find valid target positions (positions where cards can be placed)
            for entity in draggable_entity_query.iter() {
                if entity == selected_entity {
                    continue; // Skip the card being dragged
                }
                
                if let (Ok(target_transform), Ok(target_card_data)) = (draggable_transform_query.get(entity), card_data_query.get(entity)) {
                    let target_pos = target_transform.translation;
                    
                    // Only consider face-up tableau cards as potential targets
                    if !target_card_data.is_face_up {
                        continue;
                    }
                    
                    // Check if this is a tableau card by looking at its position
                    if target_pos.y < -300.0 || target_pos.y > 100.0 {
                        continue;
                    }
                    
                    // Check if this position is actually available for placement
                    // A position is available if it's the top card of a stack (no other cards above it)
                    let mut is_top_card = true;
                    for other_entity in draggable_entity_query.iter() {
                        if other_entity != entity && other_entity != selected_entity {
                            if let Ok(other_transform) = draggable_transform_query.get(other_entity) {
                                let other_pos = other_transform.translation;
                                // Check if there's another card at the same X position but stacked above (lower Y)
                                let x_same = (other_pos.x - target_pos.x).abs() < 5.0;
                                let stacked_above = other_pos.y < target_pos.y - 25.0; // 30px offset with some tolerance
                                
                                if x_same && stacked_above {
                                    // There's a card stacked above - this position is not available
                                    is_top_card = false;
                                    break;
                                }
                            }
                        }
                    }
                    
                    // Only add this as a potential target if it's the top card of its stack
                    if is_top_card {
                        let distance = (selected_pos - target_pos).length();
                        potential_targets.push((target_pos, distance, target_card_data.value, target_card_data.suit));
                        
                        println!("TARGET DEBUG: Added valid target at ({:.1}, {:.1}) - {} of {} (distance: {:.1})",
                            target_pos.x, target_pos.y, target_card_data.value, format!("{:?}", target_card_data.suit), distance);
                    }
                }
            }
            
            println!("TARGET DEBUG: Found {} potential tableau targets", potential_targets.len());
            
            // Sort potential targets by distance (closest first)
            potential_targets.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            
            // Check each potential target for valid placement
            for (target_pos, _distance, target_value, target_suit) in potential_targets {
                // Basic validation - card value must be one lower than target
                if selected_value != target_value - 1 {
                    println!("TABLEAU DROP DEBUG: Value mismatch - selected: {}, target: {}", selected_value, target_value);
                    continue; // Skip invalid placements
                }
                
                // Color validation - colors must alternate (red on black, black on red)
                let selected_is_red = matches!(selected_suit, CardSuit::Hearts | CardSuit::Diamonds);
                let target_is_red = matches!(target_suit, CardSuit::Hearts | CardSuit::Diamonds);
                
                println!("TABLEAU DROP DEBUG: Selected {} {} (red: {}), Target {} {} (red: {})", 
                    selected_value, format!("{:?}", selected_suit), selected_is_red,
                    target_value, format!("{:?}", target_suit), target_is_red);
                
                if selected_is_red == target_is_red {
                    println!("TABLEAU DROP DEBUG: Color validation failed - both cards are same color");
                    continue; // Colors must be different
                }
                
                // If we get here, this is a valid placement
                println!("TABLEAU DROP DEBUG: Valid placement found!");
                
                // Move to tableau pile
                commands.entity(selected_entity)
                    .remove::<WastePile>()
                    .remove::<SkippedWasteCard>()
                    .insert(TableauPile);
                
                // Create undo action
                create_undo_action(
                    selected_entity,
                    selected_card.original_position,
                    target_pos,
                    selected_card.original_components.clone(),
                    vec![ComponentType::TableauPile, ComponentType::Draggable],
                    Vec::new(),
                    selected_card_data.is_face_up,
                    &mut undo_stack,
                );
                
                // Clear selection
                selected_card.entity = None;
                selected_card.original_position = Vec3::ZERO;
                selected_card.original_components.clear();
                return;
            }
            
            // If we get here, the drop was invalid - snap back to original position
            if let Ok(mut transform) = draggable_transform_query.get_mut(selected_entity) {
                transform.translation = selected_card.original_position;
            }
            
            // Clear selection
            selected_card.entity = None;
            selected_card.original_position = Vec3::ZERO;
            selected_card.original_components.clear();
        }
    }
}

pub fn waste_card_click_system(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window>,
    waste_cards: Query<(Entity, &Transform, &CardData), (With<WastePile>, Without<SkippedWasteCard>)>,
    tableau_cards: Query<(Entity, &Transform, &CardData), (With<TableauPile>, Without<WastePile>)>,
    tableau_positions: Res<TableauPositions>,
    mut foundation_piles: ResMut<FoundationPiles>,
    mut stock_cards: ResMut<StockCards>,
) {
    // Only process when user clicks
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }
    
    let Ok(window) = window_query.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    
    let cursor_world_pos = Vec2::new(
        cursor_pos.x - window.width() / 2.0,
        window.height() / 2.0 - cursor_pos.y,
    );
    
    
    // Find the top waste card (highest Z position)
    let mut top_waste_card: Option<(Entity, &Transform, &CardData)> = None;
    
    for (entity, transform, card_data) in waste_cards.iter() {
        if let Some((_entity, current_transform, _card_data)) = top_waste_card {
            if transform.translation.z > current_transform.translation.z {
                top_waste_card = Some((entity, transform, card_data));
            }
        } else {
            top_waste_card = Some((entity, transform, card_data));
        }
    }
    
    // Check if user clicked on the top waste card
    if let Some((waste_entity, waste_transform, waste_card_data)) = top_waste_card {
        let card_bounds = Vec2::new(40.0, 60.0);
        let card_pos = waste_transform.translation.truncate();
        
        if (cursor_world_pos - card_pos).abs().cmplt(card_bounds).all() {
            // Check Foundation Piles FIRST (higher priority than tableau)
            let foundation_start_x = -(6.0 * 100.0) / 2.0;
            let foundation_y = WINDOW_HEIGHT / 2.0 - 100.0;
            
            let mut best_target: Option<(Vec3, f32)> = None;
            
            // First check if it can be placed on foundation piles
            for i in 0..4 {
                let foundation_x = foundation_start_x + (i as f32 * 100.0);
                let foundation_pos = Vec3::new(foundation_x, foundation_y, 1.0);
                let foundation_pile = &foundation_piles.0[i];
                
                if foundation_pile.is_empty() {
                    // Empty foundation pile - only Aces can be placed
                    if waste_card_data.value == 1 {
                        best_target = Some((foundation_pos, 0.0));
                        break;
                    }
                } else {
                    // Foundation pile has cards - check if this card can be added
                    let (top_suit, top_value) = foundation_pile.last().unwrap();
                    if waste_card_data.suit == *top_suit && waste_card_data.value == top_value + 1 {
                        best_target = Some((foundation_pos, 0.0));
                        break;
                    }
                }
            }
            
            // If no valid foundation pile found, check if it can be placed on existing tableau cards
            for (_tableau_entity, tableau_transform, tableau_card_data) in tableau_cards.iter() {
                // Only consider face-up cards as valid targets
                if !tableau_card_data.is_face_up {
                    continue;
                }
                
                // CRITICAL: Basic validation - card value must be one lower than target
                if waste_card_data.value != tableau_card_data.value - 1 {
                    continue; // Skip invalid placements
                }
                
                // Check if this is a valid placement (card value must be one lower AND colors must alternate)
                // The value check was already done above, so we just need to check colors
                
                // CRITICAL: Check for duplicate values in the target stack FIRST
                let mut target_stack_values = Vec::new();
                for (other_entity, other_transform, other_card_data) in tableau_cards.iter() {
                    // Only check X position for stacking - Y position varies due to visual stacking
                    let x_same = (other_transform.translation.x - tableau_transform.translation.x).abs() < 5.0;
                    if x_same {
                        target_stack_values.push(other_card_data.value);
                        println!("WASTE STACK DEBUG: Found card {} at X position {:.1} (target X: {:.1})", 
                            other_card_data.value, other_transform.translation.x, tableau_transform.translation.x);
                    }
                }
                
                // Check if we're trying to place a card with a value that already exists in the stack
                if target_stack_values.contains(&waste_card_data.value) {
                    println!("VALIDATION FAILED: Card value {} already exists in target stack", waste_card_data.value);
                    continue; // Skip this placement
                }
                
                // Additional check: colors must alternate (red on black, black on red)
                let waste_is_red = matches!(waste_card_data.suit, CardSuit::Hearts | CardSuit::Diamonds);
                let tableau_is_red = matches!(tableau_card_data.suit, CardSuit::Hearts | CardSuit::Diamonds);
                
                if waste_is_red != tableau_is_red { // Colors must be different
                    // Additional check: make sure we're not placing on a card that's already covered
                    // Only place on the top card of each stack
                    let mut is_top_card = true;
                    for (other_entity, other_transform, _card_data) in tableau_cards.iter() {
                        if other_entity != _tableau_entity {
                            // Check if this other card is on top of our target
                            // Only check X position for stacking - Y position varies due to visual stacking
                            let x_same = (other_transform.translation.x - tableau_transform.translation.x).abs() < 5.0;
                            let z_higher = other_transform.translation.z > tableau_transform.translation.z;
                            
                            if x_same && z_higher {
                                is_top_card = false;
                                break;
                            }
                        }
                    }
                    
                    // Additional check: prevent placing waste card on its own stack
                    let mut is_valid_target = true;
                    let waste_x = waste_transform.translation.x;
                    let waste_y = waste_transform.translation.y;
                    let tableau_x = tableau_transform.translation.x;
                    let tableau_y = tableau_transform.translation.y;
                    
                    // Check if waste card and tableau card are at the same position (same stack)
                    if (waste_x - tableau_x).abs() < 5.0 && (waste_y - tableau_y).abs() < 5.0 {
                        is_valid_target = false;
                    }
                    
                    if is_top_card && is_valid_target {
                        let distance = (waste_transform.translation - tableau_transform.translation).length();
                        
                        if let Some((_target_pos, current_distance)) = best_target {
                            if distance < current_distance {
                                best_target = Some((tableau_transform.translation, distance));
                            }
                        } else {
                            best_target = Some((tableau_transform.translation, distance));
                        }
                    }
                }
            }
            
            // If no valid foundation pile found, check if it can be placed on empty tableau piles
            if best_target.is_none() {
                // Only Kings can be placed on empty tableau piles
                if waste_card_data.value == 13 {
                    for tableau_pos in &tableau_positions.0 {
                        // Check if this tableau position is empty
                        let mut is_empty = true;
                        for (_entity, tableau_transform, _card_data) in tableau_cards.iter() {
                            if (tableau_transform.translation.x - tableau_pos.x).abs() < 5.0 
                                && (tableau_transform.translation.y - tableau_pos.y).abs() < 5.0 {
                                is_empty = false;
                                break;
                            }
                        }
                        
                        if is_empty {
                            best_target = Some((*tableau_pos, 0.0)); // Distance 0 for empty piles
                            break;
                        }
                    }
                }
            }
            
            // If we found a valid target, move the waste card there
            if let Some((target_pos, _target_distance)) = best_target {
                // Check if this is a Foundation Pile placement
                let foundation_start_x = -(6.0 * 100.0) / 2.0;
                let foundation_y = WINDOW_HEIGHT / 2.0 - 100.0;
                
                if (target_pos.x - foundation_start_x).abs() < 200.0 && (target_pos.y - foundation_y).abs() < 50.0 {
                    // This is a foundation pile placement
                    let foundation_index = ((target_pos.x - foundation_start_x) / 100.0) as usize;
                    
                    // Move the waste card
                    commands.entity(waste_entity)
                        .remove::<WastePile>()
                        .remove::<SkippedWasteCard>()
                        .insert(FoundationPile)
                        .insert(OriginalPosition(target_pos));
                    
                    // Update the transform to move to foundation pile
                    commands.entity(waste_entity).insert(Transform::from_translation(target_pos));
                    
                    // Update foundation pile
                    foundation_piles.0[foundation_index].push((waste_card_data.suit, waste_card_data.value));
                    
                    // Remove from stock_cards (waste pile)
                    stock_cards.0.retain(|card| card.0 != waste_card_data.suit || card.1 != waste_card_data.value);
                } else {
                    // This is a tableau placement
                    let new_position = Vec3::new(target_pos.x, target_pos.y, target_pos.z + 1.0);
                    
                    // Move the waste card
                    commands.entity(waste_entity)
                        .remove::<WastePile>()
                        .remove::<SkippedWasteCard>()
                        .insert(TableauPile)
                        .insert(OriginalPosition(new_position))
                        .insert(Draggable);
                    
                    // Update the transform to move to tableau position
                    commands.entity(waste_entity).insert(Transform::from_translation(new_position));
                    
                    // Remove from stock_cards (waste pile)
                    stock_cards.0.retain(|card| card.0 != waste_card_data.suit || card.1 != waste_card_data.value);
                }
                
                // Mark all other waste cards as skipped since the top one moved
                for (entity, _transform, _card_data) in waste_cards.iter() {
                    if entity != waste_entity {
                        commands.entity(entity).insert(SkippedWasteCard);
                    }
                }
            }
        }
    }
}

pub fn double_click_foundation_system(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut foundation_piles: ResMut<FoundationPiles>,
    mut draggable_cards: Query<(Entity, &mut Transform, &CardData), (With<Draggable>, Or<(With<TableauPile>, With<WastePile>)>)>,
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
                    // Check if the double-clicked card can be moved to foundation
                    // Use the proper Bevy query pattern to get the card data and transform
                    for (entity, mut transform, card_data) in draggable_cards.iter_mut() {
                        if entity == last_entity && card_data.is_face_up {
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
                                let foundation_y = WINDOW_HEIGHT / 2.0 - 100.0;
                                
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
                                
                                // Reset double-click tracking
                                *last_click_time = None;
                                return;
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
