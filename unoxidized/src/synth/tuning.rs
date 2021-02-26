use crate::synth::Synth;
use crate::synth::FLUID_FAILED;
use crate::synth::FLUID_OK;

impl Synth {
    /**
    Create a new key-based tuning with given name, number, and
    pitches. The array 'pitches' should have length 128 and contains
    the pitch in cents of every key in cents. However, if 'pitches' is
    NULL, a new tuning is created with the well-tempered scale.
     */

    pub unsafe fn create_key_tuning(
        &mut self,
        bank: i32,
        prog: i32,
        name: &[u8],
        pitch: &[f64; 128],
    ) -> i32 {
        return match self.create_tuning(bank, prog, name) {
            Some(tuning) => {
                tuning.set_all(pitch);
                FLUID_OK as i32
            }
            None => FLUID_FAILED as i32,
        };
    }

    /**
    Create a new octave-based tuning with given name, number, and
    pitches.  The array 'pitches' should have length 12 and contains
    derivation in cents from the well-tempered scale. For example, if
    pitches[0] equals -33, then the C-keys will be tuned 33 cents
    below the well-tempered C.
     */
    pub unsafe fn create_octave_tuning(
        &mut self,
        bank: i32,
        prog: i32,
        name: &[u8],
        pitch: &[f64; 12],
    ) -> i32 {
        if !(bank >= 0 as i32 && bank < 128 as i32) {
            return FLUID_FAILED as i32;
        }
        if !(prog >= 0 as i32 && prog < 128 as i32) {
            return FLUID_FAILED as i32;
        }
        return match self.create_tuning(bank, prog, name) {
            Some(tuning) => {
                tuning.set_octave(pitch);
                FLUID_OK as i32
            }
            None => FLUID_FAILED as i32,
        };
    }

    pub unsafe fn activate_octave_tuning(
        &mut self,
        bank: i32,
        prog: i32,
        name: &[u8],
        pitch: &[f64; 12],
        _apply: i32,
    ) -> i32 {
        return self.create_octave_tuning(bank, prog, name, pitch);
    }

    /**
    Request a note tuning changes. Both they 'keys' and 'pitches'
    arrays should be of length 'num_pitches'. If 'apply' is non-zero,
    the changes should be applied in real-time, i.e. sounding notes
    will have their pitch updated. 'APPLY' IS CURRENTLY IGNORED. The
    changes will be available for newly triggered notes only.
     */
    pub unsafe fn tune_notes(
        &mut self,
        bank: i32,
        prog: i32,
        len: i32,
        key: *mut i32,
        pitch: *mut f64,
        _apply: i32,
    ) -> i32 {
        if !(bank >= 0 as i32 && bank < 128 as i32) {
            return FLUID_FAILED as i32;
        }
        if !(prog >= 0 as i32 && prog < 128 as i32) {
            return FLUID_FAILED as i32;
        }
        if !(len > 0 as i32) {
            return FLUID_FAILED as i32;
        }
        if key.is_null() {
            return FLUID_FAILED as i32;
        }
        if pitch.is_null() {
            return FLUID_FAILED as i32;
        }
        match self.create_tuning(bank, prog, b"Unnamed\x00") {
            Some(tuning) => {
                for i in 0..len {
                    tuning.set_pitch(*key.offset(i as isize), *pitch.offset(i as isize));
                }
                return FLUID_OK as i32;
            }
            None => {
                return FLUID_FAILED as i32;
            }
        }
    }

    /**
    Select a tuning for a channel.
     */
    pub unsafe fn select_tuning(&mut self, chan: u8, bank: i32, prog: i32) -> i32 {
        let tuning;
        if !(bank >= 0 as i32 && bank < 128 as i32) {
            return FLUID_FAILED as i32;
        }
        if !(prog >= 0 as i32 && prog < 128 as i32) {
            return FLUID_FAILED as i32;
        }
        tuning = self.get_tuning(bank, prog);
        if tuning.is_none() {
            return FLUID_FAILED as i32;
        }
        if chan >= self.settings.synth.midi_channels {
            log::warn!("Channel out of range",);
            return FLUID_FAILED as i32;
        }
        self.channel[chan as usize].tuning = Some(tuning.unwrap().clone());
        return FLUID_OK as i32;
    }

    pub unsafe fn activate_tuning(&mut self, chan: u8, bank: i32, prog: i32, _apply: i32) -> i32 {
        return self.select_tuning(chan, bank, prog);
    }

    /**
    Set the tuning to the default well-tempered tuning on a channel.
     */
    pub unsafe fn reset_tuning(&mut self, chan: u8) -> i32 {
        if chan >= self.settings.synth.midi_channels {
            log::warn!("Channel out of range",);
            return FLUID_FAILED as i32;
        }
        self.channel[chan as usize].tuning = None;
        return FLUID_OK as i32;
    }

    pub unsafe fn tuning_iteration_start(&mut self) {
        self.cur_tuning = None;
    }

    pub unsafe fn tuning_iteration_next(&mut self, bank: *mut i32, prog: *mut i32) -> i32 {
        let mut b = 0;
        let mut p = 0;
        match self.cur_tuning.as_ref() {
            Some(tuning) => {
                b = tuning.bank;
                p = tuning.prog + 1;
                if p >= 128 {
                    p = 0;
                    b += 1
                }
            }
            None => {}
        }
        while b < 128 {
            while p < 128 {
                match self.tuning[b as usize][p as usize] {
                    Some(_) => {
                        *bank = b;
                        *prog = p;
                        return 1;
                    }
                    None => {}
                }
                p += 1
            }
            p = 0 as i32;
            b += 1
        }
        return 0 as i32;
    }

    pub unsafe fn tuning_dump(
        &self,
        bank: i32,
        prog: i32,
        name: *mut i8,
        len: i32,
        pitch: *mut f64,
    ) -> i32 {
        match self.get_tuning(bank, prog) {
            Some(tuning) => {
                if !name.is_null() {
                    libc::strncpy(
                        name,
                        tuning.get_name().as_ptr() as _,
                        (len - 1 as i32) as libc::size_t,
                    );
                    *name.offset((len - 1 as i32) as isize) = 0 as i32 as i8
                }
                if !pitch.is_null() {
                    libc::memcpy(
                        pitch as *mut libc::c_void,
                        tuning.pitch.as_ptr().offset(0 as i32 as isize) as *mut f64
                            as *const libc::c_void,
                        (128 as i32 as libc::size_t)
                            .wrapping_mul(::std::mem::size_of::<f64>() as libc::size_t),
                    );
                }
                return FLUID_OK as i32;
            }
            None => {
                return FLUID_FAILED as i32;
            }
        }
    }
}
