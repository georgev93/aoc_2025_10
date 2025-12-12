use std::collections::VecDeque;

use good_lp::{Expression, ProblemVariables, Solution, SolverModel, highs, variable};

type LightPos = usize;
type Joltages = Vec<usize>;
type ButtonIdx = Vec<LightPos>;
type Button = Vec<usize>;

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
        let mut buttons: Vec<Vec<LightPos>> = Vec::new();
        let mut button_lights_vec: Vec<LightPos> = Vec::with_capacity(MAX_LIGHTS_PER_BUTTON);
        let mut lights: Vec<bool> = Vec::with_capacity(MAX_LIGHTS);
        let mut current_joltage: String = String::new();

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

        let mut joltage_idx: usize = 0;
        let mut joltages: Joltages = vec![0; lights.len()];

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
            button_arrays.push(Self::generate_button_arr(button, lights.len()));
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

    fn generate_button_arr(idx_button: &ButtonIdx, size: usize) -> Button {
        let mut ret_val = vec![0; size];
        for idx in idx_button {
            ret_val[*idx] = 1;
        }
        ret_val
    }

    // Praise be to wilkotom: https://github.com/wilkotom/AdventOfCode/blob/main/rust/2025/day10/src/main.rs
    pub fn min_presses_to_get_joltage_good_lp(&self) -> u64 {
        // Define variables to tweak
        let mut vars = ProblemVariables::new();
        let mut button_presses = Vec::new();
        for _ in 0..self.buttons.len() {
            let variable = vars.add(variable().min(0).integer());
            button_presses.push(variable);
        }

        // Define problem to solve (and select solver; highs in this case)
        let mut problem = vars
            .minimise(button_presses.iter().sum::<Expression>())
            .using(highs);

        // Create a vector where each element is a expression with capacity for every button, then
        // the vector is joltages long. So if you have 5 buttons for 10 joltages, this creates a 10
        // element array where each element is an expression with room for 5 buttons per expression
        let mut expressions =
            vec![Expression::with_capacity(self.buttons.len()); self.joltages.len()];

        // Each expression in the above gets a variable for each button push. This is relying on
        // the self.buttons array containing the correct indexes for the indexing in each
        // expression. The result of this should be each expression vector being linked to the
        // correct combination of variables, and each element in the overall vector corresponding
        // to a given joltage. Row is a joltage, column is a button (variable)
        for button_pos in 0..self.buttons.len() {
            for &joltage_pos in self.buttons[button_pos].iter() {
                expressions[joltage_pos] += button_presses[button_pos];
            }
        }

        // For each expression row (joltage row) and target joltage, add a constraint defining that
        // the expression should match the joltage
        for (e, &j) in expressions.into_iter().zip(&self.joltages) {
            problem.add_constraint(e.eq(j as f64));
        }

        // SOLVE!
        let solution = problem.solve().unwrap();

        // Use each variable to pass into the solver's `solve` method, which returns an f64. Sum it
        // up and cast it!
        button_presses
            .iter()
            .map(|v| solution.value(*v))
            .sum::<f64>() as u64
    }

    // Praise be to wilkotom: https://github.com/wilkotom/AdventOfCode/blob/main/rust/2025/day10/src/main.rs
    pub fn min_presses_to_get_lights(&self) -> u64 {
        // ========== SECTION 1: Add Variables ==========
        // Define variables to tweak
        let mut vars = ProblemVariables::new();

        // Dependent variable 1, button pushes
        let mut button_presses = Vec::new();
        for _ in 0..self.buttons.len() {
            let pushes = vars.add(variable().min(0).integer());
            button_presses.push(pushes);
        }

        // Dependent variable 2, pairs of button pushes (auxiliary variable we add to turn modulo
        // into linear math
        let mut pairs_vars = Vec::with_capacity(self.lights.len());
        for _ in 0..self.lights.len() {
            let pairs = vars.add(variable().min(0).integer());
            pairs_vars.push(pairs);
        }

        // ========== SECTION 2: Define Expressions ==========

        // Create a vector where each element is a expression with capacity for every button, then
        // the vector is joltages long. So if you have 5 buttons for 10 joltages, this creates a 10
        // element array where each element is an expression with room for 5 buttons per expression
        let mut expressions =
            vec![Expression::with_capacity(self.buttons.len()); self.lights.len()];

        // Each expression in the above gets a variable for each button push. This is relying on
        // the self.buttons array containing the correct indexes for the indexing in each
        // expression. The result of this should be each expression vector being linked to the
        // correct combination of variables, and each element in the overall vector corresponding
        // to a given joltage. Row is a joltage, column is a button (variable)
        for (button_pos, button) in self.buttons.iter().enumerate() {
            for &light_pos in button {
                expressions[light_pos] += button_presses[button_pos];
            }
        }

        // ========== SECTION 3: Define Problem ==========

        // Define problem to solve (and select solver; highs in this case)
        let mut problem = vars
            .minimise(button_presses.iter().sum::<Expression>())
            .using(highs);

        // ========== SECTION 4: Define Constraints ==========
        for (idx, e) in expressions.into_iter().enumerate() {
            let desired_parity = if self.lights[idx] { 1.0 } else { 0.0 };
            let pairs = pairs_vars[idx];
            problem.add_constraint(e.eq((2.0 * pairs) + desired_parity));
        }

        // SOLVE!
        let solution = problem.solve().unwrap();

        // Use each variable to pass into the solver's `solve` method, which returns an f64. Sum it
        // up and cast it!
        button_presses
            .iter()
            .map(|v| solution.value(*v))
            .sum::<f64>() as u64
    }

    // pub fn min_presses_to_get_joltage(&mut self) -> Option<u64> {
    //     let mut min_presses: Option<u64> = None;
    //
    //     let mut idx_to_remove: Vec<usize> = Vec::new();
    //     for (idx, button) in &mut self.button_arrays.iter().enumerate() {
    //         // Check to ensure a button press won't make it go negative
    //         if self.joltages.iter().zip(button).any(|(j, b)| b > j) {
    //             idx_to_remove.push(idx);
    //         }
    //     }
    //
    //     idx_to_remove.reverse();
    //
    //     for idx in idx_to_remove {
    //         self.button_arrays.remove(idx);
    //     }
    //
    //     if self.button_arrays.is_empty() {
    //         if self.joltages.abs().max() == 0 {
    //             return Some(0);
    //         } else {
    //             // println!("Reached the end but no cigar");
    //             return None;
    //         }
    //     }
    //
    //     let mut max_button_presses = 0isize;
    //     let mut button = self.button_arrays[0];
    //     while !self.button_arrays.is_empty() {
    //         button = self.button_arrays.pop().unwrap();
    //         // println!("Buttons available: {}", self.button_arrays.len());
    //
    //         // Check to ensure a button press won't make it go negative
    //         if self.joltages.iter().zip(&button).any(|(j, b)| b > j) {
    //             // println!("CAUGHT ONE");
    //             continue;
    //         }
    //
    //         max_button_presses = button
    //             .component_mul(&self.joltages)
    //             .iter()
    //             .filter(|&&x| x != 0)
    //             .min()
    //             .copied()
    //             .unwrap_or(0);
    //
    //         if max_button_presses != 0 {
    //             // println!("Max presses found: {}", max_button_presses);
    //             break;
    //         }
    //     }
    //
    //     for presses in (0..=max_button_presses).rev() {
    //         let mut clone_machine = self.clone();
    //         if (clone_machine.joltages - button * presses).min() < 0 {
    //             dbg!(max_button_presses);
    //             dbg!(&clone_machine.joltages);
    //             dbg!(presses);
    //             dbg!(button);
    //             panic!();
    //         }
    //         clone_machine.joltages -= button * presses;
    //         if let Some(mut local_min) = clone_machine.min_presses_to_get_joltage() {
    //             // println!(
    //             //     "Found ONE! on obj with {} buttons left",
    //             //     self.button_arrays.len()
    //             // );
    //             local_min += presses as u64;
    //             if min_presses.is_none() {
    //                 // println!("Storing local min: {}", local_min);
    //                 min_presses = Some(local_min);
    //             }
    //             if local_min < min_presses.unwrap() {
    //                 min_presses = Some(local_min);
    //             }
    //         }
    //     }
    //     // if let some(presses) = min_presses {
    //     //     // println!("RETURNING {}", presses);
    //     // }
    //     //     println!("Returning none");
    //     // }
    //     min_presses
    // }

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
