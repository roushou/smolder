import { Link } from "@tanstack/react-router";
import { useCallback, useEffect, useState } from "react";
import { api } from "../api/client";
import type { ArtifactInfo, Deployment, Network } from "../api/types";
import { DeployModal } from "../components/deploy-modal";
import {
	CardSkeleton,
	EmptyState,
	ErrorState,
	StatCard,
} from "../components/ui";
import { formatRelativeDate, truncateAddress } from "../lib/format";

type FilterType = "all" | "deployed" | "available";

export function Contracts() {
	const [artifacts, setArtifacts] = useState<ArtifactInfo[]>([]);
	const [deployments, setDeployments] = useState<Deployment[]>([]);
	const [networks, setNetworks] = useState<Network[]>([]);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);
	const [filter, setFilter] = useState<FilterType>("all");
	const [deployingArtifact, setDeployingArtifact] = useState<string | null>(
		null,
	);

	const loadData = useCallback(async () => {
		try {
			const [artifactsData, deploymentsData, networksData] = await Promise.all([
				api.artifacts.list(),
				api.deployments.list(),
				api.networks.list(),
			]);
			setArtifacts(artifactsData);
			setDeployments(deploymentsData);
			setNetworks(networksData);
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

	// Group deployments by contract name
	const deploymentsByContract = deployments.reduce(
		(acc, d) => {
			if (!acc[d.contract_name]) {
				acc[d.contract_name] = [];
			}
			acc[d.contract_name].push(d);
			return acc;
		},
		{} as Record<string, Deployment[]>,
	);

	// Filter artifacts based on selected filter
	const filteredArtifacts = artifacts.filter((artifact) => {
		if (!artifact.has_bytecode) return false; // Only show deployable artifacts
		if (filter === "deployed") return artifact.in_registry;
		if (filter === "available") return !artifact.in_registry;
		return true;
	});

	const deployedCount = artifacts.filter(
		(a) => a.has_bytecode && a.in_registry,
	).length;
	const availableCount = artifacts.filter(
		(a) => a.has_bytecode && !a.in_registry,
	).length;

	const handleDeploySuccess = (_result: {
		address: string;
		txHash: string;
		network: string;
	}) => {
		setDeployingArtifact(null);
		// Reload data to show the new deployment
		setLoading(true);
		loadData();
	};

	if (error) {
		return (
			<ErrorState
				title="Unable to connect"
				message={error}
				hint={
					<p className="text-sm text-text-muted">
						Start the server with{" "}
						<code className="font-mono text-accent">smolder serve</code>
					</p>
				}
			/>
		);
	}

	return (
		<div className="mx-auto max-w-6xl animate-fade-in px-6 py-8">
			{/* Page header */}
			<div className="mb-8">
				<h1 className="mb-2 font-semibold text-2xl text-text tracking-tight">
					Contracts
				</h1>
				<p className="text-text-secondary">
					Manage and deploy your smart contracts across networks
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

			{/* Filter tabs */}
			<div className="mb-6">
				<div className="flex items-center gap-2">
					<FilterButton
						active={filter === "all"}
						onClick={() => setFilter("all")}
						count={artifacts.filter((a) => a.has_bytecode).length}
					>
						All
					</FilterButton>
					<FilterButton
						active={filter === "deployed"}
						onClick={() => setFilter("deployed")}
						count={deployedCount}
					>
						Deployed
					</FilterButton>
					<FilterButton
						active={filter === "available"}
						onClick={() => setFilter("available")}
						count={availableCount}
					>
						Available
					</FilterButton>
				</div>
			</div>

			{/* Contracts list */}
			{loading ? (
				<CardSkeleton count={3} />
			) : filteredArtifacts.length === 0 ? (
				<EmptyState
					icon={
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
								d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
							/>
						</svg>
					}
					title={
						filter === "deployed"
							? "No deployed contracts"
							: filter === "available"
								? "No contracts available to deploy"
								: "No contracts found"
					}
					description={
						filter === "deployed"
							? "Deploy your first contract to see it here"
							: "Compile your contracts with forge build"
					}
					action={
						filter !== "deployed" ? (
							<div className="inline-flex items-center gap-2 rounded-lg border border-border bg-bg-surface px-4 py-2.5">
								<code className="text-sm text-text-secondary">
									<span className="text-text-muted">$</span>{" "}
									<span className="text-accent">forge build</span>
								</code>
							</div>
						) : undefined
					}
				/>
			) : (
				<div className="space-y-3">
					{filteredArtifacts.map((artifact) => (
						<ContractCard
							key={artifact.name}
							artifact={artifact}
							deployments={deploymentsByContract[artifact.name] || []}
							onDeploy={() => setDeployingArtifact(artifact.name)}
						/>
					))}
				</div>
			)}

			{/* Deploy Modal */}
			{deployingArtifact && (
				<DeployModal
					artifactName={deployingArtifact}
					onClose={() => setDeployingArtifact(null)}
					onSuccess={handleDeploySuccess}
				/>
			)}
		</div>
	);
}

function FilterButton({
	active,
	onClick,
	count,
	children,
}: {
	active: boolean;
	onClick: () => void;
	count: number;
	children: React.ReactNode;
}) {
	return (
		<button
			type="button"
			onClick={onClick}
			className={`flex items-center gap-2 rounded-full px-3 py-1.5 font-medium text-sm transition-all ${
				active
					? "bg-accent text-bg-base"
					: "bg-bg-surface text-text-secondary hover:bg-bg-muted hover:text-text"
			}`}
		>
			{children}
			<span
				className={`rounded-full px-1.5 py-0.5 text-xs ${
					active ? "bg-bg-base/20 text-bg-base" : "bg-bg-muted text-text-muted"
				}`}
			>
				{count}
			</span>
		</button>
	);
}

function ContractCard({
	artifact,
	deployments,
	onDeploy,
}: {
	artifact: ArtifactInfo;
	deployments: Deployment[];
	onDeploy: () => void;
}) {
	const [expanded, setExpanded] = useState(false);
	const isDeployed = deployments.length > 0;

	return (
		<div className="rounded-xl border border-border bg-bg-elevated">
			{/* Main card content */}
			<div className="p-5">
				<div className="flex items-start gap-4">
					{/* Contract icon */}
					<div
						className={`flex h-10 w-10 shrink-0 items-center justify-center rounded-lg ${
							isDeployed ? "bg-accent/10" : "bg-bg-muted"
						}`}
					>
						<svg
							aria-hidden="true"
							className={`h-5 w-5 ${isDeployed ? "text-accent" : "text-text-muted"}`}
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
						<div className="mb-2 flex items-center gap-3">
							<h3 className="font-medium text-text">{artifact.name}</h3>
							{isDeployed ? (
								<span className="rounded-md bg-success/10 px-2 py-0.5 text-success text-xs">
									{deployments.length} deployment
									{deployments.length !== 1 ? "s" : ""}
								</span>
							) : (
								<span className="rounded-md bg-bg-muted px-2 py-0.5 text-text-muted text-xs">
									Not deployed
								</span>
							)}
						</div>

						<p className="truncate font-mono text-sm text-text-secondary">
							{artifact.source_path}
						</p>
					</div>

					{/* Actions */}
					<div className="flex shrink-0 items-center gap-2">
						<button
							type="button"
							onClick={onDeploy}
							className="rounded-lg border border-accent bg-accent/10 px-3 py-1.5 text-accent text-sm transition-colors hover:bg-accent hover:text-bg-base"
						>
							Deploy
						</button>
						{isDeployed && (
							<button
								type="button"
								onClick={() => setExpanded(!expanded)}
								className="rounded-lg border border-border bg-bg-surface p-2 text-text-muted transition-colors hover:border-accent hover:text-accent"
							>
								<svg
									aria-hidden="true"
									className={`h-4 w-4 transition-transform ${expanded ? "rotate-180" : ""}`}
									fill="none"
									viewBox="0 0 24 24"
									stroke="currentColor"
								>
									<path
										strokeLinecap="round"
										strokeLinejoin="round"
										strokeWidth={2}
										d="M19 9l-7 7-7-7"
									/>
								</svg>
							</button>
						)}
					</div>
				</div>
			</div>

			{/* Expanded content - deployments */}
			{expanded && isDeployed && (
				<div className="border-border border-t bg-bg-surface/50 px-5 py-4">
					<p className="mb-3 font-medium text-sm text-text-muted">
						Deployments
					</p>
					<div className="space-y-2">
						{deployments.map((deployment) => (
							<Link
								key={deployment.id}
								to="/deployment/$contract/$network"
								params={{
									contract: deployment.contract_name,
									network: deployment.network_name,
								}}
								className="flex items-center justify-between rounded-lg border border-border bg-bg-elevated p-3 transition-colors hover:border-accent"
							>
								<div className="flex items-center gap-3">
									<span className="flex items-center gap-1.5 text-sm text-text-secondary">
										<span className="h-1.5 w-1.5 rounded-full bg-success" />
										{deployment.network_name}
									</span>
									<span className="font-mono text-sm text-text-muted">
										{truncateAddress(deployment.address)}
									</span>
								</div>
								<div className="flex items-center gap-2">
									<span className="rounded-md bg-bg-muted px-2 py-0.5 text-text-muted text-xs">
										v{deployment.version}
									</span>
									<span className="text-text-faint text-xs">
										{formatRelativeDate(deployment.deployed_at)}
									</span>
								</div>
							</Link>
						))}
					</div>
				</div>
			)}
		</div>
	);
}
