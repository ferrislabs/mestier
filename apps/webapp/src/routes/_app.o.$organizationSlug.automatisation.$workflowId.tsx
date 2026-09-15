import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/automatisation/$workflowId',
)({
	component: WorkflowPlaceholder,
})

function WorkflowPlaceholder() {
	return (
		<div className="flex min-h-0 flex-1 flex-col">
			<div className="p-8 text-center text-muted-foreground text-sm">
				L’éditeur de workflow arrive bientôt.
			</div>
			<Outlet />
		</div>
	)
}
