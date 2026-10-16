// this code fucking sucks and i need to rewrite it, it's piecemeal from a bunch
// of sources and i barely understand it, sorry

use std::{collections::VecDeque, f64::consts::TAU, sync::Arc};

use rustfft::{Fft, FftPlanner, num_complex::Complex};

#[derive(Clone, Debug)]
pub struct Config {
    pub sample_rate: f32,
    pub channels: usize,
    pub block_len: usize,
    pub overlap: usize,
    pub kaiser_beta: f64,
    pub formant_compensation: bool,
    pub formant_width_hz: f32,
}

impl Config {
    pub fn new(sample_rate: f32, channels: usize) -> Self {
        Self::with_block_seconds(sample_rate, channels, 0.12, 4)
    }

    pub fn fast(sample_rate: f32, channels: usize) -> Self {
        Self::with_block_seconds(sample_rate, channels, 0.08, 4)
    }

    pub fn with_block_seconds(
        sample_rate: f32,
        channels: usize,
        seconds: f32,
        overlap: usize,
    ) -> Self {
        let target = (sample_rate * seconds).round() as usize;

        Config {
            sample_rate,
            channels,
            block_len: smooth_block_len(target.max(overlap * 8), overlap),
            overlap,
            kaiser_beta: 6.0,
            formant_compensation: false,
            formant_width_hz: 300.0,
        }
    }
}

fn smooth_block_len(target: usize, overlap: usize) -> usize {
    let mut n = target.next_multiple_of(overlap);

    loop {
        let mut n_check = n;

        for p in [2, 3, 5] {
            while n_check.is_multiple_of(p) {
                n_check /= p;
            }
        }

        if n.is_multiple_of(2) && n_check == 1 {
            return n;
        }

        n += overlap;
    }
}

fn bessel_i0(x: f64) -> f64 {
    let q = x * x / 4.0;
    let (mut sum, mut term) = (1.0, 1.0);

    for k in 1..100 {
        term *= q / (k * k) as f64;
        sum += term;

        if term < sum * 1e-17 {
            break;
        }
    }

    sum
}

fn kaiser(n: usize, beta: f64) -> Vec<f32> {
    let d = bessel_i0(beta);

    (0..n)
        .map(|i| {
            let x = 2.0 * i as f64 / n as f64 - 1.0;

            (bessel_i0(beta * (1.0 - x * x).max(0.0).sqrt()) / d) as f32
        })
        .collect()
}

#[inline]
fn wrap(x: f64) -> f64 {
    x - TAU * (x / TAU).round()
}

#[inline]
fn pw_at(pw: &[f32], k: isize) -> f32 {
    if k < 0 || k as usize >= pw.len() {
        0.0
    } else {
        pw[k as usize]
    }
}

fn boxf(src: &[f32], dst: &mut [f32], prefix: &mut Vec<f64>, r: usize) {
    prefix.clear();
    prefix.push(0.0);

    let mut acc = 0.0f64;

    for &v in src {
        acc += v as f64;

        prefix.push(acc);
    }

    let n = src.len();

    for (k, dst) in dst[0..n].iter_mut().enumerate() {
        let lo = k.saturating_sub(r);
        let hi = (k + r + 1).min(n);

        *dst = ((prefix[hi] - prefix[lo]) / (hi - lo) as f64) as f32;
    }
}

fn smooth(src: &[f32], dst: &mut [f32], tmp: &mut [f32], prefix: &mut Vec<f64>, radius: usize) {
    boxf(src, tmp, prefix, radius);
    boxf(tmp, dst, prefix, radius);
}

const FREEZE_DECAY: f32 = 0.9;

pub struct Stretcher {
    cfg: Config,
    n: usize,
    half: usize,
    hop: usize,
    fft: Arc<dyn Fft<f32>>,
    ifft: Arc<dyn Fft<f32>>,
    fft_scratch: Vec<Complex<f32>>,
    window: Vec<f32>,
    norm: Vec<f32>,

    speed: f64,
    pitch_ratio: f64,

    started: bool,
    lag: usize,
    skip: usize,
    owed: f64,
    freeze_count: u32,
    underruns: u64,

    in_buf: Vec<Vec<f32>>,
    in_origin: i64,
    next_pos: f64,
    prev_pos: i64,
    have_prev: bool,
    prev_ref: Vec<Complex<f32>>,
    prev_out_ref: Vec<Complex<f32>>,
    acc: Vec<Vec<f32>>,
    out_q: Vec<VecDeque<f32>>,

    spec: Vec<Vec<Complex<f32>>>,
    r: Vec<Complex<f32>>,
    pw: Vec<f32>,
    max_pw: f32,
    peaks: Vec<usize>,
    bounds: Vec<usize>,
    peak_fest: Vec<f64>,
    peak_omega: Vec<f64>,

    fftbuf: Vec<Complex<f32>>,
    out_spec: Vec<Vec<Complex<f32>>>,
    pw_out: Vec<f32>,
    owner: Vec<f32>,
    env_in: Vec<f32>,
    env_out: Vec<f32>,
    env_tmp: Vec<f32>,
    prefix: Vec<f64>,
}

impl Stretcher {
    pub fn new(cfg: Config) -> Self {
        let n = cfg.block_len;
        let half = n / 2;
        let hop = n / cfg.overlap;
        let nb = half + 1;
        let ch = cfg.channels;

        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(n);
        let ifft = planner.plan_fft_inverse(n);
        let scratch_len = fft
            .get_inplace_scratch_len()
            .max(ifft.get_inplace_scratch_len());

        let window = kaiser(n, cfg.kaiser_beta);

        let norm: Vec<f32> = (0..hop)
            .map(|i| {
                (0..cfg.overlap)
                    .map(|j| window[i + j * hop].powi(2))
                    .sum::<f32>()
            })
            .collect();

        let z = Complex::<f32>::new(0.0, 0.0);

        let mut s = Stretcher {
            n,
            half,
            hop,
            fft,
            ifft,
            fft_scratch: vec![z; scratch_len],
            window,
            norm,
            speed: 1.0,
            pitch_ratio: 1.0,
            started: false,
            lag: 0,
            skip: 0,
            owed: 0.0,
            freeze_count: 0,
            underruns: 0,
            in_buf: vec![Vec::new(); ch],
            in_origin: 0,
            next_pos: 0.0,
            prev_pos: 0,
            have_prev: false,
            prev_ref: vec![z; nb],
            prev_out_ref: vec![z; nb],
            acc: vec![vec![0.0; n]; ch],
            out_q: vec![VecDeque::new(); ch],
            spec: vec![vec![z; nb]; ch],
            r: vec![z; nb],
            pw: vec![0.0; nb],
            max_pw: 0.0,
            peaks: Vec::new(),
            bounds: Vec::new(),
            peak_fest: Vec::new(),
            peak_omega: Vec::new(),
            fftbuf: vec![z; n],
            out_spec: vec![vec![z; nb]; ch],
            pw_out: vec![0.0; nb],
            owner: vec![0.0; nb],
            env_in: vec![0.0; nb],
            env_out: vec![0.0; nb],
            env_tmp: vec![0.0; nb],
            prefix: Vec::new(),
            cfg,
        };

        s.reset();

        s
    }

    pub fn set_speed(&mut self, speed: f64) {
        self.speed = speed;
    }

    pub fn speed(&self) -> f64 {
        self.speed
    }

    pub fn output_ratio(&self) -> f64 {
        1.0 / self.speed
    }

    pub fn set_pitch_ratio(&mut self, ratio: f64) {
        self.pitch_ratio = ratio;
    }

    pub fn set_pitch_semitones(&mut self, semitones: f64) {
        self.set_pitch_ratio((semitones / 12.0).exp2());
    }

    pub fn pitch_ratio(&self) -> f64 {
        self.pitch_ratio
    }

    pub fn set_formant_compensation(&mut self, on: bool) {
        self.cfg.formant_compensation = on;
    }

    fn lag_for(n: usize, speed: f64) -> usize {
        ((n as f64 / 2.0) * (1.0 + speed)).ceil() as usize + 2
    }

    pub fn latency(&self) -> usize {
        if self.started {
            self.lag
        } else {
            Self::lag_for(self.n, self.speed)
        }
    }

    pub fn output_latency(&self) -> usize {
        (self.latency() as f64 / self.speed).round() as usize
    }

    pub fn block_len(&self) -> usize {
        self.n
    }

    pub fn hop(&self) -> usize {
        self.hop
    }

    pub fn owed(&self) -> usize {
        (self.owed + 1e-9).floor().max(0.0) as usize
    }

    pub fn underruns(&self) -> u64 {
        self.underruns
    }

    pub fn reset(&mut self) {
        self.started = false;
        self.owed = 0.0;
        self.freeze_count = 0;
        self.underruns = 0;

        for b in &mut self.in_buf {
            b.clear();
        }

        for a in &mut self.acc {
            a.fill(0.0);
        }

        for q in &mut self.out_q {
            q.clear();
        }

        self.have_prev = false;
        self.prev_ref.fill(Complex::<f32>::new(0.0, 0.0));
        self.prev_out_ref.fill(Complex::<f32>::new(0.0, 0.0));
        self.peaks.clear();
        self.bounds.clear();
        self.peak_fest.clear();
        self.peak_omega.clear();
        self.max_pw = 0.0;
    }

    fn start(&mut self) {
        if self.started {
            return;
        }

        let n = self.n as f64;

        self.lag = Self::lag_for(self.n, self.speed);

        let o0 = -(n - self.hop as f64);
        let pos0 = (o0 + n / 2.0) * self.speed - n / 2.0 - self.lag as f64;

        self.next_pos = pos0;
        self.in_origin = pos0.round() as i64;

        let pad = (-self.in_origin).max(0) as usize;

        for b in &mut self.in_buf {
            b.clear();

            b.resize(pad, 0.0);
        }

        self.skip = self.n - self.hop;
        self.started = true;
    }

    pub fn push<S: AsRef<[f32]>>(&mut self, input: &[S]) {
        let len = input[0].as_ref().len();

        self.start();

        for (c, inp) in input.iter().enumerate() {
            self.in_buf[c].extend_from_slice(inp.as_ref());
        }

        self.owed += len as f64 / self.speed;
    }

    pub fn push_interleaved(&mut self, input: &[f32]) {
        let ch = self.cfg.channels;
        let planar: Vec<Vec<f32>> = (0..ch)
            .map(|c| input.iter().skip(c).step_by(ch).copied().collect())
            .collect();

        self.push(&planar);
    }

    fn push_silence(&mut self, count: usize) {
        self.start();

        for b in &mut self.in_buf {
            b.resize(b.len() + count, 0.0);
        }
    }

    pub fn pull_into<S: AsMut<[f32]>>(&mut self, out: &mut [S]) {
        let want = out[0].as_mut().len();

        self.start();

        while self.out_q[0].len() < want {
            let pos = self.next_pos.round() as i64;
            let end = self.in_origin + self.in_buf[0].len() as i64;

            if pos + self.n as i64 <= end {
                self.process_frame(pos);
            } else {
                self.freeze_frame();
            }
        }

        for (c, o) in out.iter_mut().enumerate() {
            let o = o.as_mut();

            for v in o.iter_mut() {
                *v = self.out_q[c].pop_front().unwrap();
            }
        }

        self.owed -= want as f64;
    }

    pub fn pull(&mut self, output: &mut [Vec<f32>], n: usize) {
        let starts: Vec<usize> = output.iter().map(|o| o.len()).collect();

        for o in output.iter_mut() {
            o.resize(o.len() + n, 0.0);
        }

        let mut slices: Vec<&mut [f32]> = output
            .iter_mut()
            .zip(starts)
            .map(|(o, s)| &mut o[s..])
            .collect();

        self.pull_into(&mut slices);
    }

    pub fn pull_owed(&mut self, output: &mut [Vec<f32>]) {
        let n = self.owed();

        self.pull(output, n);
    }

    pub fn pull_interleaved(&mut self, out: &mut [f32]) {
        let ch = self.cfg.channels;
        let frames = out.len() / ch;
        let mut planar = vec![vec![0.0f32; frames]; ch];

        self.pull_into(&mut planar);

        for f in 0..frames {
            for c in 0..ch {
                out[f * ch + c] = planar[c][f];
            }
        }
    }

    pub fn process<S: AsRef<[f32]>>(&mut self, input: &[S], output: &mut [Vec<f32>]) {
        self.push(input);

        self.pull_owed(output);
    }

    pub fn process_interleaved(&mut self, input: &[f32], output: &mut Vec<f32>) {
        self.push_interleaved(input);

        let ch = self.cfg.channels;
        let n = self.owed();
        let at = output.len();

        output.resize(at + n * ch, 0.0);

        self.pull_interleaved(&mut output[at..]);
    }

    pub fn flush(&mut self, output: &mut [Vec<f32>]) {
        self.push_silence(self.flush_zeros());

        let n = (self.lag as f64 / self.speed + self.owed.max(0.0)).ceil() as usize;

        self.pull(output, n);
        self.owed = 0.0;
    }

    fn flush_zeros(&self) -> usize {
        (self.n as f64 / 2.0 * (1.0 + self.speed)).ceil() as usize + self.hop + 4
    }

    pub fn process_buffer(&mut self, input: &[Vec<f32>]) -> Vec<Vec<f32>> {
        self.reset();

        self.start();

        let target = (input[0].len() as f64 / self.speed).round() as usize;
        let lead = self.output_latency();

        for (c, inp) in input.iter().enumerate() {
            self.in_buf[c].extend_from_slice(inp);
        }

        self.push_silence(self.flush_zeros());

        let mut out = vec![Vec::new(); self.cfg.channels];

        self.pull(&mut out, lead + target);

        for o in &mut out {
            o.drain(..lead);
        }

        self.reset();

        out
    }

    fn process_frame(&mut self, pos: i64) {
        let (n, half) = (self.n, self.half);
        let nb = half + 1;
        let chs = self.cfg.channels;
        let start = (pos - self.in_origin) as usize;
        let zero = Complex::<f32>::new(0.0, 0.0);

        for (in_buf, spec) in self.in_buf[0..chs].iter().zip(&mut self.spec[0..chs]) {
            let src = &in_buf[start..start + n];

            for (i, fftbuf) in self.fftbuf[0..n].iter_mut().enumerate() {
                let j = if i < half { i + half } else { i - half };

                *fftbuf = Complex::<f32>::new(src[j] * self.window[j], 0.0);
            }

            self.fft
                .process_with_scratch(&mut self.fftbuf, &mut self.fft_scratch);

            spec.copy_from_slice(&self.fftbuf[..nb]);
        }

        let mut max_pw = 0.0f32;

        for (k, (r, pw)) in self.r[0..nb]
            .iter_mut()
            .zip(&mut self.pw[0..nb])
            .enumerate()
        {
            let mut s = zero;
            let mut p = 0.0f32;

            for spec in &self.spec[0..chs] {
                let v = spec[k];

                s += v;
                p += v.norm_sqr();
            }

            *r = s;
            *pw = p;

            max_pw = max_pw.max(p);
        }

        self.max_pw = max_pw;

        self.peaks.clear();
        self.bounds.clear();

        if max_pw > 1e-20 {
            let floor = max_pw * 1e-10;

            for (k, p) in self.pw[0..nb].iter().enumerate() {
                if *p <= floor {
                    continue;
                }

                let ki = k as isize;

                if *p > pw_at(&self.pw, ki - 1)
                    && *p > pw_at(&self.pw, ki - 2)
                    && *p >= pw_at(&self.pw, ki + 1)
                    && *p >= pw_at(&self.pw, ki + 2)
                {
                    self.peaks.push(k);
                }
            }

            for i in 0..self.peaks.len() {
                if i == 0 {
                    self.bounds.push(0);
                } else {
                    let (a, b) = (self.peaks[i - 1], self.peaks[i]);
                    let mut best = a + 1;

                    for k in a + 1..=b {
                        if self.pw[k] < self.pw[best] {
                            best = k;
                        }
                    }

                    self.bounds.push(best);
                }
            }
        }

        self.peak_fest.clear();
        self.peak_omega.clear();

        let ha = if self.have_prev {
            (pos - self.prev_pos) as f64
        } else {
            0.0
        };

        for kp in &self.peaks {
            let cr = self.r[*kp];

            let d = if *kp > 0 && *kp < half {
                let a = ((self.pw[*kp - 1] as f64) + 1e-30).ln();
                let b = ((self.pw[*kp] as f64) + 1e-30).ln();
                let c = ((self.pw[*kp + 1] as f64) + 1e-30).ln();

                let den = a - 2.0 * b + c;

                if den < -1e-12 {
                    (0.5 * (a - c) / den).clamp(-0.499, 0.499)
                } else {
                    0.0
                }
            } else {
                0.0
            };

            let f_est = *kp as f64 + d;
            let mut omega = TAU * f_est / n as f64;
            let pr = self.prev_ref[*kp];

            if self.have_prev && ha >= 1.0 && pr.norm_sqr() > 1e-6 * cr.norm_sqr() {
                let dphi = (cr * pr.conj()).arg() as f64;

                omega += wrap(dphi - omega * ha) / ha;
            }

            self.peak_fest.push(f_est);
            self.peak_omega.push(omega);
        }

        self.freeze_count = 0;

        self.synthesize();

        self.prev_ref.copy_from_slice(&self.r);
        self.prev_pos = pos;
        self.have_prev = true;
        self.next_pos += self.hop as f64 * self.speed;

        let keep = self.next_pos.round() as i64;
        let drop = ((keep - self.in_origin).max(0) as usize).min(self.in_buf[0].len());

        if drop > 0 {
            for b in &mut self.in_buf {
                b.drain(..drop);
            }

            self.in_origin += drop as i64;
        }
    }

    fn freeze_frame(&mut self) {
        self.freeze_count += 1;
        self.underruns += 1;

        self.synthesize();
    }

    fn synthesize(&mut self) {
        let (n, half, hop) = (self.n, self.half, self.hop);
        let nb = half + 1;
        let chs = self.cfg.channels;
        let pitch = self.pitch_ratio;
        let zero = Complex::<f32>::new(0.0, 0.0);

        for out_spec in &mut self.out_spec[0..chs] {
            out_spec.fill(zero);
        }

        self.owner.fill(0.0);

        for (i, (((kp, lo), peak_fest), peak_omega)) in self
            .peaks
            .iter()
            .zip(&self.bounds)
            .zip(&self.peak_fest)
            .zip(&self.peak_omega)
            .enumerate()
        {
            let hi = if i + 1 < self.peaks.len() {
                self.bounds[i + 1] - 1
            } else {
                nb - 1
            };
            let cr = self.r[*kp];
            let kt = (*peak_fest * pitch).round() as i64;

            if kt < 0 || kt > half as i64 {
                continue;
            }

            let shift = kt - *kp as i64;

            let mut g = Complex::<f32>::new(1.0, 0.0);
            let yp = self.prev_out_ref[kt as usize];
            let (ey, ec) = (yp.norm_sqr(), cr.norm_sqr());

            if ec > 0.0 && ey > 1e-3 * ec {
                let adv = wrap(*peak_omega * pitch * hop as f64);
                let rot = Complex::<f32>::new(adv.cos() as f32, adv.sin() as f32);

                g = (yp / ey.sqrt()) * rot * (cr.conj() / ec.sqrt());
            }

            for k in *lo..=hi {
                let kk = k as i64 + shift;

                if kk < 0 || kk > half as i64 {
                    continue;
                }

                let kk = kk as usize;
                let e = self.pw[k];

                if e > self.owner[kk] {
                    self.owner[kk] = e;

                    for (out_spec, spec) in self.out_spec[0..chs].iter_mut().zip(&self.spec[0..chs])
                    {
                        out_spec[kk] = spec[k] * g;
                    }
                }
            }
        }

        for (k, prev_out_ref) in self.prev_out_ref[0..nb].iter_mut().enumerate() {
            let mut s = zero;

            for out_spec in &self.out_spec[0..chs] {
                s += out_spec[k];
            }

            *prev_out_ref = s;
        }

        if self.cfg.formant_compensation && (pitch - 1.0).abs() > 1e-6 && self.max_pw > 0.0 {
            for (k, pw_out) in self.pw_out[0..nb].iter_mut().enumerate() {
                *pw_out = self.out_spec[0..chs]
                    .iter()
                    .map(|out_spec| out_spec[k].norm_sqr())
                    .sum();
            }

            let radius = ((self.cfg.formant_width_hz * n as f32 / self.cfg.sample_rate) * 0.5)
                .round()
                .max(2.0) as usize;

            smooth(
                &self.pw,
                &mut self.env_in,
                &mut self.env_tmp,
                &mut self.prefix,
                radius,
            );

            smooth(
                &self.pw_out,
                &mut self.env_out,
                &mut self.env_tmp,
                &mut self.prefix,
                radius,
            );

            let eps = self.max_pw * 1e-8;

            for (k, (env_in, env_out)) in self.env_in[0..nb]
                .iter()
                .zip(&self.env_out[0..nb])
                .enumerate()
            {
                let g = ((*env_in + eps) / (*env_out + eps)).sqrt().clamp(0.1, 10.0);

                for out_spec in &mut self.out_spec[0..chs] {
                    out_spec[k] *= g;
                }
            }
        }

        if self.freeze_count > 0 {
            let fade = FREEZE_DECAY.powi(self.freeze_count as i32);

            for c in &mut self.out_spec[0..chs] {
                for v in c {
                    *v *= fade;
                }
            }
        }

        let inv_n = 1.0 / n as f32;
        let drop_now = self.skip.min(hop);

        for ((out_spec, acc), out_q) in self.out_spec[0..chs]
            .iter()
            .zip(&mut self.acc[0..chs])
            .zip(&mut self.out_q[0..chs])
        {
            self.fftbuf[..nb].copy_from_slice(out_spec);
            self.fftbuf[0].im = 0.0;
            self.fftbuf[half].im = 0.0;

            for k in 1..half {
                self.fftbuf[n - k] = self.fftbuf[k].conj();
            }

            self.ifft
                .process_with_scratch(&mut self.fftbuf, &mut self.fft_scratch);

            for (j, (acc, window)) in acc[0..n].iter_mut().zip(&self.window[0..n]).enumerate() {
                let i = if j < half { j + half } else { j - half };

                *acc += self.fftbuf[i].re * inv_n * window;
            }

            for (acc, norm) in acc[drop_now..hop].iter().zip(&self.norm[drop_now..hop]) {
                out_q.push_back(acc / norm);
            }

            acc.copy_within(hop.., 0);

            for v in &mut acc[n - hop..] {
                *v = 0.0;
            }
        }

        self.skip -= drop_now;
    }
}
