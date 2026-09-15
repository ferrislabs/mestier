import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/automatisation/$workflowId/executions/$runId',
)({
	component: RunDetailPlaceholder,
})

function RunDetailPlaceholder() {
	return (
		<div className="flex flex-1 items-center justify-center p-8 text-muted-foreground text-sm">
			Le détail de l’exécution arrive bientôt.
		</div>
	)
}
