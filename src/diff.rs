#[derive(Debug, Clone)]
pub enum DiffSegment {
    Common(String),
    OnlyA(String),
    OnlyB(String),
}

/// Maximum words for full LCS diff (prevents memory explosion).
const MAX_DIFF_WORDS: usize = 2000;

/// Word-level diff using LCS (longest common subsequence).
pub fn word_diff(a: &str, b: &str) -> Vec<DiffSegment> {
    let words_a: Vec<&str> = a.split_whitespace().collect();
    let words_b: Vec<&str> = b.split_whitespace().collect();

    // Guard against memory explosion: O(n*m) table
    if words_a.len() > MAX_DIFF_WORDS || words_b.len() > MAX_DIFF_WORDS {
        return vec![
            DiffSegment::OnlyA(words_a.join(" ")),
            DiffSegment::OnlyB(words_b.join(" ")),
        ];
    }

    let n = words_a.len();
    let m = words_b.len();

    // Build LCS table
    let mut dp = vec![vec![0u32; m + 1]; n + 1];
    for i in 1..=n {
        for j in 1..=m {
            if words_a[i - 1] == words_b[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    // Backtrack to build diff
    let mut segments = Vec::new();
    let mut i = n;
    let mut j = m;

    // We'll collect in reverse then flip
    let mut rev = Vec::new();
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && words_a[i - 1] == words_b[j - 1] {
            rev.push(DiffSegment::Common(words_a[i - 1].to_string()));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            rev.push(DiffSegment::OnlyB(words_b[j - 1].to_string()));
            j -= 1;
        } else {
            rev.push(DiffSegment::OnlyA(words_a[i - 1].to_string()));
            i -= 1;
        }
    }

    rev.reverse();

    // Merge consecutive segments of the same kind
    for seg in rev {
        match (&seg, segments.last_mut()) {
            (DiffSegment::Common(w), Some(DiffSegment::Common(prev))) => {
                prev.push(' ');
                prev.push_str(w);
            }
            (DiffSegment::OnlyA(w), Some(DiffSegment::OnlyA(prev))) => {
                prev.push(' ');
                prev.push_str(w);
            }
            (DiffSegment::OnlyB(w), Some(DiffSegment::OnlyB(prev))) => {
                prev.push(' ');
                prev.push_str(w);
            }
            _ => segments.push(seg),
        }
    }

    segments
}

/// Calculate agreement percentage between two texts.
pub fn agreement_percentage(a: &str, b: &str) -> f32 {
    let words_a: Vec<&str> = a.split_whitespace().collect();
    let words_b: Vec<&str> = b.split_whitespace().collect();
    let total = words_a.len().max(words_b.len()) as f32;
    if total == 0.0 {
        return 100.0;
    }

    // Guard against memory explosion
    if words_a.len() > MAX_DIFF_WORDS || words_b.len() > MAX_DIFF_WORDS {
        return 0.0;
    }

    let n = words_a.len();
    let m = words_b.len();
    let mut dp = vec![vec![0u32; m + 1]; n + 1];
    for i in 1..=n {
        for j in 1..=m {
            if words_a[i - 1] == words_b[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    (dp[n][m] as f32 / total) * 100.0
}
