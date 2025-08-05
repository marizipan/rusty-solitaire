use crate::components::CardSuit;

pub fn get_card_back_image(_suit: CardSuit) -> &'static str {
    "sprites/cards/CardBack.png"
}

pub fn get_card_front_image(suit: CardSuit, value: u8) -> String {
    let (suit_name, use_card_prefix) = match suit {
        CardSuit::Hearts => ("King", true),
        CardSuit::Diamonds => ("EvilFerris", false), 
        CardSuit::Clubs => ("Stabby", true),
        CardSuit::Spades => ("Corro", true),
    };
    
    let value_name = match value {
        1 => "A",
        11 => "J", 
        12 => "Q",
        13 => "K",
        _ => &value.to_string(),
    };
    
    if use_card_prefix {
        format!("sprites/cards/{}/{}Card{}.png", suit_name, suit_name, value_name)
    } else {
        format!("sprites/cards/{}/{}{}.png", suit_name, suit_name, value_name)
    }
}

pub fn get_card_display_value(value: u8) -> String {
    match value {
        1 => "A".to_string(),
        11 => "J".to_string(),
        12 => "Q".to_string(),
        13 => "K".to_string(),
        _ => value.to_string(),
    }
}

pub fn create_deck() -> Vec<(CardSuit, u8)> {
    let mut deck = Vec::new();
    let suits = [CardSuit::Hearts, CardSuit::Diamonds, CardSuit::Clubs, CardSuit::Spades];
    
    for suit in suits {
        for value in 1..=13 {
            deck.push((suit, value));
        }
    }
    
    // Shuffle the deck (simplified - just reverse for now)
    deck.reverse();
    deck
}

// Solitaire rules helper functions
pub fn can_place_on_foundation(card_value: u8, _foundation_suit: Option<CardSuit>) -> bool {
    // Foundation starts with Ace (1), then builds up in same suit
    if card_value == 1 {
        return true; // Ace can always start a foundation
    }
    // For now, allow any card on foundation (simplified)
    true
}

pub fn can_place_on_tableau(card_value: u8, card_suit: CardSuit, tableau_card_value: u8, tableau_card_suit: CardSuit) -> bool {
    // Tableau builds down in alternating colors
    let is_red = matches!(card_suit, CardSuit::Hearts | CardSuit::Diamonds);
    let tableau_is_red = matches!(tableau_card_suit, CardSuit::Hearts | CardSuit::Diamonds);
    
    // Must be different color and one value lower
    // In solitaire: King (13) can be placed on empty tableau piles
    // Other cards must be placed on cards with value one higher
    if card_value == 13 {
        // Kings can be placed on empty tableau piles (we'll handle this separately)
        return false; // For now, don't allow kings on other cards
    } else {
        // Other cards must be placed on cards with value one higher
        is_red != tableau_is_red && card_value == tableau_card_value - 1
    }
} 