/// Type of reverser instruction, matching the Java ReverserInstruction.Type enum.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InstructionType {
    NextInt,
    FilteredSkip,
    Skip,
    MutableSkip,
}

/// A single instruction for the reverser.
#[derive(Clone, Debug)]
pub struct ReverserInstruction {
    pub instruction_type: InstructionType,
    pub min_call_count: i32,
    pub max_call_count: i32,
}

impl ReverserInstruction {
    pub fn new(instruction_type: InstructionType, min_calls: i32, max_calls: i32) -> Self {
        ReverserInstruction {
            instruction_type,
            min_call_count: min_calls,
            max_call_count: max_calls,
        }
    }

    pub fn single(instruction_type: InstructionType) -> Self {
        ReverserInstruction::new(instruction_type, 1, 1)
    }

    /// Convert a floor tile index to a ReverserInstruction.
    /// 0 = mossy -> FILTEREDSKIP
    /// 1 = cobble -> NEXTINT
    /// 2 = air -> None (skipped)
    /// 3 = unknown -> MUTABLE_SKIP (0 or 1 calls)
    /// 4 = unknown_solid -> SKIP
    pub fn from_tile_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(ReverserInstruction::single(InstructionType::FilteredSkip)),
            1 => Some(ReverserInstruction::single(InstructionType::NextInt)),
            3 => Some(ReverserInstruction::new(InstructionType::MutableSkip, 0, 1)),
            4 => Some(ReverserInstruction::single(InstructionType::Skip)),
            _ => None, // air (2) returns None
        }
    }
}
