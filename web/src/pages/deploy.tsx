import { useNavigate } from "@tanstack/react-router";
import { useCallback, useEffect, useState } from "react";
import { api } from "../api/client";
import type {
	ArtifactDetails,
	ArtifactInfo,
	Network,
	Wallet,
} from "../api/types";
import { ParamInput } from "../components/param-input";

export function Deploy() {
	const navigate = useNavigate();
	const [artifacts, setArtifacts] = useState<ArtifactInfo[]>([]);
	const [networks, setNetworks] = useState<Network[]>([]);
	const [wallets, setWallets] = useState<Wallet[]>([]);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);

	const [selectedArtifact, setSelectedArtifact] = useState<string>("");
	const [artifactDetails, setArtifactDetails] =
		useState<ArtifactDetails | null>(null);
	const [selectedNetwork, setSelectedNetwork] = useState<string>("");
	const [selectedWallet, setSelectedWallet] = useState<string>("");
	const [constructorArgs, setConstructorArgs] = useState<
		Record<string, string>
	>({});
	const [value, setValue] = useState<string>("");

	const [deploying, setDeploying] = useState(false);
	const [deployResult, setDeployResult] = useState<{
		success: boolean;
		txHash?: string;
		address?: string;
		deploymentId?: number;
		error?: string;
	} | null>(null);

	const loadData = useCallback(async () => {
		try {
			const [artifactsData, networksData, walletsData] = await Promise.all([
				api.artifacts.list(),
				api.networks.list(),
				api.wallets.list(),
			]);
			setArtifacts(artifactsData);
			setNetworks(networksData);
			setWallets(walletsData);
			setError(null);
		} catch (e) {
			setError(e instanceof Error ? e.message : "Failed to load data");
		} finally {
			setLoading(false);
		}
	}, []);

	useEffect(() => {
		loadData();
	}, [loadData]);

	// Load artifact details when selection changes
	useEffect(() => {
		if (!selectedArtifact) {
			setArtifactDetails(null);
			setConstructorArgs({});
			return;
		}

		api.artifacts
			.get(selectedArtifact)
			.then((details) => {
				setArtifactDetails(details);
				setConstructorArgs({});
			})
			.catch((e) => {
				console.error("Failed to load artifact details:", e);
				setArtifactDetails(null);
			});
	}, [selectedArtifact]);

	const handleArgChange = (name: string, argValue: string) => {
		setConstructorArgs((prev) => ({ ...prev, [name]: argValue }));
	};

	const parseConstructorArgs = (): unknown[] => {
		if (!artifactDetails?.constructor) return [];

		return artifactDetails.constructor.inputs.map((input) => {
			const rawValue = constructorArgs[input.name] ?? "";

			if (rawValue === "") {
				if (input.param_type === "bool") return false;
				if (
					input.param_type.startsWith("uint") ||
					input.param_type.startsWith("int")
				)
					return "0";
				return "";
			}

			if (input.param_type === "bool") {
				return rawValue.toLowerCase() === "true";
			}

			if (
				input.param_type.endsWith("[]") ||
				input.param_type === "tuple" ||
				input.components
			) {
				try {
					return JSON.parse(rawValue);
				} catch {
					return rawValue;
				}
			}

			return rawValue;
		});
	};

	const handleDeploy = async (e: React.FormEvent) => {
		e.preventDefault();
		setDeploying(true);
		setDeployResult(null);

		try {
			const args = parseConstructorArgs();
			const response = await api.deploy({
				artifact_name: selectedArtifact,
				network_name: selectedNetwork,
				wallet_name: selectedWallet,
				constructor_args: args,
				value: value || undefined,
			});

			setDeployResult({
				success: true,
				txHash: response.tx_hash,
				address: response.contract_address ?? undefined,
				deploymentId: response.deployment_id ?? undefined,
			});
		} catch (err) {
			setDeployResult({
				success: false,
				error: err instanceof Error ? err.message : "Deployment failed",
			});
		} finally {
			setDeploying(false);
		}
	};

	const isPayable =
		artifactDetails?.constructor?.state_mutability === "payable";
	const hasConstructorArgs =
		artifactDetails?.constructor?.inputs &&
		artifactDetails.constructor.inputs.length > 0;
	const canDeploy =
		selectedArtifact && selectedNetwork && selectedWallet && !deploying;

	if (loading) {
		return (
			<div className="mx-auto max-w-4xl px-6 py-20">
				<div className="flex flex-col items-center justify-center">
					<div className="mb-4 h-8 w-8 animate-spin rounded-full border-2 border-accent border-t-transparent" />
					<p className="text-sm text-text-muted">Loading...</p>
				</div>
			</div>
		);
	}

	if (error) {
		return (
			<div className="mx-auto max-w-4xl px-6 py-20">
				<div className="flex flex-col items-center justify-center text-center">
					<div className="mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-error/10">
						<svg
							aria-hidden="true"
							className="h-6 w-6 text-error"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							<path
								strokeLinecap="round"
								strokeLinejoin="round"
								strokeWidth={1.5}
								d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
							/>
						</svg>
					</div>
					<h2 className="mb-2 font-medium text-lg text-text">
						Unable to load data
					</h2>
					<p className="mb-6 max-w-sm text-sm text-text-secondary">{error}</p>
					<button
						type="button"
						onClick={() => {
							setLoading(true);
							loadData();
						}}
						className="rounded-lg bg-accent px-4 py-2 font-medium text-bg-base text-sm transition-colors hover:bg-accent-hover"
					>
						Retry
					</button>
				</div>
			</div>
		);
	}

	return (
		<div className="mx-auto max-w-4xl animate-fade-in px-6 py-8">
			{/* Page header */}
			<div className="mb-8">
				<h1 className="mb-2 font-semibold text-2xl text-text tracking-tight">
					Deploy Contract
				</h1>
				<p className="text-text-secondary">
					Deploy a compiled contract to a network
				</p>
			</div>

			{/* Deploy form */}
			<form
				onSubmit={handleDeploy}
				className="overflow-hidden rounded-xl border border-border bg-bg-elevated"
			>
				<div className="space-y-6 p-6">
					{/* Artifact selection */}
					<div>
						<label
							htmlFor="artifact"
							className="mb-1.5 block text-sm text-text-secondary"
						>
							Contract
						</label>
						<select
							id="artifact"
							value={selectedArtifact}
							onChange={(e) => setSelectedArtifact(e.target.value)}
							disabled={deploying}
							className="w-full rounded-lg border border-border bg-bg-surface px-4 py-2.5 text-sm text-text focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent disabled:opacity-50"
							required
						>
							<option value="">Select a contract...</option>
							{artifacts.map((artifact) => (
								<option key={artifact.name} value={artifact.name}>
									{artifact.name} ({artifact.source_path})
									{artifact.in_registry ? " [registered]" : ""}
								</option>
							))}
						</select>
						{artifacts.length === 0 && (
							<p className="mt-1.5 text-text-faint text-xs">
								No compiled artifacts found. Run `forge build` first.
							</p>
						)}
					</div>

					{/* Network selection */}
					<div>
						<label
							htmlFor="network"
							className="mb-1.5 block text-sm text-text-secondary"
						>
							Network
						</label>
						<select
							id="network"
							value={selectedNetwork}
							onChange={(e) => setSelectedNetwork(e.target.value)}
							disabled={deploying}
							className="w-full rounded-lg border border-border bg-bg-surface px-4 py-2.5 text-sm text-text focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent disabled:opacity-50"
							required
						>
							<option value="">Select a network...</option>
							{networks.map((network) => (
								<option key={network.id} value={network.name}>
									{network.name} (Chain ID: {network.chain_id})
								</option>
							))}
						</select>
						{networks.length === 0 && (
							<p className="mt-1.5 text-text-faint text-xs">
								No networks configured. Add a network first.
							</p>
						)}
					</div>

					{/* Wallet selection */}
					<div>
						<label
							htmlFor="wallet"
							className="mb-1.5 block text-sm text-text-secondary"
						>
							Wallet
						</label>
						<select
							id="wallet"
							value={selectedWallet}
							onChange={(e) => setSelectedWallet(e.target.value)}
							disabled={deploying}
							className="w-full rounded-lg border border-border bg-bg-surface px-4 py-2.5 text-sm text-text focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent disabled:opacity-50"
							required
						>
							<option value="">Select a wallet...</option>
							{wallets.map((wallet) => (
								<option key={wallet.id} value={wallet.name}>
									{wallet.name} ({truncateAddress(wallet.address)})
								</option>
							))}
						</select>
						{wallets.length === 0 && (
							<p className="mt-1.5 text-text-faint text-xs">
								No wallets available. Add a wallet first.
							</p>
						)}
					</div>

					{/* Constructor arguments */}
					{hasConstructorArgs && artifactDetails?.constructor && (
						<div>
							<h3 className="mb-3 font-medium text-sm text-text">
								Constructor Arguments
							</h3>
							<div className="space-y-3">
								{artifactDetails.constructor.inputs.map((input) => (
									<div key={input.name}>
										<label
											htmlFor={`arg-${input.name}`}
											className="mb-1.5 flex items-center gap-2 text-sm text-text-secondary"
										>
											<span>{input.name}</span>
											<span className="font-mono text-text-faint text-xs">
												{input.param_type}
											</span>
										</label>
										<ParamInput
											param={{
												name: input.name,
												param_type: input.param_type,
												components: input.components,
											}}
											value={constructorArgs[input.name] ?? ""}
											onChange={(v) => handleArgChange(input.name, v)}
											disabled={deploying}
										/>
									</div>
								))}
							</div>
						</div>
					)}

					{/* Value for payable constructor */}
					{isPayable && (
						<div>
							<label
								htmlFor="value"
								className="mb-1.5 flex items-center gap-2 text-sm text-text-secondary"
							>
								<span>Value</span>
								<span className="font-mono text-text-faint text-xs">wei</span>
							</label>
							<input
								id="value"
								type="text"
								value={value}
								onChange={(e) => setValue(e.target.value)}
								placeholder="0"
								disabled={deploying}
								className="w-full rounded-lg border border-border bg-bg-surface px-4 py-2.5 font-mono text-sm text-text placeholder-text-faint focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent disabled:opacity-50"
							/>
						</div>
					)}
				</div>

				{/* Submit button */}
				<div className="border-border border-t bg-bg-surface px-6 py-4">
					<button
						type="submit"
						disabled={!canDeploy}
						className="w-full rounded-lg bg-accent px-4 py-2.5 font-medium text-bg-base text-sm transition-colors hover:bg-accent-hover disabled:cursor-not-allowed disabled:opacity-50"
					>
						{deploying ? "Deploying..." : "Deploy Contract"}
					</button>
				</div>
			</form>

			{/* Deployment result */}
			{deployResult && (
				<div
					className={`mt-6 overflow-hidden rounded-xl border ${
						deployResult.success
							? "border-success/20 bg-success/5"
							: "border-error/20 bg-error/5"
					}`}
				>
					<div className="p-6">
						{deployResult.error ? (
							<div className="flex items-start gap-3">
								<svg
									aria-hidden="true"
									className="mt-0.5 h-5 w-5 shrink-0 text-error"
									fill="none"
									viewBox="0 0 24 24"
									stroke="currentColor"
								>
									<path
										strokeLinecap="round"
										strokeLinejoin="round"
										strokeWidth={2}
										d="M6 18L18 6M6 6l12 12"
									/>
								</svg>
								<div>
									<h3 className="mb-1 font-medium text-error">
										Deployment Failed
									</h3>
									<p className="break-all font-mono text-error/80 text-sm">
										{deployResult.error}
									</p>
								</div>
							</div>
						) : (
							<div className="flex items-start gap-3">
								<svg
									aria-hidden="true"
									className="mt-0.5 h-5 w-5 shrink-0 text-success"
									fill="none"
									viewBox="0 0 24 24"
									stroke="currentColor"
								>
									<path
										strokeLinecap="round"
										strokeLinejoin="round"
										strokeWidth={2}
										d="M5 13l4 4L19 7"
									/>
								</svg>
								<div className="min-w-0 flex-1">
									<h3 className="mb-3 font-medium text-success">
										Contract Deployed Successfully
									</h3>
									<div className="space-y-2">
										{deployResult.address && (
											<div>
												<p className="mb-1 text-text-secondary text-xs">
													Contract Address
												</p>
												<p className="break-all font-mono text-sm text-text">
													{deployResult.address}
												</p>
											</div>
										)}
										{deployResult.txHash && (
											<div>
												<p className="mb-1 text-text-secondary text-xs">
													Transaction Hash
												</p>
												<p className="break-all font-mono text-sm text-text-muted">
													{deployResult.txHash}
												</p>
											</div>
										)}
									</div>
									{deployResult.deploymentId && (
										<button
											type="button"
											onClick={() =>
												navigate({
													to: "/deployment/$contract/$network",
													params: {
														contract: selectedArtifact,
														network: selectedNetwork,
													},
												})
											}
											className="mt-4 inline-flex items-center gap-2 rounded-lg bg-success/10 px-4 py-2 font-medium text-sm text-success transition-colors hover:bg-success/20"
										>
											View Deployment
											<svg
												aria-hidden="true"
												className="h-4 w-4"
												fill="none"
												viewBox="0 0 24 24"
												stroke="currentColor"
											>
												<path
													strokeLinecap="round"
													strokeLinejoin="round"
													strokeWidth={2}
													d="M13 7l5 5m0 0l-5 5m5-5H6"
												/>
											</svg>
										</button>
									)}
								</div>
							</div>
						)}
					</div>
				</div>
			)}
		</div>
	);
}

function truncateAddress(address: string): string {
	if (address.length <= 13) return address;
	return `${address.slice(0, 6)}...${address.slice(-4)}`;
}
