use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_web_client::BevyWebClientPlugin;
use bevy_web_server::{BevyWebServerPlugin, Connection, SendPacket};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
enum Msg {
    Ping(u64),
    Pong(u64),
}

fn server_system(connections:Query<&Connection>, mut writer:EventWriter<bevy_web_server::SendPacket<Msg>>) {
    for conn in connections.iter() {
        writer.send(SendPacket {
            connection_id:conn.id.clone(),
            msg:Msg::Ping(0)
        });
    }
}

fn client_system() {
}

fn main() {
    // Start an App that acts both as a Server and a Client.
    // i.e. the app connects to itself.
    App::new()
        .add_plugins(BevyWebServerPlugin::new() as BevyWebServerPlugin<Msg>)
        .add_plugins(BevyWebClientPlugin::new() as BevyWebClientPlugin<Msg>)
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(100))))
        .add_systems(Update, server_system)
        .add_systems(Update, client_system)
        .run();
}
