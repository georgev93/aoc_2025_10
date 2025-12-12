pub mod file_parser;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::thread;

use rayon::prelude::*;

use crate::file_parser::FileParser;

mod machine;
use crate::machine::{Machine, MachineShop};

pub fn solve_pt1(input_file_text: &str) -> u64 {
    let my_machine_shop = MachineShop::new(input_file_text);

    my_machine_shop
        .machines
        .into_par_iter()
        .map(|machine| machine.min_presses_to_get_lights())
        .sum()
}

pub fn solve_pt2(input_file_text: &str) -> u64 {
    let my_machine_shop = MachineShop::new(input_file_text);

    my_machine_shop
        .machines
        .into_par_iter()
        .map(|machine| machine.min_presses_to_get_joltage_good_lp())
        .sum()
}

pub fn solve(input_file_text: &str) -> (u64, u64) {
    let my_machine_shop = MachineShop::new(input_file_text);

    let machine_vec_ref = &my_machine_shop.machines;

    let pt1 = machine_vec_ref
        .into_par_iter()
        .map(|machine| machine.min_presses_to_get_lights())
        .sum();
    let pt2 = machine_vec_ref
        .into_par_iter()
        .map(|machine| machine.min_presses_to_get_joltage_good_lp())
        .sum();
    (pt1, pt2)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_PT1: u64 = 7;
    const EXAMPLE_PT2: u64 = 33;
    const ACTUAL_PT1: u64 = 390;
    const ACTUAL_PT2: u64 = 14677;

    // #[test]
    // fn example() {
    //     let my_file = FileParser::new("data/example.txt");
    //     let (part_1, part_2) = solve(my_file.get_str());
    //     assert_eq!(part_1, EXAMPLE_PT1);
    //     assert_eq!(part_2, EXAMPLE_PT2);
    // }

    #[test]
    fn example_pts() {
        let my_file = FileParser::new("data/example.txt");
        assert_eq!(solve_pt1(my_file.get_str()), EXAMPLE_PT1);
        assert_eq!(solve_pt2(my_file.get_str()), EXAMPLE_PT2);
    }

    // #[test]
    // fn actual() {
    //     let my_file = FileParser::new("data/input.txt");
    //     let (part_1, part_2) = solve(my_file.get_str());
    //     assert_eq!(part_1, ACTUAL_PT1);
    //     assert_eq!(part_2, ACTUAL_PT2);
    // }
    //
    #[test]
    fn actual_pts() {
        let my_file = FileParser::new("data/input.txt");
        assert_eq!(solve_pt1(my_file.get_str()), ACTUAL_PT1);
        assert_eq!(solve_pt2(my_file.get_str()), ACTUAL_PT2);
    }
}
