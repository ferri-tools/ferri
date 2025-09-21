use std::cmp::min;

fn main() {
    let n = 5;
    let values = vec![1, 2, 5, 1, 4];

    let min_coins = calculate_min_coins(n, &values);

    println!("Minimum coins needed: {}", min_coins);
}

fn calculate_min_coins(n: usize, values: &[usize]) -> usize {
    let mut dp = vec![usize::MAX; n + 1];
    dp[0] = 0;

    for i in 1..=n {
        for &value in values {
            if value <= i && dp[i - value] != usize::MAX {
                dp[i] = min(dp[i], dp[i - value] + 1);
            }
        }
    }

    if dp[n] == usize::MAX {
        return 0; // Or handle the case where no combination is possible
    }

    dp[n]
}
