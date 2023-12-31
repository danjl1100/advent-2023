use std::{cmp::Reverse, collections::BTreeMap};

fn main() -> anyhow::Result<()> {
    println!("hello, camel cards!");

    let input = advent_2023::get_input_string()?;

    let total_winnings = analyze_hands(&input, Rules::Jokers)?;
    println!("Total winnings for all hands: {total_winnings}");

    Ok(())
}

#[derive(Clone, Copy, Debug)]
enum Rules {
    #[allow(unused)]
    Jacks,
    Jokers,
}

fn analyze_hands(input: &str, rules: Rules) -> anyhow::Result<u32> {
    let mut hands = input
        .lines()
        .map(|line| parse_hand(line, rules))
        .collect::<Result<Vec<_>, _>>()?;

    // // DEBUG
    // println!("Input Hands:");
    // for (hand, Bid(bid)) in &hands {
    //     let Hand { cards, ty } = hand;
    //     println!("\tHand {cards:?} {ty:?}, bid {bid}");
    // }

    hands.sort_by_key(|&(hand, _bid)| hand);

    // DEBUG
    println!("Sorted Hands:");
    for (index, (hand, Bid(bid))) in hands.iter().enumerate() {
        let Hand { cards, ty } = hand;
        // println!("\tHand {cards:?} {ty:?}, bid {bid}");

        // DEBUG, for comparison
        let ty_number = match ty {
            Type::HighCard => 1,
            Type::OnePair => 2,
            Type::TwoPair => 3,
            Type::ThreeOfAKind => 4,
            Type::FullHouse => 5,
            Type::FourOfAKind => 6,
            Type::FiveOfAKind => 7,
        };
        let cards_str = cards.iter().fold(String::new(), |mut acc, c| {
            if *c == Card::Joker {
                acc += "J";
            } else {
                acc += &format!("{c}");
            }
            acc
        });
        println!(
            "{count:3}. {cards_str} ({bid:3}) {ty_number}",
            count = index + 1,
        );
    }

    let total_winnings = hands
        .into_iter()
        .enumerate()
        .map(|(index, (_hand, bid))| {
            let rank = u32::try_from(index + 1).expect("no overflow");
            bid.0 * rank
        })
        .sum();
    Ok(total_winnings)
}
fn parse_hand(line: &str, rules: Rules) -> anyhow::Result<(Hand, Bid)> {
    let mut line_parts = line.split_whitespace();
    let Some(hand) = line_parts.next() else {
        anyhow::bail!("empty line")
    };
    let Some(bid) = line_parts.next() else {
        anyhow::bail!("missing bid on line {line:?}")
    };
    if let Some(extra) = line_parts.next() {
        anyhow::bail!("unexpected part on line {line:?}: {extra:?}")
    }

    let hand: Hand = hand.parse().map_err(|s| anyhow::anyhow!("hand {s:?}"))?;
    let bid = bid.parse().map_err(|s| anyhow::anyhow!("bid {s:?}"))?;

    let debug_original_ty = hand.ty;

    let hand = match rules {
        Rules::Jacks => hand,
        Rules::Jokers => hand.reinterpret_as_jokers(),
    };
    if hand.ty != debug_original_ty {
        let cards_str = hand
            .cards
            .iter()
            .copied()
            .fold(String::new(), |mut acc, c| {
                use std::fmt::Write;
                let _ = write!(acc, "{c}");
                acc
            });
        let ty = hand.ty;
        let direction = if ty > debug_original_ty {
            "higher"
        } else {
            "lower"
        };
        eprintln!("{cards_str:?} changed {direction} from {debug_original_ty:?} to {ty:?}");
    }

    Ok((hand, Bid(bid)))
}

#[derive(Clone, Copy, Debug)]
struct Bid(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Hand {
    cards: [Card; 5],
    ty: Type,
}
impl Hand {
    fn reinterpret_as_jokers(self) -> Self {
        let Self { mut cards, ty: _ } = self;
        for card in cards.iter_mut() {
            *card = card.reinterpret_as_joker();
        }
        let ty = Type::from(&cards);
        Self { cards, ty }
    }
}
impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Hand {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ty.cmp(&other.ty).then(self.cards.cmp(&other.cards))
    }
}
impl std::str::FromStr for Hand {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cards = s
            .chars()
            .map(Card::try_from)
            .collect::<Result<Vec<Card>, _>>()?
            .try_into()
            .map_err(|cards_vec| format!("invalid number of cards, {cards_vec:?}"))?;
        let ty = Type::from(&cards);
        Ok(Self { cards, ty })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
// PartialOrd derive: lowest to highest
enum Type {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}
impl From<&[Card; 5]> for Type {
    fn from(cards: &[Card; 5]) -> Self {
        let counts = {
            let mut counts: BTreeMap<Card, usize> = BTreeMap::new();
            for card in cards {
                let count = counts.entry(*card).or_default();
                *count += 1;
            }
            counts
        };

        let count_3_no_jokers = counts.iter().filter(|(_, &count)| count == 3).count();
        let count_2_no_jokers = counts.iter().filter(|(_, &count)| count == 2).count();

        let count_wild = counts.get(&Card::Joker).copied().unwrap_or_default();
        dbg!(cards, &counts, count_wild);
        let counts_wild = {
            let mut counts_wild = counts.clone();

            'outer: for n in (0..5).rev() {
                for (&card, count) in counts_wild.iter_mut() {
                    if *count == n && card != Card::Joker {
                        *count += count_wild;
                        break 'outer;
                    }
                }
            }
            counts_wild
        };

        let count_5 = counts_wild.iter().filter(|(_, &count)| count == 5).count();
        let count_4 = counts_wild.iter().filter(|(_, &count)| count == 4).count();
        let count_3 = counts_wild.iter().filter(|(_, &count)| count == 3).count();
        let count_2 = counts_wild.iter().filter(|(_, &count)| count == 2).count();
        // let count_1 = counts_wild.iter().filter(|(_, &count)| count == 1).count();

        // no longer accurate with wild cards
        // debug_assert_eq!(
        //     5 * count_5 + 4 * count_4 + 3 * count_3 + 2 * count_2 + count_1,
        //     5
        // );

        if count_5 >= 1 {
            Self::FiveOfAKind
        } else if count_4 >= 1 {
            Self::FourOfAKind
        } else {
            let mut allow_full_house = count_3_no_jokers >= 1 && count_2_no_jokers >= 1;
            dbg!(allow_full_house, count_wild, count_3, count_2);
            if !allow_full_house && count_wild >= 1 && count_3 >= 1 && count_2 >= 1 {
                let mut count_wild_remaining = count_wild;
                let mut counts_added: Vec<_> = counts
                    .iter()
                    .filter_map(|(&card, &count)| (card != Card::Joker).then_some(count))
                    .collect();
                counts_added.sort_by_key(|&n| Reverse(n));

                // assign wilds to 3, then 2, without duplicates
                let mut counts_added = counts_added.into_iter();
                let count_first = counts_added.next();
                let count_second = counts_added.next();
                dbg!(count_first, count_second);
                if let Some((mut count_first, mut count_second)) = count_first.zip(count_second) {
                    // threes
                    while count_first < 3 && count_wild_remaining > 0 {
                        count_wild_remaining -= 1;
                        count_first += 1;
                    }
                    while count_second < 2 && count_wild_remaining > 0 {
                        count_wild_remaining -= 1;
                        count_second += 1;
                    }
                    dbg!(&counts, count_wild, count_first, count_second);
                    // check again
                    if count_first >= 3 && count_second >= 2 {
                        allow_full_house = true;
                    }
                }
            }

            if allow_full_house {
                Self::FullHouse
            } else if count_3 >= 1 {
                Self::ThreeOfAKind
            } else if count_2 >= 2 {
                Self::TwoPair
            } else if count_2 >= 1 {
                Self::OnePair
            } else {
                // no longer accurate with wild cards
                // debug_assert_eq!(count_1 - count_wild, 5);
                Self::HighCard
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// PartialOrd derive: lowest to highest
enum Card {
    Joker,
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
    N9,
    T,
    Jack,
    Q,
    K,
    A,
}
impl Card {
    fn reinterpret_as_joker(self) -> Self {
        match self {
            Self::Jack => Self::Joker,
            other => other,
        }
    }
}

impl TryFrom<char> for Card {
    type Error = char;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        let value = match value {
            '2' => Self::N2,
            '3' => Self::N3,
            '4' => Self::N4,
            '5' => Self::N5,
            '6' => Self::N6,
            '7' => Self::N7,
            '8' => Self::N8,
            '9' => Self::N9,
            'T' => Self::T,
            'J' => Self::Jack, // NOTE: changed to Joker by `reinterpret_as_joker`
            'Q' => Self::Q,
            'K' => Self::K,
            'A' => Self::A,
            _ => {
                return Err(value);
            }
        };
        Ok(value)
    }
}
impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Card::Joker => '?',
            Card::N2 => '2',
            Card::N3 => '3',
            Card::N4 => '4',
            Card::N5 => '5',
            Card::N6 => '6',
            Card::N7 => '7',
            Card::N8 => '8',
            Card::N9 => '9',
            Card::T => 'T',
            Card::Jack => 'J',
            Card::Q => 'Q',
            Card::K => 'K',
            Card::A => 'A',
        };
        write!(f, "{c}")
    }
}
impl std::fmt::Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use crate::{analyze_hands, parse_hand, Card, Rules, Type};

    macro_rules! test_types {
        (
            $rules:expr;
            $(
                $str:expr => $expected_ty:expr;
            )+
        ) => {
            let rules: Rules = $rules;
            $({
                let hand_str: &'static str = $str;
                let (hand, _) = parse_hand(&format!("{hand_str} 0"), rules).expect(hand_str);
                assert_eq!(hand.ty, $expected_ty, "hand {hand_str} {hand:?}");
                println!("------------------------------");
            })+
        };
    }

    const SAMPLE_INPUT: &str = "32T3K 765
T55J5 684
KK677 28
KTJJT 220
QQQJA 483";

    #[test]
    fn sample_input() {
        let total_winnings = analyze_hands(SAMPLE_INPUT, Rules::Jacks).unwrap();
        assert_eq!(total_winnings, 6440);
    }
    #[test]
    fn sample_input_jokers() {
        let total_winnings = analyze_hands(SAMPLE_INPUT, Rules::Jokers).unwrap();
        assert_eq!(total_winnings, 5905);
    }

    #[test]
    fn card_from_char() {
        let a = Card::try_from('A').unwrap();
        let nine = Card::try_from('9').unwrap();
        let two = Card::try_from('2').unwrap();
        assert!(a > nine);
        assert!(nine > two);
        assert!(a > two);

        let joker = Card::try_from('J').unwrap().reinterpret_as_joker();
        assert!(a > joker);
        assert!(nine > joker);
        assert!(two > joker);
    }

    #[test]
    fn classify_types() {
        test_types! {
            Rules::Jacks;
            "KKQQ2" => Type::TwoPair;
            "KKQQK" => Type::FullHouse;
            "2222K" => Type::FourOfAKind;
            "56652" => Type::TwoPair;
            "56642" => Type::OnePair;
            "44444" => Type::FiveOfAKind;
            "3459T" => Type::HighCard;
            "666QK" => Type::ThreeOfAKind;
        }
    }

    #[test]
    fn classify_joker_to_highest() {
        test_types! {
            Rules::Jokers;
            // HighCard not possible, will auto-promote to OnePair
            "2345J" => Type::OnePair;
            // TwoPair - not possible, will auto-promote to ThreeOfAKind
            "2234J" => Type::ThreeOfAKind;
            // FullHouse - not possible, will auto-promote to FourOfAKind
            "2223J" => Type::FourOfAKind;
            "2222J" => Type::FiveOfAKind;
        }
    }

    #[test]
    fn no_sharing_jokers_in_fullhouse() {
        test_types! {
            Rules::Jokers;
            "24J8J" => Type::ThreeOfAKind;
            "4JTAJ" => Type::ThreeOfAKind;
            "6AJJ2" => Type::ThreeOfAKind;
            "J564J" => Type::ThreeOfAKind;
            "QJ2J8" => Type::ThreeOfAKind;
        }
    }
    #[test]
    fn full_houses() {
        test_types! {
            Rules::Jokers;
            "226J6" => Type::FullHouse;
            "2J525" => Type::FullHouse;
            "AATJT" => Type::FullHouse;
            "JA8A8" => Type::FullHouse;
            "Q33JQ" => Type::FullHouse;
            "QQ5J5" => Type::FullHouse;
        }
    }
}
