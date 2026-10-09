import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window

import engineering.fizzy.slipstream_embedded

import "qrc:/test_images"
import "qrc:/shaders"

FlexboxLayout {
    id: root
    required property EngineBridge engine
    required property int player_number

    direction: FlexboxLayout.Row
    justifyContent: FlexboxLayout.JustifySpaceAround
    alignItems: FlexboxLayout.AlignCenter

    Item {
        id: waveform_container

        clip: true

        Layout.fillWidth: true
        Layout.fillHeight: true

        function position_element(at) {
    	    let center_factor = (waveform_container.x + (waveform_container.width / 2));

    	    let waveform_width = (root.engine.deck_state.mixer_channel(root.player_number).player.waveform_length_seconds * root.engine.waveform_pixels_per_second) / Math.max(root.engine.deck_state.mixer_channel(root.player_number).player.tempo_percent, 0.1);

    	    let relative_time;

         	if (root.engine.deck_state.mixer_channel(root.player_number).player.beat_loop_adjust_mode == BeatLoopAdjustMode.In) {
        		relative_time = root.engine.deck_state.mixer_channel(root.player_number).player.beat_loop_start - at;
          	} else if (root.engine.deck_state.mixer_channel(root.player_number).player.beat_loop_adjust_mode == BeatLoopAdjustMode.Out) {
         		relative_time = root.engine.deck_state.mixer_channel(root.player_number).player.beat_loop_end - at;
           	} else {
           		relative_time = root.engine.deck_state.mixer_channel(root.player_number).player.time - at;
           	}

    	    let progress = (relative_time / 1000000000) / root.engine.deck_state.mixer_channel(root.player_number).player.waveform_length_seconds;

    	    return center_factor - (waveform_width * progress);
        }

        function size_element(length_seconds) {
        	return (length_seconds * root.engine.waveform_pixels_per_second) / Math.max(root.engine.deck_state.mixer_channel(root.player_number).player.tempo_percent, 0.1);
        }

        Repeater {
            model: root.engine.deck_state.mixer_channel(root.player_number).player.beat_grid

            Rectangle {
            	required property var beat_number
            	required property var time
                required property int index

                x: waveform_container.position_element(time)
                width: 1
                height: waveform_container.height
                y: 0

                visible: (beat_number == 0) ? true : (root.engine.waveform_pixels_per_second > 25)

                color: (beat_number == 0) ? "#ff0000" : palette.light
            }
        }

        Rectangle {
        	x: 0
        	width: waveform_container.width
        	height: waveform_container.height - 16
        	y: 8

         	color: palette.window
        }

        // waveform
        ShaderEffect {
            x: waveform_container.position_element(0)

            y: 8
            width: waveform_container.size_element(root.engine.deck_state.mixer_channel(root.player_number).player.waveform_length_seconds)
            height: waveform_container.height - 16

            vertexShader: "qrc:/shaders/waveform.vert.qsb"
            fragmentShader: "qrc:/shaders/waveform.frag.qsb"

            property real stride: root.engine.deck_state.mixer_channel(root.player_number).player.waveform_texture_stride
            property variant waveform: root.engine.deck_state.mixer_channel(root.player_number).player.waveform_texture_source
        }

        // beat loop area
        Rectangle {
            x: waveform_container.position_element(root.engine.deck_state.mixer_channel(root.player_number).player.beat_loop_start)
            y: 0
            width: waveform_container.size_element((root.engine.deck_state.mixer_channel(root.player_number).player.beat_loop_end - root.engine.deck_state.mixer_channel(root.player_number).player.beat_loop_start) / 1000000000)
            height: waveform_container.height

            visible: root.engine.deck_state.mixer_channel(root.player_number).player.beat_loop_start_set && root.engine.deck_state.mixer_channel(root.player_number).player.beat_loop_end_set

            color: "#3eff820e"
        }

        // beat loop in
        Rectangle {
            x: waveform_container.position_element(root.engine.deck_state.mixer_channel(root.player_number).player.beat_loop_start)
            y: 0
            width: 1
            height: waveform_container.height

            visible: root.engine.deck_state.mixer_channel(root.player_number).player.beat_loop_start_set

            color: "#ff820e"
        }

        Rectangle {
            x: waveform_container.position_element(root.engine.deck_state.mixer_channel(root.player_number).player.beat_loop_start)
            y: 0
            width: 8
            height: 1

            visible: root.engine.deck_state.mixer_channel(root.player_number).player.beat_loop_start_set

            color: "#ff820e"
        }

        Rectangle {
            x: waveform_container.position_element(root.engine.deck_state.mixer_channel(root.player_number).player.beat_loop_start)
            y: waveform_container.height - 1
            width: 8
            height: 1

            visible: root.engine.deck_state.mixer_channel(root.player_number).player.beat_loop_start_set

            color: "#ff820e"
        }

        // beat loop end
        Rectangle {
            x: waveform_container.position_element(root.engine.deck_state.mixer_channel(root.player_number).player.beat_loop_end)
            y: 0
            width: 1
            height: waveform_container.height

            visible: root.engine.deck_state.mixer_channel(root.player_number).player.beat_loop_end_set

            color: "#ff820e"
        }

        Rectangle {
            x: waveform_container.position_element(root.engine.deck_state.mixer_channel(root.player_number).player.beat_loop_end) - 8
            y: 0
            width: 8
            height: 1

            visible: root.engine.deck_state.mixer_channel(root.player_number).player.beat_loop_end_set

            color: "#ff820e"
        }

        Rectangle {
            x: waveform_container.position_element(root.engine.deck_state.mixer_channel(root.player_number).player.beat_loop_end) - 8
            y: waveform_container.height - 1
            width: 8
            height: 1

            visible: root.engine.deck_state.mixer_channel(root.player_number).player.beat_loop_end_set

            color: "#ff820e"
        }

        // touch cue
        Rectangle {
            x: waveform_container.position_element(root.engine.deck_state.mixer_channel(root.player_number).player.touch_cue_time)
            y: 0
            width: 1
            height: waveform_container.height

            visible: root.engine.deck_state.mixer_channel(root.player_number).player.touch_cue_time_set

            color: "#00ff00"
        }

        // slip
        Rectangle {
            x: waveform_container.position_element(root.engine.deck_state.mixer_channel(root.player_number).player.slip_time)
            y: 0
            width: 1
            height: waveform_container.height

            visible: {
            	let player = root.engine.deck_state.mixer_channel(root.player_number).player;

             	return player.slip_playing || player.play_direction == PlayDirection.SlipReverse;
            }

            color: "#ff0000"
        }

        // cue
        Rectangle {
            x: waveform_container.position_element(root.engine.deck_state.mixer_channel(root.player_number).player.cue_time)
            y: 0
            width: 1
            height: waveform_container.height

            visible: root.engine.deck_state.mixer_channel(root.player_number).player.cue_time_set

            color: "#ff820e"
        }

        // playhead
        Rectangle {
            x: {
            	if (root.engine.deck_state.mixer_channel(root.player_number).player.beat_loop_adjust_mode == BeatLoopAdjustMode.None) {
            		return waveform_container.x + (waveform_container.width / 2);
             	} else {
              		return waveform_container.position_element(root.engine.deck_state.mixer_channel(root.player_number).player.time);
              	}
            }
            y: 0
            width: 1
            height: waveform_container.height

            color: palette.light
        }
    }
}
