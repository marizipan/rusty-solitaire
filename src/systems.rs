use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButton;
use crate::components::*;
use crate::utils::{get_card_front_image, get_card_back_image, can_place_on_card, has_complete_stack, get_card_data_from_filename};



// Helper function to create a card entity with sprite
fn create_card_entity(
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
                    // Collect waste card data in the order they were dealt (oldest first)
                    let mut waste_cards_info: Vec<(Entity, CardSuit, u8, f32)> = waste_cards
                        .iter()
                        .map(|(entity, transform, card_data, _)| (entity, card_data.suit, card_data.value, transform.translation.z))
                        .collect();
                    
                    // Sort by Z position to ensure correct order (lowest Z = oldest = dealt first)
                    waste_cards_info.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap());
                    
                    // Now despawn all the entities
                    for (entity, _, _, _) in &waste_cards_info {
                        commands.entity(*entity).despawn();
                    }
                    
                    // Put all waste cards back into stock (oldest first, so they'll be dealt last)
                    let waste_card_data: Vec<(CardSuit, u8)> = waste_cards_info
                        .iter()
                        .map(|(_, suit, value, _)| (*suit, *value))
                        .collect();
                    stock_cards.0 = waste_card_data;
                    

                }
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
    mut last_clicked_entity: Local<Option<Entity>>,
) {
    let Ok(window) = window_query.single() else { return };
    
    if mouse_input.just_pressed(MouseButton::Left) {
        // Get mouse position
        if let Some(cursor_pos) = window.cursor_position() {
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
                                        let x_same = (other_transform.translation.x - current_pos.x).abs() < 5.0;
                                        let y_same = (other_transform.translation.y - current_pos.y).abs() < 5.0;
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
                            
                            // Check if this card is a stack leader (can drag entire stack)
                            let mut can_lead_stack = false;
                            
                            // A card can lead a stack if:
                            // 1. It's a single card (no cards above it), OR
                            // 2. It's part of a valid descending sequence with alternating suits
                            if let Ok(card_data) = card_data_query.get(entity) {
                                // First, collect all cards that are stacked above this card
                                let mut cards_above = Vec::new();
                                let mut stack_entities = Vec::new();
                                
                                for other_entity in &entity_query {
                                    if other_entity != entity {
                                        if let Ok(other_transform) = transform_query.get_mut(other_entity) {
                                            let x_same = (other_transform.translation.x - current_pos.x).abs() < 5.0;
                                            let y_same = (other_transform.translation.y - current_pos.y).abs() < 5.0;
                                            let z_higher = other_transform.translation.z > current_pos.z + 0.5;
                                            
                                            if x_same && y_same && z_higher {
                                                if let Ok(other_card_data) = card_data_query.get(other_entity) {
                                                    cards_above.push((other_card_data.suit, other_card_data.value));
                                                    stack_entities.push(other_entity);
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                // If no cards above, this card can lead a stack
                                if cards_above.is_empty() {
                                    can_lead_stack = true;
                                } else {
                                    // Check if this forms a valid descending sequence with alternating suits
                                    let mut all_cards = vec![(card_data.suit, card_data.value)];
                                    all_cards.extend(cards_above);
                                    
                                    // Sort by value in descending order (highest to lowest)
                                    all_cards.sort_by(|a, b| b.1.cmp(&a.1));
                                    
                                    // Check if the sequence is valid
                                    let mut is_valid_sequence = true;
                                    for i in 0..all_cards.len() - 1 {
                                        let current = all_cards[i];
                                        let next = all_cards[i + 1];
                                        
                                        // Check descending order (current value should be higher than next)
                                        if current.1 <= next.1 {
                                            is_valid_sequence = false;
                                            break;
                                        }
                                        
                                        // Check alternating suits (current and next should have different suits)
                                        if current.0 == next.0 {
                                            is_valid_sequence = false;
                                            break;
                                        }
                                    }
                                    
                                    can_lead_stack = is_valid_sequence;
                                }
                            }
                            
                            // Only allow dragging if this card can lead a stack
                            if !can_lead_stack {
                                continue;
                            }
                            
                            // Check for double-click
                            let now = std::time::Instant::now();
                            let mut should_handle_ace = false;
                            
                            if let Some(last_time) = *last_click_time {
                                if let Some(last_entity) = *last_clicked_entity {
                                    if last_entity == entity && now.duration_since(last_time).as_millis() < 500 {
                                        // Double-click detected! Check if it's an Ace
                                        if let Ok(card_data) = card_data_query.get(entity) {
                                            if card_data.value == 1 { // This is an Ace
                                                should_handle_ace = true;
                                            }
                                        }
                                    }
                                }
                            }
                            
                            if should_handle_ace {
                                // Handle Ace double-click
                                if let Ok(card_data) = card_data_query.get(entity) {
                                    // Determine which foundation pile this Ace should go to based on suit
                                    let foundation_index = match card_data.suit {
                                        CardSuit::Hearts => 0,    // Foundation Pile 1
                                        CardSuit::Diamonds => 1,  // Foundation Pile 2
                                        CardSuit::Clubs => 2,     // Foundation Pile 3
                                        CardSuit::Spades => 3,    // Foundation Pile 4
                                    };
                                    
                                    // Check if this foundation pile is empty
                                    if foundation_piles.0[foundation_index].is_empty() {
                                        // Calculate foundation pile position
                                        let foundation_start_x = -(6.0 * 100.0) / 2.0;
                                        let foundation_x = foundation_start_x + (foundation_index as f32 * 100.0);
                                        let foundation_y = WINDOW_HEIGHT / 2.0 - 100.0;
                                        
                                        // Move the Ace to the foundation pile
                                        if let Ok(mut transform) = transform_query.get_mut(entity) {
                                            transform.translation = Vec3::new(foundation_x, foundation_y, 1.0);
                                            
                                            // Update the FoundationPiles resource
                                            foundation_piles.0[foundation_index].push((card_data.suit, card_data.value));
                                            
                                            // Remove tableau/waste/stock components and add foundation component
                                            commands.entity(entity)
                                                .remove::<TableauPile>()
                                                .remove::<WastePile>()
                                                .remove::<SkippedWasteCard>()
                                                .remove::<StockPile>()
                                                .insert(FoundationPile)
                                                .insert(OriginalPosition(Vec3::new(foundation_x, foundation_y, 1.0)));
                                        }
                                        
                                        // Reset double-click tracking
                                        *last_click_time = None;
                                        *last_clicked_entity = None;
                                        selected_card.0 = None;
                                        return; // Exit early since we handled the double-click
                                    }
                                }
                            }
                            
                            // Update double-click tracking
                            *last_click_time = Some(now);
                            *last_clicked_entity = Some(entity);
                            selected_card.0 = Some(entity);
                            break;
                        }
                    }
                }
            }
        }
    }

    if mouse_input.just_released(MouseButton::Left) {
        // Clear selected card on mouse release
        selected_card.0 = None;
    }
}

pub fn card_drop_system(
    selected_card: ResMut<SelectedCard>,
    mut draggable_transform_query: Query<&mut Transform, (With<Card>, With<Draggable>)>,
    card_data_query: Query<&CardData, With<Card>>,
    draggable_entity_query: Query<Entity, (With<Card>, With<Draggable>)>,
    window_query: Query<&Window>,
    mut commands: Commands,

    tableau_positions: Res<TableauPositions>,
    mut foundation_piles: ResMut<FoundationPiles>,
) {
    if let Some(selected_entity) = selected_card.0 {
        let Ok(window) = window_query.single() else { return };
        
        if let Some(cursor_pos) = window.cursor_position() {
            let cursor_world_pos = Vec2::new(
                cursor_pos.x - window.width() / 2.0,
                window.height() / 2.0 - cursor_pos.y,
            );
            
            // First, collect all potential targets and tableau pile information
            let mut potential_targets = Vec::new();
            let mut tableau_pile_info = Vec::new();
            
            for entity in &draggable_entity_query {
                if entity == selected_entity {
                    continue; // Skip the card being dragged
                }
                
                if let Ok(target_card_data) = card_data_query.get(entity) {
                    // Only consider tableau cards as potential targets
                    if !target_card_data.is_face_up {
                        continue;
                    }
                    
                    if let Ok(target_transform) = draggable_transform_query.get(entity) {
                        // Check if this is a tableau card by looking at its position
                        // Tableau cards are at Y positions around -250 to 50 (allowing for stacked cards)
                        let target_pos = target_transform.translation;
                        if target_pos.y < -300.0 || target_pos.y > 100.0 {
                            continue;
                        }
                        
                        let distance = (cursor_world_pos - target_pos.truncate()).length();
                        potential_targets.push((target_pos, distance, target_card_data.value, target_card_data.suit));
                    }
                }
            }
            
            // Check tableau pile positions
            for tableau_pos in &tableau_positions.0 {
                let distance = (cursor_world_pos - tableau_pos.truncate()).length();
                if distance < 50.0 {
                    let mut pile_has_cards = false;
                    for entity in &draggable_entity_query {
                        if let Ok(card_transform) = draggable_transform_query.get(entity) {
                            if (card_transform.translation.x - tableau_pos.x).abs() < 5.0 
                                && (card_transform.translation.y - tableau_pos.y).abs() < 5.0 {
                                pile_has_cards = true;
                                break;
                            }
                        }
                    }
                    tableau_pile_info.push((*tableau_pos, !pile_has_cards));
                    break;
                }
            }
            
            // First, get the selected card data and store what we need
            let (selected_value, selected_suit) = if let Ok(card_data) = card_data_query.get(selected_entity) {
                (card_data.value, card_data.suit)
            } else {
                return; // Can't get selected card data, exit early
            };
            
            // Now collect all the information we need before any mutable operations
            let mut best_target: Option<(Vec3, f32, &'static str)> = None;
            let mut original_position = Vec3::ZERO;
            let mut cards_to_move = Vec::new();
            let mut stack_cards = Vec::new();
            
            // Get the original position from the selected card first
            if let Ok(transform) = draggable_transform_query.get(selected_entity) {
                original_position = transform.translation;
                
                // Collect all cards that are stacked on top of the selected card
                for card_entity in &draggable_entity_query {
                    if card_entity != selected_entity {
                        if let Ok(card_transform) = draggable_transform_query.get(card_entity) {
                            if let Ok(card_data) = card_data_query.get(card_entity) {
                                // Check if this card is at the same X,Y position but higher Z (stacked on top)
                                let same_position = (card_transform.translation.x - original_position.x).abs() < 5.0 
                                    && (card_transform.translation.y - original_position.y).abs() < 5.0;
                                let higher_z = card_transform.translation.z > original_position.z + 0.5; // Small tolerance for Z positioning
                                
                                if same_position && higher_z {
                                    // Calculate the relative position in the stack (how many cards above the selected card)
                                    let stack_index = ((card_transform.translation.z - original_position.z) / 1.0) as u32;
                                    cards_to_move.push((card_entity, stack_index));
                                    stack_cards.push((card_data.suit, card_data.value));
                                }
                            }
                        }
                    }
                }
                
                // Add the selected card to the stack
                stack_cards.push((selected_suit, selected_value));
            }
            
            // Check if trying to drop on stock pile or waste pile (prevent this)
            let stock_x = -(6.0 * 100.0) / 2.0 + (6.0 * 100.0); // Above Stack 7 (x = 300)
            let stock_y = WINDOW_HEIGHT / 2.0 - 100.0; // Above the tableau stacks
            let waste_x = -(6.0 * 100.0) / 2.0 + (5.0 * 100.0); // Above Stack 6 (x = 200)
            let waste_y = stock_y;
            
            // Check distance to stock pile center
            let stock_distance = (cursor_world_pos - Vec2::new(stock_x, stock_y)).length();
            // Check distance to waste pile center  
            let waste_distance = (cursor_world_pos - Vec2::new(waste_x, waste_y)).length();
            
            if stock_distance < 50.0 || waste_distance < 50.0 {
                // Don't allow dropping on stock pile or waste pile - snap back to original position
                if let Ok(mut transform) = draggable_transform_query.get_mut(selected_entity) {
                    transform.translation = original_position;
                }
                return;
            }
            
            // If the selected card is a waste card being moved to tableau, remove waste pile components
            // Check if this card was in the waste pile
            let waste_bounds = Vec2::new(50.0, 60.0);
            let waste_center = Vec2::new(waste_x, waste_y);
            let original_pos_2d = Vec2::new(original_position.x, original_position.y);
            
            if (original_pos_2d - waste_center).abs().cmplt(waste_bounds).all() {
                // This was a waste card - remove waste pile components and add tableau components
                commands.entity(selected_entity)
                    .remove::<WastePile>()
                    .remove::<SkippedWasteCard>()
                    .insert(TableauPile);
            }
            
            // Check if trying to drop on a Foundation Pile
            let foundation_start_x = -(6.0 * 100.0) / 2.0; // Same starting X as tableau stacks
            let foundation_y = WINDOW_HEIGHT / 2.0 - 100.0; // Above the tableau stacks
            
            for i in 0..4 {
                let foundation_x = foundation_start_x + (i as f32 * 100.0);
                let foundation_distance = (cursor_world_pos - Vec2::new(foundation_x, foundation_y)).length();
                
                if foundation_distance < 120.0 { // Increased detection radius to allow easier foundation placement
                    // Check if this card can be placed on this foundation pile
                    if let Ok(card_data) = card_data_query.get(selected_entity) {
                        let foundation_pile = &foundation_piles.0[i];
                        
                        // Allow foundation placement if:
                        // 1. This is NOT an Ace (Aces are handled by auto-click only), AND
                        // 2. This is the next card in sequence for a non-empty foundation pile
                        let can_place_on_foundation = if foundation_pile.is_empty() {
                            // Empty foundation pile - no cards can be placed via drag (Aces use auto-click)
                            false
                        } else {
                            // Foundation pile has cards - check if this card can be added
                            let (top_suit, top_value) = foundation_pile.last().unwrap();
                            // Prevent placing Aces on foundation piles via drag
                            if card_data.value == 1 {
                                false
                            } else {
                            let is_next_in_sequence = card_data.suit == *top_suit && card_data.value == top_value + 1;
                            is_next_in_sequence
                            }
                        };
                    
                        if can_place_on_foundation {
                            best_target = Some((Vec3::new(foundation_x, foundation_y, 1.0), 0.0, "foundation"));
                            break;
                        }
                    }
                    // Don't break here - continue checking other foundation piles
                    // Only break if we found a valid target
                }
            }
            
            // Check tableau targets only if we haven't found a foundation pile target
            if best_target.is_none() {
                for (target_pos, distance, target_value, target_suit) in potential_targets {
                    // Check if this is a valid placement (card value must be one lower AND colors must alternate)
                    if can_place_on_card(selected_value, target_value) {
                        // Additional check: colors must alternate (red on black, black on red)
                        let selected_is_red = matches!(selected_suit, CardSuit::Hearts | CardSuit::Diamonds);
                        let target_is_red = matches!(target_suit, CardSuit::Hearts | CardSuit::Diamonds);
                        
                        if selected_is_red == target_is_red {
                            // Colors are the same - this placement is invalid
                            continue;
                        }
                        
                        // Check if this target card is actually the top card of its stack
                        let mut is_top_card = true;
                        for other_entity in &draggable_entity_query {
                            if other_entity != selected_entity {
                                if let Ok(other_transform) = draggable_transform_query.get(other_entity) {
                                    let x_same = (other_transform.translation.x - target_pos.x).abs() < 5.0;
                                    let y_same = (other_transform.translation.y - target_pos.y).abs() < 5.0;
                                    let z_higher = other_transform.translation.z > target_pos.z;
                                    
                                    if x_same && y_same && z_higher {
                                        is_top_card = false;
                                        break;
                                    }
                                }
                            }
                        }
                        
                        if is_top_card {
                            // Allow placement on top of cards (we'll handle Z positioning when actually moving)
                            if let Some((_target_pos, current_distance, _target_type)) = best_target {
                                if distance < current_distance {
                                    best_target = Some((target_pos, distance, "tableau"));
                                }
                            } else {
                                best_target = Some((target_pos, distance, "tableau"));
                            }
                        }
                    }
                }
            }
            
            // Check if dropped on an empty tableau pile first (before borrowing transform)
            let mut target_tableau_pos = None;
            
            // Use the collected tableau pile information
            for (tableau_pos, is_empty) in &tableau_pile_info {
                if *is_empty {
                    // Empty tableau pile - allow if the top card (being dragged) is a King
                    if selected_value == 13 {
                        // For single cards or incomplete stacks, just check if it's a King
                        if stack_cards.len() == 1 {
                            // Single King card - allow placement
                            target_tableau_pos = Some(*tableau_pos);
                        } else {
                            // Multiple cards - check if it's a complete stack
                            let mut sorted_stack = stack_cards.clone();
                            sorted_stack.sort_by(|a, b| b.1.cmp(&a.1));
                            
                            if has_complete_stack(&sorted_stack) {
                                target_tableau_pos = Some(*tableau_pos);
                            }
                        }
                    }
                }
            }
            
            // Now get the selected card data and apply the movement
            if let Ok(mut transform) = draggable_transform_query.get_mut(selected_entity) {
                if let Some((target_pos, _distance, target_type)) = best_target {
                    if target_type == "foundation" {
                        // Placing on foundation pile
                        let new_position = target_pos;
                        transform.translation = new_position;
                        commands.entity(selected_entity).insert(OriginalPosition(new_position));
                        
                        // Update the FoundationPiles resource
                        let foundation_start_x = -(6.0 * 100.0) / 2.0;
                        let foundation_index = ((new_position.x - foundation_start_x) / 100.0) as usize;
                        
                        if let Ok(card_data) = card_data_query.get(selected_entity) {
                            // Add the card to the foundation pile stack
                            foundation_piles.0[foundation_index].push((card_data.suit, card_data.value));
                        }
                        
                        // Remove tableau/waste/stock components and add foundation component
                        commands.entity(selected_entity)
                            .remove::<TableauPile>()
                            .remove::<WastePile>()
                            .remove::<SkippedWasteCard>()
                            .remove::<StockPile>()
                            .insert(FoundationPile);
                        
                    } else {
                        // Placing on tableau
                        // Position the selected card on top of the target card
                        let new_position = Vec3::new(target_pos.x, target_pos.y, target_pos.z + 1.0);
                        transform.translation = new_position;
                        
                        // Update the original position for future reference
                        commands.entity(selected_entity).insert(OriginalPosition(new_position));
                        
                        // Move all stacked cards to maintain their relative positions with proper stacking
                        for (card_entity, stack_index) in &cards_to_move {
                            // Use stacking offset: 25 pixels per card to show enough of each card
                            let stacked_y = new_position.y - (*stack_index as f32 * 25.0);
                            // Z position should match the visual stacking: each card gets a Z offset that corresponds to its visual position
                            let new_card_position = Vec3::new(
                                new_position.x, 
                                stacked_y, // Stack cards with 25px offset for reasonable visual spacing
                                new_position.z + *stack_index as f32 + 1.0 // +1.0 to ensure proper layering
                            );
                            
                            // Update the transform directly for stacked cards
                            if let Ok(mut card_transform) = draggable_transform_query.get_mut(*card_entity) {
                                card_transform.translation = new_card_position;
                            }
                            
                            commands.entity(*card_entity).insert(OriginalPosition(new_card_position));
                            
                            // Also update the card's components to ensure it's properly marked as tableau
                            commands.entity(*card_entity)
                                .remove::<WastePile>()
                                .remove::<SkippedWasteCard>()
                                .remove::<StockPile>()
                                .insert(TableauPile)
                                .insert(Draggable); // Ensure stacked cards remain draggable
                        }
                        
                        // Update the visual stacking for the entire stack to ensure proper appearance
                        // This ensures all cards in the stack are visually stacked with proper offsets
                        let mut all_stack_cards = vec![(selected_entity, 0)]; // Selected card is at index 0
                        all_stack_cards.extend(cards_to_move.iter().map(|(entity, index)| (*entity, *index + 1)));
                        
                        // Sort by stack index to ensure proper visual stacking
                        all_stack_cards.sort_by(|a, b| a.1.cmp(&b.1));
                        
                        // Apply visual stacking offsets
                        for (stack_index, (card_entity, _)) in all_stack_cards.iter().enumerate() {
                            if let Ok(mut card_transform) = draggable_transform_query.get_mut(*card_entity) {
                                let visual_offset = stack_index as f32 * 25.0;
                                card_transform.translation.y = new_position.y - visual_offset;
                                // Set Z position to ensure proper layering: each card gets a higher Z than the one below
                                card_transform.translation.z = new_position.z + stack_index as f32 + 1.0;
                            }
                        }
                        
                        // Check if there are face-down cards at the original position that need flipping
                        // Since we can't access all_transform_query here due to query conflicts,
                        // we'll add the flip component more conservatively and let the flip system handle detection
                        // Only add it when moving to tableau positions (not empty piles)
                        if target_tableau_pos.is_some() || best_target.as_ref().map_or(false, |(_target_pos, _distance, target_type)| *target_type == "tableau") {
                            // Add the flip component - the flip system will check if there are actually cards to flip
                            // We'll be more permissive here and let the flip system do the actual position checking
                            commands.entity(selected_entity).insert(NeedsFlipUnderneath(original_position));
                                                // Added flip component
                        }
                    }
                } else if let Some(tableau_pos) = target_tableau_pos {
                    // Apply movement to empty tableau pile
                    transform.translation = tableau_pos;
                    commands.entity(selected_entity).insert(OriginalPosition(tableau_pos));
                    
                    // If this was a stock card, remove stock components and add tableau components
                    commands.entity(selected_entity)
                        .remove::<StockPile>()
                        .insert(TableauPile);
                    
                    // Move all stacked cards to the empty tableau pile
                    for (card_entity, stack_index) in &cards_to_move {
                        let stacked_y = tableau_pos.y - (*stack_index as f32 * 25.0);
                        let new_card_position = Vec3::new(
                            tableau_pos.x,
                            stacked_y,
                            tableau_pos.z + *stack_index as f32 + 1.0
                        );
                        
                        if let Ok(mut card_transform) = draggable_transform_query.get_mut(*card_entity) {
                            card_transform.translation = new_card_position;
                        }
                        
                        commands.entity(*card_entity).insert(OriginalPosition(new_card_position));
                        
                        commands.entity(*card_entity)
                            .remove::<WastePile>()
                            .remove::<SkippedWasteCard>()
                            .remove::<StockPile>()
                            .insert(TableauPile)
                            .insert(Draggable);
                    }
                    
                    // Update the visual stacking for the entire stack to ensure proper appearance
                    let mut all_stack_cards = vec![(selected_entity, 0)]; // Selected card is at index 0
                    all_stack_cards.extend(cards_to_move.iter().map(|(entity, index)| (*entity, *index + 1)));
                    
                    // Sort by stack index to ensure proper visual stacking
                    all_stack_cards.sort_by(|a, b| a.1.cmp(&b.1));
                    
                                            // Apply visual stacking offsets
                        for (stack_index, (card_entity, _)) in all_stack_cards.iter().enumerate() {
                            if let Ok(mut card_transform) = draggable_transform_query.get_mut(*card_entity) {
                                let visual_offset = stack_index as f32 * 25.0;
                                card_transform.translation.y = tableau_pos.y - visual_offset;
                                // Set Z position to ensure proper layering: each card gets a higher Z than the one below
                                card_transform.translation.z = tableau_pos.z + stack_index as f32 + 1.0;
                            }
                        }
                    
                    // Check if there are face-down cards at the original position that need flipping
                    // This is needed when moving to empty tableau piles as well
                    commands.entity(selected_entity).insert(NeedsFlipUnderneath(original_position));
                    // Added flip component for empty tableau move
                } else {
                    // If not dropped on any valid target or empty pile, snap back to original position
                    transform.translation = original_position;
                }
            }
        }
    }
}



pub fn flip_cards_system(
    mut commands: Commands,
    needs_flip_query: Query<(Entity, &NeedsFlipUnderneath), With<Card>>,
    all_transform_query: Query<&Transform, With<Card>>,
    all_card_data_query: Query<&CardData, With<Card>>,
    all_entity_query: Query<Entity, With<Card>>,
    asset_server: Res<AssetServer>,
    mut foundation_piles: ResMut<FoundationPiles>,
) {
    // Count entities that need flipping
    let needs_flip_count = needs_flip_query.iter().count();
    
    for (entity, needs_flip) in needs_flip_query.iter() {
        let original_position = needs_flip.0;
        
        // Processing flip for entity
        
        // Remove the component immediately to prevent duplicate processing
        commands.entity(entity).remove::<NeedsFlipUnderneath>();
        
        // Find face-down cards at the original position that need to be flipped
        let mut cards_at_position = Vec::new();
        
        // Collect all cards at the original X,Y position with more reasonable positioning
        for card_entity in all_entity_query.iter() { 
            if card_entity != entity { // Don't check the card that was moved
                if let Ok(transform) = all_transform_query.get(card_entity) {
                    if let Ok(card_data) = all_card_data_query.get(card_entity) {
                        // Skip cards that have already been flipped in this movement
                        if card_data.is_face_up {
                            continue;
                        }
                        
                        let x_distance = (transform.translation.x - original_position.x).abs();
                        let y_distance = (transform.translation.y - original_position.y).abs();
                        
                        // Use more reasonable thresholds: 5.0 for X and 15.0 for Y to allow for proper detection
                        // This prevents cards that are visually stacked but not exactly at the same position
                        if x_distance < 5.0 && y_distance < 15.0 {
                            // For tableau flipping, we want to be more permissive with Z positioning
                            // since cards are stacked closely together
                            // Allow cards that are at the same X,Y position regardless of Z
                            // Only consider cards that are actually underneath the moved card (lower Z position)
                            // Use a small tolerance since cards are stacked closely together
                            if transform.translation.z <= original_position.z + 0.5 {
                            cards_at_position.push((card_entity, transform.translation.z, card_data));
                                // Found potential flip card
                            }
                        }
                    }
                }
            }
        }
        
        // Sort by Z position to find the card that was directly underneath
        cards_at_position.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        // Only flip the card that was at the lowest Z position (closest to the table) that's face down
        // and only if we haven't already processed this entity
        if let Some((card_entity, _z_pos, card_data)) = cards_at_position.iter().find(|(_entity, _z_pos, card_data)| !card_data.is_face_up) {
            // Check if this card has already been flipped to prevent duplicate flips
            if !card_data.is_face_up {
                // Add a marker to prevent this card from being flipped again in this movement
                commands.entity(*card_entity).insert(AlreadyFlipped);
                
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
            commands.entity(*card_entity).remove::<CardBack>();
            commands.entity(*card_entity).insert(CardFront);
            
                // Card has been successfully flipped - no need for auto-move logic here
                // Users can manually move cards to foundation piles
            }
            
            // Only flip one card per movement, then exit
            break;
        }
    }
}



pub fn setup_initial_tableau_and_stock(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut stock_cards: ResMut<StockCards>,
) {
    // Create a standard 52-card deck
    let mut deck = Vec::new();
    let suits = [CardSuit::Hearts, CardSuit::Diamonds, CardSuit::Clubs, CardSuit::Spades];
    let values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
    
    for suit in suits {
        for value in values {
            deck.push((suit, value));
        }
    }
    
    // Shuffle the deck using a simple but effective algorithm
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    // Use a seed based on current time for some randomness
    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut hasher);
    let seed = hasher.finish();
    
    // Simple shuffle: swap each card with a random position
    for i in 0..deck.len() {
        let j = (seed.wrapping_add(i as u64 * 17)) % deck.len() as u64;
        let j = j as usize;
        deck.swap(i, j);
    }
    
    // Deal exactly 28 cards to the tableau (7 piles with 1, 2, 3, 4, 5, 6, 7 cards)
    let tableau_start_x = -(6.0 * 100.0) / 2.0;
    let tableau_y = WINDOW_HEIGHT / 2.0 - 250.0; // Moved down to align with top row
    
    let mut card_index = 0;
    for pile in 0..7 {
        let pile_size = pile + 1; // Stack 1 has 1 card, Stack 2 has 2 cards, etc.
        let x_pos = tableau_start_x + (pile as f32 * 100.0);
        
        for card_in_pile in 0..pile_size {
            if card_index < deck.len() {
                let (suit, value) = deck[card_index];
                
                // Only the top card of each pile is face-up
                let is_face_up = card_in_pile == pile_size - 1;
            
                // Calculate vertical offset for stacked appearance
                // Each card is offset downward by 25 pixels for reasonable visual spacing
                let vertical_offset = if card_in_pile == 0 {
                    0.0 // Bottom card stays at base position
                } else {
                    card_in_pile as f32 * 25.0 // Each card above is offset by 25 pixels
                };
                
                let y_pos = tableau_y - vertical_offset;
                
                // Create card entity
                let _card_entity = if is_face_up {
                    // Only face-up cards get Draggable component
                    create_card_entity(
                        commands,
                        &asset_server,
                        Vec3::new(x_pos, y_pos, card_in_pile as f32),
                        suit,
                        value,
                        is_face_up,
                        (
                            Draggable,
                            TableauPile,
                            OriginalPosition(Vec3::new(x_pos, y_pos, card_in_pile as f32)),
                            CoveredCard(None), // Top card is not covered
                        ),
                    )
                } else {
                    // Face-down cards are not draggable and get CardBack component
                    create_card_entity(
                        commands,
                        &asset_server,
                        Vec3::new(x_pos, y_pos, card_in_pile as f32),
                        suit,
                        value,
                        is_face_up,
                        (
                            TableauPile,
                            OriginalPosition(Vec3::new(x_pos, y_pos, card_in_pile as f32)),
                            CoveredCard(None),
                            CardBack, // Ensure face-down cards have CardBack component
                        ),
                    )
                };
                
                // Sprite is now handled by the create_card_entity helper function
                
                card_index += 1;
            }
        }
    }
    
    // Store the remaining 24 cards in the stock pile
    let remaining_cards: Vec<(CardSuit, u8)> = deck.iter().cloned().skip(28).collect();
    stock_cards.0 = remaining_cards;
    
    // Create stock pile above Stack 7 (rightmost stack)
    let stock_x = -(6.0 * 100.0) / 2.0 + (6.0 * 100.0); // Above Stack 7 (x = 300)
    let stock_y = WINDOW_HEIGHT / 2.0 - 100.0; // Above the tableau stacks
    
    // Create stock pile visual representation (always shows card back initially)
    create_card_entity(
        commands,
        &asset_server,
        Vec3::new(stock_x, stock_y, 0.0),
        CardSuit::Hearts, // Dummy suit - not important for stock pile
        1, // Dummy value - not important for stock pile
        false, // Always face down
        (
            StockPile,
            CardBack, // Add the CardBack component
        ),
    );
    
    // Update tableau positions resource
    let mut tableau_positions = Vec::new();
    for pile in 0..7 {
        let x_pos = tableau_start_x + (pile as f32 * 100.0);
        tableau_positions.push(Vec3::new(x_pos, tableau_y, 0.0));
    }
    commands.insert_resource(TableauPositions(tableau_positions));
}

pub fn waste_card_click_system(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window>,
    waste_cards: Query<(Entity, &Transform, &CardData), (With<WastePile>, Without<SkippedWasteCard>)>,
    tableau_cards: Query<(Entity, &Transform, &CardData), (With<TableauPile>, Without<WastePile>)>,
    tableau_positions: Res<TableauPositions>,
    mut foundation_piles: ResMut<FoundationPiles>,
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
            

            
            // Check if this waste card can be moved to any tableau position
            let mut best_target: Option<(Vec3, f32)> = None;
        
            // First check if it can be placed on existing tableau cards
            for (_tableau_entity, tableau_transform, tableau_card_data) in tableau_cards.iter() {
                // Only consider face-up cards as valid targets
                if !tableau_card_data.is_face_up {
                    continue;
                }
                
                // Check if this is a valid placement (card value must be one lower AND colors must alternate)
                if can_place_on_card(waste_card_data.value, tableau_card_data.value) {
                // Additional check: colors must alternate (red on black, black on red)
                let waste_is_red = matches!(waste_card_data.suit, CardSuit::Hearts | CardSuit::Diamonds);
                let tableau_is_red = matches!(tableau_card_data.suit, CardSuit::Hearts | CardSuit::Diamonds);
                
                // Debug: Print suit compatibility information
                
                if waste_is_red != tableau_is_red { // Colors must be different
                    // Additional check: make sure we're not placing on a card that's already covered
                    // Only place on the top card of each stack
                    let mut is_top_card = true;
                    for (other_entity, other_transform, _) in tableau_cards.iter() {
                        if other_entity != _tableau_entity {
                            // Check if this other card is on top of our target
                            let x_same = (other_transform.translation.x - tableau_transform.translation.x).abs() < 5.0;
                            let y_same = (other_transform.translation.y - tableau_transform.translation.y).abs() < 5.0;
                            let z_higher = other_transform.translation.z > tableau_transform.translation.z;
                            
                            if x_same && y_same && z_higher {
                                is_top_card = false;
                                break;
                            }
                        }
                    }
                    
                    if is_top_card {
                        let distance = (waste_transform.translation - tableau_transform.translation).length();
                        
                        if let Some((_target_pos, current_distance)) = best_target {
                            if distance < current_distance {
                                best_target = Some((tableau_transform.translation, distance));
                            }
                        } else {
                            best_target = Some((tableau_transform.translation, distance));
                        }
                    } else {
                        
                    }
                } else {
                    // Debug: Print why cards can't be placed together
                    
                }
            } else {
                // Debug: Print why cards can't be placed together (wrong values)
                
            }
            }
            
            // If no valid tableau card found, check if it can be placed on empty tableau piles
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
            
                        // If still no valid target, check if it can be placed on Foundation Piles
            if best_target.is_none() {
                // Check Foundation Piles - Aces can go on empty piles, other cards can go on matching suits
                let foundation_start_x = -(6.0 * 100.0) / 2.0;
                let foundation_y = WINDOW_HEIGHT / 2.0 - 100.0;
                
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
            }
            
            // If we found a valid target, move the waste card there
            if let Some((target_pos, _)) = best_target {
                // Check if this is a Foundation Pile placement
                let foundation_start_x = -(6.0 * 100.0) / 2.0;
                let foundation_y = WINDOW_HEIGHT / 2.0 - 100.0;
                let is_foundation_placement = (target_pos.y - foundation_y).abs() < 5.0 
                    && (target_pos.x - foundation_start_x).abs() < 200.0; // Within foundation pile X range
                
                if is_foundation_placement {
                    // Placing on Foundation Pile
                    let new_position = target_pos;
                    

                    
                    // Update the FoundationPiles resource
                    let foundation_index = ((new_position.x - foundation_start_x) / 100.0) as usize;
                                                foundation_piles.0[foundation_index].push((waste_card_data.suit, waste_card_data.value));

                    
                    // Move the waste card
                    commands.entity(waste_entity)
                        .remove::<WastePile>()
                        .remove::<SkippedWasteCard>()
                        .insert(FoundationPile)
                        .insert(OriginalPosition(new_position));
                    
                    // Update the transform to avoid destroying visual stacking
                    commands.entity(waste_entity).insert(Transform::from_translation(new_position));
                } else {
                    // Placing on Tableau
                    // Find the highest Z position at this X,Y location to ensure we're on top
                    let mut highest_z = target_pos.z;
                    for (_entity, card_transform, _card_data) in tableau_cards.iter() {
                        let x_same = (card_transform.translation.x - target_pos.x).abs() < 5.0;
                        let y_same = (card_transform.translation.y - target_pos.y).abs() < 5.0;
                        if x_same && y_same && card_transform.translation.z > highest_z {
                            highest_z = card_transform.translation.z;
                        }
                    }
                    
                    // Calculate new position using the same stacking logic as tableau cards
                    // Position the card at the target location with proper stacking offset
                    let new_position = Vec3::new(target_pos.x, target_pos.y, highest_z + 1.0);
                    
                    // Move the waste card
                    commands.entity(waste_entity)
                        .remove::<WastePile>()
                        .remove::<SkippedWasteCard>()
                        .insert(TableauPile)
                        .insert(OriginalPosition(new_position))
                        .insert(Draggable); // Also add Draggable component so it can be moved
                    
                    // Update the transform to avoid destroying visual stacking
                    commands.entity(waste_entity).insert(Transform::from_translation(new_position));
                }
                
                // Mark all other waste cards as skipped since the top one moved
                for (entity, _transform, _card_data) in waste_cards.iter() {
                    if entity != waste_entity {
                        commands.entity(entity).insert(SkippedWasteCard);
                    }
                }
                
                // If this was placed on an existing card, we need to flip the card underneath
                // Check if there's a face-down card at the target position that needs flipping
                let target_x = target_pos.x;
                let target_y = target_pos.y;
                
                // Look for face-down cards at the target position that might need flipping
                // Only flip the card that was directly underneath the moved card
                for (_card_entity, card_transform, card_data) in tableau_cards.iter() {
                    if !card_data.is_face_up {
                        let card_x = card_transform.translation.x;
                        let card_y = card_transform.translation.y;
                        
                        // Check if this card is at the target position (within small tolerance)
                        if (card_x - target_x).abs() < 5.0 && (card_y - target_y).abs() < 5.0 {
                            // Instead of directly flipping, add the flip component to use the proper system
                            commands.entity(waste_entity).insert(NeedsFlipUnderneath(Vec3::new(target_x, target_y, 0.0)));
                            break; // Only flip one card
                        }
                    }
                }
            } else {
                
            }
        }
    }
}



pub fn validate_card_draggability_system(
    mut commands: Commands,
    tableau_cards: Query<(Entity, &Transform, &CardData), (With<TableauPile>, With<Draggable>)>,
    all_transform_query: Query<&Transform, With<Card>>,
) {
    // Check each draggable tableau card to ensure it's actually the top card
    for (entity, transform, _card_data) in tableau_cards.iter() {
        let mut is_top_card = true;
        
        // Check if any other card is on top of this one
        for other_entity in all_transform_query.iter() {
            if other_entity != transform {
                let x_same = (other_entity.translation.x - transform.translation.x).abs() < 5.0;
                let y_same = (other_entity.translation.y - transform.translation.y).abs() < 5.0;
                let z_higher = other_entity.translation.z > transform.translation.z + 0.5;
                
                if x_same && y_same && z_higher {
                    is_top_card = false;
                    break;
                }
            }
        }
        
        // Only remove Draggable if this card is definitely not the top card
        // and if it's not part of a stack that's being moved together
        if !is_top_card {
            // Check if this card is part of a valid stack (has cards below it)
            let mut has_cards_below = false;
            for other_entity in all_transform_query.iter() {
                if other_entity != transform {
                    let x_same = (other_entity.translation.x - transform.translation.x).abs() < 5.0;
                    let y_same = (other_entity.translation.y - transform.translation.y).abs() < 5.0;
                    let z_lower = other_entity.translation.z < transform.translation.z - 0.5;
                    
                    if x_same && y_same && z_lower {
                        has_cards_below = true;
                        break;
                    }
                }
            }
            
            // Only remove Draggable if this card is covered and has no cards below it
            if !has_cards_below {
                commands.entity(entity).remove::<Draggable>();
            }
        }
    }
}

pub fn cleanup_flip_markers_system(
    mut commands: Commands,
    already_flipped_query: Query<Entity, With<AlreadyFlipped>>,
) {
    // Remove the AlreadyFlipped component from all cards at the end of each frame
    // This prevents the component from persisting and blocking future flips
    for entity in already_flipped_query.iter() {
        commands.entity(entity).remove::<AlreadyFlipped>();
    }
}

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

pub fn update_tableau_visual_stacking_system(
    mut tableau_cards: Query<(Entity, &mut Transform, &CardData), (With<TableauPile>, Or<(With<CardFront>, With<Draggable>)>)>,
) {
    // Group cards by their X,Y position to identify stacks
    let mut stacks: std::collections::HashMap<(i32, i32), Vec<(Entity, f32, usize)>> = std::collections::HashMap::new();
    
    // Collect all tableau cards that are either face-up or draggable
    for (entity, transform, card_data) in tableau_cards.iter() {
        // Round to nearest 5 pixels to group cards that are "at the same position"
        let x_key = (transform.translation.x / 5.0).round() as i32;
        let y_key = (transform.translation.y / 5.0).round() as i32;
        let z_pos = transform.translation.z;
        
        stacks.entry((x_key, y_key)).or_insert_with(Vec::new).push((entity, z_pos, 0));
    }
    
    // Visual stacking system processing stacks
    
    // For each stack, sort by Z position and apply visual stacking
    for (_pos, mut cards) in stacks.iter_mut() {
        if cards.len() > 1 {
            // Sort by Z position (lowest Z = bottom of stack)
            cards.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            
            // Processing stack
            
            // Get the bottom card's Y position as the base for stacking
            let bottom_card = cards.first().unwrap();
            let base_y = if let Ok((_entity_id, transform, _card_data)) = tableau_cards.get(bottom_card.0) {
                transform.translation.y
            } else {
                continue; // Skip this stack if we can't get the bottom card
            };
            
            // Apply stacking offsets to all cards in the stack
            for (stack_index, (entity, _z_pos, _card_data)) in cards.iter().enumerate() {
                if let Ok((_entity_id, mut transform, _card_data)) = tableau_cards.get_mut(*entity) {
                    // Apply stacking offset: each card above gets a 25-pixel Y offset
                    // This ensures each card shows enough of itself to remain clickable
                    let stacked_y = base_y - (stack_index as f32 * 30.0);
                    
                    // Update the transform to show proper visual stacking
                    transform.translation.y = stacked_y;
                }
            }
        }
    }
}