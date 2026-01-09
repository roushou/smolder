import { Link } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { api } from "../api/client";
import type { Network } from "../api/types";
import { CardSkeleton, EmptyState, ErrorState } from "../components/ui";

export function Networks() {
	const [networks, setNetworks] = useState<Network[]>([]);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);

	useEffect(() => {
		async function loadData() {
			try {
				const data = await api.networks.list();
				setNetworks(data);
				setError(null);
			} catch (e) {
				setError(e instanceof Error ? e.message : "Failed to load networks");
			} finally {
				setLoading(false);
			}
		}
		loadData();
	}, []);

	if (error) {
		return <ErrorState title="Unable to load networks" message={error} />;
	}

	return (
		<div className="mx-auto max-w-6xl animate-fade-in px-6 py-8">
			{/* Page header */}
			<div className="mb-8">
				<h1 className="mb-2 font-semibold text-2xl text-text tracking-tight">
					Networks
				</h1>
				<p className="text-text-secondary">
					Configured blockchain networks from your foundry.toml
				</p>
			</div>

			{/* Networks list */}
			{loading ? (
				<CardSkeleton count={2} />
			) : networks.length === 0 ? (
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
								d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9"
							/>
						</svg>
					}
					title="No networks configured"
					description="Add networks to your foundry.toml under [rpc_endpoints]"
				/>
			) : (
				<div className="space-y-3">
					{networks.map((network) => (
						<NetworkCard key={network.id} network={network} />
					))}
				</div>
			)}
		</div>
	);
}

function NetworkCard({ network }: { network: Network }) {
	const [copied, setCopied] = useState<string | null>(null);

	const copyToClipboard = (text: string, field: string) => {
		navigator.clipboard.writeText(text);
		setCopied(field);
		setTimeout(() => setCopied(null), 2000);
	};

	return (
		<div className="rounded-xl border border-border bg-bg-elevated p-5">
			<div className="flex items-start gap-4">
				{/* Network icon */}
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
							d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9"
						/>
					</svg>
				</div>

				{/* Network info */}
				<div className="min-w-0 flex-1">
					<div className="mb-3 flex items-center gap-3">
						<h3 className="font-medium text-text">{network.name}</h3>
						<span className="flex items-center gap-1.5 rounded-md bg-bg-muted px-2 py-0.5 text-xs">
							<span className="h-1.5 w-1.5 rounded-full bg-success" />
							Chain ID: {network.chain_id}
						</span>
					</div>

					<div className="space-y-2">
						{/* RPC URL */}
						<div className="flex items-center gap-2">
							<span className="w-20 shrink-0 text-sm text-text-muted">
								RPC URL
							</span>
							<button
								type="button"
								onClick={() => copyToClipboard(network.rpc_url, "rpc")}
								className="truncate font-mono text-sm text-text-secondary transition-colors hover:text-accent"
							>
								{copied === "rpc" ? "Copied!" : network.rpc_url}
							</button>
						</div>

						{/* Explorer URL */}
						{network.explorer_url && (
							<div className="flex items-center gap-2">
								<span className="w-20 shrink-0 text-sm text-text-muted">
									Explorer
								</span>
								<a
									href={network.explorer_url}
									target="_blank"
									rel="noopener noreferrer"
									className="flex items-center gap-1 truncate font-mono text-accent text-sm transition-colors hover:text-accent-hover"
								>
									{network.explorer_url}
									<svg
										aria-hidden="true"
										className="h-3 w-3 shrink-0"
										fill="none"
										viewBox="0 0 24 24"
										stroke="currentColor"
									>
										<path
											strokeLinecap="round"
											strokeLinejoin="round"
											strokeWidth={2}
											d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
										/>
									</svg>
								</a>
							</div>
						)}
					</div>
				</div>

				{/* View deployments link */}
				<Link
					to="/"
					search={{ network: network.name }}
					className="shrink-0 rounded-lg border border-border bg-bg-surface px-3 py-1.5 text-sm text-text-secondary transition-colors hover:border-accent hover:text-accent"
				>
					View deployments
				</Link>
			</div>
		</div>
	);
}
