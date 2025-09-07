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
        
        // Check descending order (current value should be higher than next)
        if current.1 <= next.1 {
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
        return false;
    }
    
    // Use the existing validation function from utils
    can_place_on_tableau(selected_card.value, selected_card.suit, target_card.value, target_card.suit)
}
