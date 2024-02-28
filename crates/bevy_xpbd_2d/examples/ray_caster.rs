//! A simple raycasting example that uses the [`RayCaster`] component.
//!
//! An alternative, more controlled approach is to use the methods of
//! the [`SpatialQuery`] system parameter.

#![allow(clippy::unnecessary_cast)]

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_xpbd_2d::{math::*, prelude::*};
use examples_common_2d::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, XpbdExamplePlugin))
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.1)))
        .add_systems(Update, render_rays)
        .add_systems(Update, transform_parent.run_if(in_state(AppState::Running)))
        .add_systems(Startup, (setup,).chain())
        .run();
}

#[derive(Component)]
struct ParentTransform;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let mut box_parent = commands.spawn((TransformBundle { ..default() }, ParentTransform));

    box_parent.with_children(|commands| {
        // Spawn a perimeter of circles that the ray will be cast against
        let radius = 16.0;
        for x in -4..=4 {
            for y in -4..=4 {
                if (-3..4).contains(&x) && (-3..4).contains(&y) {
                    continue;
                }

                commands.spawn((
                    MaterialMesh2dBundle {
                        mesh: meshes.add(Circle::new(radius)).into(),
                        material: materials.add(Color::rgb(0.2, 0.7, 0.9)),
                        transform: Transform::from_xyz(
                            x as f32 * radius * 3.0,
                            y as f32 * radius * 3.0,
                            0.0,
                        ),
                        ..default()
                    },
                    Collider::circle(radius as Scalar),
                ));
            }
        }
    });
    // Spawn a rotating kinematic body with a ray caster
    commands.spawn((
        RigidBody::Kinematic,
        AngularVelocity(0.02),
        RayCaster::new(Vector::ZERO, Direction2d::X),
    ));
}

fn transform_parent(mut parent: Query<&mut Transform, With<ParentTransform>>, time: Res<Time>) {
    if let Ok(mut parent_transform) = parent.get_single_mut() {
        parent_transform
            .rotate_around(Vec3::Z, Quat::from_rotation_z(0.001 * time.delta_seconds()));
        parent_transform.translation.x = (time.elapsed().as_secs_f32()).sin() * 10.;
        parent_transform.translation.y = (time.elapsed().as_secs_f32()).cos() * 10.;
    }
}

// Note: The `PhysicsDebugPlugin` can also render rays, hit points, and normals.
//       This system is primarily for demonstration purposes.
fn render_rays(
    mut rays: Query<(&mut RayCaster, &mut RayHits)>,
    mut gizmos: Gizmos,
    sq: SpatialQuery,
) {
    for (ray, hits) in &mut rays {
        // Convert to Vec3 for lines
        let origin = ray.global_origin().f32();
        let direction = ray.global_direction().f32();

        for hit in hits.iter() {
            let mut some_ricochet: Option<RayHitData> = Some(*hit);

            let mut n_hits = 1;
            let mut last_hit_location = origin + direction * hit.time_of_impact as f32;
            gizmos.line_2d(origin, last_hit_location, Color::GREEN);
            while some_ricochet.is_some() && n_hits < 64 {
                if let Some(ricochet) = some_ricochet {
                    some_ricochet = sq.cast_ray(
                        last_hit_location,
                        Direction2d::new_unchecked(ricochet.normal),
                        1000.0,
                        true,
                        SpatialQueryFilter::default().with_excluded_entities([ricochet.entity]),
                    );

                    if let Some(this_hit) = some_ricochet {
                        if this_hit.time_of_impact == 0. {
                            break;
                        }

                        let new_hit_location =
                            last_hit_location + (ricochet.normal) * this_hit.time_of_impact as f32;

                        gizmos.circle_2d(last_hit_location, 5., Color::YELLOW);
                        gizmos.arrow_2d(last_hit_location, new_hit_location, Color::GREEN);
                        last_hit_location = new_hit_location;
                    } else {
                        gizmos.circle_2d(last_hit_location, 5., Color::ORANGE);
                        gizmos.ray_2d(
                            last_hit_location,
                            ricochet.normal * 1000.,
                            Color::ORANGE_RED,
                        );
                    }
                } else {
                    break;
                }
                n_hits += 1;
            }
        }
        if hits.is_empty() {
            gizmos.line_2d(origin, origin + direction * 1_000_000.0, Color::ORANGE_RED);
        }
    }
}
