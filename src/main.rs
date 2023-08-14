use multiplayer_game::*;

fn main() {
    let mut app = App::new();
    GGRSPlugin::<GgrsConfig>::new()
        .with_input_system(input)
        .register_rollback_component::<Transform>()
        .register_rollback_component::<Velocity>()
        .build(&mut app);
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .insert_resource(InputDirection { dir: None, })
        .insert_resource(Playing { ready: false, playing: 0, })
        .insert_resource(BoardSize { width: 244., height: 112., ball_diameter: 6., })
        .add_startup_systems((connect, setup_table))
        .add_system(wait_for_players)
        .add_systems((
                pool_cue,
                shoot_ball.after(pool_cue),
                ball_collisions.after(shoot_ball),
                move_balls.after(ball_collisions),
                set_ready.after(move_balls),
                update_text.after(set_ready),
            ).in_schedule(GGRSSchedule),
        ).run();
}