export interface Network {
	id: number;
	name: string;
	chain_id: number;
	rpc_url: string;
	explorer_url: string | null;
	created_at: string;
}

export interface Contract {
	id: number;
	name: string;
	source_path: string;
	abi: string;
	bytecode_hash: string;
	created_at: string;
}

export interface Deployment {
	id: number;
	contract_name: string;
	network_name: string;
	chain_id: number;
	address: string;
	deployer: string;
	tx_hash: string;
	block_number: number | null;
	version: number;
	deployed_at: string;
	is_current: boolean;
	abi: string;
}

export interface HealthResponse {
	status: string;
	version: string;
}

export interface Wallet {
	id: number;
	name: string;
	address: string;
	created_at: string;
}

export interface ParamInfo {
	name: string;
	param_type: string;
	components?: ParamInfo[];
}

export interface FunctionInfo {
	name: string;
	signature: string;
	inputs: ParamInfo[];
	outputs: ParamInfo[];
	state_mutability: string;
}

export interface FunctionsResponse {
	read: FunctionInfo[];
	write: FunctionInfo[];
}

export interface CallRequest {
	function_name: string;
	params: unknown[];
}

export interface CallResponse {
	result: unknown;
}

export interface SendRequest {
	function_name: string;
	params: unknown[];
	wallet_name: string;
	value?: string;
}

export interface SendResponse {
	tx_hash: string;
	history_id: number;
}

export interface CallHistory {
	id: number;
	deployment_id: number;
	contract_name: string;
	network_name: string;
	contract_address: string;
	wallet_name: string | null;
	function_name: string;
	function_signature: string;
	input_params: string;
	call_type: string;
	result: string | null;
	tx_hash: string | null;
	block_number: number | null;
	gas_used: number | null;
	gas_price: string | null;
	status: string | null;
	error_message: string | null;
	created_at: string;
	confirmed_at: string | null;
}

export interface ArtifactInfo {
	name: string;
	source_path: string;
	has_constructor: boolean;
	has_bytecode: boolean;
	in_registry: boolean;
}

export interface ConstructorInput {
	name: string;
	param_type: string;
	components?: ConstructorInput[];
}

export interface ConstructorInfo {
	inputs: ConstructorInput[];
	state_mutability: string;
}

export interface ArtifactDetails {
	name: string;
	source_path: string;
	abi: unknown;
	constructor: ConstructorInfo | null;
	has_bytecode: boolean;
	in_registry: boolean;
}

export interface DeployRequest {
	artifact_name: string;
	network_name: string;
	wallet_name: string;
	constructor_args: unknown[];
	value?: string;
}

export interface DeployResponse {
	tx_hash: string;
	contract_address: string | null;
	deployment_id: number | null;
}
