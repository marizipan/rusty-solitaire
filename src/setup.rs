use bevy::prelude::*;
use crate::components::*;

use crate::systems::setup_initial_tableau_and_stock;

pub fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>, stock_cards: ResMut<StockCards>) {
    commands.spawn(Camera2d::default());
    
    let waste_x = -(6.0 * 100.0) / 2.0 + (5.0 * 100.0);
    let waste_y = WINDOW_HEIGHT / 2.0 - 100.0;
    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(80.0, 120.0)),
            ..default()
        },
        Transform::from_xyz(waste_x, waste_y, 0.0),
        WastePile,
    ));

    let foundation_start_x = -(6.0 * 100.0) / 2.0;
    for i in 0..4 {
        let x_pos = foundation_start_x + (i as f32 * 100.0);
        commands.spawn((
            Sprite {
                color: Color::srgb(0.2, 0.2, 0.2),
                custom_size: Some(Vec2::new(80.0, 120.0)),
                ..default()
            },
            Transform::from_xyz(x_pos, WINDOW_HEIGHT / 2.0 - 100.0, 0.0),
            FoundationPile,
        ));
    }

    setup_initial_tableau_and_stock(&mut commands, asset_server, stock_cards);

    commands.spawn((
        Text::new("Score: 0"),
        Transform::from_xyz(-WINDOW_WIDTH / 2.0 + 100.0, WINDOW_HEIGHT / 2.0 - 50.0, 2.0),
        Score,
    ));
} 
