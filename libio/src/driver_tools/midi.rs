pub const fn combine_addr_channel(addr: u16, channel: u16) -> u16 {
    addr + (channel << 8)
}

pub const fn addr_value_to_midi(addr: u16, command: u8) -> [u8; 3] {
    #[allow(clippy::cast_possible_truncation)]
    [(addr >> 8) as u8, (addr & 0xFF) as u8, command]
}

pub const fn addr_channel_value_to_midi(addr: u16, channel: u16, command: u8) -> [u8; 3] {
    let addr = combine_addr_channel(addr, channel);

    #[allow(clippy::cast_possible_truncation)]
    [(addr >> 8) as u8, (addr & 0xFF) as u8, command]
}

pub const fn midi_to_addr_value(midi: [u8; 3]) -> (u16, u8) {
    (((midi[0] as u16) << 8) + (midi[1] as u16), midi[2])
}

pub fn midi_split_pair_to_f32_range(msb: u8, lsb: u8, min: f32, max: f32) -> f32 {
    f32::from((u16::from(msb) << 7) | u16::from(lsb)) / 16383.0 * (max - min) + min
}

#[allow(clippy::cast_lossless)]
pub fn midi_rpm_to_f32(jog: u8) -> f32 {
    if jog >= 0x41 {
        // +
        (6.0 * jog as f32) - 390.0
    } else if jog <= 0x3F {
        // -
        (6.0 * jog as f32) - 378.0
    } else {
        0.0
    }
}

pub fn midi_ticks_to_relative_ticks(
    value: u8,
    clockwise_start: u8,
    counterclockwise_start: u8,
    center_threshold: u8,
) -> f32 {
    if value <= center_threshold {
        (f32::from(value) - f32::from(clockwise_start)) + 1.0
    } else {
        -((f32::from(counterclockwise_start) - f32::from(value)) + 1.0)
    }
}
