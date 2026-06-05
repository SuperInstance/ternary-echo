# ternary-echo

**Space in a signal. Reflection, delay, and the sound of rooms that don't exist.**

An echo is the simplest spatial effect: the sound goes out, hits a wall, comes back. The delay time tells you how far the wall is. The decay tells you what the wall is made of. A cathedral has long, slow decay. A bathroom has short, bright reflections. An open field has no echo at all.

This crate implements echo, multi-tap delay, and feedback loops for ternary signals. Each echo is a delayed copy of the original signal, attenuated and added back. With feedback, the echoes echo — creating cascading reflections that eventually die out (or don't, if you push the feedback too high, and then it *screams*).

## What's Inside

- **`simple_echo(signal, delay, decay)`** — one echo at `delay` ticks with `decay` attenuation (0-1)
- **`multi_tap(signal, taps)`** — multiple echoes at different delays and decays. The sound of a complex room
- **`feedback_echo(signal, delay, decay, feedback)`** — echoes that echo. `feedback` controls how much of each echo feeds back into the next iteration
- **`ping_pong(signal, delay, decay)`** — echoes that alternate between left and right channels. Stereo space from a mono signal
- **`slapback(signal, delay)`** — a single, fast, loud echo. The rockabilly sound. Sun Studios, 1956
- **`reverb_approx(signal, room_size)`** — many overlapping echoes at different delays. An approximation of room reverb using ternary feedback

## Quick Example

```rust
use ternary_echo::*;

let signal = vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

// Simple echo: one repeat after 4 ticks, at 50% volume
let echoed = simple_echo(&signal, 4, 0.5);
// [1, 0, 0, 0, 0, 0, 0, 0, ...] + delayed copy at half amplitude

// Slapback: fast echo at 75% volume (rockabilly!)
let slap = slapback(&signal, 2);

// Feedback echo: cascading reflections
let cascading = feedback_echo(&signal, 3, 0.6, 0.5);
// Echo → echo of echo → echo of echo of echo → ... fading out

// Multi-tap: three different delay times
let taps = vec![(2, 0.7), (5, 0.5), (8, 0.3)];
let complex = multi_tap(&signal, &taps);
```

## The Deeper Truth

**Echo is the only way to hear space in a flat signal.** A ternary signal is a sequence of values — no space, no dimension, no room. But add a delayed copy and suddenly you can hear the walls. The delay time maps to distance (1ms ≈ 34cm). The decay maps to absorption (hard walls reflect more, soft walls absorb more). The feedback maps to how many bounces before the sound dies.

In ternary, echo has a unique property: because the signal only has three values, the echoes can't gradually fade to a smooth silence. They fade in *steps* — from ±1 to 0. This creates a quantized decay that sounds distinctly different from analog echo. Each echo is either there (±1) or gone (0), with nothing in between. It's the echo equivalent of pixel art.

The feedback echo is where it gets dangerous. Set feedback too high and the echoes build up instead of dying out. In continuous audio, this creates infinite resonance. In ternary, the values saturate at ±1 and the signal becomes a standing wave — a permanent, unchanging pattern. The room fills up and can't hold any more. Ternary feedback echoes have a *hard ceiling* that continuous echoes don't.

**Use cases:**
- **Spatial audio** — give flat signals a sense of space and depth
- **Sound design** — create rhythmic echo patterns (dub, reggae, electronic)
- **Music production** — slapback for vocals, feedback for synths, multi-tap for atmospheres
- **Game audio** — simulate room acoustics with ternary echoes
- **Education** — hear the relationship between delay time and perceived distance

## See Also

- **ternary-pan** — stereo positioning (echo + pan = spatial audio)
- **ternary-bite** — degraded echoes for lo-fi textures
- **ternary-needledrop** — vinyl imperfection + echo = dub techno
- **ternary-rack** — wire echo into a modular signal chain
- **ternary-reverb** — (future) true convolution reverb for ternary signals

## Install

```bash
cargo add ternary-echo
```

## License

MIT
