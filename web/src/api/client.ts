import type {
	ArtifactDetails,
	ArtifactInfo,
	CallHistory,
	CallRequest,
	CallResponse,
	Contract,
	Deployment,
	DeployRequest,
	DeployResponse,
	FunctionsResponse,
	HealthResponse,
	Network,
	SendRequest,
	SendResponse,
	Wallet,
} from "./types";

const API_BASE = "/api";

async function fetchJson<T>(url: string): Promise<T> {
	const response = await fetch(url);
	if (!response.ok) {
		const text = await response.text();
		throw new Error(text || `API error: ${response.status}`);
	}
	return response.json();
}

async function postJson<T, R>(url: string, data: T): Promise<R> {
	const response = await fetch(url, {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify(data),
	});
	if (!response.ok) {
		const text = await response.text();
		throw new Error(text || `API error: ${response.status}`);
	}
	return response.json();
}

async function deleteRequest(url: string): Promise<void> {
	const response = await fetch(url, { method: "DELETE" });
	if (!response.ok && response.status !== 204) {
		const text = await response.text();
		throw new Error(text || `API error: ${response.status}`);
	}
}

export const api = {
	health: (): Promise<HealthResponse> => fetchJson(`${API_BASE}/health`),

	networks: {
		list: (): Promise<Network[]> => fetchJson(`${API_BASE}/networks`),
		get: (name: string): Promise<Network> =>
			fetchJson(`${API_BASE}/networks/${name}`),
	},

	contracts: {
		list: (): Promise<Contract[]> => fetchJson(`${API_BASE}/contracts`),
		get: (name: string): Promise<Contract> =>
			fetchJson(`${API_BASE}/contracts/${name}`),
	},

	deployments: {
		list: (network?: string): Promise<Deployment[]> => {
			const params = network ? `?network=${encodeURIComponent(network)}` : "";
			return fetchJson(`${API_BASE}/deployments${params}`);
		},
		get: (contract: string, network: string): Promise<Deployment> =>
			fetchJson(`${API_BASE}/deployments/${contract}/${network}`),
		getFunctions: (id: number): Promise<FunctionsResponse> =>
			fetchJson(`${API_BASE}/deployments/${id}/functions`),
		call: (id: number, request: CallRequest): Promise<CallResponse> =>
			postJson(`${API_BASE}/deployments/${id}/call`, request),
		send: (id: number, request: SendRequest): Promise<SendResponse> =>
			postJson(`${API_BASE}/deployments/${id}/send`, request),
		getHistory: (id: number): Promise<CallHistory[]> =>
			fetchJson(`${API_BASE}/deployments/${id}/history`),
	},

	wallets: {
		list: (): Promise<Wallet[]> => fetchJson(`${API_BASE}/wallets`),
		get: (name: string): Promise<Wallet> =>
			fetchJson(`${API_BASE}/wallets/${name}`),
		create: (name: string, privateKey: string): Promise<Wallet> =>
			postJson(`${API_BASE}/wallets`, { name, private_key: privateKey }),
		remove: (name: string): Promise<void> =>
			deleteRequest(`${API_BASE}/wallets/${name}`),
	},

	artifacts: {
		list: (): Promise<ArtifactInfo[]> => fetchJson(`${API_BASE}/artifacts`),
		get: (name: string): Promise<ArtifactDetails> =>
			fetchJson(`${API_BASE}/artifacts/${encodeURIComponent(name)}`),
	},

	deploy: (request: DeployRequest): Promise<DeployResponse> =>
		postJson(`${API_BASE}/deploy`, request),
};
