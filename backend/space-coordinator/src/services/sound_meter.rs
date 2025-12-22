use std::collections::HashMap;

use tasi_sound_level_meter::TasiSoundLevelMeter;

pub struct Tasi653bSoundLevelMeter {
    meter: TasiSoundLevelMeter,
}

pub enum SoundMeterDevice {
    Tasi653b(Tasi653bSoundLevelMeter),
}

pub struct SoundMeterService {
    devices: HashMap<String, SoundMeterDevice>,
}
