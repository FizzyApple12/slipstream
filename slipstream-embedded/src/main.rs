pub mod components;
pub mod event_mapper;
pub mod logger;
pub mod statuses;
pub mod waveform_loader;

use std::{array, pin::Pin, time::Duration};

use clap::Parser;
use cxx_qt_lib::{QFont, QGuiApplication, QQmlApplicationEngine, QString, QUrl};
use libdatabase::device_manager::DeviceManager;
use libdj::{AUDIO_CHANNELS, engine::DJEngine};
use libio::ipc::{HostIPC, IPCMessage};
use log::{debug, info};
use tokio::task;

use crate::{
    components::{engine_bridge::EngineBridgeRust, ffi::set_qfont_feature},
    event_mapper::{MappedEvent, map_controller_event},
    logger::setup_logger,
};

const LOCAL_AUTOMOUNTS: [(&str, usize); 2] = [("/djusb/usb0", 0), ("/djusb/usb1", 1)];

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    log_path: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Safety: This happens before any additional threads get spawned
    unsafe {
        std::env::set_var("QT_IM_MODULE", "qtvirtualkeyboard");
        std::env::set_var("QT_VIRTUALKEYBOARD_STYLE", "slipstream");
    };

    let args = Args::parse();

    setup_logger(args.log_path)?;

    info!("Starting tokio runtime...");

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;

    debug!("Starting IO Manager...");

    let mut host_ipc = runtime.block_on(HostIPC::listen())?;

    let enter_handle = runtime.enter();

    debug!("Starting Device Manager...");

    let mut device_manager = DeviceManager::default();

    device_manager.start_local_watch(&LOCAL_AUTOMOUNTS)?;
    // device_manager.start_prodj_link_watch(0x10)?;

    debug!("Starting DJ Engine...");

    let dj_engine = DJEngine::new();

    let mut relay_engine_handle = dj_engine.subscribe();

    let (ui_control_event_sender, ui_control_event_receiver) =
        tokio::sync::mpsc::unbounded_channel();

    task::spawn(async move {
        host_ipc.select_driver_id(Some("ddj_flx10".to_string()));

        let mut host_ipc_handle = host_ipc.subscribe();

        loop {
            tokio::select! {
                new_driver = host_ipc_handle.listen_for_driver_change() => {
                    if new_driver.is_none() {
                        tokio::time::sleep(Duration::from_secs(1)).await;

                        host_ipc.select_driver_id(Some("ddj_flx10".to_string()));
                    }
                }
                controller_event = host_ipc.receive_event() => {
                    match map_controller_event(controller_event) {
                        MappedEvent::UIControlEvent(ui_control_event) => {
                            let _ = ui_control_event_sender.send(ui_control_event);
                        },
                        MappedEvent::DeckControlEvent(deck_control_event) => {
                            relay_engine_handle.send_control_event(deck_control_event);
                        },
                    }
                }
                () = relay_engine_handle.wait_for_new_deck_state_changed() => {
                    let new_deck_state = relay_engine_handle.get_deck_state().clone();

                    #[allow(clippy::cast_precision_loss)]
                    let _ = host_ipc.send_message(IPCMessage::UpdateCurrentMeter(array::from_fn(
                        |index| {
                            new_deck_state
                                .mixer_channels
                                .get(index)
                                .map_or([0.0; AUDIO_CHANNELS], |channel| {
                                    channel
                                        .average_amplitude
                                })
                        },
                    )));

                    let _ = host_ipc.send_message(IPCMessage::UpdateDeckState(new_deck_state));
                }
            }
        }
    });

    drop(enter_handle);

    let handle = runtime.handle().clone();

    EngineBridgeRust::register(
        dj_engine.subscribe(),
        device_manager,
        ui_control_event_receiver,
        handle,
    );

    let mut app = QGuiApplication::new();

    if let Some(app) = app.as_mut() {
        let mut font = QFont::default();

        font.set_family(&QString::from("Helvetica"));
        // font.clear_features();

        // i hate this but cxx_qt is out of date and i can't be bothered to open a pr
        // right now
        set_qfont_feature(Pin::new(&mut font), &QString::from("kern"), 1);
        set_qfont_feature(Pin::new(&mut font), &QString::from("liga"), 1);
        set_qfont_feature(Pin::new(&mut font), &QString::from("tnum"), 1);

        app.set_application_font(&font);
    }

    let mut engine = QQmlApplicationEngine::new();

    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from(
            "qrc:/qt/qml/engineering/fizzy/slipstream_embedded/qml/root.qml",
        ));
    }

    if let Some(app) = app.as_mut() {
        app.exec();
    }

    runtime.shutdown_background();

    Ok(())
}
