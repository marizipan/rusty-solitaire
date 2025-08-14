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
    
    // Different suits use different naming conventions:
    match suit {
        CardSuit::Hearts | CardSuit::Clubs | CardSuit::Spades => {
            // Hearts (King), Clubs (Stabby), Spades (Corro): suitName + "Card" + valueName
            format!("sprites/cards/{}/{}Card{}.png", suit_name, suit_name, value_name)
        }
        CardSuit::Diamonds => {
            // Diamonds (EvilFerris): suitName + valueName (no "Card" prefix)
            format!("sprites/cards/{}/{}{}.png", suit_name, suit_name, value_name)
        }
    }
}

pub fn can_place_on_card(card_value: u8, target_card_value: u8) -> bool {
    // Cards can only be placed on cards with value +1 (descending order)
    // For example: Queen (12) on King (13), Jack (11) on Queen (12), etc.
    card_value == target_card_value - 1
}

pub fn has_complete_stack(cards: &[(CardSuit, u8)]) -> bool {
    // A complete stack must start with King (13) and end with Ace (1)
    // All cards must be in descending order with alternating colors
    if cards.is_empty() || cards[0].1 != 13 {
        return false;
    }
    
    for i in 0..cards.len() - 1 {
        let current = cards[i];
        let next = cards[i + 1];
        
        // Check descending order
        if current.1 != next.1 + 1 {
            return false;
        }
        
        // Check alternating colors
        let current_is_red = matches!(current.0, CardSuit::Hearts | CardSuit::Diamonds);
        let next_is_red = matches!(next.0, CardSuit::Hearts | CardSuit::Diamonds);
        
        
        if current_is_red == next_is_red {
            return false;
        }
    }
    
    // Must end with Ace (1)
    cards.last().map_or(false, |card| card.1 == 1)
}
