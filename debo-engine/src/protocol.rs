//! Versioned, bounded JSON commands accepted from DEBO's local controller.
use crate::face::Expression;
use serde_json::Value;

pub const PROTOCOL_VERSION: u64 = 1;
pub const MAX_MESSAGE_BYTES: usize = 4096;

#[derive(Clone, Debug, PartialEq)]
pub enum FaceCommand {
    SetExpression(Expression),
    SetEyeColor(String),
    Gaze { x: f32, y: f32 },
    Blink,
    CentreGaze,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProtocolError(pub &'static str);

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(self.0) }
}
impl std::error::Error for ProtocolError {}

/// Parse and validate one JSON command. This function never mutates face state.
pub fn parse_message(input: &str) -> Result<FaceCommand, ProtocolError> {
    if input.len() > MAX_MESSAGE_BYTES { return Err(ProtocolError("message_too_large")); }
    let value: Value = serde_json::from_str(input).map_err(|_| ProtocolError("invalid_json"))?;
    let object = value.as_object().ok_or(ProtocolError("expected_object"))?;
    if object.get("version").and_then(Value::as_u64) != Some(PROTOCOL_VERSION) {
        return Err(ProtocolError("unsupported_version"));
    }
    let command = object.get("command").and_then(Value::as_str).ok_or(ProtocolError("missing_command"))?;
    match command {
        "set_expression" => {
            let name = object.get("value").and_then(Value::as_str).ok_or(ProtocolError("missing_value"))?;
            let expression = match name.to_ascii_lowercase().as_str() {
                "neutral" => Expression::Neutral,
                "curious" => Expression::Curious,
                "happy" => Expression::Happy,
                "surprised" => Expression::Surprised,
                "sleepy" => Expression::Sleepy,
                _ => return Err(ProtocolError("unknown_expression")),
            };
            Ok(FaceCommand::SetExpression(expression))
        }
        "set_eye_color" => {
            let name = object.get("value").and_then(Value::as_str).ok_or(ProtocolError("missing_value"))?;
            let normalized = name.to_ascii_lowercase();
            if !crate::appearance::EYE_COLOR_PRESETS.iter().any(|p| p.name == normalized) {
                return Err(ProtocolError("unknown_eye_color"));
            }
            Ok(FaceCommand::SetEyeColor(normalized))
        }
        "gaze" => {
            let x = object.get("x").and_then(Value::as_f64).ok_or(ProtocolError("missing_or_invalid_x"))?;
            let y = object.get("y").and_then(Value::as_f64).ok_or(ProtocolError("missing_or_invalid_y"))?;
            if !x.is_finite() || !y.is_finite() || !(-1.0..=1.0).contains(&x) || !(-1.0..=1.0).contains(&y) {
                return Err(ProtocolError("gaze_out_of_range"));
            }
            Ok(FaceCommand::Gaze { x: x as f32, y: y as f32 })
        }
        "blink" => Ok(FaceCommand::Blink),
        "centre_gaze" => Ok(FaceCommand::CentreGaze),
        _ => Err(ProtocolError("unknown_command")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_expression_and_case_insensitive_colour() {
        assert_eq!(parse_message(r#"{"version":1,"command":"set_expression","value":"happy"}"#).unwrap(), FaceCommand::SetExpression(Expression::Happy));
        assert_eq!(parse_message(r#"{"version":1,"command":"set_eye_color","value":"PuRpLe"}"#).unwrap(), FaceCommand::SetEyeColor("purple".into()));
    }

    #[test]
    fn accepts_gaze_only_inside_bounds() {
        assert_eq!(parse_message(r#"{"version":1,"command":"gaze","x":0.5,"y":-1}"#).unwrap(), FaceCommand::Gaze { x: 0.5, y: -1.0 });
        assert_eq!(parse_message(r#"{"version":1,"command":"gaze","x":2,"y":0}"#), Err(ProtocolError("gaze_out_of_range")));
    }

    #[test]
    fn rejects_unknown_commands_expressions_and_colours() {
        assert_eq!(parse_message(r#"{"version":1,"command":"do_magic"}"#), Err(ProtocolError("unknown_command")));
        assert_eq!(parse_message(r#"{"version":1,"command":"set_expression","value":"angry"}"#), Err(ProtocolError("unknown_expression")));
        assert_eq!(parse_message(r#"{"version":1,"command":"set_eye_color","value":"ultraviolet"}"#), Err(ProtocolError("unknown_eye_color")));
    }

    #[test]
    fn rejects_bad_versions_malformed_json_and_oversized_messages() {
        assert_eq!(parse_message(r#"{"version":2,"command":"blink"}"#), Err(ProtocolError("unsupported_version")));
        assert_eq!(parse_message("not json"), Err(ProtocolError("invalid_json")));
        assert_eq!(parse_message(&"x".repeat(MAX_MESSAGE_BYTES + 1)), Err(ProtocolError("message_too_large")));
    }
}
