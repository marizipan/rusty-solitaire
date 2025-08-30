use bevy::prelude::*;
use crate::components::*;

use crate::systems::setup_initial_tableau_and_stock;

pub fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>, stock_cards: ResMut<StockCards>) {
    
    // Spawn a 2D camera
    commands.spawn(Camera2d::default());
    
    // Stock pile will be created by setup_initial_tableau_and_stock function

    // Create waste pile above Stack 6
    let waste_x = -(6.0 * 100.0) / 2.0 + (5.0 * 100.0); // Above Stack 6 (x = 200)
    let waste_y = WINDOW_HEIGHT / 2.0 - 100.0; // Aligned with Stock Pile and Foundation Piles
    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(80.0, 120.0)),
            ..default()
        },
        Transform::from_xyz(waste_x, waste_y, 0.0),
        WastePile,
    ));

    // Create foundation piles above the first 4 stack positions
    let foundation_start_x = -(6.0 * 100.0) / 2.0; // Same starting X as tableau stacks
    for i in 0..4 {
        let x_pos = foundation_start_x + (i as f32 * 100.0);
        commands.spawn((
            Sprite {
                color: Color::srgb(0.2, 0.2, 0.2),
                custom_size: Some(Vec2::new(80.0, 120.0)),
                ..default()
            },
            Transform::from_xyz(x_pos, WINDOW_HEIGHT / 2.0 - 100.0, 0.0), // Aligned with Stock Pile
            FoundationPile,
        ));
    }

    // Set up the initial tableau and stock pile distribution
    setup_initial_tableau_and_stock(&mut commands, asset_server, stock_cards);

    // Score display
    commands.spawn((
        Text2d::new("Score: 0"),
        Transform::from_xyz(-WINDOW_WIDTH / 2.0 + 100.0, WINDOW_HEIGHT / 2.0 - 50.0, 2.0),
        Score,
    ));

    // Undo button
    commands.spawn((
        Sprite {
            color: Color::srgb(0.4, 0.4, 0.8),
            custom_size: Some(Vec2::new(100.0, 40.0)),
            ..default()
        },
        Transform::from_xyz(WINDOW_WIDTH / 2.0 - 100.0, WINDOW_HEIGHT / 2.0 - 50.0, 2.0),
        UndoButton,
    ));

    // Undo button text
    commands.spawn((
        Text2d::new("Undo"),
        Transform::from_xyz(WINDOW_WIDTH / 2.0 - 100.0, WINDOW_HEIGHT / 2.0 - 50.0, 2.0),
    ));

} 
