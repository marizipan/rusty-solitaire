mod components;
mod utils;
mod setup;
mod systems;

use bevy::prelude::*;
use components::*;
use setup::setup_game;
use systems::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.1, 0.4, 0.1))) // Green background for solitaire
        .insert_resource(GameScore(0))
        .insert_resource(SelectedCard(None))
        .insert_resource(StockCards(Vec::new()))
        .insert_resource(TableauPositions(Vec::new()))
        .insert_resource(FoundationPiles(vec![Vec::new(); 4])) // Initialize 4 empty foundation piles
        .add_plugins(DefaultPlugins)      
        .add_systems(Startup, setup_game)
        .add_systems(
            Update,
            (
                card_drag_system,
                card_drop_system,
                stock_click_system,
                waste_card_click_system, // User-initiated waste card placement
                flip_cards_system, // Handle flipping cards underneath moved cards
                validate_card_draggability_system, // Validate that only top cards can be dragged
                cleanup_flip_markers_system, // Clean up flip markers to prevent blocking future flips
                update_tableau_visual_stacking_system, // Maintain visual stacking of tableau cards

                auto_move_to_foundation_system, // Auto-move cards to foundation piles when possible

            ),
        )
        .run();
} 




// Things to fix: 
// Cards are not moving together after they have been paired
// Cards below the top layer are not flipping