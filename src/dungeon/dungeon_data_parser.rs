use super::reverser_instruction::{InstructionType, ReverserInstruction};

/// Port of Kludwisz DungeonDataParser.
/// Parses a floor tile sequence into ReverserInstructions, then generates
/// all possible instruction lists (due to MUTABLE_SKIPs branching).
pub struct DungeonDataParser;

impl DungeonDataParser {
    /// Parse the floor sequence string and generate all possible instruction lists.
    /// Returns None if there are too many possibilities (>128).
    pub fn get_all_possibilities(sequence: &str) -> Option<Vec<Vec<ReverserInstruction>>> {
        // Build initial instruction list, merging consecutive unknowns
        let mut instructions: Vec<ReverserInstruction> = Vec::new();
        let mut last_char: Option<char> = None;

        for ch in sequence.chars() {
            if ch == '2' {
                // Air: doesn't produce a call but doesn't interrupt sequences
                continue;
            }

            if !instructions.is_empty()
                && (ch == '3' || ch == '4')
                && last_char == Some(ch)
            {
                // Merge consecutive unknowns
                let last = instructions.last_mut().unwrap();
                last.max_call_count += 1;
            } else {
                let index = ch.to_digit(10).unwrap() as u8;
                if let Some(instr) = ReverserInstruction::from_tile_index(index) {
                    instructions.push(instr);
                }
            }

            last_char = Some(ch);
        }

        // Remove trailing SKIP and MUTABLE_SKIP instructions
        while let Some(last) = instructions.last() {
            if last.instruction_type == InstructionType::Skip
                || last.instruction_type == InstructionType::MutableSkip
            {
                instructions.pop();
            } else {
                break;
            }
        }

        // Generate all possibilities by expanding MUTABLE_SKIPs
        let mut result: Vec<Vec<ReverserInstruction>> = Vec::new();
        let mut counter = 0;
        Self::generate_recursive(&instructions, &mut Vec::new(), 0, &mut result, &mut counter);

        if counter > 128 {
            return None;
        }

        Some(result)
    }

    fn generate_recursive(
        original: &[ReverserInstruction],
        current: &mut Vec<ReverserInstruction>,
        ix: usize,
        result: &mut Vec<Vec<ReverserInstruction>>,
        counter: &mut i32,
    ) {
        if *counter > 128 {
            return;
        }

        let mut idx = ix;
        while idx < original.len() {
            let instr = &original[idx];

            if instr.instruction_type == InstructionType::MutableSkip {
                // Branch for each possible call count
                for calls in instr.min_call_count..=instr.max_call_count {
                    let mut new_list = current.clone();
                    if calls != 0 {
                        new_list.push(ReverserInstruction::new(
                            InstructionType::Skip,
                            calls,
                            calls,
                        ));
                    }
                    if idx + 1 < original.len() {
                        Self::generate_recursive(original, &mut new_list, idx + 1, result, counter);
                    } else {
                        result.push(new_list);
                        *counter += 1;
                    }
                }
                return;
            } else {
                current.push(instr.clone());
                idx += 1;
                if idx >= original.len() {
                    result.push(current.clone());
                    *counter += 1;
                }
            }
        }
    }
}
