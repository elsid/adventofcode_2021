use std::collections::HashMap;
use std::io::BufRead;
use std::str::FromStr;

fn main() {
    println!("{:?}", play_dirac_dice(std::io::stdin().lock()));
}

fn play_dirac_dice(buffer: impl BufRead) -> (u64, u64) {
    let positions: Vec<u8> = buffer
        .lines()
        .map(|v| parse_position(&v.unwrap()))
        .collect();
    let deterministic = {
        let mut die = DeterministicDice::default();
        let mut players: Vec<Player> = positions
            .iter()
            .map(|v| Player {
                position: *v,
                score: 0,
            })
            .collect();
        play_with_deterministic_dice(&mut players, &mut die);
        players.iter().map(|v| v.score).min().unwrap() as u64 * die.count
    };
    (deterministic, play_with_quantum_dice(&positions))
}

const QUANTUM_DICE_WIN_SCORE: u8 = 21;

fn play_with_quantum_dice(positions: &[u8]) -> u64 {
    let mut wins: [u64; 2] = [0; 2];
    let mut min_scores = [0u8; 4 * 10];
    let mut max_scores = [0u8; 4 * 10];
    for position in 1u8..=10 {
        for first_roll in 1u8..=3 {
            let (min_score, max_score) = get_minmax_score_with_first_roll(first_roll, position);
            min_scores[get_scores_index(first_roll, position)] = min_score;
            max_scores[get_scores_index(first_roll, position)] = max_score;
        }
        let (min_score, max_score) = get_minmax_score(position);
        min_scores[get_scores_index(0, position)] = min_score;
        max_scores[get_scores_index(0, position)] = max_score;
    }
    let players = [
        QuantumPlayer {
            position: positions[0],
            score: 0,
        },
        QuantumPlayer {
            position: positions[1],
            score: 0,
        },
    ];
    let mut snapshots = HashMap::new();
    for first_roll in 1u8..=3 {
        for second_roll in 1u8..=3 {
            for third_roll in 1u8..=3 {
                play_with_quantum_dice_recursive(
                    &mut Context {
                        min_scores: &min_scores,
                        snapshots: &mut snapshots,
                    },
                    &players,
                    first_roll + second_roll + third_roll,
                    0,
                    &mut wins,
                );
            }
        }
    }
    *wins.iter().max().unwrap()
}

struct Context<'a> {
    min_scores: &'a [u8; 4 * 10],
    snapshots: &'a mut HashMap<Snapshot, [u64; 2]>,
}

#[derive(Eq, PartialEq, Hash)]
struct Snapshot {
    player_index: u8,
    players: [QuantumPlayer; 2],
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct QuantumPlayer {
    position: u8,
    score: u8,
}

fn play_with_quantum_dice_recursive(
    ctx: &mut Context,
    players: &[QuantumPlayer; 2],
    roll: u8,
    step: u8,
    wins: &mut [u64; 2],
) {
    let player_index = (step % 2) as usize;
    let position = (players[player_index].position + roll - 1) % 10 + 1;
    let score = players[player_index].score + position;
    if score >= QUANTUM_DICE_WIN_SCORE {
        wins[player_index] += 1;
        return;
    }
    let next_player_index = (player_index + 1) % 2;
    if players[next_player_index].score
        + ctx.min_scores[get_scores_index(0, players[next_player_index].position)]
        >= QUANTUM_DICE_WIN_SCORE
    {
        wins[next_player_index] += 27;
        return;
    }
    let mut new_players = players.clone();
    new_players[player_index].position = position;
    new_players[player_index].score = score;
    let snapshot = Snapshot {
        player_index: player_index as u8,
        players: new_players.clone(),
    };
    if let Some(v) = ctx.snapshots.get(&snapshot) {
        add_wins(v, wins);
        return;
    }
    let mut new_wins = [0; 2];
    for first_roll in 1..=3 {
        let next_player_index = (player_index + 1) % 2;
        if players[next_player_index].score
            + ctx.min_scores[get_scores_index(first_roll, players[next_player_index].position)]
            >= QUANTUM_DICE_WIN_SCORE
        {
            new_wins[next_player_index] += 9;
            continue;
        }
        for second_roll in 1..=3 {
            for third_roll in 1..=3 {
                play_with_quantum_dice_recursive(
                    ctx,
                    &new_players,
                    first_roll + second_roll + third_roll,
                    step + 1,
                    &mut new_wins,
                );
            }
        }
    }
    ctx.snapshots.insert(snapshot, new_wins);
    add_wins(&new_wins, wins);
}

fn add_wins(add: &[u64; 2], wins: &mut [u64; 2]) {
    for i in 0..2 {
        wins[i] += add[i];
    }
}

fn get_scores_index(first_roll: u8, position: u8) -> usize {
    (first_roll + (position - 1) * 4) as usize
}

fn get_minmax_score(position: u8) -> (u8, u8) {
    let mut min_score = u8::MAX;
    let mut max_score = u8::MIN;
    for first_roll in 1..=3 {
        let (cur_min_score, cur_max_score) = get_minmax_score_with_first_roll(first_roll, position);
        min_score = min_score.min(cur_min_score);
        max_score = max_score.max(cur_max_score);
    }
    (min_score, max_score)
}

fn get_minmax_score_with_first_roll(first_roll: u8, position: u8) -> (u8, u8) {
    let mut min_score = u8::MAX;
    let mut max_score = u8::MIN;
    for second_roll in 1..=3 {
        for third_roll in 1..=3 {
            let next_position = get_next_position(position, first_roll + second_roll + third_roll);
            if min_score > next_position {
                min_score = next_position;
            }
            if max_score < next_position {
                max_score = next_position;
            }
        }
    }
    (min_score, max_score)
}

fn get_next_position(position: u8, roll: u8) -> u8 {
    (position + roll - 1) % 10 + 1
}

fn play_with_deterministic_dice(players: &mut [Player], die: &mut DeterministicDice) {
    loop {
        for player in players.iter_mut() {
            let roll = die.roll3();
            player.position = ((player.position as u16 + roll - 1) % 10 + 1) as u8;
            player.score += player.position as u16;
            if player.score >= 1000 {
                return;
            }
        }
    }
}

struct DeterministicDice {
    value: u16,
    count: u64,
}

impl Default for DeterministicDice {
    fn default() -> Self {
        DeterministicDice { value: 1, count: 0 }
    }
}

impl DeterministicDice {
    fn roll3(&mut self) -> u16 {
        let result = self.value + self.value % 100 + 1 + (self.value + 1) % 100 + 1;
        self.value = (self.value + 2) % 100 + 1;
        self.count += 3;
        result
    }
}

#[derive(Clone)]
struct Player {
    position: u8,
    score: u16,
}

fn parse_position(line: &str) -> u8 {
    let (_, position) = line.split_once("starting position: ").unwrap();
    u8::from_str(position).unwrap()
}

#[test]
fn example_test() {
    let buffer = r#"Player 1 starting position: 4
Player 2 starting position: 8
"#
    .as_bytes();
    assert_eq!(play_dirac_dice(buffer), (739785, 444356092776315));
}
