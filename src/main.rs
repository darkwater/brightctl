use std::env;
use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::str::FromStr;

enum Step {
    Up,
    Down,
}

#[derive(Debug, PartialEq)]
struct Brightness(pub u64);

impl Brightness {
    /// Step like: 0 - 1 - 500 - 1000 - 1500 - ... - 7000 - 7500
    fn step(&self, dir: Step) -> Self {
        match (self.0, dir) {
            (0, Step::Up) => Brightness(1),
            (1, Step::Up) => Brightness(500),
            (old, Step::Up) => {
                let rounded   = old / 500 * 500; // round down to nearest multiple of 500
                let increased = rounded + 500;
                let clipped   = u64::min(increased, 7500);
                Brightness(clipped)
            },
            (0, Step::Down) => Brightness(0),
            (1, Step::Down) => Brightness(0),
            (old, Step::Down) => {
                let rounded   = (old + 499) / 500 * 500; // round up to nearest multiple of 500
                let increased = rounded - 500;
                let clipped   = u64::max(increased, 1);
                Brightness(clipped)
            },
        }
    }

    fn tween_to(&self, other: Brightness, steps: u64) -> Vec<Brightness> {
        (1..=steps).map(|step| {
            let i = step as f64 / steps as f64;

            Brightness(
                (self.0 as f64 * (1.0 - i) + other.0 as f64 * i) as u64
            )
        })
        .collect()
    }
}

fn main() {
    match env::args().nth(1).as_ref().map(String::as_ref) {
        Some("up") => change(Step::Up),
        Some("down") => change(Step::Down),
        Some(other) => eprintln!("invalid argument {:?} -- try \"up\" or \"down\"", other),
        None => eprintln!("usage: brightctl <up/down>"),
    }
}

fn change(dir: Step) {
    let path = "/sys/class/backlight/intel_backlight/brightness";
    let old = parse_from_file(path);
    let old = Brightness(old);
    let new = old.step(dir);

    let tween = old.tween_to(new, 100);

    let mut file = File::create(path).unwrap();
    for step in tween {
        write!(file, "{}", step.0).unwrap();
        std::thread::sleep_ms(1);
    }
}

fn parse_from_file<T: FromStr>(path: &str) -> T
where
    <T as FromStr>::Err: Debug
{
    let mut file = File::open(path).unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    buf.trim_end().parse().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brightness_steps() {
        use Step::*;

        assert_eq!(Brightness(0).step(Up),    Brightness(1));
        assert_eq!(Brightness(1).step(Up),    Brightness(500));
        assert_eq!(Brightness(200).step(Up),  Brightness(500));
        assert_eq!(Brightness(500).step(Up),  Brightness(1000));
        assert_eq!(Brightness(1000).step(Up), Brightness(1500));
        assert_eq!(Brightness(1005).step(Up), Brightness(1500));
        assert_eq!(Brightness(1499).step(Up), Brightness(1500));
        assert_eq!(Brightness(1500).step(Up), Brightness(2000));
        assert_eq!(Brightness(1501).step(Up), Brightness(2000));
        assert_eq!(Brightness(7000).step(Up), Brightness(7500));
        assert_eq!(Brightness(7400).step(Up), Brightness(7500));
        assert_eq!(Brightness(7500).step(Up), Brightness(7500));
        assert_eq!(Brightness(7600).step(Up), Brightness(7500));

        assert_eq!(Brightness(7600).step(Down), Brightness(7500));
        assert_eq!(Brightness(7500).step(Down), Brightness(7000));
        assert_eq!(Brightness(7400).step(Down), Brightness(7000));
        assert_eq!(Brightness(7000).step(Down), Brightness(6500));
        assert_eq!(Brightness(1501).step(Down), Brightness(1500));
        assert_eq!(Brightness(1500).step(Down), Brightness(1000));
        assert_eq!(Brightness(1499).step(Down), Brightness(1000));
        assert_eq!(Brightness(1005).step(Down), Brightness(1000));
        assert_eq!(Brightness(1000).step(Down), Brightness(500));
        assert_eq!(Brightness(500).step(Down),  Brightness(1));
        assert_eq!(Brightness(200).step(Down),  Brightness(1));
        assert_eq!(Brightness(1).step(Down),    Brightness(0));
        assert_eq!(Brightness(0).step(Down),    Brightness(0));
    }

    #[test]
    fn tweening() {
        assert_eq!(
            Brightness(1000).tween_to(Brightness(1500), 4),
            vec![
                Brightness(1125),
                Brightness(1250),
                Brightness(1375),
                Brightness(1500),
            ]
        );

        assert_eq!(
            Brightness(2000).tween_to(Brightness(8000), 6),
            vec![
                Brightness(3000),
                Brightness(4000),
                Brightness(5000),
                Brightness(6000),
                Brightness(7000),
                Brightness(8000),
            ]
        );
    }
}
