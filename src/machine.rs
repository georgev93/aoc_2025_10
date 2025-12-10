use nalgebra::SVector;
use std::collections::VecDeque;

type LightPos = usize;
type Joltages = SVector<isize, MAX_JOLTAGES>;
type ButtonIdx = Vec<LightPos>;
type Button = SVector<isize, MAX_LIGHTS_PER_BUTTON>;

const MAX_LIGHTS: usize = 10;
const MAX_LIGHTS_PER_BUTTON: usize = MAX_LIGHTS;
const MAX_BUTTONS: usize = 13;
const MAX_JOLTAGES: usize = MAX_LIGHTS;
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
    buttons: Vec<ButtonIdx>,
    button_arrays: Vec<Button>,
    joltages: Joltages,
    button_presses: u64,
}

impl Machine {
    pub fn new(input: &str) -> Self {
        let mut buttons: Vec<Vec<LightPos>> = Vec::with_capacity(MAX_BUTTONS);
        let mut button_lights_vec: Vec<LightPos> = Vec::with_capacity(MAX_LIGHTS_PER_BUTTON);
        let mut lights: Vec<bool> = Vec::with_capacity(MAX_LIGHTS);
        let mut current_joltage: String = String::new();
        let mut joltages: Joltages = SVector::<isize, MAX_LIGHTS>::zeros();
        let mut joltage_idx: usize = 0;

        let mut char_iter = input.chars();

        // Lights
        for c in &mut char_iter {
            match c {
                '.' => lights.push(false),
                '#' => lights.push(true),
                ']' => break,
                _ => {}
            }
        }

        // Buttons
        for c in &mut char_iter {
            match c {
                '(' => button_lights_vec.clear(),
                ')' => buttons.push(button_lights_vec.clone()),
                '0'..='9' => button_lights_vec.push((c as u32 - '0' as u32) as usize),
                '{' => break,
                _ => {}
            }
        }

        // Joltage
        for c in &mut char_iter {
            match c {
                '0'..='9' => current_joltage.push(c),
                ',' | '}' => {
                    joltages[joltage_idx] =
                        current_joltage.parse().expect("Failed to parse joltage");
                    joltage_idx += 1;
                    current_joltage.clear();
                }
                _ => {}
            }
        }

        buttons.sort_unstable_by_key(|b| b.len() as isize);
        let mut button_arrays: Vec<Button> = Vec::with_capacity(buttons.len());
        for button in &buttons {
            button_arrays.push(Self::generate_button_arr(button));
        }

        Self {
            lights_expected: vec![false; lights.len()],
            lights,
            buttons,
            button_arrays,
            joltages,
            button_presses: 0,
        }
    }

    fn generate_button_arr(idx_button: &ButtonIdx) -> Button {
        let mut ret_val = SVector::<isize, MAX_LIGHTS>::zeros();
        for idx in idx_button {
            ret_val[*idx] = 1;
        }
        ret_val
    }

    pub fn min_presses_to_get_joltage(&mut self) -> Option<u64> {
        let mut min_presses: Option<u64> = None;

        let mut idx_to_remove: Vec<usize> = Vec::new();
        for (idx, button) in &mut self.button_arrays.iter().enumerate() {
            // Check to ensure a button press won't make it go negative
            if self.joltages.iter().zip(button).any(|(j, b)| b > j) {
                idx_to_remove.push(idx);
            }
        }

        idx_to_remove.reverse();

        for idx in idx_to_remove {
            self.button_arrays.remove(idx);
        }

        if self.button_arrays.is_empty() {
            if self.joltages.abs().max() == 0 {
                return Some(0);
            } else {
                // println!("Reached the end but no cigar");
                return None;
            }
        }

        let mut max_button_presses = 0isize;
        let mut button = self.button_arrays[0];
        while !self.button_arrays.is_empty() {
            button = self.button_arrays.pop().unwrap();
            // println!("Buttons available: {}", self.button_arrays.len());

            // Check to ensure a button press won't make it go negative
            if self.joltages.iter().zip(&button).any(|(j, b)| b > j) {
                // println!("CAUGHT ONE");
                continue;
            }

            max_button_presses = button
                .component_mul(&self.joltages)
                .iter()
                .filter(|&&x| x != 0)
                .min()
                .copied()
                .unwrap_or(0);

            if max_button_presses != 0 {
                // println!("Max presses found: {}", max_button_presses);
                break;
            }
        }

        for presses in (0..=max_button_presses).rev() {
            let mut clone_machine = self.clone();
            if (clone_machine.joltages - button * presses).min() < 0 {
                dbg!(max_button_presses);
                dbg!(&clone_machine.joltages);
                dbg!(presses);
                dbg!(button);
                panic!();
            }
            clone_machine.joltages -= button * presses;
            if let Some(mut local_min) = clone_machine.min_presses_to_get_joltage() {
                // println!(
                //     "Found ONE! on obj with {} buttons left",
                //     self.button_arrays.len()
                // );
                local_min += presses as u64;
                if min_presses.is_none() {
                    // println!("Storing local min: {}", local_min);
                    min_presses = Some(local_min);
                }
                if local_min < min_presses.unwrap() {
                    min_presses = Some(local_min);
                }
            }
        }
        // if let some(presses) = min_presses {
        //     // println!("RETURNING {}", presses);
        // }
        //     println!("Returning none");
        // }
        min_presses
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

            // // SAFETY!!!
            // if pass_counter > 100000000 {
            //     panic!("TOO MANY POSSIBILITIES");
            // }
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
