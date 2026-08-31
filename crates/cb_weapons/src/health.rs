use bevy::prelude::*;

use crate::ballistics::DamageEvent;
use crate::components::Health;

pub use cb_shared::components::ImmortalPlayer;

/// Component attached to dead entities to delay despawning by 1.0 second
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct DespawnDelay(pub Timer);

impl Default for DespawnDelay {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Once))
    }
}

#[derive(Message, Debug, Clone)]
pub struct EntityKilledEvent {
    pub entity: Entity,
}

pub fn process_damage(
    mut commands: Commands,
    mut events: MessageReader<DamageEvent>,
    mut query: Query<(Entity, &mut Health, Option<&ImmortalPlayer>, Option<&DespawnDelay>)>,
    mut killed_events: MessageWriter<EntityKilledEvent>,
) {
    for event in events.read() {
        if let Ok((entity, mut health, immortal, despawn_delay)) = query.get_mut(event.target) {
            if despawn_delay.is_some() {
                continue; // Already in 1.0s despawn countdown
            }
            health.current -= event.amount;
            if health.current <= 0.0 {
                killed_events.write(EntityKilledEvent { entity });
                commands.entity(entity).remove::<avian3d::prelude::Collider>();
                if immortal.is_none() {
                    // set 1.0s despawn timer
                    commands.entity(entity)
                        .insert(DespawnDelay::default());
                }
            }
        }
    }
}

pub fn update_despawn_delays(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DespawnDelay)>,
) {
    for (entity, mut delay) in query.iter_mut() {
        delay.0.tick(time.delta());
        if delay.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}




