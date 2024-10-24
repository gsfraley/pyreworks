pub mod raw;

use std::collections::HashMap;

use nusb::Interface;
use pyreworks_common::Color;
use raw::{
    detach_and_claim_interface,
    ClaimInterfaceError,
    SendCommandError
};
use tokio::time::{sleep, Duration};

const COMMAND_RUN_TIMES: u8 = 3;
const COMMAND_RETRY_TIMES: u8 = 3;
const COMMAND_RETRY_INTERVAL_MS: u16 = 30;

pub struct Driver {
    interface: Interface,
}

impl Driver {
    pub fn connect() -> Result<Driver, ClaimInterfaceError> {
        Ok(Driver {
            interface: detach_and_claim_interface()?,
        })
    }

    pub async fn run(&self, commands: &[Command]) -> Result<(), SendCommandError> {
        self.run_direct(&Command::compress(commands)).await
    }

    async fn run_direct(&self, commands: &[Command]) -> Result<(), SendCommandError> {
        for command in commands {
            let value = match command {
                Command::SetColorSolid(data) => {
                    let mut value = [0u8; 10];
                    value[0] = data.target.addr();
                    value[1] = command.mode_value();
                    value[2] = data.color.r;
                    value[3] = data.color.g;
                    value[4] = data.color.b;
                    value
                },
                Command::SetColorCycle(data) => {
                    let mut value = [0u8; 10];
                    value[0] = data.target.addr();
                    value[1] = command.mode_value();
                    value[2..7].copy_from_slice(&[0u8; 5]);
                    value[7] = (data.rate >> 8) as u8;
                    value[8] = data.rate as u8;
                    value[9] = data.brightness;
                    value
                },
                Command::SetColorBreathe(data) => {
                    let mut value = [0u8; 10];
                    value[0] = data.target.addr();
                    value[1] = command.mode_value();
                    value[2] = data.color.r;
                    value[3] = data.color.g;
                    value[4] = data.color.b;
                    value[5] = (data.rate >> 8) as u8;
                    value[6] = data.rate as u8;
                    value[7] = 0x00;
                    value[8] = data.brightness;
                    value[9] = 0x00;
                    value
                },
                Command::SetColorOff(data) => {
                    let mut value = [0u8; 10];
                    value[0] = data.target.addr();
                    value[1] = command.mode_value();
                    value[2] = 0x00;
                    value[3] = 0x00;
                    value[4] = 0x00;
                    value
                },
            };

            // Following block is meant to run `send_raw_command` multiple times with a retry mechanism.
            let mut raw_cmd_result;
            for _ in 0..COMMAND_RUN_TIMES {
                for _ in 0..COMMAND_RETRY_TIMES {
                    raw_cmd_result = raw::send_raw_command(&self.interface, &value).await;
                    
                    if raw_cmd_result.is_ok() {
                        break;
                    }

                    sleep(Duration::from_millis(COMMAND_RETRY_INTERVAL_MS.into())).await;
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum Target {
    LeftPrimary,
    LeftSecondary,
    RightPrimary,
    RightSecondary,
}

impl Target {
    pub fn all() -> [Target; 4] {
        [
            Target::LeftPrimary,
            Target::LeftSecondary,
            Target::RightPrimary,
            Target::RightSecondary,
        ]
    }

    pub fn lookup(name: &str) -> Box<[Target]> {
        // TODO: This could really use some regex.
        match name {
            "leftfront" | "left-front" | "left_front" | "leftprimary" | "left-primary" | "left_primary"
                => Box::new([Target::LeftPrimary]),
            "leftback" | "left-back" | "left_back" | "leftsecondary" | "left-secondary" | "left_secondary"
                => Box::new([Target::LeftSecondary]),
            "rightfront" | "right-front" | "right_front" | "rightprimary" | "right-primary" | "right_primary"
                => Box::new([Target::RightPrimary]),
            "rightback" | "right-back" | "right_back" | "rightsecondary" | "right-secondary" | "right_secondary"
                => Box::new([Target::RightSecondary]),
            "left" => Box::new([Target::LeftPrimary, Target::LeftSecondary]),
            "right" => Box::new([Target::RightPrimary, Target::RightSecondary]),
            "front" | "primary" => Box::new([Target::LeftPrimary, Target::RightPrimary]),
            "back" | "secondary" => Box::new([Target::LeftSecondary, Target::RightSecondary]),
            "all" => Box::new(Target::all()),
            _ => Box::new([]),
        }
    }
    
    fn addr(&self) -> u8 {
        match self {
            Target::LeftPrimary => 0x00,
            Target::LeftSecondary => 0x02,
            Target::RightPrimary => 0x01,
            Target::RightSecondary => 0x03,
        }
    }
}

#[allow(private_interfaces)]
#[derive(Clone)]
pub enum Command {
    SetColorSolid(SetColorSolidCommandData),
    SetColorCycle(SetColorCycleCommandData),
    SetColorBreathe(SetColorBreatheCommandData),
    SetColorOff(SetColorOffCommandData),
}

impl Command {
    pub fn new_color_solid(target: Target, color: Color) -> Self {
        Self::SetColorSolid(SetColorSolidCommandData {
            target,
            color
        })
    }

    pub fn new_color_cycle(target: Target, rate: u16, brightness: u8) -> Self {
        Self::SetColorCycle(SetColorCycleCommandData {
            target,
            rate: num::clamp(rate, 100, 65535),
            brightness: num::clamp(brightness, 1, 100),
        })
    }

    pub fn new_color_breathe(target: Target, color: Color, rate: u16, brightness: u8) -> Self {
        Self::SetColorBreathe(SetColorBreatheCommandData {
            target,
            color,
            rate: num::clamp(rate, 100, 65535),
            brightness: num::clamp(brightness, 1, 100),
        })
    }

    pub fn new_color_off(target: Target) -> Self {
        Self::SetColorOff(SetColorOffCommandData {
            target
        })
    }

    pub fn target(&self) -> Target {
        match self {
            Self::SetColorSolid(data) => data.target,
            Self::SetColorCycle(data) => data.target,
            Self::SetColorBreathe(data) => data.target,
            Self::SetColorOff(data) => data.target,
        }
    }

    fn mode_value(&self) -> u8 {
        match self {
            Self::SetColorSolid(_) => 0x01,
            Self::SetColorCycle(_) => 0x02,
            Self::SetColorBreathe(_) => 0x04,
            Self::SetColorOff(_) => 0x01,
        }
    }

    fn compress(commands: &[Command]) -> Vec<Command> {
        let mut command_sets: HashMap<Target, Vec<Command>> = HashMap::new();
        for target in Target::all() {
            command_sets.insert(target, Vec::new());
        }

        for command in commands {
            // The `unwrap()` here assumes all Targets in the command_sets HashMap have
            // been "initialized" with empty Vecs.
            command_sets.get_mut(&command.target()).unwrap().push(command.clone());
        }

        command_sets.into_iter()
            .filter_map(|mut c|
                if c.1.len() == 0 {
                    None
                } else {
                    Some(c.1.remove(c.1.len()-1))
                })
            .collect()
    }
}

#[derive(Clone)]
struct SetColorSolidCommandData {
    target: Target,
    color: Color,
}

#[derive(Clone)]
struct SetColorCycleCommandData {
    target: Target,
    rate: u16,
    brightness: u8,
}

#[derive(Clone)]
struct SetColorBreatheCommandData {
    target: Target,
    color: Color,
    rate: u16,
    brightness: u8,
}

#[derive(Clone)]
struct SetColorOffCommandData {
    target: Target,
}
