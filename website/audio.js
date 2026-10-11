/**
 * Original, generated demo music. No samples, downloads, or dependencies.
 * Call play() from a user gesture. beat and bar are zero-based, each 0–3.
 */
export class SoundDemo {
  constructor() {
    this.context = null;
    this.playing = false;
    this.bpm = 126;
    this.mix = 0.5;
    this.filter = 1;
    this._timer = null;
    this._voices = new Set();
    this._timeline = [];
    this._step = 0;
    this._nextTime = 0;
    this._generation = 0;
    this._pendingPlay = null;
    this._destroyed = false;
  }

  async play() {
    if (this._destroyed) throw new Error("This sound demo has been closed. Reload the page to play again.");
    if (this.playing) return true;
    if (this._pendingPlay) return this._pendingPlay;

    const generation = this._generation;
    const task = (async () => {
      try {
        this._createAudio();
        await this.context.resume();
        if (generation !== this._generation || this._destroyed) {
          throw new Error("Playback was cancelled.");
        }
        if (this.context.state !== "running") {
          throw new Error("Your browser paused audio. Tap Play again to enable sound.");
        }
        this.playing = true;
        this._step = 0;
        this._timeline = [];
        this._nextTime = this.context.currentTime + 0.055;
        this._ramp(this._output.gain, 0.65, 0.025);
        this._schedule();
        this._timer = globalThis.setInterval(() => this._schedule(), 25);
        return true;
      } catch (error) {
        if (generation === this._generation) this.pause();
        throw error;
      }
    })();
    this._pendingPlay = task;
    try {
      return await task;
    } finally {
      if (this._pendingPlay === task) this._pendingPlay = null;
    }
  }

  pause() {
    this._generation += 1;
    this.playing = false;
    globalThis.clearInterval(this._timer);
    this._timer = null;
    this._timeline = [];
    if (!this.context) return;
    const now = this.context.currentTime;
    this._ramp(this._output.gain, 0, 0.012);
    for (const voice of this._voices) {
      try { voice.source.stop(now + 0.015); } catch { /* A voice may have just ended. */ }
    }
  }

  setMix(value) {
    this.mix = this._unit(value, this.mix);
    if (!this.context) return;
    this._ramp(this._deckA.gain, Math.cos(this.mix * Math.PI / 2));
    this._ramp(this._deckB.gain, Math.sin(this.mix * Math.PI / 2));
  }

  setFilter(value) {
    this.filter = this._unit(value, this.filter);
    if (this.context) this._ramp(this._lowpass.frequency, this._cutoff());
  }

  setTempo(bpm) {
    const value = Number(bpm);
    if (Number.isFinite(value)) this.bpm = Math.min(160, Math.max(90, value));
  }

  getState() {
    let step = 0;
    if (this.playing && this.context) {
      const now = this.context.currentTime;
      for (const item of this._timeline) {
        if (item.time > now) break;
        step = item.step;
      }
    }
    return {
      playing: this.playing,
      bpm: this.bpm,
      beat: Math.floor(step / 4) % 4,
      bar: Math.floor(step / 16) % 4,
      mix: this.mix,
      filter: this.filter,
    };
  }

  destroy() {
    this.pause();
    this._destroyed = true;
    for (const voice of this._voices) {
      for (const node of voice.nodes) node.disconnect();
    }
    this._voices.clear();
    if (this.context && this.context.state !== "closed") {
      this.context.close().catch(() => {});
    }
  }

  _unit(value, fallback) {
    const number = Number(value);
    return Number.isFinite(number) ? Math.min(1, Math.max(0, number)) : fallback;
  }

  _cutoff() {
    return 180 * Math.pow(18000 / 180, this.filter);
  }

  _ramp(parameter, value, duration = 0.035) {
    const now = this.context.currentTime;
    if (typeof parameter.cancelAndHoldAtTime === "function") {
      parameter.cancelAndHoldAtTime(now);
    } else {
      parameter.cancelScheduledValues(now);
      parameter.setValueAtTime(parameter.value, now);
    }
    parameter.linearRampToValueAtTime(value, now + duration);
  }

  _createAudio() {
    if (this.context) return;
    const AudioContext = globalThis.AudioContext || globalThis.webkitAudioContext;
    if (!AudioContext) {
      throw new Error("This browser does not support Web Audio. Try a recent Safari, Chrome, or Firefox.");
    }
    try {
      this.context = new AudioContext({ latencyHint: "interactive" });
    } catch {
      throw new Error("Audio could not start. Check your audio output and try Play again.");
    }
    const context = this.context;
    this._deckA = context.createGain();
    this._deckB = context.createGain();
    this._lowpass = context.createBiquadFilter();
    this._lowpass.type = "lowpass";
    this._lowpass.frequency.value = this._cutoff();
    this._lowpass.Q.value = 0.65;
    this._deckA.gain.value = Math.cos(this.mix * Math.PI / 2);
    this._deckB.gain.value = Math.sin(this.mix * Math.PI / 2);
    this._deckA.connect(this._lowpass);
    this._deckB.connect(this._lowpass);
    this._limiter = context.createDynamicsCompressor();
    this._limiter.threshold.value = -9;
    this._limiter.knee.value = 3;
    this._limiter.ratio.value = 20;
    this._limiter.attack.value = 0.003;
    this._limiter.release.value = 0.12;
    this._output = context.createGain();
    this._output.gain.value = 0;
    this._lowpass.connect(this._limiter);
    this._limiter.connect(this._output);
    this._output.connect(context.destination);

    this._noise = context.createBuffer(1, context.sampleRate, context.sampleRate);
    const samples = this._noise.getChannelData(0);
    // Deterministic noise keeps the original percussion repeatable.
    let seed = 7381;
    for (let i = 0; i < samples.length; i += 1) {
      seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
      samples[i] = (seed / 4294967296) * 2 - 1;
    }
  }

  _schedule() {
    if (!this.playing) return;
    const now = this.context.currentTime;
    const sixteenth = 60 / this.bpm / 4;
    // Skip elapsed notes if a background tab throttles the scheduler.
    // This avoids a burst of old notes on returning to the page.
    if (this._nextTime < now - 0.08) {
      const missed = Math.ceil((now + 0.025 - this._nextTime) / sixteenth);
      this._step += missed;
      this._nextTime += missed * sixteenth;
    }
    while (this._nextTime < now + 0.12) {
      this._playStep(this._step, this._nextTime, sixteenth);
      this._timeline.push({ step: this._step, time: this._nextTime });
      this._step += 1;
      this._nextTime += sixteenth;
    }
    while (this._timeline.length > 1 && this._timeline[1].time <= now) {
      this._timeline.shift();
    }
  }

  _playStep(step, time, sixteenth) {
    const position = step % 16;
    const bar = Math.floor(step / 16) % 4;
    const roots = [41, 41, 44, 39];
    const root = roots[bar];

    // Deck A: a dry, syncopated bass groove with a four-on-the-floor kick.
    if (position % 4 === 0) this._kick(this._deckA, time, 0.6);
    if (position === 4 || position === 12) this._noiseHit(this._deckA, time, 0.1, 0.18, 1500, "bandpass");
    if (position % 2 === 0) this._noiseHit(this._deckA, time, position % 4 === 2 ? 0.07 : 0.033, 0.08, 6500);
    const bassSteps = [0, 3, 6, 8, 10, 14];
    if (bassSteps.includes(position)) {
      const offset = position === 6 || position === 14 ? 12 : 0;
      this._tone(this._deckA, time + 0.007, root + offset, sixteenth * 1.55, 0.22, "triangle", 850);
    }

    // Deck B: a contrasting, original F-minor motif with lighter percussion.
    if (position % 4 === 0) this._kick(this._deckB, time, 0.43);
    if (position === 4 || position === 12) this._noiseHit(this._deckB, time, 0.13, 0.14, 2200, "bandpass");
    if (position % 4 === 2) this._noiseHit(this._deckB, time, 0.1, 0.105, 7200);
    if (position === 7 || position === 15) this._noiseHit(this._deckB, time, 0.035, 0.038, 8100);
    const melody = [0, 7, 12, 15, 10, 7, 3, 10];
    if (position % 2 === 0) {
      const note = root + 24 + melody[position / 2];
      this._tone(this._deckB, time, note, sixteenth * 2.9, 0.095, "sine", 5000);
      this._tone(this._deckB, time + sixteenth * 1.5, note, sixteenth * 1.3, 0.022, "sine", 3000);
    }
    if (position === 0 || position === 8) {
      this._tone(this._deckB, time + 0.005, root, sixteenth * 5.5, 0.15, "sine", 700);
    }
  }

  _kick(destination, time, volume) {
    const oscillator = this.context.createOscillator();
    const envelope = this.context.createGain();
    oscillator.type = "sine";
    oscillator.frequency.setValueAtTime(135, time);
    oscillator.frequency.exponentialRampToValueAtTime(47, time + 0.095);
    envelope.gain.setValueAtTime(0, time);
    envelope.gain.linearRampToValueAtTime(volume, time + 0.004);
    envelope.gain.exponentialRampToValueAtTime(0.0001, time + 0.28);
    oscillator.connect(envelope);
    envelope.connect(destination);
    this._startVoice(oscillator, [oscillator, envelope], time, time + 0.3);
  }

  _noiseHit(destination, time, duration, volume, frequency, type = "highpass") {
    const source = this.context.createBufferSource();
    source.buffer = this._noise;
    const filter = this.context.createBiquadFilter();
    filter.type = type;
    filter.frequency.value = frequency;
    filter.Q.value = type === "bandpass" ? 0.8 : 0.6;
    const envelope = this.context.createGain();
    envelope.gain.setValueAtTime(0, time);
    envelope.gain.linearRampToValueAtTime(volume, time + 0.002);
    envelope.gain.exponentialRampToValueAtTime(0.0001, time + duration);
    source.connect(filter);
    filter.connect(envelope);
    envelope.connect(destination);
    this._startVoice(source, [source, filter, envelope], time, time + duration + 0.012);
  }

  _tone(destination, time, midi, duration, volume, shape, cutoff) {
    const oscillator = this.context.createOscillator();
    oscillator.type = shape;
    oscillator.frequency.value = 440 * Math.pow(2, (midi - 69) / 12);
    const filter = this.context.createBiquadFilter();
    filter.type = "lowpass";
    filter.frequency.setValueAtTime(cutoff, time);
    filter.frequency.exponentialRampToValueAtTime(Math.max(200, cutoff * 0.35), time + duration);
    const envelope = this.context.createGain();
    envelope.gain.setValueAtTime(0, time);
    envelope.gain.linearRampToValueAtTime(volume, time + 0.008);
    envelope.gain.exponentialRampToValueAtTime(0.0001, time + duration);
    oscillator.connect(filter);
    filter.connect(envelope);
    envelope.connect(destination);
    this._startVoice(oscillator, [oscillator, filter, envelope], time, time + duration + 0.02);
  }

  _startVoice(source, nodes, start, stop) {
    const voice = { source, nodes };
    this._voices.add(voice);
    source.onended = () => {
      for (const node of nodes) node.disconnect();
      this._voices.delete(voice);
    };
    source.start(start);
    source.stop(stop);
  }
}
