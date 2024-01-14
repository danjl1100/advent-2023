use anyhow::Context;

fn main() -> anyhow::Result<()> {
    eprintln!("hello, lava manual!");

    let input = advent_2023::get_input_string()?;
    let Stats {
        sum_of_hashes,
        focusing_power,
    } = analyze(&input)?;

    eprintln!("Sum of hashes: {sum_of_hashes}");
    eprintln!("Boxes total focusing power: {focusing_power}");

    Ok(())
}

struct Stats {
    sum_of_hashes: u32,
    focusing_power: u32,
}

fn analyze(input: &str) -> anyhow::Result<Stats> {
    let mut lines = input.lines();

    let Some(line) = lines.next() else {
        anyhow::bail!("no lines in the input")
    };
    if let Some(extra) = lines.next() {
        anyhow::bail!("extra line {extra:?}")
    }

    let sum_of_hashes = line.split(',').map(ascii_hash).map(u32::from).sum();

    let boxes = Boxes::try_from(line)?;
    let focusing_power = boxes.get_focus_power();

    Ok(Stats {
        sum_of_hashes,
        focusing_power,
    })
}

fn ascii_hash(input: &str) -> u8 {
    let result = input.chars().fold(0, |accum, c| {
        assert!(c.is_ascii());
        let sum = accum + u32::from(c);
        let multiplied = sum * 17;
        multiplied % (u32::from(u8::MAX) + 1)
    });
    // println!("{input:?} -> {result}");
    u8::try_from(result).expect("mod u8::MAX fits in u8")
}

struct Boxes<'a> {
    boxes: Vec<LensSlots<'a>>,
}
impl<'a> Boxes<'a> {
    fn new() -> Self {
        Self {
            boxes: vec![LensSlots::default(); usize::from(u8::MAX) + 1],
        }
    }
    fn get_mut(&mut self, which: u8) -> &mut LensSlots<'a> {
        let which = usize::from(which);
        self.boxes
            .get_mut(which)
            .expect("boxes initialized with u8::MAX entries")
    }
    fn update(&mut self, which: u8, command: Command<'a>) {
        let entry = self.get_mut(which);
        match command {
            Command::Insert(label_lens) => {
                entry.push(label_lens);
            }
            Command::Remove { label } => {
                entry.remove(label);
            }
        }
    }
    fn get_focus_power(&self) -> u32 {
        self.boxes
            .iter()
            .enumerate()
            .map(|(box_index, elem)| elem.get_focus_power(box_index))
            .sum()
    }
}
impl<'a, 'b: 'a> TryFrom<&'b str> for Boxes<'a> {
    type Error = anyhow::Error;
    fn try_from(value: &'b str) -> Result<Self, Self::Error> {
        let mut boxes = Boxes::new();
        for command_str in value.split(',') {
            let command = Command::try_from(command_str).with_context(|| command_str.to_owned())?;
            let which = ascii_hash(command.label());
            boxes.update(which, command);

            println!("After {command_str:?}:");
            println!("{boxes}");
        }
        Ok(boxes)
    }
}
impl std::fmt::Display for Boxes<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (n, slots) in self.boxes.iter().enumerate() {
            if !slots.0.is_empty() {
                write!(f, "Box {n}:")?;
                for label_lens in &slots.0 {
                    write!(f, " {label_lens}")?;
                }
                writeln!(f)?;
            }
        }
        Ok(())
    }
}
impl std::fmt::Display for LabelLens<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { label, lens } = self;
        write!(f, "[{label} {lens}]")
    }
}

#[derive(Clone, Default)]
struct LensSlots<'a>(Vec<LabelLens<'a>>);
impl<'a> LensSlots<'a> {
    fn push(&mut self, new: LabelLens<'a>) {
        if let Some(existing) = self.get_mut(new.label) {
            *existing = new;
        } else {
            self.0.push(new);
        }
    }
    fn remove(&mut self, label: &'a str) -> Option<LabelLens<'a>> {
        let index = self
            .0
            .iter()
            .enumerate()
            .find_map(|(idx, &elem)| (label == elem.label).then_some(idx))?;
        let label_lens = self.0.remove(index);
        Some(label_lens)
    }
    fn get_mut(&mut self, label: &'a str) -> Option<&mut LabelLens<'a>> {
        self.0
            .iter_mut()
            .find_map(|elem| (label == elem.label).then_some(elem))
    }
    fn get_focus_power(&self, box_index: usize) -> u32 {
        let box_index = u32::try_from(box_index + 1).unwrap();
        self.0
            .iter()
            .enumerate()
            .map(|(index, elem)| {
                let slot_index = u32::try_from(index + 1).unwrap();
                let lens = u32::from(elem.lens);
                // dbg!((elem.label, box_index, slot_index, lens, result));
                box_index * lens * slot_index
            })
            .sum()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LabelLens<'a> {
    label: &'a str,
    lens: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Command<'a> {
    Insert(LabelLens<'a>),
    Remove { label: &'a str },
}
impl<'a> Command<'a> {
    fn label(&self) -> &'a str {
        match self {
            Command::Insert(LabelLens { label, .. }) => label,
            Command::Remove { label } => label,
        }
    }
}
impl<'a, 'b: 'a> TryFrom<&'b str> for Command<'a> {
    type Error = anyhow::Error;
    fn try_from(value: &'b str) -> Result<Self, Self::Error> {
        const EQUALS: &str = "=";
        const DASH: &str = "-";

        let command = if let Some((label, lens_str)) = value.split_once(EQUALS) {
            let Ok(lens) = lens_str.parse() else {
                anyhow::bail!("invalid lens number {lens_str:?}")
            };
            Self::Insert(LabelLens { label, lens })
        } else if let Some(label) = value.strip_suffix(DASH) {
            Self::Remove { label }
        } else {
            anyhow::bail!("entry does not contain {EQUALS:?} or {DASH:?}")
        };
        Ok(command)
    }
}

#[cfg(test)]
mod tests {
    use crate::{analyze, ascii_hash, Boxes, Command, LabelLens};

    #[test]
    fn sample_hash() {
        assert_eq!(ascii_hash("HASH"), 52);
    }

    #[test]
    fn sample_input_hash_sum() {
        let input = "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7\n";
        let stats = analyze(input).unwrap();
        assert_eq!(stats.sum_of_hashes, 1320);
    }
    #[test]
    fn sample_input_box() {
        let input = "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7\n";
        let stats = analyze(input).unwrap();
        assert_eq!(stats.focusing_power, 145);
    }
    #[test]
    fn commands() {
        assert_eq!(
            Command::try_from("abc-").unwrap(),
            Command::Remove { label: "abc" }
        );
    }

    #[test]
    fn build_a_box() {
        let mut b = Boxes::try_from("abc=1,cde=5,abc=2").unwrap();

        let box_abc = b.get_mut(ascii_hash("abc"));
        assert_eq!(
            box_abc.get_mut("abc"),
            Some(&mut LabelLens {
                label: "abc",
                lens: 2
            })
        );

        let box_cde = b.get_mut(ascii_hash("cde"));
        assert_eq!(
            box_cde.get_mut("cde"),
            Some(&mut LabelLens {
                label: "cde",
                lens: 5
            })
        );
    }
    #[test]
    fn remove_from_box() {
        let mut b = Boxes::try_from("abc=1,cde=5,abc=2,cde-,abc-").unwrap();

        let box_abc = b.get_mut(ascii_hash("abc"));
        assert_eq!(box_abc.get_mut("abc"), None);

        let box_cde = b.get_mut(ascii_hash("cde"));
        assert_eq!(box_cde.get_mut("cde"), None);
    }
}
