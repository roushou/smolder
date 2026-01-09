//! ABI parsing and encoding utilities
//!
//! Provides the [`Abi`] struct for declarative access to contract ABI information
//! including functions, constructor, and parameter details.
//!
//! Also provides utilities for converting between JSON and Solidity types:
//! - [`json_to_sol_value`] - Convert JSON values to Solidity dynamic values
//! - [`sol_value_to_json`] - Convert Solidity dynamic values to JSON

use alloy::dyn_abi::{DynSolType, DynSolValue};
use alloy::json_abi::{Function, JsonAbi, Param, StateMutability as AlloyStateMutability};
use alloy::primitives::{Bytes, I256, U256};
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::types::StateMutability;

// =============================================================================
// Abi Struct
// =============================================================================

/// Wrapper around alloy's JsonAbi providing a declarative interface
/// for all ABI operations.
#[derive(Debug, Clone)]
pub struct Abi(JsonAbi);

impl Abi {
    /// Parse a JSON ABI string into an Abi struct
    pub fn parse(json: &str) -> Result<Self, Error> {
        let abi: JsonAbi = serde_json::from_str(json)
            .map_err(|e| Error::AbiParse(format!("Failed to parse ABI: {}", e)))?;
        Ok(Self(abi))
    }

    /// Parse from a serde_json::Value
    pub fn from_value(value: &serde_json::Value) -> Result<Self, Error> {
        let abi: JsonAbi = serde_json::from_value(value.clone())
            .map_err(|e| Error::AbiParse(format!("Failed to parse ABI: {}", e)))?;
        Ok(Self(abi))
    }

    /// Get the inner JsonAbi for advanced operations
    pub fn inner(&self) -> &JsonAbi {
        &self.0
    }

    // -------------------------------------------------------------------------
    // Constructor
    // -------------------------------------------------------------------------

    /// Get constructor information if present
    pub fn constructor(&self) -> Option<ConstructorInfo> {
        self.0.constructor.as_ref().map(|c| ConstructorInfo {
            inputs: c.inputs.iter().map(ParamInfo::from_abi_param).collect(),
            state_mutability: convert_state_mutability(c.state_mutability),
        })
    }

    /// Check if the contract has a constructor with arguments
    pub fn has_constructor_with_args(&self) -> bool {
        self.0
            .constructor
            .as_ref()
            .is_some_and(|c| !c.inputs.is_empty())
    }

    // -------------------------------------------------------------------------
    // Functions
    // -------------------------------------------------------------------------

    /// Get all functions categorized as read (view/pure) and write (nonpayable/payable)
    pub fn functions(&self) -> ParsedFunctions {
        let (mut read, mut write): (Vec<_>, Vec<_>) = self
            .0
            .functions
            .values()
            .flatten()
            .map(FunctionInfo::from_abi_function)
            .partition(|f| f.is_read_only());

        read.sort_by(|a, b| a.name.cmp(&b.name));
        write.sort_by(|a, b| a.name.cmp(&b.name));

        ParsedFunctions { read, write }
    }

    /// Get a specific function by name (returns first overload if multiple exist)
    pub fn function(&self, name: &str) -> Option<&Function> {
        self.0.functions.get(name).and_then(|funcs| funcs.first())
    }

    /// Get all overloads of a function by name
    pub fn function_overloads(&self, name: &str) -> Option<&Vec<Function>> {
        self.0.functions.get(name)
    }
}

// =============================================================================
// Constructor Types
// =============================================================================

/// Constructor information extracted from ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructorInfo {
    pub inputs: Vec<ParamInfo>,
    pub state_mutability: StateMutability,
}

impl ConstructorInfo {
    /// Check if this constructor can receive ETH
    pub fn is_payable(&self) -> bool {
        self.state_mutability.is_payable()
    }
}

// =============================================================================
// Function Types
// =============================================================================

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
    pub state_mutability: StateMutability,
}

impl FunctionInfo {
    /// Create FunctionInfo from an alloy Function
    pub fn from_abi_function(func: &Function) -> Self {
        Self {
            name: func.name.clone(),
            signature: func.signature(),
            inputs: func.inputs.iter().map(ParamInfo::from_abi_param).collect(),
            outputs: func.outputs.iter().map(ParamInfo::from_abi_param).collect(),
            state_mutability: convert_state_mutability(func.state_mutability),
        }
    }

    /// Check if this is a read-only function (view or pure)
    pub fn is_read_only(&self) -> bool {
        self.state_mutability.is_read_only()
    }

    /// Check if this function can receive ETH
    pub fn is_payable(&self) -> bool {
        self.state_mutability.is_payable()
    }
}

// =============================================================================
// Parameter Types
// =============================================================================

/// Information about a function or constructor parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamInfo {
    pub name: String,
    pub param_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<ParamInfo>>,
}

impl ParamInfo {
    /// Create ParamInfo from an alloy Param
    pub fn from_abi_param(param: &Param) -> Self {
        Self {
            name: param.name.clone(),
            param_type: param.ty.to_string(),
            components: if param.components.is_empty() {
                None
            } else {
                Some(param.components.iter().map(Self::from_abi_param).collect())
            },
        }
    }
}

// =============================================================================
// Helpers
// =============================================================================

fn convert_state_mutability(sm: AlloyStateMutability) -> StateMutability {
    match sm {
        AlloyStateMutability::Pure => StateMutability::Pure,
        AlloyStateMutability::View => StateMutability::View,
        AlloyStateMutability::NonPayable => StateMutability::NonPayable,
        AlloyStateMutability::Payable => StateMutability::Payable,
    }
}

// =============================================================================
// JSON <-> Solidity Value Conversion
// =============================================================================

/// Convert a JSON value to a Solidity dynamic value based on the type string.
///
/// Supports common Solidity types: address, bool, uint*, int*, bytes, string,
/// fixed bytes, and arrays.
pub fn json_to_sol_value(type_str: &str, value: &serde_json::Value) -> Result<DynSolValue, Error> {
    let sol_type: DynSolType = type_str
        .parse()
        .map_err(|e| Error::AbiEncode(format!("Unknown type '{}': {}", type_str, e)))?;

    match sol_type {
        DynSolType::Address => {
            let addr_str = value
                .as_str()
                .ok_or_else(|| Error::AbiEncode("Expected string for address".into()))?;
            let addr: alloy::primitives::Address = addr_str
                .parse()
                .map_err(|e| Error::AbiEncode(format!("Invalid address '{}': {}", addr_str, e)))?;
            Ok(DynSolValue::Address(addr))
        }
        DynSolType::Bool => {
            let b = value
                .as_bool()
                .ok_or_else(|| Error::AbiEncode("Expected boolean".into()))?;
            Ok(DynSolValue::Bool(b))
        }
        DynSolType::Uint(bits) => {
            let n = parse_uint(value)?;
            Ok(DynSolValue::Uint(n, bits))
        }
        DynSolType::Int(bits) => {
            let n = parse_int(value)?;
            Ok(DynSolValue::Int(n, bits))
        }
        DynSolType::Bytes => {
            let hex_str = value
                .as_str()
                .ok_or_else(|| Error::AbiEncode("Expected hex string for bytes".into()))?;
            let bytes: Bytes = hex_str
                .parse()
                .map_err(|e| Error::AbiEncode(format!("Invalid hex: {}", e)))?;
            Ok(DynSolValue::Bytes(bytes.to_vec()))
        }
        DynSolType::String => {
            let s = value
                .as_str()
                .ok_or_else(|| Error::AbiEncode("Expected string".into()))?;
            Ok(DynSolValue::String(s.to_string()))
        }
        DynSolType::FixedBytes(size) => {
            let hex_str = value
                .as_str()
                .ok_or_else(|| Error::AbiEncode("Expected hex string".into()))?;
            let bytes: Bytes = hex_str
                .parse()
                .map_err(|e| Error::AbiEncode(format!("Invalid hex: {}", e)))?;
            if bytes.len() != size {
                return Err(Error::AbiEncode(format!(
                    "Expected {} bytes, got {}",
                    size,
                    bytes.len()
                )));
            }
            Ok(DynSolValue::FixedBytes(
                alloy::primitives::FixedBytes::from_slice(&bytes),
                size,
            ))
        }
        DynSolType::Array(inner) => {
            let arr = value
                .as_array()
                .ok_or_else(|| Error::AbiEncode("Expected array".into()))?;
            let inner_str = inner.to_string();
            let values: Result<Vec<_>, _> = arr
                .iter()
                .map(|v| json_to_sol_value(&inner_str, v))
                .collect();
            Ok(DynSolValue::Array(values?))
        }
        _ => Err(Error::AbiEncode(format!("Unsupported type: {}", type_str))),
    }
}

/// Parse a JSON value as a U256 unsigned integer.
pub fn parse_uint(value: &serde_json::Value) -> Result<U256, Error> {
    match value {
        serde_json::Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                Ok(U256::from(u))
            } else if let Some(i) = n.as_i64() {
                if i >= 0 {
                    Ok(U256::from(i as u64))
                } else {
                    Err(Error::AbiEncode(
                        "Negative number not allowed for uint".into(),
                    ))
                }
            } else {
                Err(Error::AbiEncode("Number too large".into()))
            }
        }
        serde_json::Value::String(s) => s
            .parse::<U256>()
            .map_err(|e| Error::AbiEncode(format!("Invalid uint: {}", e))),
        _ => Err(Error::AbiEncode(
            "Expected number or string for uint".into(),
        )),
    }
}

/// Parse a JSON value as an I256 signed integer.
pub fn parse_int(value: &serde_json::Value) -> Result<I256, Error> {
    match value {
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(I256::try_from(i).unwrap())
            } else {
                Err(Error::AbiEncode("Number out of range".into()))
            }
        }
        serde_json::Value::String(s) => s
            .parse::<I256>()
            .map_err(|e| Error::AbiEncode(format!("Invalid int: {}", e))),
        _ => Err(Error::AbiEncode("Expected number or string for int".into())),
    }
}

/// Convert a Solidity dynamic value to a JSON value.
pub fn sol_value_to_json(value: &DynSolValue) -> serde_json::Value {
    match value {
        DynSolValue::Address(a) => serde_json::json!(format!("{:?}", a)),
        DynSolValue::Bool(b) => serde_json::json!(b),
        DynSolValue::Uint(n, _) => serde_json::json!(n.to_string()),
        DynSolValue::Int(n, _) => serde_json::json!(n.to_string()),
        DynSolValue::Bytes(b) => serde_json::json!(format!("0x{}", alloy::hex::encode(b))),
        DynSolValue::FixedBytes(b, _) => serde_json::json!(format!("0x{}", alloy::hex::encode(b))),
        DynSolValue::String(s) => serde_json::json!(s),
        DynSolValue::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(sol_value_to_json).collect())
        }
        DynSolValue::Tuple(arr) => {
            serde_json::Value::Array(arr.iter().map(sol_value_to_json).collect())
        }
        _ => serde_json::Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ABI: &str = r#"[
        {
            "type": "constructor",
            "inputs": [
                {"name": "name", "type": "string"},
                {"name": "symbol", "type": "string"}
            ],
            "stateMutability": "payable"
        },
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
        let abi = Abi::parse(TEST_ABI).unwrap();
        assert_eq!(abi.inner().functions.len(), 4);
    }

    #[test]
    fn test_constructor() {
        let abi = Abi::parse(TEST_ABI).unwrap();
        let constructor = abi.constructor().unwrap();

        assert_eq!(constructor.inputs.len(), 2);
        assert_eq!(constructor.inputs[0].name, "name");
        assert_eq!(constructor.inputs[0].param_type, "string");
        assert_eq!(constructor.inputs[1].name, "symbol");
        assert_eq!(constructor.inputs[1].param_type, "string");
        assert_eq!(constructor.state_mutability, StateMutability::Payable);
        assert!(constructor.is_payable());
    }

    #[test]
    fn test_has_constructor_with_args() {
        let abi = Abi::parse(TEST_ABI).unwrap();
        assert!(abi.has_constructor_with_args());

        let no_args = Abi::parse(
            r#"[{"type": "constructor", "inputs": [], "stateMutability": "nonpayable"}]"#,
        )
        .unwrap();
        assert!(!no_args.has_constructor_with_args());

        let no_constructor = Abi::parse(r#"[{"type": "function", "name": "foo", "inputs": [], "outputs": [], "stateMutability": "view"}]"#).unwrap();
        assert!(!no_constructor.has_constructor_with_args());
    }

    #[test]
    fn test_categorize_functions() {
        let abi = Abi::parse(TEST_ABI).unwrap();
        let parsed = abi.functions();

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
        let abi = Abi::parse(TEST_ABI).unwrap();
        let parsed = abi.functions();

        let balance_of = parsed.read.iter().find(|f| f.name == "balanceOf").unwrap();
        assert_eq!(balance_of.state_mutability, StateMutability::View);
        assert!(balance_of.is_read_only());
        assert!(!balance_of.is_payable());
        assert_eq!(balance_of.inputs.len(), 1);
        assert_eq!(balance_of.inputs[0].name, "owner");
        assert_eq!(balance_of.inputs[0].param_type, "address");
        assert_eq!(balance_of.outputs.len(), 1);
        assert_eq!(balance_of.outputs[0].param_type, "uint256");
    }

    #[test]
    fn test_get_function() {
        let abi = Abi::parse(TEST_ABI).unwrap();
        let func = abi.function("transfer").unwrap();
        assert_eq!(func.name, "transfer");
        assert_eq!(func.inputs.len(), 2);
    }

    #[test]
    fn test_get_function_not_found() {
        let abi = Abi::parse(TEST_ABI).unwrap();
        assert!(abi.function("nonexistent").is_none());
    }

    #[test]
    fn test_constructor_with_components() {
        let abi_json = r#"[{
            "type": "constructor",
            "inputs": [{
                "name": "config",
                "type": "tuple",
                "components": [
                    {"name": "value", "type": "uint256"},
                    {"name": "enabled", "type": "bool"}
                ]
            }],
            "stateMutability": "nonpayable"
        }]"#;

        let abi = Abi::parse(abi_json).unwrap();
        let constructor = abi.constructor().unwrap();

        assert_eq!(constructor.inputs.len(), 1);
        assert_eq!(constructor.inputs[0].name, "config");
        assert_eq!(constructor.inputs[0].param_type, "tuple");

        let components = constructor.inputs[0].components.as_ref().unwrap();
        assert_eq!(components.len(), 2);
        assert_eq!(components[0].name, "value");
        assert_eq!(components[0].param_type, "uint256");
        assert_eq!(components[1].name, "enabled");
        assert_eq!(components[1].param_type, "bool");
    }
}
