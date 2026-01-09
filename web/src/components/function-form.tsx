import { useState } from "react";
import { api } from "../api/client";
import type { FunctionInfo, Wallet } from "../api/types";
import { truncateAddress } from "../lib/format";
import { ParamInput } from "./param-input";

interface FunctionFormProps {
	deploymentId: number;
	func: FunctionInfo;
	wallets: Wallet[];
	isWrite: boolean;
	onTxSent?: (txHash: string) => void;
}

interface CallResult {
	success: boolean;
	data?: unknown;
	txHash?: string;
	error?: string;
}

export function FunctionForm({
	deploymentId,
	func,
	wallets,
	isWrite,
	onTxSent,
}: FunctionFormProps) {
	const [params, setParams] = useState<Record<string, string>>({});
	const [selectedWallet, setSelectedWallet] = useState<string>("");
	const [value, setValue] = useState<string>("");
	const [loading, setLoading] = useState(false);
	const [expanded, setExpanded] = useState(false);
	const [result, setResult] = useState<CallResult | null>(null);

	const handleParamChange = (name: string, paramValue: string) => {
		setParams((prev) => ({ ...prev, [name]: paramValue }));
	};

	const parseParams = (): unknown[] => {
		return func.inputs.map((input) => {
			const rawValue = params[input.name] ?? "";

			// Handle empty values
			if (rawValue === "") {
				if (input.param_type === "bool") return false;
				if (
					input.param_type.startsWith("uint") ||
					input.param_type.startsWith("int")
				)
					return "0";
				return "";
			}

			// Parse based on type
			if (input.param_type === "bool") {
				return rawValue.toLowerCase() === "true";
			}

			// Arrays and tuples: try to parse as JSON
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

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault();
		setLoading(true);
		setResult(null);

		try {
			const parsedParams = parseParams();

			if (isWrite) {
				if (!selectedWallet) {
					throw new Error("Please select a wallet");
				}

				const response = await api.deployments.send(deploymentId, {
					function_name: func.name,
					params: parsedParams,
					wallet_name: selectedWallet,
					value: value || undefined,
				});

				setResult({ success: true, txHash: response.tx_hash });
				onTxSent?.(response.tx_hash);
			} else {
				const response = await api.deployments.call(deploymentId, {
					function_name: func.name,
					params: parsedParams,
				});

				setResult({ success: true, data: response.result });
			}
		} catch (err) {
			setResult({
				success: false,
				error: err instanceof Error ? err.message : "Unknown error",
			});
		} finally {
			setLoading(false);
		}
	};

	const isPayable = func.state_mutability === "payable";
	const hasInputs = func.inputs.length > 0;

	return (
		<div className="rounded-lg border border-border bg-bg-elevated">
			{/* Header - always visible */}
			<button
				type="button"
				onClick={() => setExpanded(!expanded)}
				className="flex w-full items-center justify-between px-4 py-3 text-left transition-colors hover:bg-bg-surface"
			>
				<div className="flex items-center gap-3">
					<span className="font-medium text-sm text-text">{func.name}</span>
					{func.inputs.length > 0 && (
						<span className="text-text-faint text-xs">
							({func.inputs.map((i) => i.param_type).join(", ")})
						</span>
					)}
				</div>
				<div className="flex items-center gap-2">
					{isPayable && (
						<span className="rounded bg-warning/10 px-2 py-0.5 font-medium text-warning text-xs">
							payable
						</span>
					)}
					<span
						className={`rounded px-2 py-0.5 text-xs ${
							isWrite
								? "bg-accent/10 text-accent"
								: "bg-success/10 text-success"
						}`}
					>
						{func.state_mutability}
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

			{/* Form - expandable */}
			{expanded && (
				<form onSubmit={handleSubmit} className="border-border border-t p-4">
					{/* Inputs */}
					{hasInputs && (
						<div className="mb-4 space-y-3">
							{func.inputs.map((input) => (
								<div key={input.name}>
									<label
										htmlFor={`${func.name}-${input.name}`}
										className="mb-1.5 flex items-center gap-2 text-sm text-text-secondary"
									>
										<span>{input.name}</span>
										<span className="font-mono text-text-faint text-xs">
											{input.param_type}
										</span>
									</label>
									<ParamInput
										param={input}
										value={params[input.name] ?? ""}
										onChange={(v) => handleParamChange(input.name, v)}
										disabled={loading}
									/>
								</div>
							))}
						</div>
					)}

					{/* Wallet selection for write functions */}
					{isWrite && (
						<div className="mb-4">
							<label
								htmlFor={`${func.name}-wallet`}
								className="mb-1.5 block text-sm text-text-secondary"
							>
								Wallet
							</label>
							<select
								id={`${func.name}-wallet`}
								value={selectedWallet}
								onChange={(e) => setSelectedWallet(e.target.value)}
								disabled={loading}
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
					)}

					{/* Value input for payable functions */}
					{isPayable && (
						<div className="mb-4">
							<label
								htmlFor={`${func.name}-value`}
								className="mb-1.5 flex items-center gap-2 text-sm text-text-secondary"
							>
								<span>Value</span>
								<span className="font-mono text-text-faint text-xs">wei</span>
							</label>
							<input
								id={`${func.name}-value`}
								type="text"
								value={value}
								onChange={(e) => setValue(e.target.value)}
								placeholder="0"
								disabled={loading}
								className="w-full rounded-lg border border-border bg-bg-surface px-4 py-2.5 font-mono text-sm text-text placeholder-text-faint focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent disabled:opacity-50"
							/>
						</div>
					)}

					{/* Submit button */}
					<button
						type="submit"
						disabled={loading || (isWrite && !selectedWallet)}
						className={`w-full rounded-lg px-4 py-2.5 font-medium text-sm transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
							isWrite
								? "bg-accent text-bg-base hover:bg-accent-hover"
								: "bg-success text-bg-base hover:bg-success/90"
						}`}
					>
						{loading ? "Executing..." : isWrite ? "Send Transaction" : "Call"}
					</button>

					{/* Result display - inline */}
					{result && (
						<div
							className={`mt-4 rounded-lg p-3 ${
								result.success
									? "border border-success/20 bg-success/5"
									: "border border-error/20 bg-error/5"
							}`}
						>
							{result.error ? (
								<div className="flex items-start gap-2">
									<svg
										aria-hidden="true"
										className="mt-0.5 h-4 w-4 shrink-0 text-error"
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
									<p className="break-all font-mono text-error text-xs">
										{result.error}
									</p>
								</div>
							) : result.txHash ? (
								<div className="flex items-start gap-2">
									<svg
										aria-hidden="true"
										className="mt-0.5 h-4 w-4 shrink-0 text-success"
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
									<div className="min-w-0">
										<p className="mb-1 font-medium text-success text-xs">
											Transaction sent
										</p>
										<p className="break-all font-mono text-text-secondary text-xs">
											{result.txHash}
										</p>
									</div>
								</div>
							) : (
								<div className="flex items-start gap-2">
									<svg
										aria-hidden="true"
										className="mt-0.5 h-4 w-4 shrink-0 text-success"
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
										<p className="mb-1 font-medium text-success text-xs">
											Result
										</p>
										<pre className="whitespace-pre-wrap break-all font-mono text-text-secondary text-xs">
											{JSON.stringify(result.data, null, 2)}
										</pre>
									</div>
								</div>
							)}
						</div>
					)}
				</form>
			)}
		</div>
	);
}
