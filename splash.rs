use bevy::input::keyboard::KeyCode;




#[derive(Component)]
struct SplashScreen;

#[derive(Component)]
struct StartButton;


// Under "fn main"
        .insert_state(GameState::Splash)
        .add_systems(OnEnter(GameState::Splash), setup_splash)
        .add_systems(Update, start_button.run_if(in_state(GameState::Splash)))
        .add_systems(OnEnter(GameState::Playing), setup_game)
        
        fn setup_splash(mut commands: Commands, asset_server: Res<AssetServer>) {
    // UI Camera for splash screen
    commands.spawn((Camera2d, IsDefaultUiCamera));

    // Splash image as background
    commands.spawn((
        Sprite {
            image: asset_server.load("splash.png"),
            custom_size: Some(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        SplashScreen,
    ));

    // Start button - a large, visible colored rectangle
    commands.spawn((
        Sprite {
            color: Color::srgb(0.25, 0.25, 0.85),
            custom_size: Some(Vec2::new(300.0, 100.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -100.0, 1.0),
        Button,
        StartButton,
    ));
    
    // Start button text
    commands.spawn((
        Text2d("Press Spacebar to Start".to_string()), // Changed text
        Transform::from_xyz(0.0, -100.0, 2.0),
        StartButton,
    ));

}

// Start button mechanics

fn start_button(
    mut interaction_query: Query<
        (&Interaction, &mut Sprite),
        (Changed<Interaction>, With<StartButton>),
    >,
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    splash_query: Query<Entity, With<SplashScreen>>,
    button_query: Query<Entity, With<StartButton>>,
) {
    // Check for mouse interaction
    for (interaction, mut sprite) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                for entity in &splash_query {
                    commands.entity(entity).despawn();
                }
                for entity in &button_query {
                    commands.entity(entity).despawn();
                }
                next_state.set(GameState::Playing);
            }
            Interaction::Hovered => {
                sprite.color = Color::srgb(0.35, 0.35, 0.95);
            }
            Interaction::None => {
                sprite.color = Color::srgb(0.25, 0.25, 0.85);
            }
        }
    }

    // Check for keyboard input (spacebar)
    if input.just_pressed(KeyCode::Space) {
        for entity in &splash_query {
            commands.entity(entity).despawn();
        }
        for entity in &button_query {
            commands.entity(entity).despawn();
        }
        next_state.set(GameState::Playing);
    }
}

