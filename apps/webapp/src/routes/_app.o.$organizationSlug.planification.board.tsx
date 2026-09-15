import { createFileRoute } from '@tanstack/react-router'
import { BoardFeature } from '#/pages/planification/feature/board-feature'

/**
 * The board (#466): five columns of cards, drag one to advance it, schedule
 * one without leaving the screen. The route was created by #468 ahead of the
 * screen; WS5 filled it in and registered the section in `modules/registry.ts`.
 */
export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planification/board',
)({
	component: BoardFeature,
})
