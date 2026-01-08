import { Link } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { api } from "../api/client";
import type { Deployment, Network } from "../api/types";

export function Dashboard() {
	const [networks, setNetworks] = useState<Network[]>([]);
	const [deployments, setDeployments] = useState<Deployment[]>([]);
	const [selectedNetwork, setSelectedNetwork] = useState<string | null>(null);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);

	useEffect(() => {
		async function loadData() {
			try {
				const [networksData, deploymentsData] = await Promise.all([
					api.networks.list(),
					api.deployments.list(selectedNetwork ?? undefined),
				]);
				setNetworks(networksData);
				setDeployments(deploymentsData);
				setError(null);
			} catch (e) {
				setError(e instanceof Error ? e.message : "Failed to load data");
			} finally {
				setLoading(false);
			}
		}
		loadData();
	}, [selectedNetwork]);

	const handleNetworkSelect = (networkName: string | null) => {
		setSelectedNetwork(networkName);
		setLoading(true);
	};

	if (error) {
		return (
			<div className="mx-auto max-w-6xl px-6 py-20">
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
						Unable to connect
					</h2>
					<p className="mb-6 max-w-sm text-sm text-text-secondary">{error}</p>
					<div className="rounded-lg border border-border bg-bg-surface px-4 py-3">
						<p className="text-sm text-text-muted">
							Start the server with{" "}
							<code className="font-mono text-accent">smolder serve</code>
						</p>
					</div>
				</div>
			</div>
		);
	}

	return (
		<div className="mx-auto max-w-6xl animate-fade-in px-6 py-8">
			{/* Page header */}
			<div className="mb-8">
				<h1 className="mb-2 font-semibold text-2xl text-text tracking-tight">
					Deployments
				</h1>
				<p className="text-text-secondary">
					Track and manage your smart contract deployments across networks
				</p>
			</div>

			{/* Stats cards */}
			<div className="mb-8 grid grid-cols-3 gap-4">
				<StatCard
					label="Total Deployments"
					value={deployments.length}
					loading={loading}
				/>
				<StatCard label="Networks" value={networks.length} loading={loading} />
				<StatCard
					label="Contracts"
					value={new Set(deployments.map((d) => d.contract_name)).size}
					loading={loading}
				/>
			</div>

			{/* Network filter */}
			{networks.length > 0 && (
				<div className="mb-6">
					<div className="flex flex-wrap items-center gap-2">
						<button
							type="button"
							onClick={() => handleNetworkSelect(null)}
							className={`rounded-full px-3 py-1.5 font-medium text-sm transition-all ${
								selectedNetwork === null
									? "bg-accent text-bg-base"
									: "bg-bg-surface text-text-secondary hover:bg-bg-muted hover:text-text"
							}`}
						>
							All networks
						</button>
						{networks.map((network) => (
							<button
								type="button"
								key={network.id}
								onClick={() => handleNetworkSelect(network.name)}
								className={`flex items-center gap-2 rounded-full px-3 py-1.5 font-medium text-sm transition-all ${
									selectedNetwork === network.name
										? "bg-accent text-bg-base"
										: "bg-bg-surface text-text-secondary hover:bg-bg-muted hover:text-text"
								}`}
							>
								<span
									className={`h-1.5 w-1.5 rounded-full ${
										selectedNetwork === network.name
											? "bg-bg-base"
											: "bg-success"
									}`}
								/>
								{network.name}
							</button>
						))}
					</div>
				</div>
			)}

			{/* Deployments list */}
			<DeploymentsList deployments={deployments} loading={loading} />
		</div>
	);
}

function StatCard({
	label,
	value,
	loading,
}: {
	label: string;
	value: number;
	loading: boolean;
}) {
	return (
		<div className="rounded-xl border border-border bg-bg-elevated p-5">
			<p className="mb-1 text-sm text-text-muted">{label}</p>
			{loading ? (
				<div className="h-8 w-16 animate-pulse rounded bg-bg-muted" />
			) : (
				<p className="font-semibold text-2xl text-text">{value}</p>
			)}
		</div>
	);
}

function DeploymentsList({
	deployments,
	loading,
}: {
	deployments: Deployment[];
	loading: boolean;
}) {
	if (loading) {
		return (
			<div className="space-y-3">
				{["skeleton-1", "skeleton-2", "skeleton-3"].map((key) => (
					<div
						key={key}
						className="animate-pulse rounded-xl border border-border bg-bg-elevated p-5"
					>
						<div className="flex items-center gap-4">
							<div className="h-10 w-10 rounded-lg bg-bg-muted" />
							<div className="flex-1">
								<div className="mb-2 h-4 w-32 rounded bg-bg-muted" />
								<div className="h-3 w-48 rounded bg-bg-muted" />
							</div>
						</div>
					</div>
				))}
			</div>
		);
	}

	if (deployments.length === 0) {
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
							d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"
						/>
					</svg>
				</div>
				<h3 className="mb-2 font-medium text-lg text-text">
					No deployments yet
				</h3>
				<p className="mb-6 text-sm text-text-secondary">
					Deploy your first contract to see it here
				</p>
				<div className="inline-flex items-center gap-2 rounded-lg border border-border bg-bg-surface px-4 py-2.5">
					<code className="text-sm text-text-secondary">
						<span className="text-text-muted">$</span>{" "}
						<span className="text-accent">smolder deploy</span>{" "}
						<span className="text-text-muted">
							script/Deploy.s.sol --network mainnet
						</span>
					</code>
				</div>
			</div>
		);
	}

	return (
		<div className="space-y-3">
			{deployments.map((deployment) => (
				<DeploymentCard key={deployment.id} deployment={deployment} />
			))}
		</div>
	);
}

function DeploymentCard({ deployment }: { deployment: Deployment }) {
	const [copied, setCopied] = useState(false);

	const copyAddress = () => {
		navigator.clipboard.writeText(deployment.address);
		setCopied(true);
		setTimeout(() => setCopied(false), 2000);
	};

	return (
		<Link
			to="/deployment/$contract/$network"
			params={{
				contract: deployment.contract_name,
				network: deployment.network_name,
			}}
			className="group block rounded-xl border border-border bg-bg-elevated p-5 transition-all hover:border-border hover:bg-bg-surface"
		>
			<div className="flex items-center gap-4">
				{/* Contract icon */}
				<div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-accent/10">
					<svg
						aria-hidden="true"
						className="h-5 w-5 text-accent"
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

				{/* Contract info */}
				<div className="min-w-0 flex-1">
					<div className="mb-1 flex items-center gap-3">
						<h3 className="font-medium text-text transition-colors group-hover:text-accent">
							{deployment.contract_name}
						</h3>
						<span className="rounded-md bg-bg-muted px-2 py-0.5 font-medium text-text-muted text-xs">
							v{deployment.version}
						</span>
					</div>
					<div className="flex items-center gap-3 text-sm">
						<span className="flex items-center gap-1.5 text-text-secondary">
							<span className="h-1.5 w-1.5 rounded-full bg-success" />
							{deployment.network_name}
						</span>
						<span className="text-text-faint">·</span>
						<button
							type="button"
							onClick={(e) => {
								e.preventDefault();
								copyAddress();
							}}
							className="font-mono text-text-muted transition-colors hover:text-accent"
						>
							{copied ? "Copied!" : truncateAddress(deployment.address)}
						</button>
					</div>
				</div>

				{/* Metadata */}
				<div className="shrink-0 text-right">
					<p className="text-sm text-text-secondary">
						{formatDate(deployment.deployed_at)}
					</p>
					{deployment.block_number && (
						<p className="font-mono text-text-faint text-xs">
							Block {deployment.block_number.toLocaleString()}
						</p>
					)}
				</div>

				{/* Arrow */}
				<svg
					aria-hidden="true"
					className="h-5 w-5 shrink-0 text-text-faint transition-all group-hover:translate-x-0.5 group-hover:text-accent"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						strokeLinecap="round"
						strokeLinejoin="round"
						strokeWidth={1.5}
						d="M9 5l7 7-7 7"
					/>
				</svg>
			</div>
		</Link>
	);
}

function truncateAddress(address: string): string {
	if (address.length <= 13) return address;
	return `${address.slice(0, 6)}...${address.slice(-4)}`;
}

function formatDate(dateString: string): string {
	const date = new Date(dateString);
	const now = new Date();
	const diffMs = now.getTime() - date.getTime();
	const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

	if (diffDays === 0) {
		const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
		if (diffHours === 0) {
			const diffMins = Math.floor(diffMs / (1000 * 60));
			return diffMins <= 1 ? "Just now" : `${diffMins}m ago`;
		}
		return `${diffHours}h ago`;
	}
	if (diffDays === 1) return "Yesterday";
	if (diffDays < 7) return `${diffDays}d ago`;

	return date.toLocaleDateString("en-US", {
		month: "short",
		day: "numeric",
	});
}
