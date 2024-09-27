use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::{schema::SchemaType, types::Context};
use derive_more::{Deref, DerefMut};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorInput(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operand {
    Value(serde_json::Value),
    Dimension(serde_json::Value),
}

impl From<Value> for Operand {
    fn from(value: Value) -> Self {
        match value {
            Value::Object(ref o) if o.contains_key("var") => Operand::Dimension(value),
            v => Operand::Value(v),
        }
    }
}

#[derive(Debug, Clone, Deref, DerefMut, Serialize, Deserialize)]
pub struct Operands(pub Vec<Operand>);

impl FromIterator<Value> for Operands {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        Operands(
            iter.into_iter()
                .map(Operand::from)
                .collect::<Vec<Operand>>(),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionName(pub String);

impl TryFrom<(Operator, String, SchemaType)> for Operands {
    type Error = String;
    fn try_from(
        (operator, d_name, r#type): (Operator, String, SchemaType),
    ) -> Result<Self, Self::Error> {
        match operator {
            Operator::Is => Ok(Operands(vec![
                Operand::Dimension(json!({ "var": d_name })),
                Operand::Value(r#type.default_value()),
            ])),
            Operator::In => Ok(Operands(vec![
                Operand::Dimension(json!({ "var": d_name })),
                Operand::Value(r#type.default_value()),
            ])),
            Operator::Has => Ok(Operands(vec![
                Operand::Value(r#type.default_value()),
                Operand::Dimension(json!({ "var": d_name })),
            ])),
            Operator::Between => Ok(Operands(vec![
                Operand::Value(r#type.default_value()),
                Operand::Dimension(json!({ "var": d_name })),
                Operand::Value(r#type.default_value()),
            ])),
            _ => Err(String::from("unsupported operator")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Operator {
    Is,
    In,
    Has,
    Between,
    Other(String),
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Has => f.write_str("has"),
            Self::Is => f.write_str("is"),
            Self::In => f.write_str("in"),
            Self::Between => f.write_str("between"),
            Self::Other(o) => f.write_str(o),
        }
    }
}

impl From<OperatorInput> for Operator {
    fn from(op: OperatorInput) -> Self {
        match op.0.as_str() {
            "==" => Operator::Is,
            "<=" => Operator::Between,
            "in" => Operator::In,
            "has" => Operator::Has,
            other => Operator::Other(other.to_string()),
        }
    }
}

impl From<(String, &Vec<Value>)> for Operator {
    fn from(value: (String, &Vec<Value>)) -> Self {
        let (operator, operands) = value;
        let operand_0 = operands.first();
        let operand_1 = operands.get(1);
        let operand_2 = operands.get(2);
        match (operator.as_str(), operand_0, operand_1, operand_2) {
            // assuming there will be only two operands, one with the dimension name and other with the value
            ("==", _, _, None) => Operator::Is,
            ("<=", Some(_), Some(Value::Object(a)), Some(_)) if a.contains_key("var") => {
                Operator::Between
            }
            // assuming there will be only two operands, one with the dimension name and other with the value
            ("in", Some(Value::Object(a)), Some(_), None) if a.contains_key("var") => {
                Operator::In
            }
            // assuming there will be only two operands, one with the dimension name and other with the value
            ("in", Some(_), Some(Value::Object(a)), None) if a.contains_key("var") => {
                Operator::Has
            }
            _ => Operator::Other(operator),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Condition {
    pub dimension: String,
    pub operator: Operator,
    pub operands: Operands,
}

#[derive(
    Clone, Debug, derive_more::Deref, derive_more::DerefMut, Serialize, Deserialize, Default
)]
pub struct Conditions(pub Vec<Condition>);

impl TryFrom<(Operator, String, SchemaType)> for Condition {
    type Error = String;
    fn try_from(
        (operator, d_name, r#type): (Operator, String, SchemaType),
    ) -> Result<Self, Self::Error> {
        Ok(Condition {
            dimension: d_name.clone(),
            operator: operator.clone(),
            operands: Operands::try_from((operator, d_name, r#type))?,
        })
    }
}

impl TryFrom<&Map<String, Value>> for Condition {
    type Error = &'static str;
    fn try_from(source: &Map<String, Value>) -> Result<Self, Self::Error> {
        if let Some(operator) = source.keys().next() {
            let emty_vec = vec![];
            let operands = source[operator].as_array().unwrap_or(&emty_vec);

            let operator = Operator::from((operator.to_owned(), operands));

            let dimension_name = operands
                .iter()
                .find_map(|item| match item.as_object() {
                    Some(o) if o.contains_key("var") => {
                        Some(o["var"].as_str().unwrap_or(""))
                    }
                    _ => None,
                })
                .unwrap_or("");

            return Ok(Condition {
                operator,
                dimension: dimension_name.to_owned(),
                operands: Operands::from_iter(operands.to_owned()),
            });
        }

        Err("not a valid condition map")
    }
}

impl TryFrom<&Value> for Condition {
    type Error = &'static str;
    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let obj = value
            .as_object()
            .ok_or("not a valid condition value, should be an object")?;
        Condition::try_from(obj)
    }
}

impl Into<Value> for Condition {
    fn into(self) -> Value {
        let operator = match self.operator {
            Operator::In | Operator::Has => "in".to_owned(),
            Operator::Is => "==".to_owned(),
            Operator::Between => "<=".to_owned(),
            Operator::Other(op) => op,
        };

        let operands = self
            .operands
            .iter()
            .map(|v| match v {
                Operand::Dimension(d) => d.clone(),
                Operand::Value(v) => v.clone(),
            })
            .collect::<Vec<Value>>();

        json!({ operator: operands })
    }
}

impl TryFrom<&Context> for Conditions {
    type Error = &'static str;
    fn try_from(context: &Context) -> Result<Self, Self::Error> {
        Self::from_context_json(context.condition.clone())
    }
}

impl Conditions {
    pub fn from_context_json(context: Value) -> Result<Self, &'static str> {
        Ok(Conditions(
            context
                .as_object()
                .ok_or("failed to parse context.condition as an object")
                .and_then(|obj| match obj.get("and") {
                    Some(v) => v
                        .as_array()
                        .ok_or("failed to parse value of and as array")
                        .and_then(|arr| {
                            arr.iter().map(Condition::try_from).collect::<Result<
                                Vec<Condition>,
                                &'static str,
                            >>(
                            )
                        }),
                    None => Condition::try_from(obj).map(|v| vec![v]),
                })?,
        ))
    }
}

impl Into<Value> for Conditions {
    fn into(self) -> Value {
        let conditions = self
            .iter()
            .map(|v| v.clone().into())
            .collect::<Vec<Value>>();
        json!({ "and": conditions })
    }
}
