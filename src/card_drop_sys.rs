use bevy::prelude::*;
use crate::components::*;
use crate::utils::{is_red_suit, is_valid_stack_sequence, is_in_waste_or_stock_area};



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
            
            // First, collect all occupied positions to avoid targeting them
            let mut occupied_positions = Vec::new();
            for entity in &draggable_entity_query {
                if entity == selected_entity {
                    continue; // Skip the card being dragged
                }
                
                if let Ok(target_transform) = draggable_transform_query.get(entity) {
                    let target_pos = target_transform.translation;
                    // Check if this is a tableau card by looking at its position
                    if target_pos.y >= -300.0 && target_pos.y <= 100.0 {
                        // This is a tableau card - mark its position as occupied
                        occupied_positions.push(target_pos);
                    }
                }
            }
            
            // Now find valid target positions (positions where cards can be placed)
            for entity in &draggable_entity_query {
                if entity == selected_entity {
                    continue; // Skip the card being dragged
                }
                
                if let Ok(target_card_data) = card_data_query.get(entity) {
                    // Only consider face-up tableau cards as potential targets
                    if !target_card_data.is_face_up {
                        continue;
                    }
                    
                    if let Ok(target_transform) = draggable_transform_query.get(entity) {
                        let target_pos = target_transform.translation;
                        
                        // CRITICAL: Prevent targeting cards that are in waste or stock pile areas
                        if is_in_waste_or_stock_area(target_pos.truncate()) {
                            continue;
                        }
                        
                        // Check if this is a tableau card by looking at its position
                        if target_pos.y < -300.0 || target_pos.y > 100.0 {
                            continue;
                        }
                        
                        // CRITICAL FIX: Check if this position is actually available for placement
                        // A position is available if it's the top card of a stack (no other cards above it)
                        // In Solitaire, cards are stacked with Y offsets, not Z offsets
                        let mut is_top_card = true;
                        for other_entity in &draggable_entity_query {
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
                            let distance = (cursor_world_pos - target_pos.truncate()).length();
                            potential_targets.push((target_pos, distance, target_card_data.value, target_card_data.suit));
                        } else {
                        }
                    }
                }
            }
            
            
            // Check tableau pile positions
            for tableau_pos in &tableau_positions.0 {
                let distance = (cursor_world_pos - tableau_pos.truncate()).length();
                if distance < 50.0 {
                    // CRITICAL: Ensure we're not placing on waste or stock pile areas
                    if is_in_waste_or_stock_area(tableau_pos.truncate()) {
                        continue;
                    }
                    
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
            let mut target_tableau_pos: Option<Vec3> = None;
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
                                // Check if this card is part of the same stack
                                // Cards in a stack should have the same X position and be directly stacked above
                                // In Solitaire: cards are stacked with 30px Y offsets, so we need to check for exact stacking
                                let x_tolerance = 15.0; // Same X position tolerance
                                
                                let x_same = (card_transform.translation.x - original_position.x).abs() < x_tolerance;
                                
                                // A card is part of the stack if it's at the same X position AND directly stacked above
                                // In Solitaire: cards are stacked with 30px Y offsets, bottom card has highest Y, cards above have lower Y
                                // We need to check if this card is positioned exactly where a stacked card should be
                                let y_difference = original_position.y - card_transform.translation.y;
                                
                                // Check if this card is positioned at a valid stack offset (multiple of 30px)
                                // and is within a reasonable stack height (max 10 cards)
                                let stack_offset = (y_difference / 30.0).round();
                                let is_valid_stack_offset = stack_offset > 0.0 && stack_offset <= 10.0;
                                let is_exact_stack_position = (y_difference - (stack_offset * 30.0)).abs() < 10.0; // More tolerance
                                
                                let is_part_of_stack = x_same && is_valid_stack_offset && is_exact_stack_position;
                                
                                
                                if is_part_of_stack {
                                    // CRITICAL: Only include face-up cards in the stack
                                    if card_data.is_face_up {
                                        // Calculate the relative position in the stack based on Y offset
                                        // In Solitaire stacking: bottom card has highest Y, cards above have lower Y
                                        // Each card is offset by 30px, so calculate stack index from Y difference
                                        let y_difference = original_position.y - card_transform.translation.y;
                                        let stack_index = (y_difference / 30.0).round() as u32;
                                        
                                        // Include cards that are above the selected card (positive stack index)
                                        // Since cards above have lower Y values, y_difference will be positive
                                        if stack_index > 0 {
                                            cards_to_move.push((card_entity, stack_index));
                                            stack_cards.push((card_data.suit, card_data.value));
                                            
                                        }
                                    }
                                    // Face-down cards are ignored and will not be moved
                                }
                            }
                        }
                    }
                }
                
                // Add the selected card to the stack
                stack_cards.push((selected_suit, selected_value));
                
                // VALIDATE STACK SEQUENCE BEFORE PROCEEDING
                if stack_cards.len() > 1 {
                    let mut is_valid_stack = true;
                    let mut sorted_stack = stack_cards.clone();
                    sorted_stack.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by value descending
                    
                    // Check if the stack is in valid sequence (descending order with alternating colors)
                    for i in 0..sorted_stack.len() - 1 {
                        let current = sorted_stack[i];
                        let next = sorted_stack[i + 1];
                        
                        // Check descending order
                        if current.1 != next.1 + 1 {
                            is_valid_stack = false;
                            break;
                        }
                        
                        // Check alternating colors
                        if is_red_suit(current.0) == is_red_suit(next.0) {
                            is_valid_stack = false;
                            break;
                        }
                    }
                    
                    if !is_valid_stack {
                        // Clear the invalid stack
                        stack_cards.clear();
                        cards_to_move.clear();
                        // Only keep the selected card
                        stack_cards.push((selected_suit, selected_value));
                    }
                }
                
            }
            
            // CRITICAL: Prevent cards from being dropped on stock pile or waste pile
            // Cards can NEVER be stacked on waste or stock piles in solitaire
            let stock_x = -(6.0 * 100.0) / 2.0 + (6.0 * 100.0); // Above Stack 7 (x = 300)
            let stock_y = WINDOW_HEIGHT / 2.0 - 100.0; // Above the tableau stacks
            let waste_x = -(6.0 * 100.0) / 2.0 + (5.0 * 100.0); // Above Stack 6 (x = 200)
            let waste_y = stock_y;
            
            // Check distance to stock pile center
            let stock_distance = (cursor_world_pos - Vec2::new(stock_x, stock_y)).length();
            // Check distance to waste pile center  
            let waste_distance = (cursor_world_pos - Vec2::new(waste_x, waste_y)).length();
            
            // Use the utility function to check if we're in waste/stock areas
            if is_in_waste_or_stock_area(cursor_world_pos) {
                // Don't allow dropping on stock pile or waste pile - snap back to original position
                if let Ok(mut transform) = draggable_transform_query.get_mut(selected_entity) {
                    transform.translation = original_position;
                }
                
                // Also snap back any stacked cards
                for (card_entity, stack_index) in &cards_to_move {
                    if let Ok(mut card_transform) = draggable_transform_query.get_mut(*card_entity) {
                        let original_stacked_y = original_position.y - (*stack_index as f32 * 30.0);
                        let original_stacked_z = original_position.z + *stack_index as f32 + 1.0;
                        let original_stacked_pos = Vec3::new(
                            original_position.x,
                            original_stacked_y,
                            original_stacked_z
                        );
                        card_transform.translation = original_stacked_pos;
                        commands.entity(*card_entity).insert(OriginalPosition(original_stacked_pos));
                    }
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
            
            
            // Only allow foundation pile placement if cursor is actually near the foundation pile Y position
            // Also ensure we're not near waste or stock pile areas
            if (cursor_world_pos.y - foundation_y).abs() < 250.0 
                && !is_in_waste_or_stock_area(cursor_world_pos) {
                for i in 0..4 {
                    let foundation_x = foundation_start_x + (i as f32 * 100.0);
                    let foundation_distance = (cursor_world_pos - Vec2::new(foundation_x, foundation_y)).length();
                    
                    
                    // Use a larger detection radius for foundation piles to make them reachable
                    if foundation_distance < 120.0 { // Increased from 60.0 to make foundations reachable
                        // Check if this card can be placed on this foundation pile
                        if let Ok(card_data) = card_data_query.get(selected_entity) {
                            let foundation_pile = &foundation_piles.0[i];
                            
                        
                        // Allow foundation placement if:
                        // 1. This is an Ace for empty foundation piles, OR
                        // 2. This is the next card in sequence for a non-empty foundation pile
                        let can_place_on_foundation = if foundation_pile.is_empty() {
                            // Empty foundation pile - only Aces can be placed via drag
                            let result = card_data.value == 1;
                            result
                        } else {
                            // Foundation pile has cards - check if this card can be added
                            let (top_suit, top_value) = foundation_pile.last().unwrap();
                            let is_next_in_sequence = card_data.suit == *top_suit && card_data.value == top_value + 1;
                            is_next_in_sequence
                        };
                    
                        if can_place_on_foundation {
                            best_target = Some((Vec3::new(foundation_x, foundation_y, 1.0), 0.0, "foundation"));
                            break;
                        } else {
                        }
                        } else {
                        } // Close the if let Ok(card_data) block
                    } else {
                    }
                    // Don't break here - continue checking other foundation piles
                    // Only break if we found a valid target
                }
            } else {
            }
            
            // Foundation pile validation is already handled above
            
            // Check tableau targets only if we haven't found a foundation pile target
            if best_target.is_none() {
                // PRIORITY: Check empty tableau piles first for Kings
                // This ensures Kings go to empty piles instead of being placed on existing cards
                for (tableau_pos, is_empty) in &tableau_pile_info {
                    if *is_empty && selected_value == 13 { // King can go to empty tableau pile
                        let distance = (cursor_world_pos - tableau_pos.truncate()).length();
                        if distance < 50.0 { // Within reasonable distance
                            target_tableau_pos = Some(*tableau_pos);
                            break;
                        }
                    }
                }
                
                // If we found an empty tableau pile for the King, use it
                if target_tableau_pos.is_some() {
                    best_target = Some((target_tableau_pos.unwrap(), 0.0, "empty_tableau"));
                } else {
                    // No empty tableau pile available, check existing tableau cards
                    for (target_pos, distance, target_value, target_suit) in &potential_targets {
                        // VALIDATION: The selected card must be one value lower than the target
                        // In Solitaire: place a 5 on a 6, place a 4 on a 5, etc.
                        if selected_value != target_value - 1 {
                            continue; // Skip invalid tableau placements
                        }
                        
                        // Check if colors alternate (red on black, black on red)
                        if is_red_suit(selected_suit) == is_red_suit(*target_suit) {
                            continue; // Colors are the same - this placement is invalid
                        }
                        
                        
                        // STACK VALIDATION: Validate the entire stack structure
                        if stack_cards.len() > 1 {
                            if !is_valid_stack_sequence(&stack_cards) {
                                continue; // Skip invalid stack placements
                            }
                            
                        }
                        
                        // The target is already validated as available during the initial collection phase
                        // We can trust that it's the top card of its stack
                        // Allow placement on top of cards (we'll handle Z positioning when actually moving)
                        if let Some((_target_pos, current_distance, _target_type)) = best_target {
                            if *distance < current_distance {
                                best_target = Some((*target_pos, *distance, "tableau"));
                            }
                        } else {
                            best_target = Some((*target_pos, *distance, "tableau"));
                        }
                    }
                }
            }
            
            // Note: Tableau validation is already handled in the first loop above
            // No duplicate validation needed here
            
            // Now get the selected card data and apply the movement
            if let Ok(mut transform) = draggable_transform_query.get_mut(selected_entity) {
                if let Some((target_pos, _distance, target_type)) = best_target {
                    if target_type == "foundation" {
                        // Placing on foundation pile - SIMPLIFIED LOGIC
                        let foundation_start_x = -(6.0 * 100.0) / 2.0;
                        let foundation_index = ((target_pos.x - foundation_start_x) / 100.0) as usize;
                        
                        // Get foundation pile info before any mutable borrows
                        let foundation_pile_info = foundation_piles.0[foundation_index].clone();
                        let card_count = foundation_pile_info.len() as f32;
                        
                        // Position the new card above existing cards with proper Z layering
                        let new_position = Vec3::new(
                            target_pos.x,
                            target_pos.y,
                            target_pos.z + card_count + 1.0 // Stack above existing cards
                        );
                        
                        transform.translation = new_position;
                        commands.entity(selected_entity).insert(OriginalPosition(new_position));
                        
                        if let Ok(card_data) = card_data_query.get(selected_entity) {
                            // Add the card to the foundation pile stack
                            foundation_piles.0[foundation_index].push((card_data.suit, card_data.value));
                        }
                        
                        // Foundation placement: Only the top card goes to foundation
                        // VALIDATION: Only single cards can be placed on foundation piles
                        if cards_to_move.len() > 0 {
                            // Snap back to original position
                            transform.translation = original_position;
                            
                            // Also snap back any stacked cards
                            for (card_entity, stack_index) in &cards_to_move {
                                if let Ok(mut card_transform) = draggable_transform_query.get_mut(*card_entity) {
                                    let original_stacked_y = original_position.y - (*stack_index as f32 * 30.0);
                                    let original_stacked_z = original_position.z + *stack_index as f32 + 1.0;
                                    let original_stacked_pos = Vec3::new(
                                        original_position.x,
                                        original_stacked_y,
                                        original_stacked_z
                                    );
                                    
                                    card_transform.translation = original_stacked_pos;
                                    commands.entity(*card_entity).insert(OriginalPosition(original_stacked_pos));
                                }
                            }
                            // Don't return here - continue to the end to ensure proper cleanup
                        } else {
                            
                            // Remove tableau/waste/stock components and add foundation component
                            commands.entity(selected_entity)
                                .remove::<TableauPile>()
                                .remove::<WastePile>()
                                .remove::<SkippedWasteCard>()
                                .remove::<StockPile>()
                                .remove::<Draggable>() // Foundation cards cannot be moved
                                .insert(FoundationPile);
                        }
                        
                    } else if target_type == "empty_tableau" {
                        // Empty tableau placement: Only Kings can be placed on empty tableau positions
                        // VALIDATION: Check if the selected card is a King
                        if selected_value != 13 { // 13 = King
                            // Snap back to original position
                            transform.translation = original_position;
                            
                            // Also snap back any stacked cards
                            for (card_entity, stack_index) in &cards_to_move {
                                if let Ok(mut card_transform) = draggable_transform_query.get_mut(*card_entity) {
                                    let original_stacked_y = original_position.y - (*stack_index as f32 * 30.0);
                                    let original_stacked_z = original_position.z + *stack_index as f32 + 1.0;
                                    let original_stacked_pos = Vec3::new(
                                        original_position.x,
                                        original_stacked_y,
                                        original_stacked_z
                                    );
                                    
                                    card_transform.translation = original_stacked_pos;
                                    commands.entity(*card_entity).insert(OriginalPosition(original_stacked_pos));
                                }
                            }
                            return; // Skip this movement
                        }
                        
                        
                        // Position the selected card at the empty tableau position
                        let new_position = Vec3::new(target_pos.x, target_pos.y, target_pos.z);
                        transform.translation = new_position;
                        
                        // Update the original position for future reference
                        commands.entity(selected_entity).insert(OriginalPosition(new_position));
                        
                        // Move all stacked cards to maintain their relative positions with proper stacking
                        for (card_entity, stack_index) in &cards_to_move {
                            // Use stacking offset: 30 pixels per card to show enough of each card
                            let stacked_y = new_position.y - (*stack_index as f32 * 30.0);
                            // Z position should match the visual stacking: each card gets a Z offset that corresponds to its visual position
                            let new_card_position = Vec3::new(
                                new_position.x, 
                                stacked_y, // Stack cards with 30px offset for reasonable visual spacing
                                new_position.z + *stack_index as f32 + 2.0 // +2.0 to ensure proper layering above the target card
                            );
                            
                            
                            // Update the transform directly for stacked cards
                            if let Ok(mut card_transform) = draggable_transform_query.get_mut(*card_entity) {
                                card_transform.translation = new_card_position;
                            } else {
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
                        
                    } else {
                        // Placing on tableau
                        
                        
                        // CRITICAL: Validate that this placement is actually legal
                        // The card must be one value lower than the target AND have alternating colors
                        // Note: target_value and target_suit are available from the loop above
                        if let Some((_target_pos, _distance, _target_type)) = best_target {
                            // Validation is already done in the target selection loop above
                            // This is just a safety check
                        }
                        
                        // Basic validation: only check if the top card can be placed on the target
                        // The stack validation is handled during the drag detection phase
                        // This allows for more flexible movement while maintaining game rules
                        
                        // Position the selected card on top of the target card
                        let new_position = Vec3::new(target_pos.x, target_pos.y, target_pos.z + 1.0);
                        transform.translation = new_position;
                        
                        // Update the original position for future reference
                        commands.entity(selected_entity).insert(OriginalPosition(new_position));
                        
                        // Move all stacked cards to maintain their relative positions with proper stacking
                        for (card_entity, stack_index) in &cards_to_move {
                            // Use stacking offset: 30 pixels per card to show enough of each card
                            let stacked_y = new_position.y - (*stack_index as f32 * 30.0);
                            // Z position should match the visual stacking: each card gets a Z offset that corresponds to its visual position
                            let new_card_position = Vec3::new(
                                new_position.x, 
                                stacked_y, // Stack cards with 30px offset for reasonable visual spacing
                                new_position.z + *stack_index as f32 + 2.0 // +2.0 to ensure proper layering above the target card
                            );
                            
                            
                            // Update the transform directly for stacked cards
                            if let Ok(mut card_transform) = draggable_transform_query.get_mut(*card_entity) {
                                card_transform.translation = new_card_position;
                            } else {
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
                        
                        // Visual stacking is already handled by the individual card positioning above
                        // No need to override positions here
                        
                        // Check if there are face-down cards at the ORIGINAL position that need flipping
                        // This should happen where the card came FROM, not where it went TO
                        // Only add flip component if we're moving to a valid tableau position
                        if target_tableau_pos.is_some() || best_target.as_ref().map_or(false, |(_target_pos, _distance, target_type)| *target_type == "tableau") {
                            // Check if the selected card is face-up before adding flip component
                            if let Ok(selected_card_data) = card_data_query.get(selected_entity) {
                                if selected_card_data.is_face_up {
                                    // Create a separate entity to trigger the flip system
                                    // This avoids the issue of adding the component to the moved card
                                    commands.spawn(NeedsFlipUnderneath(original_position));
                                }
                            }
                        }
                    }
                } else if let Some(tableau_pos) = target_tableau_pos {
                    // Apply movement to empty tableau pile
                    
                    // For empty tableau piles, only require that the top card is a King
                    // The stack validation is handled during the drag detection phase
                    // This allows for more flexible movement while maintaining game rules
                    if let Some((_top_suit, top_value)) = stack_cards.first() {
                        // Only Kings (value 13) can be placed on empty tableau piles
                        if *top_value != 13 {
                            // Not a King - snap back to original position
                            transform.translation = original_position;
                            return;
                        }
                    }
                    
                    transform.translation = tableau_pos;
                    commands.entity(selected_entity).insert(OriginalPosition(tableau_pos));
                    
                    // If this was a stock card, remove stock components and add tableau components
                    commands.entity(selected_entity)
                        .remove::<StockPile>()
                        .insert(TableauPile);
                    
                    // Move all stacked cards to the empty tableau pile
                    for (card_entity, stack_index) in &cards_to_move {
                        let stacked_y = tableau_pos.y - (*stack_index as f32 * 30.0);
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
                    for (stack_index, (card_entity, _stack_index)) in all_stack_cards.iter().enumerate() {
                        if let Ok(mut card_transform) = draggable_transform_query.get_mut(*card_entity) {
                            let visual_offset = stack_index as f32 * 30.0;
                            card_transform.translation.y = tableau_pos.y - visual_offset;
                            // Set Z position to ensure proper layering: each card gets a higher Z than the one below
                            card_transform.translation.z = tableau_pos.z + stack_index as f32 + 1.0;
                        }
                    }
                    
                                            // Check if there are face-down cards at the ORIGINAL position that need flipping
                        // This should happen where the card came FROM, not where it went TO
                        // Check if the selected card is face-up before adding flip component
                        if let Ok(selected_card_data) = card_data_query.get(selected_entity) {
                            if selected_card_data.is_face_up {
                                // Create a separate entity to trigger the flip system
                                // This avoids the issue of adding the component to the moved card
                                commands.spawn(NeedsFlipUnderneath(original_position));
                            }
                        }
                } // Close the else if let Some(tableau_pos) = target_tableau_pos block
            } else {
                // If not dropped on any valid target or empty pile, snap back to original position
                if let Ok(mut transform) = draggable_transform_query.get_mut(selected_entity) {
                    transform.translation = original_position;
                }
            }
        } // Close the if let Ok(mut transform) block
    } // Close the cursor_pos block
} // Close the selected_entity block

