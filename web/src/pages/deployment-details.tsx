import { Link, useParams } from "@tanstack/react-router";
import { useCallback, useEffect, useState } from "react";
import { api } from "../api/client";
import type {
	CallHistory,
	Deployment,
	FunctionsResponse,
	Network,
	Wallet,
} from "../api/types";
import { FunctionForm } from "../components/function-form";
import { formatDateTime, truncateAddress } from "../lib/format";

type Tab = "details" | "interact" | "history" | "versions";
type InteractTab = "read" | "write";

export function DeploymentDetail() {
	const params = useParams({ from: "/deployment/$contract/$network" });
	const { contract, network } = params;
	const [deployment, setDeployment] = useState<Deployment | null>(null);
	const [networkData, setNetworkData] = useState<Network | null>(null);
	const [versions, setVersions] = useState<Deployment[]>([]);
	const [functions, setFunctions] = useState<FunctionsResponse | null>(null);
	const [wallets, setWallets] = useState<Wallet[]>([]);
	const [history, setHistory] = useState<CallHistory[]>([]);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);
	const [activeTab, setActiveTab] = useState<Tab>("details");
	const [interactTab, setInteractTab] = useState<InteractTab>("read");

	const loadHistory = useCallback(async (deploymentId: number) => {
		try {
			const historyData = await api.deployments.getHistory(deploymentId);
			setHistory(historyData);
		} catch {
			// History load failure is non-critical
		}
	}, []);

	useEffect(() => {
		async function loadData() {
			try {
				const deploymentData = await api.deployments.get(contract, network);
				setDeployment(deploymentData);

				// Load functions, wallets, network, and versions in parallel
				const [functionsData, walletsData, networkInfo, versionsData] =
					await Promise.all([
						api.deployments.getFunctions(deploymentData.id),
						api.wallets.list(),
						api.networks.get(network).catch(() => null),
						api.deployments.getVersions(contract, network),
					]);

				setFunctions(functionsData);
				setWallets(walletsData);
				setNetworkData(networkInfo);
				setVersions(versionsData);

				// Load history
				await loadHistory(deploymentData.id);

				setError(null);
			} catch (e) {
				setError(e instanceof Error ? e.message : "Failed to load deployment");
			} finally {
				setLoading(false);
			}
		}
		loadData();
	}, [contract, network, loadHistory]);

	const handleTxSent = () => {
		// Refresh history after a transaction
		if (deployment) {
			setTimeout(() => loadHistory(deployment.id), 2000);
		}
	};

	if (loading) {
		return (
			<div className="mx-auto max-w-4xl px-6 py-8">
				<div className="animate-pulse">
					<div className="mb-8 h-4 w-24 rounded bg-bg-muted" />
					<div className="mb-2 h-8 w-64 rounded bg-bg-muted" />
					<div className="mb-8 h-4 w-48 rounded bg-bg-muted" />
					<div className="space-y-4">
						{[
							"skeleton-1",
							"skeleton-2",
							"skeleton-3",
							"skeleton-4",
							"skeleton-5",
						].map((key) => (
							<div key={key} className="h-12 rounded-lg bg-bg-muted" />
						))}
					</div>
				</div>
			</div>
		);
	}

	if (error || !deployment) {
		return (
			<div className="mx-auto max-w-4xl px-6 py-8">
				<BackLink />
				<div className="rounded-xl border border-border bg-bg-elevated p-12 text-center">
					<div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-error/10">
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
								d="M6 18L18 6M6 6l12 12"
							/>
						</svg>
					</div>
					<h2 className="mb-2 font-medium text-lg text-text">
						Deployment not found
					</h2>
					<p className="text-sm text-text-secondary">{error}</p>
				</div>
			</div>
		);
	}

	return (
		<div className="mx-auto max-w-4xl animate-fade-in px-6 py-8">
			<BackLink />

			{/* Header */}
			<div className="mb-8">
				<div className="mb-3 flex items-center gap-4">
					<div className="flex h-12 w-12 items-center justify-center rounded-xl bg-accent/10">
						<svg
							aria-hidden="true"
							className="h-6 w-6 text-accent"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							<path
								strokeLinecap="round"
								strokeLinejoin="round"
								strokeWidth={1.5}
								d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
							/>
						</svg>
					</div>
					<div>
						<h1 className="font-semibold text-2xl text-text tracking-tight">
							{deployment.contract_name}
						</h1>
						<div className="flex items-center gap-2 text-text-secondary">
							<span className="flex items-center gap-1.5">
								<span className="h-1.5 w-1.5 rounded-full bg-success" />
								{deployment.network_name}
							</span>
							<span className="text-text-faint">·</span>
							<span>Chain ID {deployment.chain_id}</span>
							<span className="text-text-faint">·</span>
							<span className="rounded-md bg-bg-muted px-2 py-0.5 font-medium text-text-muted text-xs">
								v{deployment.version}
							</span>
						</div>
					</div>
				</div>
			</div>

			{/* Tab navigation */}
			<div className="mb-6 flex gap-1 rounded-lg bg-bg-surface p-1">
				<TabButton
					active={activeTab === "details"}
					onClick={() => setActiveTab("details")}
				>
					Details
				</TabButton>
				<TabButton
					active={activeTab === "interact"}
					onClick={() => setActiveTab("interact")}
				>
					Interact
				</TabButton>
				<TabButton
					active={activeTab === "history"}
					onClick={() => setActiveTab("history")}
				>
					History
					{history.length > 0 && (
						<span className="ml-2 rounded-full bg-bg-muted px-2 py-0.5 text-xs">
							{history.length}
						</span>
					)}
				</TabButton>
				<TabButton
					active={activeTab === "versions"}
					onClick={() => setActiveTab("versions")}
				>
					Versions
					{versions.length > 1 && (
						<span className="ml-2 rounded-full bg-bg-muted px-2 py-0.5 text-xs">
							{versions.length}
						</span>
					)}
				</TabButton>
			</div>

			{/* Tab content */}
			{activeTab === "details" && (
				<DetailsTab deployment={deployment} network={networkData} />
			)}

			{activeTab === "interact" && functions && (
				<InteractTab
					deployment={deployment}
					functions={functions}
					wallets={wallets}
					interactTab={interactTab}
					setInteractTab={setInteractTab}
					onTxSent={handleTxSent}
				/>
			)}

			{activeTab === "history" && <HistoryTab history={history} />}

			{activeTab === "versions" && (
				<VersionsTab
					versions={versions}
					currentVersion={deployment.version}
					network={networkData}
				/>
			)}
		</div>
	);
}

function TabButton({
	active,
	onClick,
	children,
}: {
	active: boolean;
	onClick: () => void;
	children: React.ReactNode;
}) {
	return (
		<button
			type="button"
			onClick={onClick}
			className={`flex-1 rounded-md px-4 py-2 font-medium text-sm transition-colors ${
				active
					? "bg-bg-elevated text-text shadow-sm"
					: "text-text-muted hover:text-text"
			}`}
		>
			{children}
		</button>
	);
}

function DetailsTab({
	deployment,
	network,
}: {
	deployment: Deployment;
	network: Network | null;
}) {
	const explorerUrl = network?.explorer_url;

	const getAddressUrl = () =>
		explorerUrl ? `${explorerUrl}/address/${deployment.address}` : null;
	const getTxUrl = () =>
		explorerUrl ? `${explorerUrl}/tx/${deployment.tx_hash}` : null;
	const getBlockUrl = () =>
		explorerUrl && deployment.block_number
			? `${explorerUrl}/block/${deployment.block_number}`
			: null;

	return (
		<div className="space-y-6">
			{/* Primary info */}
			<section className="overflow-hidden rounded-xl border border-border bg-bg-elevated">
				<div className="border-border border-b px-5 py-4">
					<h2 className="font-medium text-sm text-text">Contract Details</h2>
				</div>
				<div className="divide-y divide-border">
					<DetailRow
						label="Address"
						value={deployment.address}
						mono
						copyable
						explorerUrl={getAddressUrl()}
					/>
					<DetailRow
						label="Deployer"
						value={deployment.deployer}
						mono
						copyable
						explorerUrl={
							explorerUrl
								? `${explorerUrl}/address/${deployment.deployer}`
								: null
						}
					/>
					<DetailRow
						label="Transaction Hash"
						value={deployment.tx_hash}
						mono
						copyable
						explorerUrl={getTxUrl()}
					/>
					<DetailRow
						label="Block Number"
						value={deployment.block_number?.toLocaleString() ?? "N/A"}
						mono={!!deployment.block_number}
						explorerUrl={getBlockUrl()}
					/>
					<DetailRow
						label="Deployed At"
						value={formatDateTime(deployment.deployed_at)}
					/>
				</div>
			</section>

			{/* ABI section */}
			<section className="overflow-hidden rounded-xl border border-border bg-bg-elevated">
				<div className="flex items-center justify-between border-border border-b px-5 py-4">
					<h2 className="font-medium text-sm text-text">Contract ABI</h2>
					<CopyButton text={deployment.abi} label="Copy ABI" />
				</div>
				<div className="p-4">
					<pre className="max-h-96 overflow-x-auto overflow-y-auto rounded-lg bg-bg-surface p-4 font-mono text-text-secondary text-xs leading-relaxed">
						{formatAbi(deployment.abi)}
					</pre>
				</div>
			</section>
		</div>
	);
}

function VersionsTab({
	versions,
	currentVersion,
	network,
}: {
	versions: Deployment[];
	currentVersion: number;
	network: Network | null;
}) {
	const explorerUrl = network?.explorer_url;

	if (versions.length === 0) {
		return (
			<div className="rounded-xl border border-border bg-bg-elevated p-12 text-center">
				<p className="text-sm text-text-muted">No version history available</p>
			</div>
		);
	}

	return (
		<div className="overflow-hidden rounded-xl border border-border bg-bg-elevated">
			<div className="divide-y divide-border">
				{versions.map((version) => (
					<div
						key={version.id}
						className={`flex items-center justify-between px-5 py-4 ${
							version.version === currentVersion ? "bg-accent/5" : ""
						}`}
					>
						<div className="flex items-center gap-4">
							<span
								className={`rounded-md px-2 py-1 font-medium text-sm ${
									version.version === currentVersion
										? "bg-accent text-bg-base"
										: "bg-bg-muted text-text-muted"
								}`}
							>
								v{version.version}
							</span>
							<div>
								<div className="flex items-center gap-2">
									<span className="font-mono text-sm text-text-secondary">
										{truncateAddress(version.address)}
									</span>
									{version.is_current && (
										<span className="rounded bg-success/10 px-1.5 py-0.5 text-success text-xs">
											Current
										</span>
									)}
								</div>
								<p className="text-text-faint text-xs">
									{formatDateTime(version.deployed_at)}
								</p>
							</div>
						</div>
						<div className="flex items-center gap-2">
							{explorerUrl && (
								<a
									href={`${explorerUrl}/address/${version.address}`}
									target="_blank"
									rel="noopener noreferrer"
									className="rounded-lg border border-border bg-bg-surface p-2 text-text-muted transition-colors hover:border-accent hover:text-accent"
									title="View on explorer"
								>
									<span className="sr-only">View on explorer</span>
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
											strokeWidth={1.5}
											d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
										/>
									</svg>
								</a>
							)}
							<CopyButton text={version.address} label="Copy" />
						</div>
					</div>
				))}
			</div>
		</div>
	);
}

function InteractTab({
	deployment,
	functions,
	wallets,
	interactTab,
	setInteractTab,
	onTxSent,
}: {
	deployment: Deployment;
	functions: FunctionsResponse;
	wallets: Wallet[];
	interactTab: InteractTab;
	setInteractTab: (tab: InteractTab) => void;
	onTxSent: () => void;
}) {
	const currentFunctions =
		interactTab === "read" ? functions.read : functions.write;

	return (
		<div className="space-y-6">
			{/* Function tabs */}
			<div className="flex gap-2">
				<button
					type="button"
					onClick={() => setInteractTab("read")}
					className={`rounded-full px-4 py-1.5 font-medium text-sm transition-colors ${
						interactTab === "read"
							? "bg-success text-bg-base"
							: "bg-bg-surface text-text-muted hover:text-text"
					}`}
				>
					Read ({functions.read.length})
				</button>
				<button
					type="button"
					onClick={() => setInteractTab("write")}
					className={`rounded-full px-4 py-1.5 font-medium text-sm transition-colors ${
						interactTab === "write"
							? "bg-accent text-bg-base"
							: "bg-bg-surface text-text-muted hover:text-text"
					}`}
				>
					Write ({functions.write.length})
				</button>
			</div>

			{/* Functions list */}
			{currentFunctions.length === 0 ? (
				<div className="rounded-xl border border-border bg-bg-elevated p-8 text-center">
					<p className="text-sm text-text-muted">
						No {interactTab} functions available
					</p>
				</div>
			) : (
				<div className="space-y-3">
					{currentFunctions.map((func) => (
						<FunctionForm
							key={func.signature}
							deploymentId={deployment.id}
							func={func}
							wallets={wallets}
							isWrite={interactTab === "write"}
							onTxSent={onTxSent}
						/>
					))}
				</div>
			)}
		</div>
	);
}

function HistoryTab({ history }: { history: CallHistory[] }) {
	if (history.length === 0) {
		return (
			<div className="rounded-xl border border-border bg-bg-elevated p-12 text-center">
				<div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-bg-muted">
					<svg
						aria-hidden="true"
						className="h-6 w-6 text-text-muted"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
					>
						<path
							strokeLinecap="round"
							strokeLinejoin="round"
							strokeWidth={1.5}
							d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
						/>
					</svg>
				</div>
				<h3 className="mb-2 font-medium text-lg text-text">No history yet</h3>
				<p className="text-sm text-text-secondary">
					Interact with the contract to see call history
				</p>
			</div>
		);
	}

	return (
		<div className="overflow-hidden rounded-xl border border-border bg-bg-elevated">
			<div className="divide-y divide-border">
				{history.map((item) => (
					<HistoryRow key={item.id} item={item} />
				))}
			</div>
		</div>
	);
}

function HistoryRow({ item }: { item: CallHistory }) {
	const [expanded, setExpanded] = useState(false);

	const statusColor = {
		pending: "bg-warning/10 text-warning",
		success: "bg-success/10 text-success",
		failed: "bg-error/10 text-error",
		reverted: "bg-error/10 text-error",
	}[item.status ?? "pending"];

	return (
		<div>
			<button
				type="button"
				onClick={() => setExpanded(!expanded)}
				className="flex w-full items-center justify-between px-5 py-4 text-left transition-colors hover:bg-bg-surface"
			>
				<div className="flex items-center gap-3">
					<span className="font-medium text-sm text-text">
						{item.function_name}
					</span>
					<span
						className={`rounded px-2 py-0.5 text-xs ${
							item.call_type === "write"
								? "bg-accent/10 text-accent"
								: "bg-success/10 text-success"
						}`}
					>
						{item.call_type}
					</span>
					{item.status && (
						<span className={`rounded px-2 py-0.5 text-xs ${statusColor}`}>
							{item.status}
						</span>
					)}
				</div>
				<div className="flex items-center gap-3">
					<span className="text-text-faint text-xs">
						{formatDateTime(item.created_at)}
					</span>
					<svg
						aria-hidden="true"
						className={`h-4 w-4 text-text-faint transition-transform ${expanded ? "rotate-180" : ""}`}
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
					>
						<path
							strokeLinecap="round"
							strokeLinejoin="round"
							strokeWidth={1.5}
							d="M19 9l-7 7-7-7"
						/>
					</svg>
				</div>
			</button>
			{expanded && (
				<div className="border-border border-t bg-bg-surface px-5 py-4">
					<div className="space-y-3">
						<div>
							<p className="mb-1 font-medium text-text-muted text-xs">
								Signature
							</p>
							<p className="font-mono text-text-secondary text-xs">
								{item.function_signature}
							</p>
						</div>
						<div>
							<p className="mb-1 font-medium text-text-muted text-xs">
								Parameters
							</p>
							<pre className="font-mono text-text-secondary text-xs">
								{formatJson(item.input_params)}
							</pre>
						</div>
						{item.result && (
							<div>
								<p className="mb-1 font-medium text-text-muted text-xs">
									Result
								</p>
								<pre className="font-mono text-text-secondary text-xs">
									{formatJson(item.result)}
								</pre>
							</div>
						)}
						{item.tx_hash && (
							<div>
								<p className="mb-1 font-medium text-text-muted text-xs">
									Transaction Hash
								</p>
								<p className="font-mono text-text-secondary text-xs">
									{item.tx_hash}
								</p>
							</div>
						)}
						{item.error_message && (
							<div>
								<p className="mb-1 font-medium text-text-muted text-xs">
									Error
								</p>
								<p className="font-mono text-error text-xs">
									{item.error_message}
								</p>
							</div>
						)}
						{item.wallet_name && (
							<div>
								<p className="mb-1 font-medium text-text-muted text-xs">
									Wallet
								</p>
								<p className="text-text-secondary text-xs">
									{item.wallet_name}
								</p>
							</div>
						)}
						{item.gas_used && (
							<div>
								<p className="mb-1 font-medium text-text-muted text-xs">
									Gas Used
								</p>
								<p className="font-mono text-text-secondary text-xs">
									{item.gas_used.toLocaleString()}
								</p>
							</div>
						)}
					</div>
				</div>
			)}
		</div>
	);
}

function BackLink() {
	return (
		<Link
			to="/"
			className="group mb-6 inline-flex items-center gap-2 text-sm text-text-muted transition-colors hover:text-text"
		>
			<svg
				aria-hidden="true"
				className="h-4 w-4 transition-transform group-hover:-translate-x-0.5"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
			>
				<path
					strokeLinecap="round"
					strokeLinejoin="round"
					strokeWidth={1.5}
					d="M15 19l-7-7 7-7"
				/>
			</svg>
			Back to Dashboard
		</Link>
	);
}

function DetailRow({
	label,
	value,
	mono,
	copyable,
	explorerUrl,
}: {
	label: string;
	value: string;
	mono?: boolean;
	copyable?: boolean;
	explorerUrl?: string | null;
}) {
	const [copied, setCopied] = useState(false);

	const handleCopy = () => {
		navigator.clipboard.writeText(value);
		setCopied(true);
		setTimeout(() => setCopied(false), 2000);
	};

	return (
		<div className="flex items-start justify-between gap-4 px-5 py-4">
			<span className="w-36 shrink-0 text-sm text-text-muted">{label}</span>
			<div className="flex min-w-0 flex-1 items-center justify-end gap-2">
				<span
					className={`break-all text-right text-sm ${mono ? "font-mono text-text-secondary" : "text-text"}`}
				>
					{value}
				</span>
				{explorerUrl && (
					<a
						href={explorerUrl}
						target="_blank"
						rel="noopener noreferrer"
						className="shrink-0 rounded-md p-1.5 text-text-faint transition-colors hover:bg-bg-muted hover:text-accent"
						title="View on explorer"
					>
						<span className="sr-only">View on explorer</span>
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
								strokeWidth={1.5}
								d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
							/>
						</svg>
					</a>
				)}
				{copyable && (
					<button
						type="button"
						onClick={handleCopy}
						className="shrink-0 rounded-md p-1.5 text-text-faint transition-colors hover:bg-bg-muted hover:text-text-secondary"
						title="Copy to clipboard"
					>
						{copied ? (
							<svg
								aria-hidden="true"
								className="h-4 w-4 text-success"
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
						) : (
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
									strokeWidth={1.5}
									d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
								/>
							</svg>
						)}
					</button>
				)}
			</div>
		</div>
	);
}

function CopyButton({ text, label }: { text: string; label: string }) {
	const [copied, setCopied] = useState(false);

	const handleCopy = () => {
		navigator.clipboard.writeText(text);
		setCopied(true);
		setTimeout(() => setCopied(false), 2000);
	};

	return (
		<button
			type="button"
			onClick={handleCopy}
			className="inline-flex items-center gap-2 rounded-lg border border-border bg-bg-surface px-3 py-1.5 font-medium text-sm text-text-secondary transition-colors hover:bg-bg-muted hover:text-text"
		>
			{copied ? (
				<>
					<svg
						aria-hidden="true"
						className="h-4 w-4 text-success"
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
					Copied
				</>
			) : (
				<>
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
							strokeWidth={1.5}
							d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
						/>
					</svg>
					{label}
				</>
			)}
		</button>
	);
}

function formatAbi(abi: string): string {
	try {
		return JSON.stringify(JSON.parse(abi), null, 2);
	} catch {
		return abi;
	}
}

function formatJson(json: string): string {
	try {
		return JSON.stringify(JSON.parse(json), null, 2);
	} catch {
		return json;
	}
}
