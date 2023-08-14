use crate::*;

pub fn input(
    _: In<ggrs::PlayerHandle>,
    key: Res<Input<KeyCode>>,
    mut ready: ResMut<Playing>,
    res_direction: Res<InputDirection>,
) -> CueInput {
    if key.just_pressed(KeyCode::Space) {
        if let Some(direction) = res_direction.dir {
            ready.ready = false;
            return CueInput {
                dir_x: (direction.x * 127.) as i8,
                dir_y: (direction.y * 127.) as i8,
                power: 5,
            };
        }
    }

    CueInput {
        dir_x: 0,
        dir_y: 0,
        power: 0,
    }
}

pub fn pool_cue(
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    ball: Query<(&Ball, &Transform), Without<Cue>>,
    mut cue: Query<&mut Transform, (With<Cue>, Without<Ball>)>,
    mut res_direction: ResMut<InputDirection>,
) {
    let window = windows.single();
    let (camera, cam_transform) = camera.single();
    let mut cue_transform = cue.single_mut();
    for (ball, ball_transform) in ball.iter() {
        if ball.color == BallColor::White {
            if let Some(pos) = window.cursor_position()
                .and_then(|cursor| camera.viewport_to_world(cam_transform, cursor))
                .map(|ray| ray.origin.truncate())
            {
                let direction = (pos - ball_transform.translation.truncate()).normalize();
                cue_transform.rotation = Quat::from_rotation_z(direction.y.atan2(direction.x));
                cue_transform.translation = ball_transform.translation - Vec3::new(direction.x, direction.y, 0.) * 60.;
                cue_transform.translation.z = 10.;
                res_direction.dir = Some(direction);
            } else {
                cue_transform.translation.z = -10.;
                res_direction.dir = None;
            }
        }
    }
}

pub fn set_ready(mut ready: ResMut<Playing>, velocities: Query<&Velocity>) {
    if ready.ready { return; }

    for velocity in velocities.iter() {
        if velocity.vel.x != 0. || velocity.vel.y != 0. {
            return;
        }
    }
    info!("system is sleeping");
    ready.ready = true;
}

pub fn update_text(
    mut text: Query<&mut Text, With<PlayerText>>, 
    playing: Res<Playing>,
    ready: Res<Playing>,
) {
    let mut text = text.single_mut();
    if ready.ready {
        text.sections[0].value = format!("Player to move is {}", playing.playing);
    } else {
        text.sections[0].value = format!("Moving balls");
    }
}

pub fn shoot_ball(
    inputs: Res<PlayerInputs<GgrsConfig>>,
    mut player_query: Query<(&mut Velocity, &Ball)>,
    mut playing: ResMut<Playing>,
) {
    for (mut velocity, ball) in player_query.iter_mut() {
        if ball.color == BallColor::White {
            let (input, _) = inputs[playing.playing];

            if input.power != 0 {
                let x_component = input.dir_x as f32 / 127. * input.power as f32;
                let y_component = input.dir_y as f32 / 127. * input.power as f32;
                velocity.vel.x = x_component;
                velocity.vel.y = y_component;

                if playing.playing == 1 {
                    playing.playing = 0;
                } else {
                    playing.playing = 1;
                }
            } else {
                return;
            }
        }
    }
}

pub fn move_balls(
    mut ball: Query<(&mut Velocity, &mut Transform), With<Ball>>,
    board_size: Res<BoardSize>,
) {
    for (mut velocity, mut transform) in ball.iter_mut() {
        if velocity.vel != Vec2::ZERO {
            if velocity.vel.x.abs() < 0.1 && velocity.vel.y.abs() < 0.1 {
                *velocity = Velocity::zero();
                return;
            }

            transform.translation.x += velocity.vel.x;
            transform.translation.y += velocity.vel.y;
            velocity.vel.x *= 0.97;
            velocity.vel.y *= 0.97;

            if transform.translation.x.abs() > board_size.width / 2. {
                velocity.vel.x *= -1.;
            }
            if transform.translation.y.abs() > board_size.height / 2. {
                velocity.vel.y *= -1.;
            }
        }
    }
}

pub fn ball_collisions(
    board_size: Res<BoardSize>,
    mut ball: Query<(Entity, &Transform, &mut Velocity), With<Ball>>,
) {
    let mut new_velocities = Vec::new();
    let mut checked = Vec::new();

    // phase 1: calculate all the new velocities from the collisions
    for (entity_1, transform_1, velocity_1) in ball.iter() {
        checked.push(entity_1);
        if velocity_1.vel != Vec2::ZERO {
            for (entity_2, transform_2, velocity_2) in ball.iter() {
                if entity_1 != entity_2 && !checked.contains(&entity_2) {
                    let distance = transform_1.translation.distance(transform_2.translation);
                    if distance <= board_size.ball_diameter {
                        info!("collision between {:?} and {:?}!", entity_1, entity_2);
    
                        let collision_direction = (transform_2.translation - transform_1.translation).normalize().truncate();  
                        let perpendicular_direction = Vec2::new(-collision_direction.y, collision_direction.x);
                        let angle = velocity_1.vel.angle_between(collision_direction);
                        let force_transferred = velocity_1.vel.length() * angle.cos();    
                        let transferred_velocity_2 = collision_direction * force_transferred;
                        let new_velocity_2 = velocity_2.vel + transferred_velocity_2;
                        let new_velocity_1 = -perpendicular_direction * force_transferred;
                        new_velocities.push((entity_1, new_velocity_1));
                        new_velocities.push((entity_2, new_velocity_2));
                        // this is wrong, namely the new direction of ball 1 (as well as the amount of force transferred lolol)
                    }
                }
            }
        }
    }

    // phase 2: apply velocities
    for (entity, velocity) in new_velocities {
        if let Ok((_, _, mut vel)) = ball.get_mut(entity) {
            vel.vel = velocity;
        }
    }
}