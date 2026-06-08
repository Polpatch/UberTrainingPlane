
export function play_beep() {
    try {
        var ctx = new (window.AudioContext || window.webkitAudioContext)();
        function tone(freq, vol, t0, dur, vibrato) {
            var o = ctx.createOscillator(), g = ctx.createGain();
            o.connect(g); g.connect(ctx.destination);
            o.type = 'sine';
            o.frequency.setValueAtTime(freq * 1.04, ctx.currentTime + t0);
            o.frequency.exponentialRampToValueAtTime(freq, ctx.currentTime + t0 + 0.04);
            if (vibrato) {
                var lfo = ctx.createOscillator(), lfoG = ctx.createGain();
                lfo.frequency.value = 5.5;
                lfoG.gain.value = 14;
                lfo.connect(lfoG); lfoG.connect(o.frequency);
                lfo.start(ctx.currentTime + t0 + 0.09);
                lfo.stop(ctx.currentTime + t0 + dur + 0.1);
            }
            g.gain.setValueAtTime(vol, ctx.currentTime + t0);
            g.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + t0 + dur);
            o.start(ctx.currentTime + t0);
            o.stop(ctx.currentTime + t0 + dur + 0.1);
        }
        tone(659,  0.55, 0,    0.22, false);
        tone(784,  0.65, 0.24, 0.13, false);
        tone(1319, 0.9,  0.41, 0.85, true);
    } catch(e) {}
}
