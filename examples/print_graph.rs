use bevy::{app::App, DefaultPlugins};
use bevy_incandescent::IncandescentPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, IncandescentPlugin));
    bevy_mod_debugdump::print_render_graph(&mut app);
}
