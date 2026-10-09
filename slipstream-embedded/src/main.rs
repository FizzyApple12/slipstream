pub mod components;
pub mod event_mapper;
pub mod logger;
pub mod statuses;
pub mod waveform_loader;

use std::{array, pin::Pin};

use cxx_qt_lib::{QFont, QGuiApplication, QQmlApplicationEngine, QString, QUrl};
use libdatabase::device_manager::DeviceManager;
use libdj::{AUDIO_CHANNELS, engine::DJEngine};
use libio::{controller::Controller, types::controller::ControllerMessage};
use log::{debug, info, warn};
use tokio::task;

use crate::{
    components::{engine_bridge::EngineBridgeRust, ffi::set_qfont_feature},
    event_mapper::{MappedEvent, map_controller_event},
    logger::setup_logger,
};

const LOCAL_AUTOMOUNTS: [(&str, usize); 2] = [("/djusb/usb0", 0), ("/djusb/usb1", 1)];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger()?;

    info!("Starting tokio runtime...");

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;

    debug!("Entering tokio to start managers");

    let enter_handle = runtime.enter();

    debug!("Starting IO Manager...");

    let (controller_event_sender, mut controller_event_receiver) =
        tokio::sync::mpsc::unbounded_channel();

    // todo: this needs to be replaced with a proper io manager
    let mut controller = Controller::default();

    'controller_init: {
        if let Err(err) = controller.open_midi_input() {
            warn!("Failed to find MIDI input for testing, MIDI has been disabled: {err}");
            break 'controller_init;
        }
        if let Err(err) = controller.open_midi_output() {
            warn!("Failed to find MIDI output for testing, MIDI has been disabled: {err}");
            break 'controller_init;
        }

        if let Err(err) = controller.start(controller_event_sender) {
            warn!("Failed to start MIDI processing, MIDI has been disabled: {err}");
            break 'controller_init;
        }
    }

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
        loop {
            tokio::select! {
                controller_event = controller_event_receiver.recv() => {
                    if let Some(event) = controller_event {
                        match map_controller_event(event) {
                            MappedEvent::UIControlEvent(ui_control_event) => {
                                let _ = ui_control_event_sender.send(ui_control_event);
                            },
                            MappedEvent::DeckControlEvent(deck_control_event) => {
                                relay_engine_handle.send_control_event(deck_control_event);
                            },
                        }
                    }
                }
                _ = relay_engine_handle.wait_for_new_deck_state_changed() => {
                    let new_deck_state = relay_engine_handle.get_deck_state().clone();

                    #[allow(clippy::cast_precision_loss)]
                    let _ = controller.send(ControllerMessage::UpdateCurrentSamples(array::from_fn(
                        |index| {
                            new_deck_state
                                .mixer_channels
                                .get(index)
                                .map_or(0.0, |channel| {
                                    channel
                                        .average_amplitude
                                        .iter()
                                        .fold(0.0, |accumulator, amplitude| accumulator + amplitude)
                                        / AUDIO_CHANNELS as f32
                                })
                        },
                    )));

                    let _ = controller.send(ControllerMessage::UpdateDeckState(new_deck_state));
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

    unsafe {
        std::env::set_var("QT_IM_MODULE", "qtvirtualkeyboard");
        std::env::set_var("QT_VIRTUALKEYBOARD_STYLE", "slipstream");
    };

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
