import { Link } from "@tanstack/react-router";

export function NotFound() {
	return (
		<div className="mx-auto max-w-6xl px-6 py-20">
			<div className="flex flex-col items-center justify-center text-center">
				<div className="mb-6 flex h-20 w-20 items-center justify-center rounded-full bg-bg-muted">
					<span className="font-bold text-4xl text-text-muted">404</span>
				</div>
				<h1 className="mb-2 font-semibold text-2xl text-text">
					Page not found
				</h1>
				<p className="mb-8 max-w-md text-text-secondary">
					The page you're looking for doesn't exist or has been moved.
				</p>
				<Link
					to="/"
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
							d="M10 19l-7-7m0 0l7-7m-7 7h18"
						/>
					</svg>
					Back to Contracts
				</Link>
			</div>
		</div>
	);
}
