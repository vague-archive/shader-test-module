//! Helpers mostly related to analyzing WGSL for tests.

use std::{error::Error, fmt::Display};

use naga::{
    WithSpan,
    front::wgsl::{ParseError, parse_str},
    valid::{Capabilities, ValidationError, ValidationFlags, Validator},
};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct WgslValidator(Validator);

impl WgslValidator {
    pub fn emit_wgsl_metadata<S: AsRef<str>>(
        &mut self,
        shader_string: S,
    ) -> Result<WgslMetaData, WgslError> {
        let shader_string = shader_string.as_ref();
        let module = parse_str(shader_string)
            .map_err(|error| WgslError::from_parse_error(&error, shader_string))?;

        let types = module
            .types
            .iter()
            .fold(vec![], |mut accumulator, (_, wgsl_type)| {
                if let Some(name) = &wgsl_type.name {
                    accumulator.push(name.clone());
                }
                accumulator
            });
        let global_variables =
            module
                .global_variables
                .iter()
                .fold(vec![], |mut accumulator, (_, global_variable)| {
                    if let Some(name) = &global_variable.name {
                        accumulator.push(name.clone());
                    }
                    accumulator
                });
        let functions = module
            .functions
            .iter()
            .fold(vec![], |mut accumulator, (_, function)| {
                if let Some(name) = &function.name {
                    accumulator.push(name.clone());
                }
                accumulator
            });
        let special_types = module
            .special_types
            .predeclared_types
            .iter()
            .map(|(_, special_type)| special_type.to_wgsl(&module.to_ctx()))
            .collect();
        let constants = module
            .constants
            .iter()
            .fold(vec![], |mut accumulator, (_, constant)| {
                if let Some(name) = &constant.name {
                    accumulator.push(name.clone());
                }
                accumulator
            });
        let overrides =
            module
                .overrides
                .iter()
                .fold(vec![], |mut accumulator, (_, wgsl_override)| {
                    if let Some(name) = &wgsl_override.name {
                        accumulator.push(name.clone());
                    }
                    accumulator
                });
        let entry_points = module
            .entry_points
            .iter()
            .map(|entry_point| entry_point.name.clone())
            .collect();

        Ok(WgslMetaData {
            types,
            global_variables,
            functions,
            special_types,
            constants,
            overrides,
            entry_points,
        })
    }

    pub fn validate_wgsl_string<S: AsRef<str>>(
        &mut self,
        shader_string: S,
    ) -> Result<(), WgslError> {
        let shader_string = shader_string.as_ref();
        let module = parse_str(shader_string)
            .map_err(|error| WgslError::from_parse_error(&error, shader_string))?;

        if let Err(error) = self.0.validate(&module) {
            let message = error.emit_to_string(shader_string);
            Err(WgslError::ValidationErr {
                source: shader_string.to_string(),
                error,
                message,
            })
        } else {
            Ok(())
        }
    }
}

impl Default for WgslValidator {
    fn default() -> Self {
        Self(Validator::new(ValidationFlags::all(), Capabilities::all()))
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WgslMetaData {
    types: Vec<String>,
    special_types: Vec<String>,
    constants: Vec<String>,
    overrides: Vec<String>,
    global_variables: Vec<String>,
    functions: Vec<String>,
    entry_points: Vec<String>,
}

impl WgslMetaData {
    pub fn types_iter(&self) -> impl Iterator<Item = &'_ str> {
        self.types.iter().map(|value| value.as_str())
    }
    pub fn special_types_iter(&self) -> impl Iterator<Item = &'_ str> {
        self.special_types.iter().map(|value| value.as_str())
    }
    pub fn constants_iter(&self) -> impl Iterator<Item = &'_ str> {
        self.constants.iter().map(|value| value.as_str())
    }
    pub fn overrides_iter(&self) -> impl Iterator<Item = &'_ str> {
        self.overrides.iter().map(|value| value.as_str())
    }
    pub fn global_variables_iter(&self) -> impl Iterator<Item = &'_ str> {
        self.global_variables.iter().map(|value| value.as_str())
    }
    pub fn functions_iter(&self) -> impl Iterator<Item = &'_ str> {
        self.functions.iter().map(|value| value.as_str())
    }
    pub fn entry_points_iter(&self) -> impl Iterator<Item = &'_ str> {
        self.entry_points.iter().map(|value| value.as_str())
    }
}

#[derive(Debug)]
pub enum WgslError {
    ValidationErr {
        source: String,
        error: WithSpan<ValidationError>,
        message: String,
    },
    ParserErr {
        message: String,
        line: Option<usize>,
        position: Option<usize>,
    },
}

impl WgslError {
    pub fn from_parse_error(error: &ParseError, source: &str) -> Self {
        let message = error.emit_to_string(source);
        if let Some(location) = error.location(source) {
            Self::ParserErr {
                message,
                line: Some(location.line_number as usize),
                position: Some(location.line_position as usize),
            }
        } else {
            Self::ParserErr {
                message,
                line: None,
                position: None,
            }
        }
    }
}

impl Display for WgslError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            WgslError::ValidationErr {
                source,
                error,
                message,
            } => {
                write!(
                    f,
                    "Error validating WGSL. Error: {}, message: {}",
                    error.emit_to_string(source),
                    message
                )
            }
            WgslError::ParserErr {
                message,
                line,
                position,
            } => {
                let line_string = match line {
                    Some(line) => line.to_string(),
                    None => "not found".to_string(),
                };
                let position_string = match position {
                    Some(position) => position.to_string(),
                    None => "not found".to_string(),
                };
                write!(
                    f,
                    "Error parsing WGSL on ln {line_string} pos {position_string} : {message}"
                )
            }
        }
    }
}

impl Error for WgslError {}
