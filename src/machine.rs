type LightPos = usize;
type Joltage = u32;
type Button = Vec<LightPos>;
type ButtonGroup = Vec<Button>;

const MAX_LIGHTS: usize = 20;
const MAX_LIGHTS_PER_BUTTON: usize = 20;
const MAX_BUTTONS: usize = 20;
const MAX_JOLTAGES: usize = 20;
const MAX_MACHINES: usize = 200;

#[derive(Debug)]
pub struct MachineShop {
    machines: Vec<Machine>,
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

#[derive(Debug)]
pub struct Machine {
    lights: Vec<bool>,
    lights_expected: Vec<bool>,
    buttons: Vec<ButtonGroup>,
    joltage: Vec<Joltage>,
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

        buttons.sort_unstable_by_key(|b| -(b.len() as isize));

        let mut grouped_buttons: Vec<ButtonGroup> = Vec::with_capacity(MAX_BUTTONS);

        let mut last_size = buttons[0].len();
        let mut current_group: ButtonGroup = vec![];

        for mut button in buttons {
            let current_button_size = button.len();
            if current_button_size == last_size {
                current_group.push(std::mem::take(&mut button));
            } else {
                grouped_buttons.push(std::mem::take(&mut current_group));
                current_group = vec![std::mem::take(&mut button)];
                last_size = current_button_size;
            }
        }
        grouped_buttons.push(current_group);

        Self {
            lights_expected: vec![false; lights.len()],
            lights,
            buttons: grouped_buttons,
            joltage: vec![],
        }
    }

    fn press_button(&self, button: &Button, input_light_pattern: &[bool]) -> Vec<bool> {
        let mut light_pattern = input_light_pattern.to_vec();

        for pos in button {
            light_pattern[*pos] = !light_pattern[*pos];
        }

        light_pattern
    }

    fn score_lights(&self, possible_lights: &[bool]) -> usize {
        let mut score: usize = 0;

        possible_lights
            .iter()
            .zip(&self.lights_expected)
            .for_each(|(a, b)| {
                if *a == *b {
                    score += 1;
                }
            });

        score
    }
}
