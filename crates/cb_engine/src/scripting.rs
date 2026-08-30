use bevy::prelude::*;
use rhai::{Engine, Scope, AST};
use std::collections::HashMap;

#[derive(
    Component, Reflect, Default, serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq,
)]
#[reflect(Component, Default, Serialize, Deserialize)]
pub struct ScriptComponent {
    pub path: String,
}

#[derive(Resource)]
pub struct ScriptCache {
    pub engine: Engine,
    pub asts: HashMap<String, AST>,
    pub mtimes: HashMap<String, std::time::SystemTime>,
}

impl Default for ScriptCache {
    fn default() -> Self {
        let mut engine = Engine::new();

        // Register Vec3 math so Rhai can understand Bevy transforms
        engine
            .register_type_with_name::<Vec3>("Vec3")
            .register_fn("vec3", |x: f32, y: f32, z: f32| Vec3::new(x, y, z))
            .register_get_set("x", |v: &mut Vec3| v.x, |v: &mut Vec3, x: f32| v.x = x)
            .register_get_set("y", |v: &mut Vec3| v.y, |v: &mut Vec3, y: f32| v.y = y)
            .register_get_set("z", |v: &mut Vec3| v.z, |v: &mut Vec3, z: f32| v.z = z)
            .register_fn("+", |a: Vec3, b: Vec3| a + b)
            .register_fn("-", |a: Vec3, b: Vec3| a - b)
            .register_fn("*", |a: Vec3, b: f32| a * b)
            .register_fn("*", |a: f32, b: Vec3| a * b);

        Self {
            engine,
            asts: HashMap::new(),
            mtimes: HashMap::new(),
        }
    }
}

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScriptCache>()
            .register_type::<ScriptComponent>()
            .add_systems(
                Update,
                execute_scripts.run_if(in_state(crate::editor::EngineState::Play)),
            );
    }
}

fn execute_scripts(
    time: Res<Time>,
    mut q_scripts: Query<(&ScriptComponent, &mut Transform)>,
    mut script_cache: ResMut<ScriptCache>,
) {
    let delta = time.delta_secs();

    for (script, mut transform) in q_scripts.iter_mut() {
        if script.path.is_empty() {
            continue;
        }

        let path = if script.path.starts_with("assets/") {
            script.path.clone()
        } else {
            format!("assets/{}", script.path)
        };

        let current_mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        let last_mtime = script_cache.mtimes.get(&script.path).copied();
        let needs_reload = !script_cache.asts.contains_key(&script.path)
            || (current_mtime.is_some() && current_mtime != last_mtime);

        // Load or Hot-Reload AST if changed
        if needs_reload {
            match script_cache.engine.compile_file(path.clone().into()) {
                Ok(ast) => {
                    script_cache.asts.insert(script.path.clone(), ast);
                    if let Some(mtime) = current_mtime {
                        script_cache.mtimes.insert(script.path.clone(), mtime);
                    }
                    info!("  Live Hot-Reloaded Script: {}", path);
                }
                Err(e) => {
                    error!("Failed to compile script {}: {}", path, e);
                    continue;
                }
            }
        }

        if let Some(ast) = script_cache.asts.get(&script.path) {
            let mut scope = Scope::new();
            scope.push("delta_time", delta);
            scope.push("position", transform.translation);

            match script_cache
                .engine
                .eval_ast_with_scope::<()>(&mut scope, ast)
            {
                Ok(_) => {
                    if let Some(pos) = scope.get_value::<Vec3>("position") {
                        transform.translation = pos;
                    }
                }
                Err(e) => {
                    error!("Error running script {}: {}", path, e);
                }
            }
        }
    }
}
