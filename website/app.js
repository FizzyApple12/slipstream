import { SoundDemo } from './audio.js';

const sound = new SoundDemo();
const play = document.querySelector('#play');
const bpm = document.querySelector('#bpm');
const status = document.querySelector('#audio-status');
const crossfader = document.querySelector('#crossfader');
const filter = document.querySelector('#filter');
const signalLabel = document.querySelector('#signal-label');
const signalLed = document.querySelector('#signal-led');
const elapsed = document.querySelector('#session-time');
const playLabel = document.querySelector('#play-label');
const beatLights = [...document.querySelectorAll('.beat-lights i')];
const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)');
const canvases = [...document.querySelectorAll('.deck canvas')];
let playing = false;
let starting = false;
let startTime = 0;
let animationId = 0;
let lastDraw = 0;
let lastSecond = -1;

function paintWave(canvas, index, phase = 0) {
  const context = canvas.getContext('2d');
  if (!context) return;
  const rect = canvas.getBoundingClientRect();
  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  const width = Math.round(rect.width * dpr);
  const height = Math.round(rect.height * dpr);
  if (width <= 0 || height <= 0) return;
  if (canvas.width !== width || canvas.height !== height) { canvas.width = width; canvas.height = height; }
  context.clearRect(0, 0, width, height);
  context.fillStyle = '#263120';
  for (let x = 0; x < width; x += width / 16) context.fillRect(x, 0, 1, height);
  context.fillRect(0, height / 2, width, 1);
  const bars = Math.floor(rect.width / 3.5);
  const color = index === 0 ? '#f0643f' : '#b2ccd0';
  for (let i = 0; i < bars; i++) {
    const position = (i / bars + phase) % 1;
    const impulse = Math.pow(1 - ((position * 16) % 1), index === 0 ? 2.6 : 1.6);
    const noise = Math.abs(Math.sin((position * 963 + index * 177) * 1.87) * Math.cos(position * 317));
    const section = .68 + .32 * Math.sin(position * Math.PI * 8 + index);
    const amplitude = (.08 + .54 * impulse + .34 * noise) * section;
    const barHeight = amplitude * height * .98;
    context.fillStyle = color;
    context.globalAlpha = .5 + .5 * amplitude;
    context.fillRect(i * width / bars, (height - barHeight) / 2, Math.max(1, width / bars - 1.3 * dpr), barHeight);
  }
  context.globalAlpha = 1;
}

function drawFrame(now) {
  if (!playing) return;
  const state = sound.getState();
  const seconds = Math.max(0, Math.floor((now - startTime) / 1000));
  if (seconds !== lastSecond) {
    elapsed.textContent = `${String(Math.floor(seconds / 60)).padStart(2, '0')}:${String(seconds % 60).padStart(2, '0')}`;
    lastSecond = seconds;
  }
  beatLights.forEach((light, index) => light.classList.toggle('active', state.beat === index));
  if (!reducedMotion.matches && now - lastDraw > 32) {
    const phase = (((now - startTime) / 1000) * state.bpm / 60 / 16) % 1;
    canvases.forEach((canvas, i) => paintWave(canvas, i, phase));
    lastDraw = now;
  }
  animationId = requestAnimationFrame(drawFrame);
}

function updatePlayback() {
  play.setAttribute('aria-pressed', String(playing));
  play.setAttribute('aria-label', playing ? 'Pause sound study' : 'Play sound study');
  signalLabel.textContent = playing ? 'IN THE FLOW' : 'READY WHEN YOU ARE';
  signalLed.classList.toggle('active', playing);
  playLabel.textContent = playing ? 'IN THE MIX' : 'PRESS PLAY';
}

function pause() {
  sound.pause();
  playing = false;
  cancelAnimationFrame(animationId);
  updatePlayback();
  beatLights.forEach(light => light.classList.remove('active'));
  status.textContent = 'PAUSED. PICK IT UP WHENEVER.';
}

async function togglePlayback() {
  if (starting) return;
  if (playing) { pause(); return; }
  starting = true;
  play.disabled = true;
  status.classList.remove('error');
  try {
    await sound.play();
    if (document.hidden) { sound.pause(); return; }
    playing = true;
    startTime = performance.now();
    lastSecond = -1;
    status.textContent = 'TWO LOOPS. YOUR TRANSITION. TRY THE FADER.';
    updatePlayback();
    animationId = requestAnimationFrame(drawFrame);
  } catch (error) {
    sound.pause();
    playing = false;
    updatePlayback();
    status.textContent = 'AUDIO COULDN’T START. TRY AGAIN IN A MODERN BROWSER.';
    status.classList.add('error');
    console.warn('Slipstream sound study:', error.message);
  } finally {
    starting = false;
    play.disabled = false;
  }
}

function updateMix() {
  const value = Number(crossfader.value);
  sound.setMix(value / 100);
  document.querySelector('#mix-label').textContent = `A ${100 - value} / B ${value}`;
}
function updateFilter() {
  const value = Number(filter.value);
  sound.setFilter(value / 100);
  document.querySelector('#filter-label').textContent = `${value}%`;
}
function updateTempo(value) {
  sound.setTempo(value);
  const tempo = sound.getState().bpm;
  bpm.textContent = tempo;
  document.querySelector('#tempo-down').disabled = tempo <= 90;
  document.querySelector('#tempo-up').disabled = tempo >= 160;
}

play.addEventListener('click', togglePlayback);
crossfader.addEventListener('input', updateMix);
filter.addEventListener('input', updateFilter);
document.querySelector('#tempo-down').addEventListener('click', () => updateTempo(sound.getState().bpm - 1));
document.querySelector('#tempo-up').addEventListener('click', () => updateTempo(sound.getState().bpm + 1));
document.querySelector('#reset').addEventListener('click', () => {
  crossfader.value = 50;
  filter.value = 100;
  updateMix();
  updateFilter();
  updateTempo(126);
  status.classList.remove('error');
  status.textContent = playing ? 'BACK TO THE BEGINNING. KEEP IT MOVING.' : 'MIX RESET. PRESS PLAY WHEN YOU’RE READY.';
});
document.addEventListener('visibilitychange', () => { if (document.hidden && playing) pause(); });
window.addEventListener('pagehide', () => { pause(); });
const resizeObserver = new ResizeObserver(() => { if (!playing || reducedMotion.matches) canvases.forEach((canvas, i) => paintWave(canvas, i)); });
canvases.forEach(canvas => resizeObserver.observe(canvas));
updateMix();
updateFilter();
updateTempo(126);
canvases.forEach((canvas, i) => paintWave(canvas, i));
