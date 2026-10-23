use std::collections::BTreeMap;

use loop_unwrap::{ToOption, unwrap_continue};

use crate::{
    driver_tools::midi::{
        midi_rpm_to_f32, midi_split_pair_to_f32_range, midi_ticks_to_relative_ticks,
    },
    types::controller::ControllerEvent,
};

#[allow(clippy::type_complexity)]
#[derive(Debug)]
pub struct MidiMapping {
    direct: BTreeMap<u16, Vec<(MappingTarget, ControllerEvent)>>,
    direct_equals: BTreeMap<u16, Vec<(usize, MappingTarget, ControllerEvent)>>,
    integrated: [BTreeMap<u16, Vec<(MappingTarget, ControllerEvent)>>; 4],
}

#[derive(Debug)]
pub enum MappingTarget {
    Button(u8),
    ButtonWithInteger(u8, usize),
    ButtonWithFloat(u8, f32),
    Absolute(BytePosition, f32, f32),
    Relative(u8, u8, u8, f32),
    Jog,
}

#[derive(Debug)]
pub enum BytePosition {
    MSB,
    LSB,
}

#[derive(Debug)]
pub enum ChannelType {
    Direct,
    DirectEquals,
    Integrated,
}

pub fn load_midi_mapping(mapping: &str) -> MidiMapping {
    let mut parsed_mapping = MidiMapping {
        direct: BTreeMap::default(),
        direct_equals: BTreeMap::default(),
        integrated: [
            BTreeMap::default(),
            BTreeMap::default(),
            BTreeMap::default(),
            BTreeMap::default(),
        ],
    };

    'outer: for line in mapping.lines() {
        let mut chars = line.chars();

        if unwrap_continue!(chars.next().map(|c| c != '0'), 'outer)
            || unwrap_continue!(chars.next().map(|c| c != 'x'), 'outer)
        {
            continue 'outer;
        }

        let mut number_string: String = String::with_capacity(4);
        let mut channel_type = ChannelType::Direct;
        let mut channel: usize = 0;

        for i in (0..4).rev() {
            let character = unwrap_continue!(chars.next(), 'outer);

            if character == 'n' {
                channel_type = ChannelType::Integrated;
                channel = i;

                number_string.push('0');
            } else {
                number_string.push(character);
            }
        }

        let address = unwrap_continue!(u16::from_str_radix(&number_string, 16), 'outer);

        let mut character = unwrap_continue!(chars.next(), 'outer);

        if character == '=' {
            let mut number_string: String = String::with_capacity(1);

            number_string.push(unwrap_continue!(chars.next(), 'outer));

            channel_type = ChannelType::DirectEquals;
            channel = unwrap_continue!(number_string.parse::<usize>(), 'outer);

            character = unwrap_continue!(chars.next(), 'outer);
        }

        if character != '-' || unwrap_continue!(chars.next().map(|c| c != '>'), 'outer) {
            continue 'outer;
        }

        let character = unwrap_continue!(chars.next(), 'outer);

        if unwrap_continue!(chars.next().map(|c| c != '('), 'outer) {
            continue 'outer;
        }

        let mut args: Vec<String> = vec![String::new()];

        'arg_parser: loop {
            let character = unwrap_continue!(chars.next(), 'outer);

            if character == ' ' || character == '\t' {
            } else if character == ')' {
                break 'arg_parser;
            } else if character == ',' {
                args.push(String::new());
            } else if let Some(last_arg) = args.last_mut() {
                last_arg.push(character);
            }
        }

        if let Some(last_arg) = args.last()
            && last_arg.is_empty()
        {
            args.pop();
        }

        let target = match character {
            'b' => {
                if args.len() != 1 {
                    continue 'outer;
                }

                let trigger: &String = unwrap_continue!(args.first(), 'outer);
                let trigger_radix = if trigger.starts_with("0x") { 16 } else { 10 };
                let trigger = trigger.trim_start_matches("0x");
                let trigger: u8 =
                    unwrap_continue!(u8::from_str_radix(trigger, trigger_radix), 'outer);

                MappingTarget::Button(trigger)
            }
            'x' => {
                if args.len() != 2 {
                    continue 'outer;
                }

                let trigger: &String = unwrap_continue!(args.first(), 'outer);
                let trigger_radix = if trigger.starts_with("0x") { 16 } else { 10 };
                let trigger = trigger.trim_start_matches("0x");
                let trigger: u8 =
                    unwrap_continue!(u8::from_str_radix(trigger, trigger_radix), 'outer);

                let number: &String = unwrap_continue!(args.get(1), 'outer);
                let number_radix = if number.starts_with("0x") { 16 } else { 10 };
                let number = number.trim_start_matches("0x");
                let number: usize =
                    unwrap_continue!(usize::from_str_radix(number, number_radix), 'outer);

                MappingTarget::ButtonWithInteger(trigger, number)
            }
            'y' => {
                if args.len() != 2 {
                    continue 'outer;
                }

                let trigger: &String = unwrap_continue!(args.first(), 'outer);
                let trigger_radix = if trigger.starts_with("0x") { 16 } else { 10 };
                let trigger = trigger.trim_start_matches("0x");
                let trigger: u8 =
                    unwrap_continue!(u8::from_str_radix(trigger, trigger_radix), 'outer);

                let number: &String = unwrap_continue!(args.get(1), 'outer);
                let number: f32 = unwrap_continue!(number.parse::<f32>(), 'outer);

                MappingTarget::ButtonWithFloat(trigger, number)
            }
            'a' => {
                if args.len() != 3 {
                    continue 'outer;
                }

                let byte_position = match args.first().map(String::as_str) {
                    Some("msb") => BytePosition::MSB,
                    Some("lsb") => BytePosition::LSB,
                    _ => {
                        continue 'outer;
                    }
                };

                let min: &String = unwrap_continue!(args.get(1), 'outer);
                let min: f32 = unwrap_continue!(min.parse::<f32>(), 'outer);

                let max: &String = unwrap_continue!(args.get(2), 'outer);
                let max: f32 = unwrap_continue!(max.parse::<f32>(), 'outer);

                MappingTarget::Absolute(byte_position, min, max)
            }
            'r' => {
                if args.len() != 4 {
                    continue 'outer;
                }

                let clockwise_start: &String = unwrap_continue!(args.first(), 'outer);
                let clockwise_start_radix = if clockwise_start.starts_with("0x") {
                    16
                } else {
                    10
                };
                let clockwise_start = clockwise_start.trim_start_matches("0x");
                let clockwise_start: u8 = unwrap_continue!(u8::from_str_radix(clockwise_start, clockwise_start_radix), 'outer);

                let counter_clockwise_start: &String = unwrap_continue!(args.get(1), 'outer);
                let counter_clockwise_start_radix = if counter_clockwise_start.starts_with("0x") {
                    16
                } else {
                    10
                };
                let counter_clockwise_start = counter_clockwise_start.trim_start_matches("0x");
                let counter_clockwise_start: u8 = unwrap_continue!(u8::from_str_radix(counter_clockwise_start, counter_clockwise_start_radix), 'outer);

                let center_threshold: &String = unwrap_continue!(args.get(2), 'outer);
                let center_threshold_radix = if center_threshold.starts_with("0x") {
                    16
                } else {
                    10
                };
                let center_threshold = center_threshold.trim_start_matches("0x");
                let center_threshold: u8 = unwrap_continue!(u8::from_str_radix(center_threshold, center_threshold_radix), 'outer);

                let delta: &String = unwrap_continue!(args.get(3), 'outer);
                let delta: f32 = unwrap_continue!(delta.parse::<f32>(), 'outer);

                MappingTarget::Relative(
                    clockwise_start,
                    counter_clockwise_start,
                    center_threshold,
                    delta,
                )
            }
            'j' => {
                if !args.is_empty() {
                    continue 'outer;
                }

                MappingTarget::Jog
            }
            _ => {
                continue 'outer;
            }
        };

        if unwrap_continue!(chars.next().map(|c| c != ':'), 'outer) {
            continue 'outer;
        }

        let mut event_name: String = String::new();

        #[allow(clippy::while_let_on_iterator)]
        'name_parser: while let Some(character) = chars.next() {
            if character == ' ' || character == '\t' {
            } else if character == '#' {
                break 'name_parser;
            } else {
                event_name.push(character);
            }
        }

        let event = unwrap_continue!(ControllerEvent::from_str(&event_name));

        match channel_type {
            ChannelType::Direct => {
                if let Some(existing_mappings) = parsed_mapping.direct.get_mut(&address) {
                    existing_mappings.push((target, event));
                } else {
                    parsed_mapping.direct.insert(address, vec![(target, event)]);
                }
            }
            ChannelType::DirectEquals => {
                if let Some(existing_mappings) = parsed_mapping.direct_equals.get_mut(&address) {
                    existing_mappings.push((channel, target, event));
                } else {
                    parsed_mapping
                        .direct_equals
                        .insert(address, vec![(channel, target, event)]);
                }
            }
            ChannelType::Integrated => {
                let map = unwrap_continue!(parsed_mapping.integrated.get_mut(channel), 'outer);

                if let Some(existing_mappings) = map.get_mut(&address) {
                    existing_mappings.push((target, event));
                } else {
                    map.insert(address, vec![(target, event)]);
                }
            }
        }
    }

    parsed_mapping
}

pub fn process_midi_command(
    address: u16,
    value: u8,
    mappings: &MidiMapping,
    persistence_store: &mut BTreeMap<(&str, usize), (u8, u8)>,
) -> Option<ControllerEvent> {
    if let Some(mappings) = mappings.direct.get(&address) {
        for (mapping_target, event) in mappings {
            if let Some(event) =
                apply_mapping_to_event(value, 0, mapping_target, *event, persistence_store)
            {
                return Some(event);
            }
        }

        return None;
    }

    if let Some(mappings) = mappings.direct_equals.get(&address) {
        for (channel, mapping_target, action) in mappings {
            if let Some(event) =
                apply_mapping_to_event(value, *channel, mapping_target, *action, persistence_store)
            {
                return Some(event);
            }
        }

        return None;
    }

    for index in (0..4).rev() {
        let masked_address = address & !(0xf << (index * 4));
        let channel = usize::from((address >> (index * 4)) & 0xf);

        if let Some(map) = mappings.integrated.get(index)
            && let Some(mappings) = map.get(&masked_address)
        {
            for (mapping_target, event) in mappings {
                if let Some(event) = apply_mapping_to_event(
                    value,
                    channel,
                    mapping_target,
                    *event,
                    persistence_store,
                ) {
                    return Some(event);
                }
            }

            return None;
        }
    }

    None
}

fn apply_mapping_to_event(
    value: u8,
    channel: usize,
    target: &MappingTarget,
    event: ControllerEvent,
    persistence_store: &mut BTreeMap<(&str, usize), (u8, u8)>,
) -> Option<ControllerEvent> {
    let mut new_position: f32 = 0.0;
    let mut new_velocity: f32 = 0.0;
    let mut new_delta: f32 = 0.0;

    let mut source_parameter_integer: usize = 0;
    let mut source_parameter_float: f32 = 0.0;

    match target {
        MappingTarget::Button(target) => {
            if value != *target {
                return None;
            }
        }
        MappingTarget::ButtonWithInteger(target, number) => {
            if value != *target {
                return None;
            }

            source_parameter_integer = *number;
        }
        MappingTarget::ButtonWithFloat(target, number) => {
            if value != *target {
                return None;
            }

            source_parameter_float = *number;
        }
        MappingTarget::Absolute(byte_position, min, max) => {
            let (msb, lsb) = if let Some((msb, lsb)) =
                persistence_store.get_mut(&(ControllerEvent::to_str(event), channel))
            {
                match byte_position {
                    BytePosition::MSB => {
                        *msb = value;
                    }
                    BytePosition::LSB => {
                        *lsb = value;
                    }
                }

                (*msb, *lsb)
            } else {
                let bytes = match byte_position {
                    BytePosition::MSB => (value, 0),
                    BytePosition::LSB => (0, value),
                };

                persistence_store.insert((ControllerEvent::to_str(event), channel), bytes);

                bytes
            };

            new_position = midi_split_pair_to_f32_range(msb, lsb, *min, *max);
        }
        MappingTarget::Relative(
            clockwise_start,
            counterclockwise_start,
            center_threshold,
            size,
        ) => {
            new_delta = midi_ticks_to_relative_ticks(
                value,
                *clockwise_start,
                *counterclockwise_start,
                *center_threshold,
            ) * size;
        }
        MappingTarget::Jog => {
            new_velocity = midi_rpm_to_f32(value);
        }
    }

    let new_channel = channel;

    let mut new_event = event;

    match &mut new_event {
        ControllerEvent::BrowserEncoderPress
        | ControllerEvent::BrowserSourcePress
        | ControllerEvent::BrowserBrowsePress
        | ControllerEvent::BrowserPlaylistPress
        | ControllerEvent::BrowserBackPress
        | ControllerEvent::MixerMasterFXEnablePress
        | ControllerEvent::BrowserSearchPress
        | ControllerEvent::MixerMasterCuePress
        | ControllerEvent::MixerMasterFXParameterDecreasePress
        | ControllerEvent::MixerMasterFXParameterIncreasePress
        | ControllerEvent::MixerMasterFXTargetMasterPress
        | ControllerEvent::MixerChannelFXProximityPress
        | ControllerEvent::MixerChannelFXProximityRelease => {}

        ControllerEvent::BrowserEncoderAdjust { delta } => {
            *delta = new_delta;
        }

        ControllerEvent::MixerHeadphonesMixSet { position }
        | ControllerEvent::MixerHeadphonesGainSet { position }
        | ControllerEvent::MixerMasterFXDepthSet { position }
        | ControllerEvent::MixerMasterGainSet { position }
        | ControllerEvent::MixerCrossfaderSet { position } => {
            *position = new_position;
        }

        ControllerEvent::MixerMasterFXEffectPress { number }
        | ControllerEvent::MixerMasterFXTargetChannelPress { number }
        | ControllerEvent::MixerChannelFXEffectPress { number }
        | ControllerEvent::MixerChannelFXEffectRelease { number } => {
            *number = source_parameter_integer;
        }

        ControllerEvent::DriverInternalButton { id: channel }
        | ControllerEvent::PlayerPlayPress { channel }
        | ControllerEvent::PlayerCuePress { channel }
        | ControllerEvent::PlayerCueRelease { channel }
        | ControllerEvent::PlayerAltCuePress { channel }
        | ControllerEvent::PlayerAltCueRelease { channel }
        | ControllerEvent::PlayerBeatSyncPress { channel }
        | ControllerEvent::PlayerMasterPress { channel }
        | ControllerEvent::PlayerTempoResetPress { channel }
        | ControllerEvent::PlayerTempoRangePress { channel }
        | ControllerEvent::PlayerMasterTempoPress { channel }
        | ControllerEvent::PlayerJogTouchPress { channel }
        | ControllerEvent::PlayerJogTouchRelease { channel }
        | ControllerEvent::MixerChannelCuePress { channel }
        | ControllerEvent::MixerChannelCrossfaderAssignAPress { channel }
        | ControllerEvent::MixerChannelCrossfaderAssignNonePress { channel }
        | ControllerEvent::MixerChannelCrossfaderAssignBPress { channel }
        | ControllerEvent::PlayerQuantizePress { channel }
        | ControllerEvent::PlayerSlipPress { channel }
        | ControllerEvent::PlayerGateCuePress { channel }
        | ControllerEvent::PlayerReversePress { channel }
        | ControllerEvent::PlayerReverseRelease { channel }
        | ControllerEvent::PlayerSlipReversePress { channel }
        | ControllerEvent::PlayerSlipReverseRelease { channel }
        | ControllerEvent::PlayerBPMSyncPress { channel }
        | ControllerEvent::PlayerKeySyncPress { channel }
        | ControllerEvent::PlayerBeatLoopInPress { channel }
        | ControllerEvent::PlayerBeatLoopInAdjustPress { channel }
        | ControllerEvent::PlayerBeatLoopOutPress { channel }
        | ControllerEvent::PlayerBeatLoopOutAdjustPress { channel }
        | ControllerEvent::PlayerBeatLoopExitPress { channel }
        | ControllerEvent::PlayerReLoopPress { channel } => {
            *channel = new_channel;
        }

        ControllerEvent::PlayerJogVelocitySet { channel, velocity }
        | ControllerEvent::PlayerJogSearchVelocitySet { channel, velocity } => {
            *channel = new_channel;
            *velocity = new_velocity;
        }

        ControllerEvent::DriverInternalButtonWithAbsolute {
            id: channel,
            position,
        }
        | ControllerEvent::MixerChannelGainSet { channel, position }
        | ControllerEvent::MixerChannelEqHighSet { channel, position }
        | ControllerEvent::MixerChannelEqMidSet { channel, position }
        | ControllerEvent::MixerChannelEqLowSet { channel, position }
        | ControllerEvent::MixerChannelFXSet { channel, position }
        | ControllerEvent::PlayerTempoSet { channel, position }
        | ControllerEvent::MixerChannelFaderSet { channel, position } => {
            *channel = new_channel;
            *position = new_position;
        }

        ControllerEvent::DriverInternalButtonWithRelative { id: channel, delta } => {
            *channel = new_channel;
            *delta = new_delta;
        }

        ControllerEvent::DriverInternalButtonWithInteger {
            id: channel,
            number,
        }
        | ControllerEvent::PlayerHotCuePress { channel, number }
        | ControllerEvent::PlayerHotCueRelease { channel, number } => {
            *channel = new_channel;
            *number = source_parameter_integer;
        }

        ControllerEvent::DriverInternalButtonWithFloat {
            id: channel,
            number,
        }
        | ControllerEvent::PlayerBeatJumpPress {
            channel,
            distance: number,
        }
        | ControllerEvent::PlayerInstantLoopPress {
            channel,
            length: number,
        }
        | ControllerEvent::PlayerKeyShiftPress {
            channel,
            shift: number,
        } => {
            *channel = new_channel;
            *number = source_parameter_float;
        }

        ControllerEvent::USBEjectPress { slot } | ControllerEvent::USBEjectRelease { slot } => {
            *slot = source_parameter_integer;
        }
    }

    Some(new_event)
}
