interface ErrorStateProps {
	title: string;
	message: string;
	hint?: React.ReactNode;
}

export function ErrorState({ title, message, hint }: ErrorStateProps) {
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
				<h2 className="mb-2 font-medium text-lg text-text">{title}</h2>
				<p className="mb-6 max-w-sm text-sm text-text-secondary">{message}</p>
				{hint && (
					<div className="rounded-lg border border-border bg-bg-surface px-4 py-3">
						{hint}
					</div>
				)}
			</div>
		</div>
	);
}
