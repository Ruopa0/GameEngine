use bevy::prelude::*;
use cb_shared::components::{WeaponBundle, WeaponConfig, FireRate, Magazine, Spread, RecoilPattern, FireMode};
use avian3d::prelude::*;

#[derive(Component)]
pub struct WeaponCrate {
    pub is_open: bool,
    pub weapon_to_give: WeaponBundle,
}

#[derive(Component)]
pub struct CrateLid;

pub struct InteractablesPlugin;

impl Plugin for InteractablesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_crate_interactions);
    }
}



pub fn populate_crate_entity(commands: &mut Commands, entity: Entity, meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) {
    let crate_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.25, 0.1),
        ..default()
    });
    let lid_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.3, 0.15),
        ..default()
    });
    
    let rifle = WeaponBundle {
        config: WeaponConfig {
            name: "Assault Rifle",
            fire_mode: FireMode::FullAuto,
            damage: 35.0,
            penetration: 0.5,
            range: 150.0,
            projectile_speed: Some(300.0),
        },
        fire: FireRate::new(600.0),
        mag: Magazine::new(30, 90, 2.0),
        spread: Spread::default(),
        recoil: RecoilPattern::default(),
    };

    commands.entity(entity).insert((
        WeaponCrate {
            is_open: false,
            weapon_to_give: rifle,
        },
    )).with_children(|parent| {
        parent.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.2, 0.6, 0.6))),
            MeshMaterial3d(crate_mat),
            Transform::from_xyz(0.0, 0.3, 0.0),
            RigidBody::Static,
            Collider::cuboid(1.2, 0.6, 0.6),
        ));
        parent.spawn((
            CrateLid,
            Mesh3d(meshes.add(Cuboid::new(1.2, 0.1, 0.6))),
            MeshMaterial3d(lid_mat),
            Transform::from_xyz(0.0, 0.65, 0.0),
        ));
    });
}

fn handle_crate_interactions(
    input_opt: Option<Res<cb_input::PlayerInput>>,
    mut q_player: Query<(&Transform, &mut cb_shared::components::WeaponInventory), With<crate::player::Player>>,
    mut q_crates: Query<(&mut WeaponCrate, &Transform, &Children)>,
    mut q_lids: Query<&mut Transform, (With<CrateLid>, Without<crate::player::Player>, Without<WeaponCrate>)>,
) {
    let Some(input) = input_opt else { return };
    if !input.interact { return; }

    let Ok((player_tf, mut inventory)) = q_player.single_mut() else { return };

    let interact_range = 3.0;

    for (mut crate_item, crate_tf, children) in q_crates.iter_mut() {
        if crate_item.is_open { continue; }

        if player_tf.translation.distance(crate_tf.translation) < interact_range {
            crate_item.is_open = true;
            inventory.secondary = Some(crate_item.weapon_to_give.clone_components());
            inventory.pending_slot = Some(2);

            for child in children.iter() {
                if let Ok(mut lid_tf) = q_lids.get_mut(child) {
                    lid_tf.translation.y += 0.4;
                    lid_tf.rotation = Quat::from_rotation_x(1.0);
                }
            }
            break;
        }
    }
}

