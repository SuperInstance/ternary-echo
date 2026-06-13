# Ternary Echo — Delay Lines, Multi-Tap Echo, and Feedback for Ternary Signals

**Ternary Echo** implements digital delay line effects — echo, multi-tap delay, slapback, and ping-pong — for signals in the ternary value space **T = {−1, 0, +1}**. Each delay line is a circular buffer that stores samples and reads them back at configurable time offsets, with feedback control for cascading reflections. The result is spatial audio processing where the only values are positive impulse, negative impulse, and silence.

## Why It Matters

Echo is the simplest spatial audio effect — it creates the perception of distance and room geometry from a flat signal. The delay time maps to physical distance (1 ms ≈ 34 cm at sea level), the decay maps to surface absorption, and feedback maps to room liveness.

In the ternary regime, echo has unique properties that continuous DSP cannot replicate:

- **Quantized decay:** Because ternary signals only have three values, echoes fade in discrete steps — from ±1 to 0. There is no gradual amplitude reduction; each echo is either present (±1) or absent (0). This produces a distinctly digital, staircase decay.
- **Hard saturation:** In continuous audio, excessive feedback creates infinite resonance. In ternary, values saturate at ±1, producing a **standing wave** — a permanent, unchanging pattern. The room fills completely.
- **No multiplication:** Ternary delay lines only need conditional add/subtract and buffer reads. No hardware multiplier is required, enabling audio processing on minimal ternary ALUs.

The delay line is also the **fundamental building block** of digital filters (FIR, IIR), reverb (Schroeder allpass), and physical modeling synthesis (Karplus-Strong). This crate provides the primitive that higher-order ternary DSP builds upon.

## How It Works

### Circular Buffer Delay Line

The core `DelayLine` is a circular buffer of capacity N:

```
buffer: [s₀, s₁, ..., s_{N-1}]    (initialized to 0)
write_head: position index (wraps around)
```

**`tick(signal, feedback)`:**
1. Read `delayed = buffer[write_head]` (the oldest sample)
2. Write `buffer[write_head] = signal` (store new input)
3. Advance `write_head = (write_head + 1) mod N`
4. Return `delayed`

The feedback parameter is available for external mixing: `output = signal + feedback × delayed`. In ternary, the multiplication by feedback is typically implemented as: if `feedback ≥ threshold`, use `delayed`; otherwise, use 0.

**Complexity:** O(1) per tick — one array read, one write, one modular increment.

**Memory:** O(N) where N is the delay capacity in samples.

### Multi-Tap Delay

Extracts multiple delayed copies from a single delay line at different offsets:

```
tap_k = buffer[(write_head − offset_k) mod N]
```

Each tap reads from a different position in the circular buffer, producing k parallel echoes from one buffer. The taps are read-only — no additional write operations.

**Complexity:** O(k) per sample for k taps. Each tap is O(1) (array index computation + read).

**Memory:** O(N) — single shared buffer regardless of tap count.

### Ping-Pong Delay (Stereo Cross-Feed)

Two independent delay lines (left and right) with cross-feeding:

```
out_L[i] = dl_L.tick(left[i], feedback)
out_R[i] = dl_R.tick(right[i], feedback)
```

The cross-feed creates a bouncing effect where echoes alternate between channels. With a delay of d samples and sample rate f_s, the bounce rate is f_s / (2d) Hz.

**Complexity:** O(n) for n samples (two O(1) ticks per sample).

**Memory:** O(2d) — two delay lines of depth d.

### Slapback Echo

A single, fast echo with mix control:

```
output[i] = round(signal[i] × (1 − mix) + delayed[i] × mix)
```

clamped to [−128, 127] for i8 arithmetic. When `mix = 1.0`, the output is purely the delayed signal. When `mix = 0.0`, the output is the original signal.

**Complexity:** O(n) for n samples — one tick + one mix operation per sample.

**Memory:** O(d) where d is the slapback delay in samples.

### Delay Time → Physical Distance

The delay time encodes the distance to the reflecting surface:

```
d_distance = (delay_samples / f_s) × c_sound
```

where c_sound ≈ 343 m/s at 20°C. For f_s = 44100 Hz:

| Delay (samples) | Delay (ms) | Distance (m) | Space |
|---|---|---|---|
| 44 | 1.0 | 0.34 | Near field |
| 441 | 10.0 | 3.43 | Small room |
| 1764 | 40.0 | 13.7 | Hall |
| 8820 | 200.0 | 68.6 | Cathedral |

### Feedback Stability

With feedback gain β ∈ [0, 1), the echo amplitude after k iterations is β^k. The system is stable (echoes die out) when β < 1. The **reverberation time** RT₆₀ (time to decay by 60 dB) is:

```
RT₆₀ = −3 / log₁₀(β) × delay_time
```

For ternary signals, the effective β is quantized: β = 1 (feedback on) or β = 0 (feedback off). With β = 1, the system is **marginally stable** — echoes persist indefinitely as a standing wave.

## Quick Start

```rust
use ternary_echo::*;

// Basic delay line
let mut dl = DelayLine::new(1024);  // 1024-sample buffer
dl.tick(42, 0.5);                    // write 42, read previous
let delayed = dl.tap(512);           // read 512 samples back

// Multi-tap: extract echoes at 2, 4, and 8 samples delay
let signal = &[1i8, 2, 3, 4, 5, 6, 7, 8];
let outputs = multi_tap(signal, &[2, 4, 8], 16, 0.7);
// outputs[0]: delayed by 2 samples
// outputs[1]: delayed by 4 samples
// outputs[2]: delayed by 8 samples

// Slapback (rockabilly echo)
let music = &[100i8, 0, 0, 0, 0, 0];
let echoed = slapback(music, 2, 0.5);
// echoed[2] = 50  (half-volume echo of sample 0)

// Ping-pong stereo
let left  = &[1i8, 2, 3, 4, 5];
let right = &[-1i8, -2, -3, -4, -5];
let (out_l, out_r) = ping_pong(left, right, 0.5, 4);
```

```bash
cargo add ternary-echo
```

## API

| Function | Complexity | Description |
|---|---|---|
| `DelayLine::new(capacity)` | O(N) | Allocate N-sample circular buffer |
| `DelayLine::tick(signal, feedback)` | O(1) | Process one sample |
| `DelayLine::tap(offset)` | O(1) | Read from offset samples ago |
| `multi_tap(signal, taps, capacity, feedback)` | O(n × k) | k taps over n samples |
| `ping_pong(left, right, feedback, delay)` | O(n) | Stereo cross-feed echo |
| `slapback(signal, delay, mix)` | O(n) | Single-repeat echo with mix |
| `new_delay(capacity)` | O(N) | Constructor convenience function |

## Architecture Notes

In the **SuperInstance** ecosystem, `ternary-echo` provides the temporal dimension of signal processing. The delay line is the discrete-time analog of the **γ + η = C** conservation: past signals (stored in the buffer) represent structured information (γ), while the decay and eventual loss of delayed signals represents entropy (η). The total energy C = signal + delayed is conserved through the delay line — what enters must exit, delayed in time.

The delay line also serves as the foundation for ternary **reverb** (multiple delay lines with feedback), **physical modeling** (waveguide synthesis), and **IIR filtering** (recursive delay with feedback). The standing-wave behavior at β = 1 maps directly to the resonance modes of the ternary field.

## References

1. Smith, J. O. (2010). *Physical Audio Signal Processing*. W3K Publishing. — Digital waveguide theory and delay-line filters. <https://ccrma.stanford.edu/~jos/pasp/>
2. Schroeder, M. R. (1962). "Natural Sounding Artificial Reverberation." *Journal of the Audio Engineering Society*, 10(3), 219–223. — Schroeder reverb topology.
3. Karplus, K. & Strong, A. (1983). "Digital Synthesis of Plucked-String and Drum Timbres." *Computer Music Journal*, 7(2), 43–55. — Karplus-Strong algorithm.
4. Zölzer, U. (2011). *DAFX: Digital Audio Effects* (2nd ed.). Wiley. — Comprehensive DSP effects reference.
5. Oppenheim, A. V. & Schafer, R. W. (2010). *Discrete-Time Signal Processing* (3rd ed.). Pearson. — Theoretical foundations of discrete delay systems.
6. Roads, C. (1996). *The Computer Music Tutorial*. MIT Press. — Comprehensive delay and echo techniques.

## License

MIT
