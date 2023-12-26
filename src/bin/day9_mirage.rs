fn main() -> anyhow::Result<()> {
    println!("hello, mirage!");
    let input = advent_2023::get_input_string()?;

    let inputs = parse_input(&input)?;

    let sum_nexts = sum_next_values(inputs)?;
    println!("Sum of the next predictions: {sum_nexts}");

    Ok(())
}

fn parse_input(input: &str) -> anyhow::Result<Vec<Vec<i64>>> {
    input
        .lines()
        .map(|line| {
            line.split_whitespace()
                .map(|s| {
                    s.parse()
                        .map_err(|e| anyhow::anyhow!("invalid token {s:?}: {e}"))
                })
                .collect()
        })
        .collect()
}

fn sum_next_values(inputs: Vec<Vec<i64>>) -> anyhow::Result<i64> {
    let values = inputs
        .iter()
        .map(|series| predict_sequence(series))
        .collect::<anyhow::Result<Vec<_>, _>>()?;
    let sum = values.into_iter().sum();
    Ok(sum)
}
fn predict_sequence(series: &Vec<i64>) -> anyhow::Result<i64> {
    fn inner_fn(series: &[i64]) -> anyhow::Result<i64> {
        let diffs = series
            .windows(2)
            .map(|window| {
                let [prev, next] = window.try_into().expect("windows of 2");
                next - prev
            })
            .collect::<Vec<_>>();

        if diffs.iter().copied().all(|diff| diff == 0) {
            let first = series.first().copied().expect("nonempty series");
            let true = series.iter().all(|&n| n == first) else {
                anyhow::bail!("diff zero when series not homogenous: {series:?}")
            };
            Ok(first)
        } else {
            let next_diff = inner_fn(&diffs)?;
            let last = series.last().copied().expect("nonempty series");
            Ok(last + next_diff)
        }
    }
    if series.is_empty() {
        anyhow::bail!("empty series")
    }
    inner_fn(series)
    // .and_then(|result| {
    //     result.ok_or_else(|| anyhow::anyhow!("series top-level prediction is None"))
    // })
}

#[cfg(test)]
mod tests {
    use crate::{parse_input, sum_next_values};

    #[test]
    fn sample_input() {
        let input = "0 3 6 9 12 15
1 3 6 10 15 21
10 13 16 21 30 45";
        let inputs = parse_input(input).unwrap();
        dbg!(&inputs);

        let sum_nexts = sum_next_values(inputs).unwrap();
        assert_eq!(sum_nexts, 114);
    }
}
