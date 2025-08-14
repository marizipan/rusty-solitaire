use crate::components::CardSuit;

pub fn get_card_back_image(_suit: CardSuit) -> &'static str {
    "sprites/cards/CardBack.png"
}

pub fn get_card_front_image(suit: CardSuit, value: u8) -> String {
    let suit_name = match suit {
        CardSuit::Hearts => "King",
        CardSuit::Diamonds => "EvilFerris", 
        CardSuit::Clubs => "Stabby",
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
        _ => &value.to_string(),
    };
    
    // Check if the suit needs "Card" prefix
    let needs_card_prefix = matches!(suit, CardSuit::Hearts | CardSuit::Clubs | CardSuit::Spades);
    
    if needs_card_prefix {
        format!("sprites/cards/{}/{}Card{}.png", suit_name, suit_name, value_name)
    } else {
        format!("sprites/cards/{}/{}{}.png", suit_name, suit_name, value_name)
    }
}

pub fn get_card_display_value(value: u8) -> String {
    match value {
        1 => "A".to_string(),
        2 => "2".to_string(),
        3 => "3".to_string(),
        4 => "4".to_string(),
        5 => "5".to_string(),
        6 => "6".to_string(),
        7 => "7".to_string(),
        8 => "8".to_string(),   
        9 => "9".to_string(),
        10 => "10".to_string(),
        11 => "J".to_string(),
        12 => "Q".to_string(),
        13 => "K".to_string(),
        _ => value.to_string(),
    }
}

pub fn create_deck() -> Vec<(CardSuit, u8)> {
    let mut deck = Vec::new();
    let suits = [CardSuit::Hearts, CardSuit::Diamonds, CardSuit::Clubs, CardSuit::Spades];
    let values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
    
    for suit in suits {
        for value in values {
            deck.push((suit, value));
        }
    }
    
    // Shuffle the deck (simplified - just reverse for now)
    deck.reverse();
    deck
}

pub fn can_place_on_empty_tableau(card_value: u8) -> bool {
    // Only cards with value 13 (Kings) can be placed on empty tableau piles
    card_value == 13
} 

pub fn can_place_on_king(card_value: u8) -> bool {
    // Only a Queen (value 12) can be placed on top of a King (value 13)
    match card_value {
        12 => true,  // Queen can be placed on King
        _ => false,  // All other cards cannot be placed on King
    }
}

pub fn can_place_on_card(card_value: u8, target_card_value: u8) -> bool {
    // Cards can only be placed on cards with value +1 (descending order)
    // For example: Queen (12) on King (13), Jack (11) on Queen (12), etc.
    card_value == target_card_value - 1
}
