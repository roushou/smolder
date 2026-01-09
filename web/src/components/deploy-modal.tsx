import { useEffect, useState } from "react";
import { api } from "../api/client";
import type { ArtifactDetails, Network, Wallet } from "../api/types";
import { truncateAddress } from "../lib/format";
import { ParamInput } from "./param-input";

interface DeployModalProps {
	artifactName: string;
	onClose: () => void;
	onSuccess: (result: {
		address: string;
		txHash: string;
		network: string;
	}) => void;
}

export function DeployModal({
	artifactName,
	onClose,
	onSuccess,
}: DeployModalProps) {
	const [networks, setNetworks] = useState<Network[]>([]);
	const [wallets, setWallets] = useState<Wallet[]>([]);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);

	const [artifactDetails, setArtifactDetails] =
		useState<ArtifactDetails | null>(null);
	const [selectedNetwork, setSelectedNetwork] = useState<string>("");
	const [selectedWallet, setSelectedWallet] = useState<string>("");
	const [constructorArgs, setConstructorArgs] = useState<
		Record<string, string>
	>({});
	const [value, setValue] = useState<string>("");

	const [deploying, setDeploying] = useState(false);
	const [deployError, setDeployError] = useState<string | null>(null);

	useEffect(() => {
		async function loadData() {
			try {
				const [networksData, walletsData, details] = await Promise.all([
					api.networks.list(),
					api.wallets.list(),
					api.artifacts.get(artifactName),
				]);
				setNetworks(networksData);
				setWallets(walletsData);
				setArtifactDetails(details);
				setError(null);
			} catch (e) {
				setError(e instanceof Error ? e.message : "Failed to load data");
			} finally {
				setLoading(false);
			}
		}
		loadData();
	}, [artifactName]);

	// Close on escape key
	useEffect(() => {
		const handleEscape = (e: KeyboardEvent) => {
			if (e.key === "Escape" && !deploying) {
				onClose();
			}
		};
		document.addEventListener("keydown", handleEscape);
		return () => document.removeEventListener("keydown", handleEscape);
	}, [onClose, deploying]);

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
		setDeployError(null);

		try {
			const args = parseConstructorArgs();
			const response = await api.deploy({
				artifact_name: artifactName,
				network_name: selectedNetwork,
				wallet_name: selectedWallet,
				constructor_args: args,
				value: value || undefined,
			});

			if (response.contract_address) {
				onSuccess({
					address: response.contract_address,
					txHash: response.tx_hash,
					network: selectedNetwork,
				});
			}
		} catch (err) {
			setDeployError(err instanceof Error ? err.message : "Deployment failed");
		} finally {
			setDeploying(false);
		}
	};

	const isPayable =
		artifactDetails?.constructor?.state_mutability === "payable";
	const hasConstructorArgs =
		artifactDetails?.constructor?.inputs &&
		artifactDetails.constructor.inputs.length > 0;
	const canDeploy = selectedNetwork && selectedWallet && !deploying;

	return (
		<div className="fixed inset-0 z-50 flex items-center justify-center">
			{/* Backdrop */}
			<button
				type="button"
				aria-label="Close modal"
				className="absolute inset-0 bg-bg-base/80 backdrop-blur-sm"
				onClick={() => !deploying && onClose()}
			/>

			{/* Modal */}
			<div className="relative z-10 w-full max-w-lg animate-fade-in rounded-xl border border-border bg-bg-elevated shadow-2xl">
				{/* Header */}
				<div className="flex items-center justify-between border-border border-b px-6 py-4">
					<div>
						<h2 className="font-semibold text-lg text-text">Deploy Contract</h2>
						<p className="text-sm text-text-muted">{artifactName}</p>
					</div>
					<button
						type="button"
						onClick={onClose}
						disabled={deploying}
						className="rounded-lg p-2 text-text-muted transition-colors hover:bg-bg-muted hover:text-text disabled:opacity-50"
					>
						<svg
							aria-hidden="true"
							className="h-5 w-5"
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
					</button>
				</div>

				{/* Content */}
				<div className="max-h-[60vh] overflow-y-auto p-6">
					{loading ? (
						<div className="flex flex-col items-center justify-center py-8">
							<div className="mb-4 h-8 w-8 animate-spin rounded-full border-2 border-accent border-t-transparent" />
							<p className="text-sm text-text-muted">Loading...</p>
						</div>
					) : error ? (
						<div className="flex flex-col items-center justify-center py-8 text-center">
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
							<p className="text-sm text-text-secondary">{error}</p>
						</div>
					) : (
						<form
							id="deploy-form"
							onSubmit={handleDeploy}
							className="space-y-5"
						>
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
										<span className="font-mono text-text-faint text-xs">
											wei
										</span>
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

							{/* Deploy error */}
							{deployError && (
								<div className="rounded-lg border border-error/20 bg-error/5 p-3">
									<p className="text-error text-sm">{deployError}</p>
								</div>
							)}
						</form>
					)}
				</div>

				{/* Footer */}
				{!loading && !error && (
					<div className="flex items-center justify-end gap-3 border-border border-t px-6 py-4">
						<button
							type="button"
							onClick={onClose}
							disabled={deploying}
							className="rounded-lg px-4 py-2 text-sm text-text-secondary transition-colors hover:bg-bg-muted hover:text-text disabled:opacity-50"
						>
							Cancel
						</button>
						<button
							type="submit"
							form="deploy-form"
							disabled={!canDeploy}
							className="rounded-lg bg-accent px-4 py-2 font-medium text-bg-base text-sm transition-colors hover:bg-accent-hover disabled:cursor-not-allowed disabled:opacity-50"
						>
							{deploying ? "Deploying..." : "Deploy"}
						</button>
					</div>
				)}
			</div>
		</div>
	);
}
