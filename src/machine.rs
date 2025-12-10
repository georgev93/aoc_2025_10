use std::collections::VecDeque;

type LightPos = usize;
type Joltage = u32;
type Button = Vec<LightPos>;

const MAX_LIGHTS: usize = 20;
const MAX_LIGHTS_PER_BUTTON: usize = 20;
const MAX_BUTTONS: usize = 20;
const MAX_JOLTAGES: usize = 20;
const MAX_MACHINES: usize = 200;

enum Possibilites {
    Machines(Vec<Machine>),
    Success,
}

#[derive(Debug)]
pub struct MachineShop {
    pub machines: Vec<Machine>,
}

impl MachineShop {
    pub fn new(input: &str) -> Self {
        let mut machines: Vec<Machine> = Vec::with_capacity(MAX_MACHINES);
        for line in input.lines() {
            machines.push(Machine::new(line));
        }

        Self { machines }
    }
}

#[derive(Debug, Clone)]
pub struct Machine {
    lights: Vec<bool>,
    lights_expected: Vec<bool>,
    buttons: Vec<Button>,
    joltage: Vec<Joltage>,
    button_presses: u64,
}

impl Machine {
    pub fn new(input: &str) -> Self {
        let mut buttons: Vec<Vec<LightPos>> = Vec::with_capacity(MAX_BUTTONS);
        let mut button_lights_vec: Vec<LightPos> = Vec::with_capacity(MAX_LIGHTS_PER_BUTTON);
        let mut lights: Vec<bool> = Vec::with_capacity(MAX_LIGHTS);

        for c in input.chars() {
            match c {
                '.' => lights.push(false),
                '#' => lights.push(true),
                '(' => button_lights_vec.clear(),
                ')' => buttons.push(button_lights_vec.clone()),
                '0'..='9' => button_lights_vec.push((c as u32 - '0' as u32) as usize),
                _ => {}
            }
        }

        buttons.sort_unstable_by_key(|b| b.len() as isize);

        // lights.reverse();

        Self {
            lights_expected: vec![false; lights.len()],
            lights,
            buttons,
            joltage: vec![],
            button_presses: 0,
        }
    }

    pub fn min_presses_to_turn_off(&mut self) -> u64 {
        let mut possibilities: VecDeque<Self> = VecDeque::new();
        let mut pass_counter = 0u64;
        if let Some(successful_button_presses) = self.find_possibilites(&mut possibilities) {
            return successful_button_presses;
        }

        while !&possibilities.is_empty() {
            // dbg!(&possibilities.len());
            let mut child_possibility = possibilities.pop_front().unwrap();
            pass_counter += 1;
            // dbg!(poss_counter);

            if let Some(successful_button_presses) =
                child_possibility.find_possibilites(&mut possibilities)
            {
                return successful_button_presses;
            }

            // SAFETY!!!
            if pass_counter > 100000000 {
                panic!("TOO MANY POSSIBILITIES");
            }
        }
        dbg!(pass_counter);
        panic!("Not possible!!!")
    }

    fn find_possibilites(&mut self, possibilities: &mut VecDeque<Self>) -> Option<u64> {
        for button_pos in (0..self.buttons.len()).rev() {
            let mut possible_machine = self.clone();
            possible_machine.press_button(button_pos);

            if possible_machine.lights_are_correct() {
                return Some(possible_machine.button_presses);
            } else if !possible_machine.buttons.is_empty() {
                possibilities.push_back(possible_machine);
            }
        }
        None
    }

    fn press_button(&mut self, button_idx: usize) {
        for pos in &self.buttons[button_idx] {
            self.lights[*pos] = !self.lights[*pos];
        }
        // dbg!(self.button_presses);
        self.buttons.remove(button_idx);
        self.button_presses += 1;
    }

    fn lights_are_correct(&self) -> bool {
        // dbg!(&self.lights);
        self.lights
            .iter()
            .zip(&self.lights_expected)
            .map(|(a, b)| *a == *b)
            .fold(true, |acc, x| acc & x)
    }
}
