use crate::ports::CreateModelStepError;
port! { pub PresentTurnEvent(event: domain::TurnEvent,) -> Result<(), CreateModelStepError>; }
