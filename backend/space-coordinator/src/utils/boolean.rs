use std::collections::HashSet;
use std::hash::Hash;

use serde::Deserialize;

use crate::tables::StateTable;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonOperator {
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Condition<K> {
    pub key: K,
    pub operator: ComparisonOperator,
    pub value: serde_json::Value,
}

impl<K> Condition<K> {
    pub fn test(&self, table: &impl StateTable<Key = K>) -> Result<bool, Error> {
        if let Some(value) = table.get(&self.key) {
            match self.operator {
                ComparisonOperator::Eq => Ok(value == self.value),
                ComparisonOperator::Ne => Ok(value != self.value),
                ComparisonOperator::Ge
                | ComparisonOperator::Gt
                | ComparisonOperator::Le
                | ComparisonOperator::Lt => {
                    if value.is_null() {
                        return Ok(false);
                    }

                    let Some(table_value) = value.as_f64() else {
                        return Err(Error::NumberExpected(value));
                    };
                    let Some(config_value) = self.value.as_f64() else {
                        return Err(Error::Configuration(self.value.clone()));
                    };

                    match self.operator {
                        ComparisonOperator::Ge => Ok(table_value >= config_value),
                        ComparisonOperator::Gt => Ok(table_value > config_value),
                        ComparisonOperator::Le => Ok(table_value <= config_value),
                        ComparisonOperator::Lt => Ok(table_value < config_value),
                        _ => unreachable!(),
                    }
                }
            }
        } else if self.operator == ComparisonOperator::Eq && self.value.is_null() {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Expression<K> {
    Unary(Condition<K>),
    And(Vec<Expression<K>>),
    Or { or: Vec<Expression<K>> },
}

impl<K: Eq + Hash + Clone> Expression<K> {
    pub fn test(&self, table: &impl StateTable<Key = K>) -> Result<bool, Error> {
        match self {
            Self::Unary(condition) => condition.test(table),
            Self::And(expressions) => Ok(expressions
                .iter()
                .map(|v| v.test(table))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .all(|v| v)),
            Self::Or { or: expressions } => Ok(expressions
                .iter()
                .map(|v| v.test(table))
                .any(|v| matches!(v, Ok(true)))),
        }
    }

    pub fn keys(&self) -> HashSet<K> {
        let mut values = HashSet::new();

        match self {
            Self::Unary(condition) => {
                values.insert(condition.key.clone());
            }
            Self::And(expressions) => {
                for expression in expressions {
                    values.extend(expression.keys());
                }
            }
            Self::Or { or: expressions } => {
                for expression in expressions {
                    values.extend(expression.keys());
                }
            }
        }

        values
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Expecter number, got {0}.")]
    NumberExpected(serde_json::Value),
    #[error("Expected number, got {0} from configuration.")]
    Configuration(serde_json::Value),
}
