use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_web_client::BevyWebClientPlugin;
use bevy_web_server::{BevyWebServerPlugin, Connection, SendPacket};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
enum Msg {
    Ping(u128),
    Pong(u128),
}

fn server_system(
    connections: Query<&Connection>,
    mut writer: EventWriter<bevy_web_server::SendPacket<Msg>>,
    time: Res<Time>,
    mut reader: EventReader<bevy_web_server::RecvPacket<Msg>>,
) {
    for conn in connections.iter() {
        writer.send(SendPacket {
            connection_id: conn.id.clone(),
            msg: Msg::Ping(time.elapsed().as_millis()),
        });
    }

    for packet in reader.read() {
        match &packet.msg {
            Msg::Ping(_) => {}
            Msg::Pong(ms) => {
                let current_ms = time.elapsed().as_millis();
                let diff = current_ms - ms.to_owned();
                println!("{} ping: {}ms", packet.connection, diff);
            }
        }
    }
}

fn client_system(
    mut reader: EventReader<bevy_web_client::RecvPacket<Msg>>,
    mut writer: EventWriter<bevy_web_client::SendPacket<Msg>>,
) {
    for packet in reader.read() {
        match &packet.msg {
            Msg::Ping(ms) => {
                writer.send(bevy_web_client::SendPacket {
                    msg: Msg::Pong(ms.to_owned()),
                });
            }
            Msg::Pong(_) => {}
        }
    }
}

fn main() {
    let server_tick_rate = 1.0 / 64.0; 
    let client_tick_rate = 0.0; // unlimited;
    let server_thread = std::thread::spawn(move || {
        App::new()
            .add_plugins(BevyWebServerPlugin::new() as BevyWebServerPlugin<Msg>)
            .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
                Duration::from_secs_f32(server_tick_rate),
            )))
            .add_systems(Update, server_system)
            .run();
    });
    App::new()
        .add_plugins(BevyWebClientPlugin::new() as BevyWebClientPlugin<Msg>)
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f32(
                client_tick_rate,
            ))),
        )
        .add_systems(Update, client_system)
        .run();

    server_thread.join().expect("failed to join");
}
