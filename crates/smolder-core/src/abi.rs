//! ABI parsing and encoding utilities
//!
//! Uses alloy for ABI parsing, function categorization, and parameter encoding/decoding.

use alloy::json_abi::{Function, JsonAbi, Param, StateMutability};
use serde::{Deserialize, Serialize};

use crate::error::Error;

/// Parsed contract functions separated by read/write
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedFunctions {
    pub read: Vec<FunctionInfo>,
    pub write: Vec<FunctionInfo>,
}

/// Information about a single contract function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub signature: String,
    pub inputs: Vec<ParamInfo>,
    pub outputs: Vec<ParamInfo>,
    pub state_mutability: String,
}

/// Information about a function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamInfo {
    pub name: String,
    pub param_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<ParamInfo>>,
}

impl FunctionInfo {
    /// Create FunctionInfo from an alloy Function
    pub fn from_abi_function(func: &Function) -> Self {
        Self {
            name: func.name.clone(),
            signature: func.signature(),
            inputs: func.inputs.iter().map(ParamInfo::from_abi_param).collect(),
            outputs: func.outputs.iter().map(ParamInfo::from_abi_param).collect(),
            state_mutability: match func.state_mutability {
                StateMutability::Pure => "pure".to_string(),
                StateMutability::View => "view".to_string(),
                StateMutability::NonPayable => "nonpayable".to_string(),
                StateMutability::Payable => "payable".to_string(),
            },
        }
    }
}

impl ParamInfo {
    /// Create ParamInfo from an alloy Param
    pub fn from_abi_param(param: &Param) -> Self {
        let components = if param.components.is_empty() {
            None
        } else {
            Some(
                param
                    .components
                    .iter()
                    .map(ParamInfo::from_abi_param)
                    .collect(),
            )
        };

        Self {
            name: param.name.clone(),
            param_type: param.ty.to_string(),
            components,
        }
    }
}

/// Parse a JSON ABI string into a JsonAbi struct
pub fn parse_abi(abi_json: &str) -> Result<JsonAbi, Error> {
    serde_json::from_str(abi_json).map_err(|e| Error::Abi(format!("Failed to parse ABI: {}", e)))
}

/// Categorize contract functions into read (view/pure) and write (nonpayable/payable)
pub fn categorize_functions(abi_json: &str) -> Result<ParsedFunctions, Error> {
    let abi = parse_abi(abi_json)?;

    let mut read = Vec::new();
    let mut write = Vec::new();

    for functions in abi.functions.values() {
        for func in functions {
            let info = FunctionInfo::from_abi_function(func);
            match func.state_mutability {
                StateMutability::Pure | StateMutability::View => read.push(info),
                StateMutability::NonPayable | StateMutability::Payable => write.push(info),
            }
        }
    }

    // Sort alphabetically for consistent display
    read.sort_by(|a, b| a.name.cmp(&b.name));
    write.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(ParsedFunctions { read, write })
}

/// Get a specific function from an ABI by name (returns first overload if multiple exist)
pub fn get_function(abi_json: &str, function_name: &str) -> Result<Function, Error> {
    let abi = parse_abi(abi_json)?;

    abi.functions
        .get(function_name)
        .and_then(|funcs| funcs.first())
        .cloned()
        .ok_or_else(|| Error::FunctionNotFound(function_name.to_string()))
}

/// Get all overloads of a function by name
pub fn get_function_overloads(abi_json: &str, function_name: &str) -> Result<Vec<Function>, Error> {
    let abi = parse_abi(abi_json)?;

    abi.functions
        .get(function_name)
        .cloned()
        .ok_or_else(|| Error::FunctionNotFound(function_name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ABI: &str = r#"[
        {
            "type": "function",
            "name": "balanceOf",
            "inputs": [{"name": "owner", "type": "address"}],
            "outputs": [{"name": "", "type": "uint256"}],
            "stateMutability": "view"
        },
        {
            "type": "function",
            "name": "transfer",
            "inputs": [
                {"name": "to", "type": "address"},
                {"name": "amount", "type": "uint256"}
            ],
            "outputs": [{"name": "", "type": "bool"}],
            "stateMutability": "nonpayable"
        },
        {
            "type": "function",
            "name": "name",
            "inputs": [],
            "outputs": [{"name": "", "type": "string"}],
            "stateMutability": "pure"
        },
        {
            "type": "function",
            "name": "mint",
            "inputs": [
                {"name": "to", "type": "address"},
                {"name": "amount", "type": "uint256"}
            ],
            "outputs": [],
            "stateMutability": "payable"
        }
    ]"#;

    #[test]
    fn test_parse_abi() {
        let abi = parse_abi(TEST_ABI).unwrap();
        assert_eq!(abi.functions.len(), 4);
    }

    #[test]
    fn test_categorize_functions() {
        let parsed = categorize_functions(TEST_ABI).unwrap();

        // Read functions: balanceOf, name
        assert_eq!(parsed.read.len(), 2);
        assert!(parsed.read.iter().any(|f| f.name == "balanceOf"));
        assert!(parsed.read.iter().any(|f| f.name == "name"));

        // Write functions: transfer, mint
        assert_eq!(parsed.write.len(), 2);
        assert!(parsed.write.iter().any(|f| f.name == "transfer"));
        assert!(parsed.write.iter().any(|f| f.name == "mint"));
    }

    #[test]
    fn test_function_info() {
        let parsed = categorize_functions(TEST_ABI).unwrap();

        let balance_of = parsed.read.iter().find(|f| f.name == "balanceOf").unwrap();
        assert_eq!(balance_of.state_mutability, "view");
        assert_eq!(balance_of.inputs.len(), 1);
        assert_eq!(balance_of.inputs[0].name, "owner");
        assert_eq!(balance_of.inputs[0].param_type, "address");
        assert_eq!(balance_of.outputs.len(), 1);
        assert_eq!(balance_of.outputs[0].param_type, "uint256");
    }

    #[test]
    fn test_get_function() {
        let func = get_function(TEST_ABI, "transfer").unwrap();
        assert_eq!(func.name, "transfer");
        assert_eq!(func.inputs.len(), 2);
    }

    #[test]
    fn test_get_function_not_found() {
        let result = get_function(TEST_ABI, "nonexistent");
        assert!(result.is_err());
    }
}
