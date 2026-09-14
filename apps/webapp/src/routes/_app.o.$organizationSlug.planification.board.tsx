import { createFileRoute } from '@tanstack/react-router'

/**
 * Mount point for the board, created ahead of the screen itself (#468) so
 * WS5 (#466) has a stable address to land on and every workstream after it
 * can link here without waiting.
 *
 * Deliberately absent from `modules/registry.ts`: nothing in the navigation
 * points here until the board exists.
 */
export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planification/board',
)({
	component: BoardPlaceholder,
})

function BoardPlaceholder() {
	return (
		<div className="flex flex-1 items-center justify-center p-8 text-muted-foreground text-sm">
			Le tableau arrive bientôt.
		</div>
	)
}
