use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_web_client::{BevyWebClientPlugin, ClientStatus};
use bevy_web_server::{BevyWebServerPlugin, Connection, ServerSendPacket};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
enum Msg {
    Ping(u128),
    Pong(u128),
}

fn server_ping_system(
    connections: Query<&Connection>,
    mut writer: EventWriter<bevy_web_server::ServerSendPacket<Msg>>,
    time: Res<Time>,
) {
    for conn in connections.iter() {
        writer.send(ServerSendPacket {
            connection_id: conn.id.clone(),
            msg: Msg::Ping(time.elapsed().as_millis()),
        });
    }
}

fn server_recv_system(
    time: Res<Time>,
    mut reader: EventReader<bevy_web_server::ServerRecvPacket<Msg>>,
) {
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

fn client_is_connected_system(status:Res<ClientStatus>) {
    if status.is_changed() {
        println!("client is_connected={}", status.is_connected);

    }
}

fn client_recv_system(
    mut reader: EventReader<bevy_web_client::ClientRecvPacket<Msg>>,
    mut writer: EventWriter<bevy_web_client::ClientSendPacket<Msg>>,
) {
    for packet in reader.read() {
        match &packet.msg {
            Msg::Ping(ms) => {
                writer.send(bevy_web_client::ClientSendPacket {
                    msg: Msg::Pong(ms.to_owned()),
                });
            }
            Msg::Pong(_) => {}
        }
    }
}

fn main() {
    // start a bevy app as server in a seperate thread.
    let server_tick_rate = 1.0 / 64.0; 
    let client_tick_rate = 0.0; // unlimited;
    let server_thread = std::thread::spawn(move || {
        App::new()
            .add_plugins(BevyWebServerPlugin::new() as BevyWebServerPlugin<Msg>)
            .insert_resource(bevy_web_server::WebServerSettings { port: 8080 })
            .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
                Duration::from_secs_f32(server_tick_rate),
            )))
            .add_systems(Update, (server_ping_system, server_recv_system))
            .run();
    });

    // start a bevy app in the main thread
    App::new()
        .add_plugins(BevyWebClientPlugin::new() as BevyWebClientPlugin<Msg>)
        .insert_resource(bevy_web_client::ClientSettings {
            url:"ws://localhost:8080".to_owned()
        })
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f32(
                client_tick_rate,
            ))),
        )
        .add_systems(Update, (client_is_connected_system, client_recv_system))
        .run();

    server_thread.join().expect("failed to join");
}
