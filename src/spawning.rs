use crate::*;

pub fn setup_table(
    mut commands: Commands,
    assets: Res<AssetServer>,
    board_sizes: Res<BoardSize>,
    mut rip: ResMut<RollbackIdProvider>,
) {
    let board_size = Vec2::new(board_sizes.width, board_sizes.height);

    // green part of board
    commands.spawn((SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.0, 0.3, 0.0),
            custom_size: Some(board_size),
            ..default()
        },
        ..default()
    },));

    // edges of board
    let edge_thickness = 4.0;
    for &(x, y, width, height) in &[
        (0.0, board_size.y / 2.0, board_size.x, edge_thickness),
        (0.0, -board_size.y / 2.0, board_size.x, edge_thickness),
        (board_size.x / 2.0, 0.0, edge_thickness, board_size.y),
        (-board_size.x / 2.0, 0.0, edge_thickness, board_size.y),
    ] {
        commands.spawn((SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.5, 0.3, 0.1),
                custom_size: Some(Vec2::new(width, height)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
            ..default()
        },));
    }

    // holes
    let hole_width = 13.0;
    for &(x, y) in &[
        (0.0, board_size.y / 2.0),
        (0.0, -board_size.y / 2.0),
        (board_size.x / 2.0, board_size.y / 2.0),
        (board_size.x / 2.0, -board_size.y / 2.0),
        (-board_size.x / 2.0, board_size.y / 2.0),
        (-board_size.x / 2.0, -board_size.y / 2.0),
    ] {
        commands.spawn((SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(hole_width, hole_width)),
                ..default()
            },
            texture: assets.load("hole.png"),
            transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
            ..default()
        },
        Hole {},
    ));
    }

    // cue
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(100., 10.)),
                ..default()
            },
            texture: assets.load("stick.png"),
            ..default()
        },
        Cue {},
    ));

    // white ball
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(board_sizes.ball_diameter, board_sizes.ball_diameter)),
                ..default()
            },
            texture: assets.load("white.png"),
            ..default()
        },
        Ball {
            color: BallColor::White,
        },
        Velocity::zero(),
        rip.next(),
    ));

    // red ball?
    for pos in &[
        Vec3::new(-20., 0., 0.),
        Vec3::new(-10., 0., 0.),
        Vec3::new(10., 0., 0.),
        Vec3::new(0., 20., 0.),
        Vec3::new(0., 10., 0.),
        Vec3::new(0., -10., 0.),
        Vec3::new(0., -20., 0.),
        Vec3::new(10., 10., 0.),
    ] {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(board_sizes.ball_diameter, board_sizes.ball_diameter)),
                    ..default()
                },
                texture: assets.load("red.png"),
                transform: Transform::from_translation(*pos),
                ..default()
            },
            Ball {
                color: BallColor::Filled,
            },
            Velocity::zero(),
            rip.next(),
        ));
    }

    // ui
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::width(Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(
                    TextBundle::from_section(
                        "Text Example",
                        TextStyle {
                            font: assets.load("Marlboro.ttf"),
                            font_size: 30.0,
                            color: Color::RED,
                        },
                    )
                    .with_text_alignment(TextAlignment::Center),
                )
                .insert(PlayerText {});
        });

    // camera
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::FixedHorizontal(256.),
            ..default()
        },
        ..default()
    });
}

pub fn connect(mut commands: Commands) {
    let room_url = "ws://127.0.0.1:3536/pool?next=2";
    info!("connecting to matchbox server: {:?}", room_url);
    commands.insert_resource(MatchboxSocket::new_ggrs(room_url));
}

pub fn wait_for_players(mut commands: Commands, mut socket: ResMut<MatchboxSocket<SingleChannel>>) {
    if socket.get_channel(0).is_err() { return; }

    socket.update_peers();
    let players = socket.players();

    let num_players = 2;
    if players.len() < num_players { return; }

    info!("Players have joined, starting game");
    let mut session_builder = ggrs::SessionBuilder::<GgrsConfig>::new()
        .with_num_players(num_players)
        .with_input_delay(2);

    for (i, player) in players.into_iter().enumerate() {
        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");
    }
    let channel = socket.take_channel(0).unwrap();

    let ggrs_session = session_builder
        .start_p2p_session(channel)
        .expect("failed to start session");

    commands.insert_resource(bevy_ggrs::Session::P2PSession(ggrs_session));
}