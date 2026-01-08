import type { ParamInfo } from "../api/types";

interface ParamInputProps {
	param: ParamInfo;
	value: string;
	onChange: (value: string) => void;
	disabled?: boolean;
}

export function ParamInput({
	param,
	value,
	onChange,
	disabled,
}: ParamInputProps) {
	const placeholder = getPlaceholder(param.param_type);
	const inputType = getInputType(param.param_type);

	// Handle tuple types with components
	if (param.components && param.components.length > 0) {
		return (
			<TupleInput
				param={param}
				value={value}
				onChange={onChange}
				disabled={disabled}
			/>
		);
	}

	// Handle array types
	if (param.param_type.endsWith("[]")) {
		return (
			<ArrayInput
				param={param}
				value={value}
				onChange={onChange}
				disabled={disabled}
			/>
		);
	}

	// Handle bool type with toggle
	if (param.param_type === "bool") {
		return <BoolInput value={value} onChange={onChange} disabled={disabled} />;
	}

	// Handle bytes type with textarea
	if (param.param_type === "bytes") {
		return (
			<textarea
				value={value}
				onChange={(e) => onChange(e.target.value)}
				placeholder={placeholder}
				disabled={disabled}
				rows={3}
				className="w-full rounded-lg border border-border bg-bg-surface px-4 py-2.5 font-mono text-sm text-text placeholder-text-faint focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent disabled:opacity-50"
			/>
		);
	}

	return (
		<input
			type={inputType}
			value={value}
			onChange={(e) => onChange(e.target.value)}
			placeholder={placeholder}
			disabled={disabled}
			className="w-full rounded-lg border border-border bg-bg-surface px-4 py-2.5 font-mono text-sm text-text placeholder-text-faint focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent disabled:opacity-50"
		/>
	);
}

function BoolInput({
	value,
	onChange,
	disabled,
}: {
	value: string;
	onChange: (value: string) => void;
	disabled?: boolean;
}) {
	const isTrue = value === "true";

	return (
		<button
			type="button"
			onClick={() => onChange(isTrue ? "false" : "true")}
			disabled={disabled}
			className={`relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 focus:ring-offset-bg-base disabled:cursor-not-allowed disabled:opacity-50 ${
				isTrue ? "bg-accent" : "bg-bg-muted"
			}`}
		>
			<span
				className={`pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${
					isTrue ? "translate-x-5" : "translate-x-0"
				}`}
			/>
		</button>
	);
}

function ArrayInput({
	param,
	value,
	onChange,
	disabled,
}: {
	param: ParamInfo;
	value: string;
	onChange: (value: string) => void;
	disabled?: boolean;
}) {
	const baseType = param.param_type.replace("[]", "");
	const placeholder = `[${getPlaceholder(baseType)}, ...]`;

	return (
		<textarea
			value={value}
			onChange={(e) => onChange(e.target.value)}
			placeholder={placeholder}
			disabled={disabled}
			rows={2}
			className="w-full rounded-lg border border-border bg-bg-surface px-4 py-2.5 font-mono text-sm text-text placeholder-text-faint focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent disabled:opacity-50"
		/>
	);
}

function TupleInput({
	param,
	value,
	onChange,
	disabled,
}: {
	param: ParamInfo;
	value: string;
	onChange: (value: string) => void;
	disabled?: boolean;
}) {
	// For tuples, we expect a JSON object as input
	const placeholder = param.components
		? `{ ${param.components.map((c) => `"${c.name}": ${getPlaceholder(c.param_type)}`).join(", ")} }`
		: "{ }";

	return (
		<div className="space-y-2">
			<textarea
				value={value}
				onChange={(e) => onChange(e.target.value)}
				placeholder={placeholder}
				disabled={disabled}
				rows={3}
				className="w-full rounded-lg border border-border bg-bg-surface px-4 py-2.5 font-mono text-sm text-text placeholder-text-faint focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent disabled:opacity-50"
			/>
			{param.components && (
				<div className="rounded-lg bg-bg-surface p-3">
					<p className="mb-2 font-medium text-text-muted text-xs">
						Expected fields:
					</p>
					<div className="space-y-1">
						{param.components.map((comp) => (
							<p
								key={comp.name}
								className="font-mono text-text-secondary text-xs"
							>
								<span className="text-accent">{comp.name}</span>:{" "}
								<span className="text-text-muted">{comp.param_type}</span>
							</p>
						))}
					</div>
				</div>
			)}
		</div>
	);
}

function getPlaceholder(paramType: string): string {
	if (paramType === "address") return "0x...";
	if (paramType.startsWith("uint") || paramType.startsWith("int")) return "0";
	if (paramType === "bool") return "true/false";
	if (paramType.startsWith("bytes32")) return "0x...";
	if (paramType === "bytes") return "0x...";
	if (paramType === "string") return "text";
	return "value";
}

function getInputType(paramType: string): string {
	if (paramType.startsWith("uint") || paramType.startsWith("int")) {
		return "text"; // Use text to allow large numbers
	}
	return "text";
}
