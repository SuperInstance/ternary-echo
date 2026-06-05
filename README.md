# ternary-echo

**Delay lines, multi-tap echoes, and ping-pong effects for ternary audio.**

An echo is just a memory — the signal comes back, slightly delayed, slightly changed. In audio processing, delay lines are the foundation of reverb, chorus, flanging, and every spatial effect. This crate implements them for signals in `{-1, 0, +1}`.

The ternary constraint makes echoes *sharp*. There's no gradual fade — a delayed ternary signal either changes state or it doesn't. This creates distinctive rhythmic patterns: the echo isn't a ghost of the original, it's the original arriving again, shifted in time.

## What's Inside

- **`DelayLine`** — circular buffer with `tick()` (process one sample), `tap()` (read from offset), and feedback control
- **`multi_tap(signal, taps, capacity, feedback)`** — extract multiple delay taps from a single line. Create rhythmic patterns from one input
- **`ping_pong(left, right, feedback, delay)`** — alternating stereo echoes. Left feeds right, right feeds left. The classic spatial widener
- **`slapback(signal, delay, mix)`** — single-repeat echo. Rockabilly in a function
- **`feedback_echo(signal, delay, feedback, mix)`** — repeating echo with decay. Each repeat is quieter until it vanishes

## Quick Example

```rust
use ternary_echo::*;

// Simple delay line
let mut dl = DelayLine::new(100); // 100-sample buffer
let output = dl.tick(1, 0.7); // write 1, read with 70% feedback
assert_eq!(output, 0); // first read is empty (buffer was zero)

// Process a signal with slapback echo
let signal = vec![1, -1, 0, 1, -1, 0];
let echoed = slapback(&signal, 3, 0.5);
// Original signal + delayed copy mixed in

// Multi-tap: three echoes at different delays
let taps = multi_tap(&signal, &[50, 100, 150], 200, 0.6);
// taps[0] = 50-sample delay, taps[1] = 100-sample, taps[2] = 150-sample

// Ping-pong stereo
let left  = vec![1, 0, -1, 0, 1, 0];
let right = vec![0, 1, 0, -1, 0, 1];
let (echo_l, echo_r) = ping_pong(&left, &right, 0.5, 50);
```

## The Insight

**Ternary echoes preserve state boundaries.** In continuous audio, an echo is a smeared copy — amplitude decays, harmonics blur. In ternary, the echo arrives *exactly* as it was: -1, 0, or +1. The only transformation is time. This makes ternary delay effects especially useful for rhythm generation, pattern evolution, and any domain where you want echoes that *mean* something, not just sound like something.

**Use cases:**
- **Audio effect chains** — delay, reverb foundations, spatial widening
- **Rhythm generation** — multi-tap delays create polyrhythmic patterns from simple inputs
- **Signal processing education** — delay lines are the simplest stateful transform
- **Game audio** — lightweight echo effects without floating-point
- **Generative music** — feedback echoes as evolving pattern generators

## See Also

- **ternary-loop** — find repeating periods in echoed signals
- **ternary-phase** — phase relationships between original and delayed signals
- **ternary-mixer** — blend original and echo signals
- **ternary-vu** — meter the output level of echo processing
- **ternary-reverb** — (if you build it) multiple delay lines = reverb

## Install

```bash
cargo add ternary-echo
```

## License

MIT
