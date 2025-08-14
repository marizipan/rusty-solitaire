use bevy::prelude::*;
use crate::components::*;

use crate::systems::setup_initial_tableau_and_stock;

pub fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>, stock_cards: ResMut<StockCards>) {
    
    // Spawn a 2D camera
    commands.spawn(Camera2d::default());
    
    // Stock pile will be created by setup_initial_tableau_and_stock function

    // Create waste pile (next to stock)
    let stock_x = -WINDOW_WIDTH / 2.0 + 100.0;
    let stock_y = -WINDOW_HEIGHT / 2.0 + 100.0;
    let waste_x = stock_x + 100.0;
    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(80.0, 120.0)),
            ..default()
        },
        Transform::from_xyz(waste_x, stock_y, 0.0),
        WastePile,
    ));

    // Create foundation piles (top right)
    let foundation_start_x = WINDOW_WIDTH / 2.0 - 400.0;
    for i in 0..4 {
        let x_pos = foundation_start_x + (i as f32 * 100.0);
        commands.spawn((
            Sprite {
                color: Color::srgb(0.2, 0.2, 0.2),
                custom_size: Some(Vec2::new(80.0, 120.0)),
                ..default()
            },
            Transform::from_xyz(x_pos, WINDOW_HEIGHT / 2.0 - 50.0, 0.0),
            FoundationPile,
        ));
    }

    // Set up the initial tableau and stock pile distribution
    setup_initial_tableau_and_stock(&mut commands, asset_server, stock_cards);

    // Score display
    commands.spawn((
        Text::new("Score: 0"),
        Transform::from_xyz(-WINDOW_WIDTH / 2.0 + 100.0, WINDOW_HEIGHT / 2.0 - 50.0, 2.0),
        Score,
    ));

} 
