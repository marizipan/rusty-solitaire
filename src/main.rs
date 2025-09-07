mod components;
mod utils;
mod setup;
mod init_setup;
mod card_drag_sys;
mod card_drop_sys;
mod card_flip_sys;
mod card_entity;
mod foundation_auto;
mod foundation_click;
mod stock_click;
mod undo;
mod visual_stacking;

use bevy::prelude::*;
use components::*;
use setup::setup_game;
use card_drag_sys::*;
use card_drop_sys::*;
use card_flip_sys::*;
use card_entity::*;
use foundation_auto::*;
use foundation_click::*;
use stock_click::*;
use undo::*;
use visual_stacking::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.1, 0.4, 0.1))) // Green background for solitaire
        .insert_resource(GameScore(0))
        .insert_resource(SelectedCard(None))
        .insert_resource(StockCards(Vec::new()))
        .insert_resource(TableauPositions(Vec::new()))
        .insert_resource(FoundationPiles(vec![Vec::new(); 4])) // Initialize 4 empty foundation piles
        .insert_resource(ClickedEntity(None)) // Initialize clicked entity tracking for double-click detection
        .insert_resource(UndoStack(Vec::new())) // Initialize undo stack
        .add_plugins(DefaultPlugins)      
        .add_systems(Startup, setup_game)
        .add_systems(
            Update,
            (
                // Input systems first
                stock_click_system, // Handle stock pile cycling (deal to waste, recycle waste to stock)
                double_click_foundation_system, // Auto-move cards to foundation piles on click
                undo_button_system, // Handle undo button clicks
                // Drag and drop systems
                card_drag_system,
                card_drop_system,
                // Update systems last
                flip_cards_system, // Handle flipping cards underneath moved cards
                undo_system, // Handle undo functionality
                update_tableau_visual_stacking_system, // Maintain visual stacking of tableau cards. Never disable this.
            ),
        )
        .run();
} 



