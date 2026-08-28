use bevy::prelude::*;

use crate::ballistics::DamageEvent;
use crate::components::Health;

/// Tag to prevent entity from being despawned on lethal damage (e.g. for player death screens/match state)
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct ImmortalPlayer;

#[derive(Message, Debug, Clone)]
pub struct EntityKilledEvent {
    pub entity: Entity,
}

pub fn process_damage(
    mut commands: Commands,
    mut events: MessageReader<DamageEvent>,
    mut query: Query<(Entity, &mut Health, Option<&ImmortalPlayer>)>,
    mut killed_events: MessageWriter<EntityKilledEvent>,
) {
    for event in events.read() {
        if let Ok((entity, mut health, immortal)) = query.get_mut(event.target) {
            health.current -= event.amount;
            if health.current <= 0.0 {
                killed_events.write(EntityKilledEvent { entity });
                if immortal.is_none() {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}




