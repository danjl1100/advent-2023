use std::collections::BTreeMap;

fn main() -> anyhow::Result<()> {
    println!("hello, camel cards!");

    let input = advent_2023::get_input_string()?;

    let total_winnings = analyze_hands(&input)?;
    println!("Total winnings for all hands: {total_winnings}");

    Ok(())
}

fn analyze_hands(input: &str) -> anyhow::Result<u32> {
    let mut hands = input
        .lines()
        .map(parse_hand)
        .collect::<Result<Vec<_>, _>>()?;
    dbg!(&hands);

    hands.sort_by_key(|&(hand, _bid)| hand);

    dbg!(("sorted", &hands));

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
fn parse_hand(line: &str) -> anyhow::Result<(Hand, Bid)> {
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

    let hand = hand.parse().map_err(|s| anyhow::anyhow!("hand {s:?}"))?;
    let bid = bid.parse().map_err(|s| anyhow::anyhow!("bid {s:?}"))?;

    Ok((hand, Bid(bid)))
}

#[derive(Clone, Copy, Debug)]
struct Bid(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Hand {
    cards: [Card; 5],
    ty: Type,
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
            let mut counts: BTreeMap<Card, u8> = BTreeMap::new();
            for card in cards {
                let count = counts.entry(*card).or_default();
                *count += 1;
            }
            counts
        };

        let count_5 = counts.iter().filter(|(_, &count)| count == 5).count();
        let count_4 = counts.iter().filter(|(_, &count)| count == 4).count();
        let count_3 = counts.iter().filter(|(_, &count)| count == 3).count();
        let count_2 = counts.iter().filter(|(_, &count)| count == 2).count();
        let count_1 = counts.iter().filter(|(_, &count)| count == 1).count();

        debug_assert_eq!(
            5 * count_5 + 4 * count_4 + 3 * count_3 + 2 * count_2 + count_1,
            5
        );

        if count_5 == 1 {
            Self::FiveOfAKind
        } else if count_4 == 1 {
            Self::FourOfAKind
        } else if count_3 == 1 && count_2 == 1 {
            Self::FullHouse
        } else if count_3 == 1 {
            Self::ThreeOfAKind
        } else if count_2 == 2 {
            Self::TwoPair
        } else if count_2 == 1 {
            Self::OnePair
        } else {
            debug_assert_eq!(count_1, 5);
            Self::HighCard
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
// PartialOrd derive: lowest to highest
enum Card {
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
    N9,
    T,
    J,
    Q,
    K,
    A,
}

impl TryFrom<char> for Card {
    type Error = char;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        let value = match value {
            'A' => Self::A,
            'K' => Self::K,
            'Q' => Self::Q,
            'J' => Self::J,
            'T' => Self::T,
            '9' => Self::N9,
            '8' => Self::N8,
            '7' => Self::N7,
            '6' => Self::N6,
            '5' => Self::N5,
            '4' => Self::N4,
            '3' => Self::N3,
            '2' => Self::N2,
            _ => {
                return Err(value);
            }
        };
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{analyze_hands, Card, Hand, Type};

    macro_rules! test_types {
        (
            $(
                $str:expr => $expected_ty:expr;
            )+
        ) => {
            $({
                let hand_str = $str;
                let hand = Hand::from_str(hand_str).unwrap();
                assert_eq!(hand.ty, $expected_ty, "hand {hand_str} {hand:?}");
            })+
        };
    }

    #[test]
    fn sample_input() {
        let input = "32T3K 765
T55J5 684
KK677 28
KTJJT 220
QQQJA 483";
        let total_winnings = analyze_hands(input).unwrap();
        assert_eq!(total_winnings, 6440);
    }

    #[test]
    fn card_from_char() {
        let a = Card::try_from('A').unwrap();
        let nine = Card::try_from('9').unwrap();
        let two = Card::try_from('2').unwrap();
        assert!(a > nine);
        assert!(nine > two);
        assert!(a > two);
    }

    #[test]
    fn classify_types() {
        test_types! {
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
}
