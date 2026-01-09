import {
	createRootRoute,
	createRoute,
	createRouter,
	Link,
	Outlet,
} from "@tanstack/react-router";
import { Contracts } from "./pages/contracts";
import { DeploymentDetail } from "./pages/deployment-details";
import { Networks } from "./pages/networks";
import { NotFound } from "./pages/not-found";
import { Wallets } from "./pages/wallets";

function RootLayout() {
	return (
		<div className="flex min-h-screen flex-col bg-bg-base">
			{/* Header */}
			<header className="sticky top-0 z-50 border-border-subtle border-b bg-bg-base/80 backdrop-blur-xl">
				<div className="mx-auto max-w-6xl px-6">
					<div className="flex h-16 items-center justify-between">
						<Link to="/" className="group flex items-center gap-3">
							{/* Logo mark */}
							<div className="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-accent to-accent-hover">
								<svg
									aria-hidden="true"
									className="h-4 w-4 text-bg-base"
									fill="none"
									viewBox="0 0 24 24"
									stroke="currentColor"
									strokeWidth={2.5}
								>
									<path
										strokeLinecap="round"
										strokeLinejoin="round"
										d="M17.657 18.657A8 8 0 016.343 7.343S7 9 9 10c0-2 .5-5 2.986-7C14 5 16.09 5.777 17.656 7.343A7.975 7.975 0 0120 13a7.975 7.975 0 01-2.343 5.657z"
									/>
									<path
										strokeLinecap="round"
										strokeLinejoin="round"
										d="M9.879 16.121A3 3 0 1012.015 11L11 14H9c0 .768.293 1.536.879 2.121z"
									/>
								</svg>
							</div>
							<span className="font-semibold text-lg text-text tracking-tight transition-colors group-hover:text-accent">
								Smolder
							</span>
						</Link>

						<nav className="flex items-center gap-6">
							<Link
								to="/"
								className="text-sm text-text-muted transition-colors hover:text-text [&.active]:text-accent"
							>
								Contracts
							</Link>
							<Link
								to="/networks"
								className="text-sm text-text-muted transition-colors hover:text-text [&.active]:text-accent"
							>
								Networks
							</Link>
							<Link
								to="/wallets"
								className="text-sm text-text-muted transition-colors hover:text-text [&.active]:text-accent"
							>
								Wallets
							</Link>
						</nav>
					</div>
				</div>
			</header>

			{/* Main content */}
			<main className="flex-1">
				<Outlet />
			</main>

			{/* Footer */}
			<footer className="border-border-subtle border-t py-6">
				<div className="mx-auto max-w-6xl px-6">
					<p className="text-center text-text-faint text-xs">Smolder</p>
				</div>
			</footer>
		</div>
	);
}

// Root layout
const rootRoute = createRootRoute({
	component: RootLayout,
	notFoundComponent: NotFound,
});

// Routes
const indexRoute = createRoute({
	getParentRoute: () => rootRoute,
	path: "/",
	component: Contracts,
});

const deploymentRoute = createRoute({
	getParentRoute: () => rootRoute,
	path: "/deployment/$contract/$network",
	component: DeploymentDetail,
});

const networksRoute = createRoute({
	getParentRoute: () => rootRoute,
	path: "/networks",
	component: Networks,
});

const walletsRoute = createRoute({
	getParentRoute: () => rootRoute,
	path: "/wallets",
	component: Wallets,
});

// Build route tree
const routeTree = rootRoute.addChildren([
	indexRoute,
	deploymentRoute,
	networksRoute,
	walletsRoute,
]);

// Create router
export const router = createRouter({ routeTree });

// Register router for type safety
declare module "@tanstack/react-router" {
	interface Register {
		router: typeof router;
	}
}
