use crate::commands::CommandInput;

pub enum Action {
    BridgeCreate,
    BridgeDestroy,
}

#[derive(PartialEq, PartialOrd)]
enum PowerLevel {
    //    User = 0,
    //    Moderator = 50,
    Administrator = 100,
}

pub struct PowerLevelConstraint {
    pub minimum: i64,
    pub maximum: Option<i64>,
}

impl PowerLevelConstraint {
    fn new(minimum: i64, maximum: Option<i64>) -> Self {
        Self { minimum, maximum }
    }

    fn is_allowed(&self, power_level: i64) -> bool {
        if self.maximum.is_none() {
            return power_level >= self.minimum;
        } else {
            if self.minimum == self.maximum.unwrap() {
                return power_level == self.minimum;
            } else {
                return self.maximum.unwrap() >= power_level && power_level >= self.minimum;
            }
        }
    }
}

pub fn get_power_level_constraint(action: Action) -> PowerLevelConstraint {
    match action {
        Action::BridgeCreate => PowerLevelConstraint::new(PowerLevel::Administrator as i64, None),
        Action::BridgeDestroy => PowerLevelConstraint::new(PowerLevel::Administrator as i64, None),
    }
}

pub async fn is_allowed(command_input: &CommandInput, action: Action) -> Result<bool, String> {
    match command_input
        .room
        .get_member(&command_input.event.sender)
        .await
    {
        Ok(member) => match member {
            Some(member) => {
                if get_power_level_constraint(action).is_allowed(member.power_level()) {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            None => Err("member somehow does not exist".to_string()),
        },
        Err(error) => Err(format!("unable to get member: {error}")),
    }
}
