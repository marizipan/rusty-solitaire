use bevy::prelude::*;
use crate::components::CardSuit;

// Direct mapping from filename to card data - more verbose but completely reliable
pub fn get_card_data_from_filename(filename: &str) -> Option<(CardSuit, u8)> {
    match filename {
        // Hearts (King) - format: KingCard{value}.png
        "sprites/cards/King/KingCardA.png" => Some((CardSuit::Hearts, 1)),
        "sprites/cards/King/KingCard2.png" => Some((CardSuit::Hearts, 2)),
        "sprites/cards/King/KingCard3.png" => Some((CardSuit::Hearts, 3)),
        "sprites/cards/King/KingCard4.png" => Some((CardSuit::Hearts, 4)),
        "sprites/cards/King/KingCard5.png" => Some((CardSuit::Hearts, 5)),
        "sprites/cards/King/KingCard6.png" => Some((CardSuit::Hearts, 6)),
        "sprites/cards/King/KingCard7.png" => Some((CardSuit::Hearts, 7)),
        "sprites/cards/King/KingCard8.png" => Some((CardSuit::Hearts, 8)),
        "sprites/cards/King/KingCard9.png" => Some((CardSuit::Hearts, 9)),
        "sprites/cards/King/KingCard10.png" => Some((CardSuit::Hearts, 10)),
        "sprites/cards/King/KingCardJ.png" => Some((CardSuit::Hearts, 11)),
        "sprites/cards/King/KingCardQ.png" => Some((CardSuit::Hearts, 12)),
        "sprites/cards/King/KingCardK.png" => Some((CardSuit::Hearts, 13)),
        
        // Clubs (EvilFerris) - format: EvilFerris{value}.png
        "sprites/cards/EvilFerris/EvilFerrisA.png" => Some((CardSuit::Clubs, 1)),
        "sprites/cards/EvilFerris/EvilFerris2.png" => Some((CardSuit::Clubs, 2)),
        "sprites/cards/EvilFerris/EvilFerris3.png" => Some((CardSuit::Clubs, 3)),
        "sprites/cards/EvilFerris/EvilFerris4.png" => Some((CardSuit::Clubs, 4)),
        "sprites/cards/EvilFerris/EvilFerris5.png" => Some((CardSuit::Clubs, 5)),
        "sprites/cards/EvilFerris/EvilFerris6.png" => Some((CardSuit::Clubs, 6)),
        "sprites/cards/EvilFerris/EvilFerris7.png" => Some((CardSuit::Clubs, 7)),
        "sprites/cards/EvilFerris/EvilFerris8.png" => Some((CardSuit::Clubs, 8)),
        "sprites/cards/EvilFerris/EvilFerris9.png" => Some((CardSuit::Clubs, 9)),
        "sprites/cards/EvilFerris/EvilFerris10.png" => Some((CardSuit::Clubs, 10)),
        "sprites/cards/EvilFerris/EvilFerrisJ.png" => Some((CardSuit::Clubs, 11)),
        "sprites/cards/EvilFerris/EvilFerrisQ.png" => Some((CardSuit::Clubs, 12)),
        "sprites/cards/EvilFerris/EvilFerrisK.png" => Some((CardSuit::Clubs, 13)),
        
        // Diamonds (Stabby) - format: StabbyCard{value}.png
        "sprites/cards/Stabby/StabbyCardA.png" => Some((CardSuit::Diamonds, 1)),
        "sprites/cards/Stabby/StabbyCard2.png" => Some((CardSuit::Diamonds, 2)),
        "sprites/cards/Stabby/StabbyCard3.png" => Some((CardSuit::Diamonds, 3)),
        "sprites/cards/Stabby/StabbyCard4.png" => Some((CardSuit::Diamonds, 4)),
        "sprites/cards/Stabby/StabbyCard5.png" => Some((CardSuit::Diamonds, 5)),
        "sprites/cards/Stabby/StabbyCard6.png" => Some((CardSuit::Diamonds, 6)),
        "sprites/cards/Stabby/StabbyCard7.png" => Some((CardSuit::Diamonds, 7)),
        "sprites/cards/Stabby/StabbyCard8.png" => Some((CardSuit::Diamonds, 8)),
        "sprites/cards/Stabby/StabbyCard9.png" => Some((CardSuit::Diamonds, 9)),
        "sprites/cards/Stabby/StabbyCard10.png" => Some((CardSuit::Diamonds, 10)),
        "sprites/cards/Stabby/StabbyCardJ.png" => Some((CardSuit::Diamonds, 11)),
        "sprites/cards/Stabby/StabbyCardQ.png" => Some((CardSuit::Diamonds, 12)),
        "sprites/cards/Stabby/StabbyCardK.png" => Some((CardSuit::Diamonds, 13)),
        
        // Spades (Corro) - format: CorroCard{value}.png
        "sprites/cards/Corro/CorroCardA.png" => Some((CardSuit::Spades, 1)),
        "sprites/cards/Corro/CorroCard2.png" => Some((CardSuit::Spades, 2)),
        "sprites/cards/Corro/CorroCard3.png" => Some((CardSuit::Spades, 3)),
        "sprites/cards/Corro/CorroCard4.png" => Some((CardSuit::Spades, 4)),
        "sprites/cards/Corro/CorroCard5.png" => Some((CardSuit::Spades, 5)),
        "sprites/cards/Corro/CorroCard6.png" => Some((CardSuit::Spades, 6)),
        "sprites/cards/Corro/CorroCard7.png" => Some((CardSuit::Spades, 7)),
        "sprites/cards/Corro/CorroCard8.png" => Some((CardSuit::Spades, 8)),
        "sprites/cards/Corro/CorroCard9.png" => Some((CardSuit::Spades, 9)),
        "sprites/cards/Corro/CorroCard10.png" => Some((CardSuit::Spades, 10)),
        "sprites/cards/Corro/CorroCardJ.png" => Some((CardSuit::Spades, 11)),
        "sprites/cards/Corro/CorroCardQ.png" => Some((CardSuit::Spades, 12)),
        "sprites/cards/Corro/CorroCardK.png" => Some((CardSuit::Spades, 13)),
        
        // No match found
        _ => None
    }
}

pub fn get_card_back_image(_suit: CardSuit) -> &'static str {
    "sprites/cards/CardBack.png"
}

pub fn get_card_front_image(suit: CardSuit, value: u8) -> String {
    let suit_name = match suit {
        CardSuit::Hearts => "King",
        CardSuit::Diamonds => "Stabby", 
        CardSuit::Clubs => "EvilFerris",
        CardSuit::Spades => "Corro",
    };
    
    let value_name = match value {
        1 => "A",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        8 => "8",
        9 => "9",
        10 => "10",
        11 => "J", 
        12 => "Q",
        13 => "K",
        _ => return format!("sprites/cards/error/invalid_value_{}.png", value), // Fallback for invalid values
    };
    
    // Different suits use different naming conventions:
    match suit {
        CardSuit::Hearts | CardSuit::Spades => {
            // Hearts (King), Spades (Corro): suitName + "Card" + valueName
            format!("sprites/cards/{}/{}Card{}.png", suit_name, suit_name, value_name)
        }
        CardSuit::Diamonds => {
            // Diamonds (Stabby): suitName + "Card" + valueName
            format!("sprites/cards/{}/{}Card{}.png", suit_name, suit_name, value_name)
        }
        CardSuit::Clubs => {
            // Clubs (EvilFerris): suitName + valueName (no "Card" prefix)
            format!("sprites/cards/{}/{}{}.png", suit_name, suit_name, value_name)
        }
    }
}

pub fn can_place_on_card(card_value: u8, target_card_value: u8) -> bool {
    // Cards can only be placed on cards with value +1 (descending order)
    // For example: Queen (12) on King (13), Jack (11) on Queen (12), etc.
    card_value == target_card_value - 1
}

pub fn is_red_suit(suit: CardSuit) -> bool {
    matches!(suit, CardSuit::Hearts | CardSuit::Diamonds)
}

pub fn can_place_on_tableau(card_value: u8, card_suit: CardSuit, target_value: u8, target_suit: CardSuit) -> bool {
    // Tableau placement rules: descending order with alternating colors
    can_place_on_card(card_value, target_value) && is_red_suit(card_suit) != is_red_suit(target_suit)
}

pub fn is_valid_stack_sequence(cards: &[(CardSuit, u8)]) -> bool {
    if cards.len() <= 1 {
        return true;
    }
    
    // Sort by value in descending order (highest to lowest)
    let mut sorted_cards = cards.to_vec();
    sorted_cards.sort_by(|a, b| b.1.cmp(&a.1));
    
    // Check if the sequence is valid (descending order with alternating colors)
    for i in 0..sorted_cards.len() - 1 {
        let current = sorted_cards[i];
        let next = sorted_cards[i + 1];
        
        // Check descending order (current value should be exactly one higher than next)
        if current.1 != next.1 + 1 {
            return false;
        }
        
        // Check alternating colors (red on black, black on red)
        if is_red_suit(current.0) == is_red_suit(next.0) {
            return false;
        }
    }
    
    true
}

pub fn has_complete_stack(cards: &[(CardSuit, u8)]) -> bool {
    // A complete stack must start with King (13) and end with Ace (1)
    // All cards must be in descending order with alternating colors
    if cards.is_empty() || cards[0].1 != 13 {
        return false;
    }
    
    // Must end with Ace (1)
    if cards.last().map_or(true, |card| card.1 != 1) {
        return false;
    }
    
    // Check if the sequence is valid
    is_valid_stack_sequence(cards)
}

pub fn is_in_waste_or_stock_area(position: Vec2) -> bool {
    // Check if a position is in the waste or stock pile areas
    // These areas should never accept card drops in solitaire
    let waste_x = -(6.0 * 100.0) / 2.0 + (5.0 * 100.0); // Above Stack 6 (x = 200)
    let waste_y = 260.0; // WINDOW_HEIGHT / 2.0 - 100.0
    let stock_x = -(6.0 * 100.0) / 2.0 + (6.0 * 100.0); // Above Stack 7 (x = 300)
    let stock_y = waste_y;
    
    let waste_distance = (position - Vec2::new(waste_x, waste_y)).length();
    let stock_distance = (position - Vec2::new(stock_x, stock_y)).length();
    
    // Use a generous detection radius to prevent any cards from being placed near these areas
    waste_distance < 80.0 || stock_distance < 80.0
}

pub fn can_place_on_foundation(card_data: &crate::components::CardData, foundation_pile: &Vec<(CardSuit, u8)>) -> bool {
    if foundation_pile.is_empty() {
        // Only Ace can start a foundation pile
        return card_data.value == 1;
    }
    
    // Get the top card of the foundation pile
    if let Some((top_suit, top_value)) = foundation_pile.last() {
        // Must be same suit and one higher value
        return card_data.suit == *top_suit && card_data.value == top_value + 1;
    }
    
    false
}

pub fn can_place_on_tableau_card(selected_card: &crate::components::CardData, target_card: &crate::components::CardData) -> bool {
    // Target card must be face up
    if !target_card.is_face_up {
        tracing::debug!("TABLEAU REJECTED: Target card is face down");
        return false;
    }
    
    // Use the existing validation function from utils
    let can_place = can_place_on_tableau(selected_card.value, selected_card.suit, target_card.value, target_card.suit);
    
    if !can_place {
        tracing::debug!("TABLEAU REJECTED: Card {:?} (value: {}, suit: {:?}) cannot be placed on {:?} (value: {}, suit: {:?}) - same color: {}, valid sequence: {}", 
                       selected_card.suit, selected_card.value, selected_card.suit,
                       target_card.suit, target_card.value, target_card.suit,
                       is_red_suit(selected_card.suit) == is_red_suit(target_card.suit),
                       can_place_on_card(selected_card.value, target_card.value));
    }
    
    can_place
}

/// Finds the best tableau target for a card, reusing logic from waste_click.rs
pub fn find_best_tableau_target(
    card_data: &crate::components::CardData,
    card_position: bevy::math::Vec3,
    tableau_cards: &[(bevy::prelude::Entity, bevy::math::Vec3, crate::components::CardData)],
    tableau_positions: &[bevy::math::Vec3],
    exclude_entity: Option<bevy::prelude::Entity>,
) -> Option<bevy::math::Vec3> {
    let mut best_target: Option<(bevy::math::Vec3, f32)> = None;
    
    // First, try to find a valid tableau card to place on
    for (entity, target_transform, target_card_data) in tableau_cards.iter() {
        // Skip the excluded entity
        if let Some(exclude) = exclude_entity {
            if *entity == exclude {
                continue;
            }
        }
        
        // Only consider face-up cards as valid targets
        if !target_card_data.is_face_up {
            continue;
        }
        
        // Check if this is a valid placement
        if can_place_on_tableau_card(card_data, target_card_data) {
            tracing::debug!("TABLEAU VALID: Card {:?} (value: {}, suit: {:?}) can be placed on {:?} (value: {}, suit: {:?})", 
                           card_data.suit, card_data.value, card_data.suit,
                           target_card_data.suit, target_card_data.value, target_card_data.suit);
            
            // Check if this is the top card of its stack
            let mut is_top_card = true;
            for (other_entity, other_transform, _other_card_data) in tableau_cards.iter() {
                if *other_entity != *entity {
                    if let Some(exclude) = exclude_entity {
                        if *other_entity == exclude {
                            continue;
                        }
                    }
                    
                    // Check if this other card is on top of our target
                    let x_same = (other_transform.x - target_transform.x).abs() < 5.0;
                    let y_same = (other_transform.y - target_transform.y).abs() < 5.0;
                    let z_higher = other_transform.z > target_transform.z;
                    
                    if x_same && y_same && z_higher {
                        is_top_card = false;
                        break;
                    }
                }
            }
            
            if is_top_card {
                let distance = (card_position - *target_transform).length();
                
                if let Some((_target_pos, current_distance)) = best_target {
                    if distance < current_distance {
                        best_target = Some((*target_transform, distance));
                    }
                } else {
                    best_target = Some((*target_transform, distance));
                }
            }
        }
    }
    
    // If no valid tableau card found, try empty tableau positions (only for Kings)
    if best_target.is_none() && card_data.value == 13 {
        for tableau_pos in tableau_positions {
            // Check if this tableau position is empty
            let mut is_empty = true;
            for (_other_entity, other_transform, _other_card_data) in tableau_cards.iter() {
                if (other_transform.x - tableau_pos.x).abs() < 5.0 
                    && (other_transform.y - tableau_pos.y).abs() < 5.0 {
                    is_empty = false;
                    break;
                }
            }
            
            if is_empty {
                best_target = Some((*tableau_pos, 0.0));
                break;
            }
        }
    }
    
    best_target.map(|(pos, _)| pos)
}
