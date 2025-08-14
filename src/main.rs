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
        .add_plugins(DefaultPlugins)      
        .add_systems(Startup, setup_game)
        .add_systems(
            Update,
            (
                card_drag_system,
                card_drop_system,
                stock_click_system,
                double_click_system,
                card_movement_system,

            ),
        )
        .run();
} 




// Things to fix: 
// suits are randomly incompatible
// cards are going into the Stock pile for some reason
// Stock pile is not being recycled
// Cards are not moving together after they have been paired
// Stabby queen still disappears when clicked
// Cards do not come out of the stock pile
// Corro Queen seemingly overrides other queen cards?