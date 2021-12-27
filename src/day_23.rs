use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, HashMap};
use std::io::BufRead;

fn main() {
    println!("{:?}", relocate_amphipods(std::io::stdin().lock()));
}

const MAX_STATES: usize = 2_000_000;

fn relocate_amphipods(buffer: impl BufRead) -> (u32, u32) {
    let input: String = buffer.lines().map(|v| v.unwrap() + "\n").collect();
    let first = {
        let World { env, state } = parse_world(input.as_bytes());
        find_min_energy(&env, state)
    };
    let second = {
        let extended_input: String = input
            .as_bytes()
            .lines()
            .enumerate()
            .map(|(i, v)| {
                if i == 2 {
                    v.unwrap() + "\n  #D#C#B#A#\n  #D#B#A#C#\n"
                } else {
                    v.unwrap() + "\n"
                }
            })
            .collect();
        let World { env, state } = parse_world(extended_input.as_bytes());
        find_min_energy(&env, state)
    };
    (first, second)
}

fn find_min_energy(env: &Env, mut initial_state: State) -> u32 {
    for i in 0..initial_state.amphipods.len() {
        if is_amphipod_in_final_state(&initial_state.amphipods[i], env, &initial_state) {
            initial_state.amphipods[i].done = true;
        }
    }
    let mut explored = HashMap::new();
    explored.insert(make_state_key(&initial_state, &env.map), 0);
    let mut states = vec![initial_state];
    let mut new_states = BinaryHeap::new();
    new_states.push((Reverse(0), 0));
    while let Some((Reverse(f_score), state_index)) = new_states.pop() {
        if is_final(&states[state_index]) {
            return f_score;
        }
        if states.len() >= MAX_STATES {
            continue;
        }
        let g_score = explored[&make_state_key(&states[state_index], &env.map)];
        for amphipod_index in 0..states[state_index].amphipods.len() {
            try_move_amphipod(
                amphipod_index,
                state_index,
                g_score,
                &mut Ctx {
                    env,
                    states: &mut states,
                    new_states: &mut new_states,
                    explored: &mut explored,
                },
            );
        }
    }
    u32::MAX
}

fn is_final(state: &State) -> bool {
    state.amphipods.iter().all(|v| v.done)
}

fn make_state_key(state: &State, map: &Map) -> StateKey {
    StateKey {
        positions: state
            .amphipods
            .iter()
            .map(|v| map.index(v.position) as u8)
            .collect(),
    }
}

fn is_amphipod_in_final_state(amphipod: &Amphipod, env: &Env, state: &State) -> bool {
    if let Tile::Room(room_index) = env.map.tile(amphipod.position) {
        if room_index != amphipod.kind as u8 {
            return false;
        }
        let room_x = env.rooms[room_index as usize].x;
        let room_bottom = env.rooms[room_index as usize].depth + 2;
        if amphipod.position[1] == room_bottom - 1 {
            return true;
        }
        state
            .amphipods
            .iter()
            .filter(|v| {
                v.position[0] == room_x
                    && v.position[1] > amphipod.position[1]
                    && v.kind == amphipod.kind
            })
            .count()
            == (room_bottom - 1 - amphipod.position[1]) as usize
    } else {
        false
    }
}

const STEPS: [Vec2; 4] = [[1, 0], [-1, 0], [0, 1], [0, -1]];

struct Ctx<'a> {
    env: &'a Env,
    states: &'a mut Vec<State>,
    new_states: &'a mut BinaryHeap<(Reverse<u32>, usize)>,
    explored: &'a mut HashMap<StateKey, u32>,
}

fn try_move_amphipod(amphipod_index: usize, state_index: usize, g_score: u32, ctx: &mut Ctx) {
    if ctx.states[state_index].amphipods[amphipod_index].done {
        return;
    }
    let kind = ctx.states[state_index].amphipods[amphipod_index].kind;
    let src = ctx.states[state_index].amphipods[amphipod_index].position;
    let room_x = ctx.env.rooms[kind as usize].x;
    let tile = ctx
        .env
        .map
        .tile(ctx.states[state_index].amphipods[amphipod_index].position);
    if tile == Tile::Hallway {
        for room_y in (2..ctx.env.rooms[kind as usize].depth + 2).rev() {
            let dst = [room_x, room_y];
            if ctx.states[state_index]
                .amphipods
                .iter()
                .any(|v| v.position == dst && v.kind != kind)
            {
                break;
            }
            if ctx.states[state_index]
                .amphipods
                .iter()
                .all(|v| v.position != dst)
            {
                if let Some(length) =
                    find_shortest_path(kind, src, dst, &ctx.states[state_index], ctx.env)
                {
                    try_apply_transition(
                        amphipod_index,
                        dst,
                        state_index,
                        g_score,
                        length as u32,
                        ctx,
                    );
                    break;
                }
            }
        }
    } else {
        for hallway_x in 0..ctx.env.map.width as i8 {
            let dst = [hallway_x, 1];
            if ctx.env.map.tile(dst) != Tile::Hallway {
                continue;
            }
            if ctx.states[state_index]
                .amphipods
                .iter()
                .all(|v| v.position != dst)
            {
                if let Some(length) =
                    find_shortest_path(kind, src, dst, &ctx.states[state_index], ctx.env)
                {
                    try_apply_transition(
                        amphipod_index,
                        dst,
                        state_index,
                        g_score,
                        length as u32,
                        ctx,
                    );
                }
            }
        }
    }
}

fn try_apply_transition(
    amphipod_index: usize,
    next_position: Vec2,
    state_index: usize,
    g_score: u32,
    length: u32,
    ctx: &mut Ctx,
) {
    let mut state = ctx.states[state_index].clone();
    move_amphipod(amphipod_index, next_position, ctx.env, &mut state);
    let transition_cost = get_energy_cost(state.amphipods[amphipod_index].kind) * length;
    let tentative_g_score = g_score + transition_cost;
    let key = make_state_key(&state, &ctx.env.map);
    if let Some(min_g_score) = ctx.explored.get_mut(&key) {
        if *min_g_score <= tentative_g_score {
            return;
        }
        *min_g_score = tentative_g_score;
    } else {
        ctx.explored.insert(key, tentative_g_score);
    }
    let f_score = tentative_g_score + get_h_score(&state, ctx.env);
    ctx.new_states.push((Reverse(f_score), ctx.states.len()));
    ctx.states.push(state);
}

fn get_h_score(state: &State, env: &Env) -> u32 {
    let fill_room: u32 = env
        .rooms
        .iter()
        .enumerate()
        .map(|(i, room)| {
            let left_space = state
                .amphipods
                .iter()
                .filter(|v| v.done && v.position[0] == room.x)
                .map(|v| v.position[1] - 2)
                .min()
                .unwrap_or(room.depth) as u32;
            left_space * (left_space + 1) / 2 * ENERGY_COST[i]
        })
        .sum();
    let move_to_room: u32 = state
        .amphipods
        .iter()
        .filter(|v| !v.done)
        .map(|v| {
            let room = &env.rooms[v.kind as usize];
            let length = if env.map.tile(v.position) == Tile::Hallway {
                (v.position[0] - room.x).abs()
            } else if room.x == v.position[0] {
                v.position[1] + 1
            } else {
                (v.position[0] - room.x).abs() + (v.position[1] - 1)
            };
            get_energy_cost(v.kind) * length as u32
        })
        .sum();
    fill_room + move_to_room
}

fn find_shortest_path(
    kind: AmphipodType,
    src: Vec2,
    dst: Vec2,
    state: &State,
    env: &Env,
) -> Option<u8> {
    let mut nodes = BinaryHeap::new();
    let mut distances: Vec<u8> = std::iter::repeat(u8::MAX)
        .take(env.map.width * env.map.height)
        .collect();
    let mut predecessors: Vec<u8> = std::iter::repeat(u8::MAX)
        .take(env.map.width * env.map.height)
        .collect();
    distances[env.map.index(src)] = 0;
    nodes.push((Reverse(0u8), src));
    while let Some((Reverse(cost), position)) = nodes.pop() {
        if position == dst {
            return Some(cost);
        }
        let index = env.map.index(position) as u8;
        for step in STEPS {
            if !can_move_amphipod(kind, position, step, env, state) {
                continue;
            }
            let next_position = add_vec2(position, step);
            let new_cost = cost + 1;
            let next_index = env.map.index(next_position);
            if distances[next_index] <= new_cost {
                continue;
            }
            nodes.push((Reverse(new_cost), next_position));
            distances[next_index] = new_cost;
            predecessors[next_index] = index;
        }
    }
    None
}

fn parse_world(buffer: impl BufRead) -> World {
    let mut tiles = BTreeMap::new();
    let mut x = 0i8;
    let mut y = 0i8;
    let mut width = 0usize;
    let mut amphipods = Vec::new();
    let mut rooms: Vec<Room> = Vec::with_capacity(4);
    for line in buffer.lines() {
        for symbol in line.unwrap().as_bytes() {
            match symbol {
                b'#' => {
                    tiles.insert([x, y], Tile::Wall);
                }
                b'.' => {
                    tiles.insert([x, y], Tile::Hallway);
                }
                b'A' | b'B' | b'C' | b'D' => {
                    if let Some(Tile::Room(index)) = tiles.get(&[x, y - 1]).cloned() {
                        rooms[index as usize].depth += 1;
                        tiles.insert([x, y], Tile::Room(index));
                    } else {
                        tiles.insert([x, y], Tile::Room(rooms.len() as u8));
                    }
                    amphipods.push(Amphipod {
                        position: [x, y],
                        kind: match symbol {
                            b'A' => AmphipodType::Amber,
                            b'B' => AmphipodType::Bronze,
                            b'C' => AmphipodType::Copper,
                            b'D' => AmphipodType::Desert,
                            _ => panic!(),
                        },
                        done: false,
                    });
                    if matches!(tiles.get(&[x, y - 1]), Some(&Tile::Hallway)) {
                        tiles.insert([x, y - 1], Tile::HallwayRestricted);
                        rooms.push(Room { x, depth: 1 });
                    }
                }
                _ => (),
            }
            x += 1;
            width = width.max(x as usize);
        }
        x = 0;
        y += 1;
    }
    let height = y as usize;
    let mut map = Map {
        tiles: std::iter::repeat(Tile::Wall).take(width * height).collect(),
        width,
        height,
    };
    for (position, tile) in tiles.iter() {
        let index = map.index(*position);
        map.tiles[index] = *tile;
    }
    let env = Env { map, rooms };
    let state = State { amphipods };
    World { env, state }
}

fn can_move_amphipod(
    kind: AmphipodType,
    position: Vec2,
    step: Vec2,
    env: &Env,
    state: &State,
) -> bool {
    let next_position = add_vec2(position, step);
    if next_position[0] < 0
        || next_position[1] < 0
        || next_position[0] as usize >= env.map.width
        || next_position[1] as usize >= env.map.height
    {
        return false;
    }
    let next_tile_index = env.map.index(next_position);
    if let Tile::Room(next_room) = env.map.tiles[next_tile_index] {
        if next_room != kind as u8 {
            if let Tile::Room(current_room) = env.map.tile(position) {
                if next_room != current_room {
                    return false;
                }
            }
        }
    }
    state.amphipods.iter().all(|v| v.position != next_position)
        && env.map.tiles[next_tile_index] != Tile::Wall
}

fn move_amphipod(amphipod_index: usize, next_position: Vec2, env: &Env, state: &mut State) {
    state.amphipods[amphipod_index].position = next_position;
    if is_amphipod_in_final_state(&state.amphipods[amphipod_index], env, state) {
        state.amphipods[amphipod_index].done = true;
    }
}

const ENERGY_COST: [u32; 4] = [1, 10, 100, 1000];

fn get_energy_cost(kind: AmphipodType) -> u32 {
    ENERGY_COST[kind as usize]
}

fn add_vec2(mut a: Vec2, b: Vec2) -> Vec2 {
    for i in 0..2 {
        a[i] += b[i];
    }
    a
}

struct World {
    env: Env,
    state: State,
}

struct Env {
    map: Map,
    rooms: Vec<Room>,
}

#[derive(Clone)]
struct State {
    amphipods: Vec<Amphipod>,
}

#[derive(Eq, PartialEq, Hash)]
struct StateKey {
    positions: Vec<u8>,
}

struct Room {
    x: i8,
    depth: i8,
}

type Vec2 = [i8; 2];

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
enum AmphipodType {
    Amber,
    Bronze,
    Copper,
    Desert,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct Amphipod {
    kind: AmphipodType,
    done: bool,
    position: Vec2,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum Tile {
    Wall,
    Hallway,
    HallwayRestricted,
    Room(u8),
}

struct Map {
    tiles: Vec<Tile>,
    width: usize,
    height: usize,
}

impl Map {
    fn index(&self, position: Vec2) -> usize {
        position[0] as usize + position[1] as usize * self.width
    }

    fn tile(&self, position: Vec2) -> Tile {
        self.tiles[self.index(position)]
    }
}

#[test]
fn example_scenario_test() {
    let buffer = r#"#############
#...........#
###B#C#B#D###
  #D#C#B#A#
  #D#B#A#C#
  #A#D#C#A#
  #########
"#
    .as_bytes();
    let World { env, mut state } = parse_world(buffer);
    for i in 0..state.amphipods.len() {
        if is_amphipod_in_final_state(&state.amphipods[i], &env, &state) {
            state.amphipods[i].done = true;
        }
    }
    let scenario = [
        ([9, 2], [11, 1], 3, false),
        ([9, 3], [1, 1], 10, false),
        ([7, 2], [10, 1], 4, false),
        ([7, 3], [8, 1], 3, false),
        ([7, 4], [2, 1], 8, false),
        ([5, 2], [7, 4], 6, true),
        ([5, 3], [7, 3], 6, true),
        ([5, 4], [6, 1], 4, false),
        ([5, 5], [4, 1], 5, false),
        ([6, 1], [5, 5], 5, true),
        ([8, 1], [5, 4], 6, true),
        ([10, 1], [5, 3], 7, true),
        ([9, 4], [7, 2], 6, true),
        ([9, 5], [10, 1], 5, false),
        ([4, 1], [9, 5], 9, true),
    ];
    for (src, dst, length, s) in scenario {
        let i = state
            .amphipods
            .iter()
            .enumerate()
            .find(|(_, v)| v.position == src)
            .unwrap()
            .0;
        let kind = state.amphipods[i].kind;
        assert_eq!(
            find_shortest_path(kind, src, dst, &state, &env),
            Some(length),
            "{:?} {:?} {} {:?}",
            src,
            dst,
            length,
            s
        );
        move_amphipod(i, dst, &env, &mut state);
        assert_eq!(
            state.amphipods[i].done, s,
            "{:?} {:?} {} {:?}",
            src, dst, length, s
        );
    }
    // assert!(is_final(&state));
}

#[test]
fn find_min_energy_0_test() {
    let buffer = r#"#############
#...........#
###A#B#C#D###
  #A#B#C#D#
  #########
"#
    .as_bytes();
    let World { env, state } = parse_world(buffer);
    assert_eq!(find_min_energy(&env, state), 0);
}

#[test]
fn find_min_energy_1_test() {
    let buffer = r#"#############
#...........#
###B#A#C#D###
  #A#B#C#D#
  #########
"#
    .as_bytes();
    let World { env, state } = parse_world(buffer);
    assert_eq!(find_min_energy(&env, state), 46);
}

#[test]
fn find_min_energy_2_test() {
    let buffer = r#"#############
#...........#
###B#C#A#D###
  #A#B#C#D#
  #########
"#
    .as_bytes();
    let World { env, state } = parse_world(buffer);
    assert_eq!(find_min_energy(&env, state), 448);
}

#[test]
fn find_min_energy_3_test() {
    let buffer = r#"#############
#...........#
###B#C#D#A###
  #A#B#C#D#
  #########
"#
    .as_bytes();
    let World { env, state } = parse_world(buffer);
    assert_eq!(find_min_energy(&env, state), 4450);
}

#[test]
fn find_min_energy_4_test() {
    let buffer = r#"#############
#...........#
###A#B#C#D###
  #A#B#C#D#
  #A#B#C#D#
  #A#B#C#D#
  #########
"#
    .as_bytes();
    let World { env, state } = parse_world(buffer);
    assert_eq!(find_min_energy(&env, state), 0);
}

#[test]
fn find_min_energy_5_test() {
    let buffer = r#"#############
#...........#
###B#A#C#D###
  #A#B#C#D#
  #A#B#C#D#
  #A#B#C#D#
  #########
"#
    .as_bytes();
    let World { env, state } = parse_world(buffer);
    assert_eq!(find_min_energy(&env, state), 46);
}

#[test]
fn find_min_energy_6_test() {
    let buffer = r#"#############
#...........#
###B#C#A#D###
  #A#B#C#D#
  #A#B#C#D#
  #A#B#C#D#
  #########
"#
    .as_bytes();
    let World { env, state } = parse_world(buffer);
    assert_eq!(find_min_energy(&env, state), 448);
}

#[test]
fn find_min_energy_7_test() {
    let buffer = r#"#############
#...........#
###B#A#C#D###
  #B#A#C#D#
  #B#A#C#D#
  #A#B#C#D#
  #########
"#
    .as_bytes();
    let World { env, state } = parse_world(buffer);
    assert_eq!(find_min_energy(&env, state), 206);
}

#[test]
fn find_min_energy_8_test() {
    let buffer = r#"#############
#...........#
###B#A#C#D###
  #B#A#C#D#
  #B#A#C#D#
  #B#A#C#D#
  #########
"#
    .as_bytes();
    let World { env, state } = parse_world(buffer);
    assert_eq!(find_min_energy(&env, state), 322);
}

#[test]
fn find_min_energy_9_test() {
    let buffer = r#"#############
#...........#
###B#A#D#C###
  #B#A#C#D#
  #A#B#C#D#
  #A#B#C#D#
  #########
"#
    .as_bytes();
    let World { env, state } = parse_world(buffer);
    assert_eq!(find_min_energy(&env, state), 4714);
}

#[test]
fn find_min_energy_10_test() {
    let buffer = r#"#############
#...........#
###B#A#D#C###
  #B#A#D#C#
  #A#B#C#D#
  #A#B#C#D#
  #########
"#
    .as_bytes();
    let World { env, state } = parse_world(buffer);
    assert_eq!(find_min_energy(&env, state), 11514);
}

#[test]
fn find_min_energy_part_1_example_test() {
    let buffer = r#"#############
#...........#
###B#C#B#D###
  #A#D#C#A#
  #########
"#
    .as_bytes();
    let World { env, state } = parse_world(buffer);
    assert_eq!(find_min_energy(&env, state), 12521);
}

#[test]
fn find_min_energy_part_2_example_test() {
    let buffer = r#"#############
#...........#
###B#C#B#D###
  #D#C#B#A#
  #D#B#A#C#
  #A#D#C#A#
  #########
"#
    .as_bytes();
    let World { env, state } = parse_world(buffer);
    assert_eq!(find_min_energy(&env, state), 44169);
}

#[test]
fn example_test() {
    let buffer = r#"#############
#...........#
###B#C#B#D###
  #A#D#C#A#
  #########
"#
    .as_bytes();
    assert_eq!(relocate_amphipods(buffer), (12521, 44169));
}
