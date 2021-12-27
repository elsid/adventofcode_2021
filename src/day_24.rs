use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{BufRead, Read};
use std::str::FromStr;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 && args[1] == "execute" {
        let program = compile_program(std::io::stdin().lock()).unwrap();
        let mut alu = Alu::default();
        println!(
            "{:?}",
            execute_program(args[2].as_bytes(), &program, &mut alu)
        );
        println!("{:?}", alu);
    } else if args.len() >= 2 && args[1] == "flow" {
        let program = compile_program(std::io::stdin().lock()).unwrap();
        let data_flow = build_data_flow(&program);
        let mut buffer = Vec::new();
        dot::render(&data_flow, &mut buffer).unwrap();
        println!("{}", String::from_utf8(buffer).unwrap());
    } else if args.len() >= 2 && args[1] == "optimize" {
        let program = compile_program(std::io::stdin().lock()).unwrap();
        let mut data_flow = build_data_flow(&program);
        propagate_constants(&mut data_flow);
        let optimized = generate_program(&data_flow);
        print!("{}", program_to_string(&optimized));
    } else {
        println!("{:?}", find_the_meaning_of_monad(std::io::stdin().lock()));
    }
}

fn find_the_meaning_of_monad(buffer: impl BufRead) -> (u64, u64) {
    let program = compile_program(buffer).unwrap();
    let mut start = 0;
    let mut block = 0;
    let mut stack = Vec::new();
    let mut min_number = [b'0'; 14];
    let mut max_number = [b'0'; 14];
    while start < program.len() {
        let next_inp = program
            .iter()
            .enumerate()
            .skip(start + 1)
            .find(|(_, v)| matches!(v, Instruction::Inp(..)))
            .map(|(i, _)| i)
            .unwrap_or(program.len());
        let block_program = &program[start..next_inp];
        if let Instruction::DivConst(Variable::Z, z_divisor) = &block_program[4] {
            match z_divisor {
                1 => {
                    let y_term =
                        if let Instruction::AddConst(Variable::Y, y_term) = &block_program[15] {
                            *y_term
                        } else {
                            unreachable!()
                        };
                    stack.push((block, y_term));
                }
                26 => {
                    let (prev_block, y_term) = stack.pop().unwrap();
                    let diff = if let Instruction::AddConst(Variable::X, x_term) = &block_program[5]
                    {
                        y_term + *x_term
                    } else {
                        unreachable!();
                    };
                    let (i, j, shift) = if diff < 0 {
                        (prev_block, block, -diff)
                    } else {
                        (block, prev_block, diff)
                    };
                    max_number[i] = b'9';
                    max_number[j] = b'9' - shift as u8;
                    min_number[i] = b'1' + shift as u8;
                    min_number[j] = b'1';
                }
                _ => unreachable!(),
            }
        }
        start = next_inp;
        block += 1;
    }
    (
        u64::from_str(String::from_utf8_lossy(&max_number).as_ref()).unwrap(),
        u64::from_str(String::from_utf8_lossy(&min_number).as_ref()).unwrap(),
    )
}

#[derive(Copy, Clone, Debug)]
enum OperationType {
    Add,
    Mul,
    Div,
    Mod,
    Eql,
}

#[derive(Clone, Debug)]
enum DataFlowNode {
    ConstInput(Variable, Value),
    Input(Variable, usize),
    RightConstOperation(OperationType, Variable, Value),
    LeftConstOperation(OperationType, Value, Variable, Variable),
    Operation(OperationType, Variable, Variable),
    Out(Variable),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum Side {
    Left,
    Right,
}

#[derive(Clone)]
struct DataFlowEdge {
    src: usize,
    dst: usize,
    side: Side,
}

struct DataFlow {
    nodes: Vec<DataFlowNode>,
    edges: Vec<DataFlowEdge>,
}

impl<'a> dot::GraphWalk<'a, (usize, DataFlowNode), DataFlowEdge> for DataFlow {
    fn nodes(&self) -> dot::Nodes<(usize, DataFlowNode)> {
        self.nodes.iter().cloned().enumerate().collect()
    }

    fn edges(&self) -> dot::Edges<DataFlowEdge> {
        self.edges.iter().cloned().collect()
    }

    fn source(&self, edge: &DataFlowEdge) -> (usize, DataFlowNode) {
        (edge.src, self.nodes[edge.src].clone())
    }

    fn target(&self, edge: &DataFlowEdge) -> (usize, DataFlowNode) {
        (edge.dst, self.nodes[edge.dst].clone())
    }
}

impl<'a> dot::Labeller<'a, (usize, DataFlowNode), DataFlowEdge> for DataFlow {
    fn graph_id(&self) -> dot::Id {
        dot::Id::new("data_flow").unwrap()
    }

    fn node_id(&self, node: &(usize, DataFlowNode)) -> dot::Id {
        dot::Id::new(format!("node_{}", node.0)).unwrap()
    }

    fn node_shape(&self, (_, node): &(usize, DataFlowNode)) -> Option<dot::LabelText> {
        match node {
            DataFlowNode::Input(..) | DataFlowNode::Out(..) => {
                Some(dot::LabelText::LabelStr("box".into()))
            }
            _ => None,
        }
    }

    fn node_label(&self, node: &(usize, DataFlowNode)) -> dot::LabelText {
        let label = match node.1 {
            DataFlowNode::ConstInput(variable, value) => {
                format!("set {} {}", variable_to_char(variable), value)
            }
            DataFlowNode::Input(variable, index) => {
                format!("inp {} : {}", variable_to_char(variable), index)
            }
            DataFlowNode::RightConstOperation(op, left, right) => format!(
                "{} {} {}",
                operation_to_string(op),
                variable_to_char(left),
                right
            ),
            DataFlowNode::LeftConstOperation(op, value, left, right) => format!(
                "{} {} {} ; {} = {}",
                operation_to_string(op),
                variable_to_char(left),
                variable_to_char(right),
                variable_to_char(left),
                value
            ),
            DataFlowNode::Operation(op, left, right) => format!(
                "{} {} {}",
                operation_to_string(op),
                variable_to_char(left),
                variable_to_char(right)
            ),
            DataFlowNode::Out(v) => format!("out {}", variable_to_char(v)),
        };
        dot::LabelText::LabelStr(format!("#{} {}", node.0, label).into())
    }

    fn edge_label(&self, edge: &DataFlowEdge) -> dot::LabelText {
        dot::LabelText::LabelStr(format!("{:?}", edge.side).into())
    }
}

impl<'a> dot::GraphWalk<'a, (usize, DataFlowNode), DataFlowEdge>
    for (&'a DataFlow, &'a HashSet<usize>)
{
    fn nodes(&self) -> dot::Nodes<(usize, DataFlowNode)> {
        self.0.nodes()
    }

    fn edges(&self) -> dot::Edges<DataFlowEdge> {
        self.0.edges()
    }

    fn source(&self, edge: &DataFlowEdge) -> (usize, DataFlowNode) {
        self.0.source(edge)
    }

    fn target(&self, edge: &DataFlowEdge) -> (usize, DataFlowNode) {
        self.0.target(edge)
    }
}

impl<'a> dot::Labeller<'a, (usize, DataFlowNode), DataFlowEdge>
    for (&'a DataFlow, &'a HashSet<usize>)
{
    fn graph_id(&self) -> dot::Id {
        self.0.graph_id()
    }

    fn node_id(&self, node: &(usize, DataFlowNode)) -> dot::Id {
        self.0.node_id(node)
    }

    fn node_shape(&self, node: &(usize, DataFlowNode)) -> Option<dot::LabelText> {
        self.0.node_shape(node)
    }

    fn node_label(&self, node: &(usize, DataFlowNode)) -> dot::LabelText {
        self.0.node_label(node)
    }

    fn edge_label(&self, edge: &DataFlowEdge) -> dot::LabelText {
        self.0.edge_label(edge)
    }

    fn node_style(&self, _: &(usize, DataFlowNode)) -> dot::Style {
        dot::Style::Filled
    }

    fn node_color(&self, (index, _): &(usize, DataFlowNode)) -> Option<dot::LabelText> {
        if self.1.contains(index) {
            Some(dot::LabelText::LabelStr("green".into()))
        } else {
            Some(dot::LabelText::LabelStr("red".into()))
        }
    }
}

fn build_data_flow(program: &[Instruction]) -> DataFlow {
    let mut variables = [0; 4];
    let mut nodes = Vec::new();
    for (i, node) in variables.iter_mut().enumerate() {
        *node = nodes.len();
        nodes.push(DataFlowNode::ConstInput(usize_to_variable(i), 0));
    }
    let mut edges = Vec::new();
    let mut input_index = 0;
    for instruction in program.iter() {
        match instruction {
            Instruction::Inp(variable) => {
                variables[*variable as usize] = nodes.len();
                nodes.push(DataFlowNode::Input(*variable, input_index));
                input_index += 1;
            }
            other => {
                let mut add_operation_node =
                    |operation: OperationType, left: Variable, right: Operand| match right {
                        Operand::Value(right_value) => {
                            let left_index = variables[left as usize];
                            variables[left as usize] = nodes.len();
                            nodes.push(DataFlowNode::RightConstOperation(
                                operation,
                                left,
                                right_value,
                            ));
                            edges.push(DataFlowEdge {
                                src: left_index,
                                dst: nodes.len() - 1,
                                side: Side::Left,
                            });
                        }
                        Operand::Variable(right_variable) => {
                            let left_index = variables[left as usize];
                            let right_index = variables[right_variable as usize];
                            edges.push(DataFlowEdge {
                                src: left_index,
                                dst: nodes.len(),
                                side: Side::Left,
                            });
                            edges.push(DataFlowEdge {
                                src: right_index,
                                dst: nodes.len(),
                                side: Side::Right,
                            });
                            variables[left as usize] = nodes.len();
                            nodes.push(DataFlowNode::Operation(operation, left, right_variable));
                        }
                    };
                match other {
                    Instruction::Inp(..) => unreachable!(),
                    Instruction::Add(left, right) => {
                        add_operation_node(OperationType::Add, *left, Operand::Variable(*right))
                    }
                    Instruction::Mul(left, right) => {
                        add_operation_node(OperationType::Mul, *left, Operand::Variable(*right))
                    }
                    Instruction::Div(left, right) => {
                        add_operation_node(OperationType::Div, *left, Operand::Variable(*right))
                    }
                    Instruction::Mod(left, right) => {
                        add_operation_node(OperationType::Mod, *left, Operand::Variable(*right))
                    }
                    Instruction::Eql(left, right) => {
                        add_operation_node(OperationType::Eql, *left, Operand::Variable(*right))
                    }
                    Instruction::AddConst(left, right) => {
                        add_operation_node(OperationType::Add, *left, Operand::Value(*right))
                    }
                    Instruction::MulConst(left, right) => {
                        add_operation_node(OperationType::Mul, *left, Operand::Value(*right))
                    }
                    Instruction::DivConst(left, right) => {
                        add_operation_node(OperationType::Div, *left, Operand::Value(*right))
                    }
                    Instruction::ModConst(left, right) => {
                        add_operation_node(OperationType::Mod, *left, Operand::Value(*right))
                    }
                    Instruction::EqlConst(left, right) => {
                        add_operation_node(OperationType::Eql, *left, Operand::Value(*right))
                    }
                }
            }
        }
    }
    let mut output: Vec<(usize, Variable)> = variables
        .iter()
        .enumerate()
        .map(|(i, v)| (*v, usize_to_variable(i)))
        .collect();
    output.sort_by_key(|(v, _)| *v);
    for (node, variable) in output.iter() {
        nodes.push(DataFlowNode::Out(*variable));
        edges.push(DataFlowEdge {
            src: *node,
            dst: nodes.len() - 1,
            side: Side::Left,
        });
    }
    DataFlow { nodes, edges }
}

fn find_reachable_nodes(src: usize, data_flow: &DataFlow) -> HashSet<usize> {
    let mut nodes = vec![src];
    let mut reachable = HashSet::new();
    let mut edges = HashMap::new();
    for edge in data_flow.edges.iter() {
        edges
            .entry(edge.dst)
            .or_insert_with(Vec::new)
            .push(edge.src);
    }
    reachable.insert(src);
    while let Some(node) = nodes.pop() {
        if let Some(neighbours) = edges.get(&node) {
            for neighbour in neighbours.iter() {
                if reachable.insert(*neighbour) {
                    nodes.push(*neighbour);
                }
            }
        }
    }
    reachable
}

fn propagate_constants(data_flow: &mut DataFlow) {
    let mut nodes: Vec<usize> = data_flow
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, v)| matches!(v, DataFlowNode::ConstInput(..)))
        .map(|(i, _)| i)
        .collect();
    let mut edges = HashMap::new();
    for edge in data_flow.edges.iter() {
        edges
            .entry(edge.src)
            .or_insert_with(Vec::new)
            .push((edge.dst, edge.side));
    }
    while let Some(node) = nodes.pop() {
        let node_value = match &data_flow.nodes[node] {
            DataFlowNode::ConstInput(.., v) => *v,
            _ => unreachable!(),
        };
        if let Some(neighbours) = edges.get(&node) {
            for (neighbour, side) in neighbours.iter() {
                match side {
                    Side::Left => match data_flow.nodes[*neighbour] {
                        DataFlowNode::RightConstOperation(op, left, right) => {
                            data_flow.nodes[*neighbour] = DataFlowNode::ConstInput(
                                left,
                                perform_operation(op, node_value, right),
                            );
                            nodes.push(*neighbour);
                        }
                        DataFlowNode::Operation(op, left, right) => {
                            data_flow.nodes[*neighbour] =
                                DataFlowNode::LeftConstOperation(op, node_value, left, right);
                        }
                        _ => (),
                    },
                    Side::Right => match data_flow.nodes[*neighbour] {
                        DataFlowNode::LeftConstOperation(op, value, left, ..) => {
                            data_flow.nodes[*neighbour] = DataFlowNode::ConstInput(
                                left,
                                perform_operation(op, value, node_value),
                            );
                            nodes.push(*neighbour);
                        }
                        DataFlowNode::Operation(op, left, ..) => {
                            data_flow.nodes[*neighbour] =
                                DataFlowNode::RightConstOperation(op, left, node_value);
                        }
                        _ => (),
                    },
                }
            }
        }
    }
    let nodes = &data_flow.nodes;
    data_flow
        .edges
        .retain(|edge| !matches!(&nodes[edge.dst], DataFlowNode::ConstInput(..)));
}

fn perform_operation(operation: OperationType, left: Value, right: Value) -> Value {
    match operation {
        OperationType::Add => left + right,
        OperationType::Mul => left * right,
        OperationType::Div => left / right,
        OperationType::Mod => left % right,
        OperationType::Eql => (left == right) as Value,
    }
}

fn generate_program(data_flow: &DataFlow) -> Vec<Instruction> {
    let mut result = Vec::new();
    let mut edges = HashMap::new();
    for edge in data_flow.edges.iter() {
        edges
            .entry(edge.dst)
            .or_insert_with(Vec::new)
            .push(edge.src);
    }
    let mut nodes = VecDeque::new();
    let mut reachable: HashSet<usize> = HashSet::new();
    let mut zero_variables = [true; 4];
    for (i, node) in data_flow.nodes.iter().enumerate().rev() {
        if matches!(node, DataFlowNode::Out(..)) {
            nodes.push_back(i);
            reachable.extend(find_reachable_nodes(i, data_flow).iter());
        }
    }
    for (i, node) in data_flow.nodes.iter().enumerate() {
        if !reachable.contains(&i) {
            continue;
        }
        match node {
            DataFlowNode::ConstInput(variable, value) => {
                if zero_variables[*variable as usize] {
                    zero_variables[*variable as usize] = false;
                } else {
                    result.push(Instruction::MulConst(*variable, 0));
                }
                if *value != 0 {
                    result.push(Instruction::AddConst(*variable, *value));
                }
            }
            DataFlowNode::Input(variable, ..) => result.push(Instruction::Inp(*variable)),
            DataFlowNode::RightConstOperation(op, left, right) => {
                let instruction = match op {
                    OperationType::Add => Instruction::AddConst(*left, *right),
                    OperationType::Mul => Instruction::MulConst(*left, *right),
                    OperationType::Div => Instruction::DivConst(*left, *right),
                    OperationType::Mod => Instruction::ModConst(*left, *right),
                    OperationType::Eql => Instruction::EqlConst(*left, *right),
                };
                result.push(instruction);
            }
            DataFlowNode::LeftConstOperation(op, .., left, right)
            | DataFlowNode::Operation(op, left, right) => {
                let instruction = match op {
                    OperationType::Add => Instruction::Add(*left, *right),
                    OperationType::Mul => Instruction::Mul(*left, *right),
                    OperationType::Div => Instruction::Div(*left, *right),
                    OperationType::Mod => Instruction::Mod(*left, *right),
                    OperationType::Eql => Instruction::Eql(*left, *right),
                };
                result.push(instruction);
            }
            DataFlowNode::Out(..) => (),
        }
    }
    result
}

fn usize_to_variable(v: usize) -> Variable {
    match v {
        0 => Variable::W,
        1 => Variable::X,
        2 => Variable::Y,
        3 => Variable::Z,
        v => panic!("Invalid variable: {}", v),
    }
}

fn operation_to_string(value: OperationType) -> &'static str {
    match value {
        OperationType::Add => "add",
        OperationType::Mul => "mul",
        OperationType::Div => "div",
        OperationType::Mod => "mod",
        OperationType::Eql => "eql",
    }
}

fn program_to_string(program: &[Instruction]) -> String {
    let mut result = String::new();
    for instruction in program.iter() {
        result = format!("{}{}\n", result, instruction_to_string(instruction))
    }
    result
}

fn instruction_to_string(instruction: &Instruction) -> String {
    match instruction {
        Instruction::Inp(variable) => {
            format!("inp {}", variable_to_char(*variable))
        }
        Instruction::Add(left, right) => {
            format!(
                "add {} {}",
                variable_to_char(*left),
                variable_to_char(*right)
            )
        }
        Instruction::Mul(left, right) => {
            format!(
                "mul {} {}",
                variable_to_char(*left),
                variable_to_char(*right)
            )
        }
        Instruction::Div(left, right) => {
            format!(
                "div {} {}",
                variable_to_char(*left),
                variable_to_char(*right)
            )
        }
        Instruction::Mod(left, right) => {
            format!(
                "mod {} {}",
                variable_to_char(*left),
                variable_to_char(*right)
            )
        }
        Instruction::Eql(left, right) => {
            format!(
                "eql {} {}",
                variable_to_char(*left),
                variable_to_char(*right)
            )
        }
        Instruction::AddConst(left, right) => {
            format!("add {} {}", variable_to_char(*left), right)
        }
        Instruction::MulConst(left, right) => {
            format!("mul {} {}", variable_to_char(*left), right)
        }
        Instruction::DivConst(left, right) => {
            format!("div {} {}", variable_to_char(*left), right)
        }
        Instruction::ModConst(left, right) => {
            format!("mod {} {}", variable_to_char(*left), right)
        }
        Instruction::EqlConst(left, right) => {
            format!("eql {} {}", variable_to_char(*left), right)
        }
    }
}

fn variable_to_char(variable: Variable) -> char {
    match variable {
        Variable::W => 'w',
        Variable::X => 'x',
        Variable::Y => 'y',
        Variable::Z => 'z',
    }
}

type Value = i64;

#[derive(Eq, PartialEq, Debug, Default)]
struct Alu {
    w: Value,
    x: Value,
    y: Value,
    z: Value,
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
enum Variable {
    W,
    X,
    Y,
    Z,
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
enum Operand {
    Variable(Variable),
    Value(Value),
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
enum Instruction {
    Inp(Variable),
    Add(Variable, Variable),
    Mul(Variable, Variable),
    Div(Variable, Variable),
    Mod(Variable, Variable),
    Eql(Variable, Variable),
    AddConst(Variable, Value),
    MulConst(Variable, Value),
    DivConst(Variable, Value),
    ModConst(Variable, Value),
    EqlConst(Variable, Value),
}

type Program = Vec<Instruction>;

#[derive(PartialEq, Debug)]
enum ExecutionError {
    Read(usize),
    DivByZero(usize),
    ModNegativeDividend(usize),
    ModNonPositiveModulus(usize),
}

fn execute_program(
    mut input: impl Read,
    program: &[Instruction],
    alu: &mut Alu,
) -> Result<(), ExecutionError> {
    for (i, instruction) in program.iter().enumerate() {
        match instruction {
            Instruction::Inp(variable) => {
                let mut buf = [0; 1];
                match input.read(&mut buf) {
                    Ok(v) => {
                        if v == 0 {
                            return Err(ExecutionError::Read(i));
                        }
                        alu.store(*variable, symbol_to_value(buf[0]));
                    }
                    Err(_) => return Err(ExecutionError::Read(i)),
                }
            }
            Instruction::Add(left, right) => {
                alu.store(*left, alu.load(*left) + alu.load(*right));
            }
            Instruction::Mul(left, right) => {
                alu.store(*left, alu.load(*left) * alu.load(*right));
            }
            Instruction::Div(left, right) => {
                alu.store(*left, checked_div(alu.load(*left), alu.load(*right), i)?);
            }
            Instruction::Mod(left, right) => {
                alu.store(*left, checked_mod(alu.load(*left), alu.load(*right), i)?);
            }
            Instruction::Eql(left, right) => {
                alu.store(*left, (alu.load(*left) == alu.load(*right)) as Value)
            }
            Instruction::AddConst(left, right) => {
                alu.store(*left, alu.load(*left) + *right);
            }
            Instruction::MulConst(left, right) => {
                alu.store(*left, alu.load(*left) * *right);
            }
            Instruction::DivConst(left, right) => {
                alu.store(*left, checked_div(alu.load(*left), *right, i)?);
            }
            Instruction::ModConst(left, right) => {
                alu.store(*left, checked_mod(alu.load(*left), *right, i)?);
            }
            Instruction::EqlConst(left, right) => {
                alu.store(*left, (alu.load(*left) == *right) as Value)
            }
        }
    }
    Ok(())
}

fn checked_div(dividend: Value, divisor: Value, i: usize) -> Result<Value, ExecutionError> {
    if divisor == 0 {
        return Err(ExecutionError::DivByZero(i));
    }
    Ok(dividend / divisor)
}

fn checked_mod(dividend: Value, modulus: Value, i: usize) -> Result<Value, ExecutionError> {
    if dividend < 0 {
        return Err(ExecutionError::ModNegativeDividend(i));
    }
    if modulus <= 0 {
        return Err(ExecutionError::ModNonPositiveModulus(i));
    }
    Ok(dividend % modulus)
}

fn symbol_to_value(symbol: u8) -> Value {
    (symbol - b'0') as Value
}

impl Alu {
    fn load(&self, variable: Variable) -> Value {
        match variable {
            Variable::W => self.w,
            Variable::X => self.x,
            Variable::Y => self.y,
            Variable::Z => self.z,
        }
    }

    fn store(&mut self, variable: Variable, value: Value) {
        match variable {
            Variable::W => self.w = value,
            Variable::X => self.x = value,
            Variable::Y => self.y = value,
            Variable::Z => self.z = value,
        }
    }
}

fn compile_program(buffer: impl BufRead) -> Result<Program, String> {
    let mut instructions = Vec::new();
    for (i, line) in buffer.lines().enumerate() {
        match line {
            Ok(v) => {
                if v.is_empty() {
                    continue;
                }
                match parse_instruction(&v) {
                    Ok(v) => instructions.push(v),
                    Err(e) => return Err(format!("syntax error at line {}: {}", i + 1, e)),
                }
            }
            Err(e) => return Err(format!("failed to read line {}: {}", i + 1, e)),
        }
    }
    Ok(instructions)
}

fn parse_instruction(text: &str) -> Result<Instruction, String> {
    let (command, operands) = if let Some(v) = text.split_once(' ') {
        v
    } else {
        return Err("invalid instruction format".to_string());
    };
    match command {
        "inp" => Ok(Instruction::Inp(parse_variable(operands)?)),
        name => {
            let (left, right) = parse_operands(operands)?;
            match right {
                Operand::Variable(variable) => make_in_out_instruction(name, left, variable),
                Operand::Value(value) => make_const_in_out_instruction(name, left, value),
            }
        }
    }
}

fn make_in_out_instruction(
    name: &str,
    left: Variable,
    right: Variable,
) -> Result<Instruction, String> {
    match name {
        "add" => Ok(Instruction::Add(left, right)),
        "mul" => Ok(Instruction::Mul(left, right)),
        "div" => Ok(Instruction::Div(left, right)),
        "mod" => Ok(Instruction::Mod(left, right)),
        "eql" => Ok(Instruction::Eql(left, right)),
        v => Err(format!("invalid instruction name: {}", v)),
    }
}

fn make_const_in_out_instruction(
    name: &str,
    left: Variable,
    right: Value,
) -> Result<Instruction, String> {
    match name {
        "add" => Ok(Instruction::AddConst(left, right)),
        "mul" => Ok(Instruction::MulConst(left, right)),
        "div" => Ok(Instruction::DivConst(left, right)),
        "mod" => Ok(Instruction::ModConst(left, right)),
        "eql" => Ok(Instruction::EqlConst(left, right)),
        v => Err(format!("invalid instruction name: {}", v)),
    }
}

fn parse_operands(text: &str) -> Result<(Variable, Operand), String> {
    if let Some((a, b)) = text.split_once(' ') {
        Ok((parse_variable(a)?, parse_operand(b)?))
    } else {
        Err(format!("invalid command format: {}", text))
    }
}

fn parse_operand(text: &str) -> Result<Operand, String> {
    match parse_variable(text) {
        Ok(v) => Ok(Operand::Variable(v)),
        Err(_) => match Value::from_str(text) {
            Ok(v) => Ok(Operand::Value(v)),
            Err(e) => Err(format!("invalid value \"{}\" format: {}", text, e)),
        },
    }
}

fn parse_variable(text: &str) -> Result<Variable, String> {
    match text {
        "w" => Ok(Variable::W),
        "x" => Ok(Variable::X),
        "y" => Ok(Variable::Y),
        "z" => Ok(Variable::Z),
        v => Err(format!("invalid variable name: {}", v)),
    }
}

#[test]
fn compile_program_test() {
    let code = r#"inp w
add x y
add z 1
mod w x
mod y -2
div z w
div x 3
eql y z
eql w -4
"#
    .as_bytes();
    assert_eq!(
        compile_program(code),
        Ok(vec![
            Instruction::Inp(Variable::W),
            Instruction::Add(Variable::X, Variable::Y),
            Instruction::AddConst(Variable::Z, 1),
            Instruction::Mod(Variable::W, Variable::X),
            Instruction::ModConst(Variable::Y, -2),
            Instruction::Div(Variable::Z, Variable::W),
            Instruction::DivConst(Variable::X, 3),
            Instruction::Eql(Variable::Y, Variable::Z),
            Instruction::EqlConst(Variable::W, -4),
        ])
    );
}

#[test]
fn execute_program_1_test() {
    let code = r#"inp x
mul x -1
"#
    .as_bytes();
    let mut alu = Alu::default();
    assert_eq!(
        execute_program("9".as_bytes(), &compile_program(code).unwrap(), &mut alu),
        Ok(())
    );
    assert_eq!(
        alu,
        Alu {
            w: 0,
            x: -9,
            y: 0,
            z: 0,
        }
    );
}

#[test]
fn execute_program_2_test() {
    let code = r#"inp z
inp x
mul z 3
eql z x
"#
    .as_bytes();
    let mut alu = Alu::default();
    assert_eq!(
        execute_program("39".as_bytes(), &compile_program(code).unwrap(), &mut alu),
        Ok(())
    );
    assert_eq!(
        alu,
        Alu {
            w: 0,
            x: 9,
            y: 0,
            z: 1,
        }
    );
}

#[test]
fn execute_program_3_test() {
    let code = r#"inp w
add z w
mod z 2
div w 2
add y w
mod y 2
div w 2
add x w
mod x 2
div w 2
mod w 2
"#
    .as_bytes();
    let mut alu = Alu::default();
    let program = compile_program(code).unwrap();
    assert_eq!(execute_program("9".as_bytes(), &program, &mut alu), Ok(()));
    assert_eq!(
        alu,
        Alu {
            w: 1,
            x: 0,
            y: 0,
            z: 1,
        }
    );
    alu = Alu::default();
    assert_eq!(execute_program("5".as_bytes(), &program, &mut alu), Ok(()));
    assert_eq!(
        alu,
        Alu {
            w: 0,
            x: 1,
            y: 0,
            z: 1,
        }
    );
}

#[test]
fn generate_program_1_test() {
    let code = r#"inp z
inp x
mul z 3
eql z x
"#
    .as_bytes();
    let program = compile_program(code).unwrap();
    let data_flow = build_data_flow(&program);
    assert_eq!(generate_program(&data_flow), program);
}
