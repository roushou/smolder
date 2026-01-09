import { useCallback, useEffect, useState } from "react";
import { api } from "../api/client";
import type { Wallet } from "../api/types";
import { CardSkeleton, EmptyState, ErrorState } from "../components/ui";
import { formatDate } from "../lib/format";

export function Wallets() {
	const [wallets, setWallets] = useState<Wallet[]>([]);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);
	const [showAddForm, setShowAddForm] = useState(false);

	const loadWallets = useCallback(async () => {
		try {
			const data = await api.wallets.list();
			setWallets(data);
			setError(null);
		} catch (e) {
			setError(e instanceof Error ? e.message : "Failed to load wallets");
		} finally {
			setLoading(false);
		}
	}, []);

	useEffect(() => {
		loadWallets();
	}, [loadWallets]);

	const handleWalletAdded = () => {
		setShowAddForm(false);
		setLoading(true);
		loadWallets();
	};

	const handleWalletRemoved = () => {
		setLoading(true);
		loadWallets();
	};

	if (error) {
		return (
			<ErrorState
				title="Unable to load wallets"
				message={error}
				hint={
					<button
						type="button"
						onClick={() => {
							setLoading(true);
							loadWallets();
						}}
						className="rounded-lg bg-accent px-4 py-2 font-medium text-bg-base text-sm transition-colors hover:bg-accent-hover"
					>
						Retry
					</button>
				}
			/>
		);
	}

	return (
		<div className="mx-auto max-w-4xl animate-fade-in px-6 py-8">
			{/* Page header */}
			<div className="mb-8 flex items-center justify-between">
				<div>
					<h1 className="mb-2 font-semibold text-2xl text-text tracking-tight">
						Wallets
					</h1>
					<p className="text-text-secondary">
						Manage wallets for signing transactions
					</p>
				</div>
				<button
					type="button"
					onClick={() => setShowAddForm(true)}
					className="inline-flex items-center gap-2 rounded-lg bg-accent px-4 py-2.5 font-medium text-bg-base text-sm transition-colors hover:bg-accent-hover"
				>
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
							d="M12 4v16m8-8H4"
						/>
					</svg>
					Add Wallet
				</button>
			</div>

			{/* Add wallet form */}
			{showAddForm && (
				<AddWalletForm
					onSuccess={handleWalletAdded}
					onCancel={() => setShowAddForm(false)}
				/>
			)}

			{/* Wallets list */}
			<WalletsList
				wallets={wallets}
				loading={loading}
				onRemove={handleWalletRemoved}
			/>
		</div>
	);
}

function AddWalletForm({
	onSuccess,
	onCancel,
}: {
	onSuccess: () => void;
	onCancel: () => void;
}) {
	const [name, setName] = useState("");
	const [privateKey, setPrivateKey] = useState("");
	const [submitting, setSubmitting] = useState(false);
	const [error, setError] = useState<string | null>(null);

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault();
		setError(null);
		setSubmitting(true);

		try {
			await api.wallets.create(name.trim(), privateKey.trim());
			onSuccess();
		} catch (err) {
			setError(err instanceof Error ? err.message : "Failed to create wallet");
		} finally {
			setSubmitting(false);
		}
	};

	return (
		<div className="mb-6 overflow-hidden rounded-xl border border-border bg-bg-elevated">
			<div className="border-border border-b px-5 py-4">
				<h2 className="font-medium text-sm text-text">Add New Wallet</h2>
			</div>
			<form onSubmit={handleSubmit} className="p-5">
				{error && (
					<div className="mb-4 rounded-lg bg-error/10 px-4 py-3 text-error text-sm">
						{error}
					</div>
				)}
				<div className="space-y-4">
					<div>
						<label
							htmlFor="wallet-name"
							className="mb-1.5 block text-sm text-text-secondary"
						>
							Wallet Name
						</label>
						<input
							id="wallet-name"
							type="text"
							value={name}
							onChange={(e) => setName(e.target.value)}
							placeholder="e.g., deployer, treasury"
							className="w-full rounded-lg border border-border bg-bg-surface px-4 py-2.5 text-sm text-text placeholder-text-faint focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent"
							required
						/>
					</div>
					<div>
						<label
							htmlFor="private-key"
							className="mb-1.5 block text-sm text-text-secondary"
						>
							Private Key
						</label>
						<input
							id="private-key"
							type="password"
							value={privateKey}
							onChange={(e) => setPrivateKey(e.target.value)}
							placeholder="0x..."
							className="w-full rounded-lg border border-border bg-bg-surface px-4 py-2.5 font-mono text-sm text-text placeholder-text-faint focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent"
							required
						/>
						<p className="mt-1.5 text-text-faint text-xs">
							Your private key will be securely stored in the system keychain
						</p>
					</div>
				</div>
				<div className="mt-6 flex items-center justify-end gap-3">
					<button
						type="button"
						onClick={onCancel}
						className="rounded-lg border border-border bg-bg-surface px-4 py-2 font-medium text-sm text-text-secondary transition-colors hover:bg-bg-muted hover:text-text"
					>
						Cancel
					</button>
					<button
						type="submit"
						disabled={submitting || !name.trim() || !privateKey.trim()}
						className="rounded-lg bg-accent px-4 py-2 font-medium text-bg-base text-sm transition-colors hover:bg-accent-hover disabled:cursor-not-allowed disabled:opacity-50"
					>
						{submitting ? "Adding..." : "Add Wallet"}
					</button>
				</div>
			</form>
		</div>
	);
}

function WalletsList({
	wallets,
	loading,
	onRemove,
}: {
	wallets: Wallet[];
	loading: boolean;
	onRemove: () => void;
}) {
	if (loading) {
		return <CardSkeleton count={2} />;
	}

	if (wallets.length === 0) {
		return (
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
							d="M3 10h18M7 15h1m4 0h1m-7 4h12a3 3 0 003-3V8a3 3 0 00-3-3H6a3 3 0 00-3 3v8a3 3 0 003 3z"
						/>
					</svg>
				}
				title="No wallets yet"
				description="Add a wallet to start signing transactions"
			/>
		);
	}

	return (
		<div className="space-y-3">
			{wallets.map((wallet) => (
				<WalletCard key={wallet.id} wallet={wallet} onRemove={onRemove} />
			))}
		</div>
	);
}

function WalletCard({
	wallet,
	onRemove,
}: {
	wallet: Wallet;
	onRemove: () => void;
}) {
	const [copied, setCopied] = useState(false);
	const [showConfirm, setShowConfirm] = useState(false);
	const [removing, setRemoving] = useState(false);

	const copyAddress = () => {
		navigator.clipboard.writeText(wallet.address);
		setCopied(true);
		setTimeout(() => setCopied(false), 2000);
	};

	const handleRemove = async () => {
		setRemoving(true);
		try {
			await api.wallets.remove(wallet.name);
			onRemove();
		} catch (err) {
			console.error("Failed to remove wallet:", err);
			setRemoving(false);
			setShowConfirm(false);
		}
	};

	return (
		<div className="rounded-xl border border-border bg-bg-elevated p-5">
			<div className="flex items-center gap-4">
				{/* Wallet icon */}
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
							d="M3 10h18M7 15h1m4 0h1m-7 4h12a3 3 0 003-3V8a3 3 0 00-3-3H6a3 3 0 00-3 3v8a3 3 0 003 3z"
						/>
					</svg>
				</div>

				{/* Wallet info */}
				<div className="min-w-0 flex-1">
					<h3 className="font-medium text-text">{wallet.name}</h3>
					<button
						type="button"
						onClick={copyAddress}
						className="font-mono text-sm text-text-muted transition-colors hover:text-accent"
					>
						{copied ? "Copied!" : wallet.address}
					</button>
				</div>

				{/* Actions */}
				{showConfirm ? (
					<div className="flex items-center gap-2">
						<span className="text-sm text-text-secondary">Remove?</span>
						<button
							type="button"
							onClick={() => setShowConfirm(false)}
							disabled={removing}
							className="rounded-lg border border-border bg-bg-surface px-3 py-1.5 font-medium text-sm text-text-secondary transition-colors hover:bg-bg-muted"
						>
							Cancel
						</button>
						<button
							type="button"
							onClick={handleRemove}
							disabled={removing}
							className="rounded-lg bg-error px-3 py-1.5 font-medium text-sm text-white transition-colors hover:bg-error/90 disabled:opacity-50"
						>
							{removing ? "Removing..." : "Remove"}
						</button>
					</div>
				) : (
					<button
						type="button"
						onClick={() => setShowConfirm(true)}
						className="rounded-lg p-2 text-text-faint transition-colors hover:bg-bg-muted hover:text-error"
						title="Remove wallet"
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
								strokeWidth={1.5}
								d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
							/>
						</svg>
					</button>
				)}
			</div>

			{/* Metadata */}
			<div className="mt-3 border-border border-t pt-3">
				<p className="text-text-faint text-xs">
					Added {formatDate(wallet.created_at)}
				</p>
			</div>
		</div>
	);
}
