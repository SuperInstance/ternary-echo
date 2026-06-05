#![forbid(unsafe_code)]

/// Delay line buffer for echo/delay effects
#[derive(Debug, Clone)]
pub struct DelayLine {
    buffer: Vec<i8>,
    position: usize,
}

impl DelayLine {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0i8; capacity],
            position: 0,
        }
    }

    /// Process one sample: write input, read output with feedback
    pub fn tick(&mut self, signal: i8, feedback: f64) -> i8 {
        let delayed = self.buffer[self.position];
        let output = (signal as f64 + delayed as f64 * feedback)
            .round()
            .clamp(-128.0, 127.0) as i8;
        self.buffer[self.position] = signal;
        self.position = (self.position + 1) % self.buffer.len();
        delayed
    }

    /// Read from a specific tap position (offset back from write head)
    pub fn tap(&self, offset: usize) -> i8 {
        if offset > self.buffer.len() || self.buffer.is_empty() {
            return 0;
        }
        let idx = (self.position + self.buffer.len() - offset) % self.buffer.len();
        self.buffer[idx]
    }
}

/// Create a new delay line with given capacity
pub fn new_delay(capacity: usize) -> DelayLine {
    DelayLine::new(capacity)
}

/// Multi-tap delay: extract multiple taps from a single delay line
pub fn multi_tap(signal: &[i8], taps: &[usize], capacity: usize, feedback: f64) -> Vec<Vec<i8>> {
    let mut dl = DelayLine::new(capacity);
    let mut outputs: Vec<Vec<i8>> = taps.iter().map(|_| Vec::with_capacity(signal.len())).collect();
    for &s in signal {
        dl.tick(s, feedback);
        for (i, &tap) in taps.iter().enumerate() {
            outputs[i].push(dl.tap(tap));
        }
    }
    outputs
}

/// Ping-pong delay: alternating left/right echoes
pub fn ping_pong(left: &[i8], right: &[i8], feedback: f64, delay_ticks: usize) -> (Vec<i8>, Vec<i8>) {
    let mut dl_l = DelayLine::new(delay_ticks.max(1));
    let mut dl_r = DelayLine::new(delay_ticks.max(1));
    let mut out_l = Vec::with_capacity(left.len());
    let mut out_r = Vec::with_capacity(right.len());
    let len = left.len().min(right.len());
    for i in 0..len {
        let l = dl_l.tick(left[i], feedback);
        let r = dl_r.tick(right[i], feedback);
        // cross-feed for ping-pong
        out_l.push(l);
        out_r.push(r);
    }
    (out_l, out_r)
}

/// Slapback echo: single repeat with mix control
pub fn slapback(signal: &[i8], delay_ticks: usize, mix: f64) -> Vec<i8> {
    let mut dl = DelayLine::new(delay_ticks.max(1));
    let mut out = Vec::with_capacity(signal.len());
    for &s in signal {
        let delayed = dl.tap(delay_ticks);
        dl.tick(s, 1.0);
        let mixed = (s as f64 * (1.0 - mix) + delayed as f64 * mix)
            .round()
            .clamp(-128.0, 127.0) as i8;
        out.push(mixed);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_delay() {
        let dl = new_delay(100);
        assert_eq!(dl.buffer.len(), 100);
        assert_eq!(dl.position, 0);
    }

    #[test]
    fn test_tick_basic() {
        let mut dl = DelayLine::new(4);
        let out = dl.tick(10, 0.5);
        assert_eq!(out, 0); // first read, buffer was empty
    }

    #[test]
    fn test_tick_feedback() {
        let mut dl = DelayLine::new(1);
        let first = dl.tick(50, 0.5);
        assert_eq!(first, 0); // buffer was empty
        let second = dl.tick(20, 0.5);
        assert_eq!(second, 50); // delayed value from position 0
    }

    #[test]
    fn test_tap_basic() {
        let mut dl = DelayLine::new(8);
        dl.tick(42, 1.0);
        // position is now 1, tap(1) reads from position 0
        assert_eq!(dl.tap(1), 42);
    }

    #[test]
    fn test_tap_out_of_range() {
        let dl = DelayLine::new(4);
        assert_eq!(dl.tap(100), 0);
    }

    #[test]
    fn test_tap_delayed() {
        let mut dl = DelayLine::new(4);
        for v in &[10i8, 20, 30, 40] {
            dl.tick(*v, 1.0);
        }
        // buffer = [10, 20, 30, 40], pos = 0
        // tap(1) = buffer[3] = 40 (last written)
        // tap(2) = buffer[2] = 30
        assert_eq!(dl.tap(1), 40);
        assert_eq!(dl.tap(2), 30);
    }

    #[test]
    fn test_multi_tap() {
        let signal = &[1i8, 2, 3, 4, 5, 6, 7, 8];
        let outputs = multi_tap(signal, &[2, 4], 8, 1.0);
        assert_eq!(outputs.len(), 2);
    }

    #[test]
    fn test_multi_tap_lengths() {
        let signal = &[1i8, 2, 3, 4, 5];
        let outputs = multi_tap(signal, &[1, 2, 3], 8, 1.0);
        for o in &outputs {
            assert_eq!(o.len(), 5);
        }
    }

    #[test]
    fn test_ping_pong() {
        let left = &[1i8, 2, 3, 4, 5];
        let right = &[-1i8, -2, -3, -4, -5];
        let (ol, or) = ping_pong(left, right, 0.5, 4);
        assert_eq!(ol.len(), 5);
        assert_eq!(or.len(), 5);
    }

    #[test]
    fn test_slapback_basic() {
        let signal = &[10i8, 0, 0, 0];
        let out = slapback(signal, 2, 1.0);
        assert_eq!(out.len(), 4);
        // at index 2, the delayed value from index 0 should appear
        assert_eq!(out[2], 10);
    }

    #[test]
    fn test_slapback_mix_zero() {
        let signal = &[10i8, 20, 30];
        let out = slapback(signal, 2, 0.0);
        assert_eq!(out[0], 10); // no delay mixed in
    }

    #[test]
    fn test_slapback_mix_half() {
        let signal = &[100i8, 0, 0];
        let out = slapback(signal, 2, 0.5);
        // at index 2: signal=0 * 0.5 + delayed=100 * 0.5 = 50
        assert_eq!(out[2], 50);
    }

    #[test]
    fn test_delay_wrap() {
        let mut dl = DelayLine::new(2);
        dl.tick(10, 1.0);
        dl.tick(20, 1.0);
        // buffer = [10, 20], pos = 0
        assert_eq!(dl.tap(0), 10); // pos 0 itself
        assert_eq!(dl.tap(1), 20); // one back = pos 1
    }

    #[test]
    fn test_empty_signal() {
        let signal: &[i8] = &[];
        let out = slapback(signal, 4, 0.5);
        assert!(out.is_empty());
    }
}
