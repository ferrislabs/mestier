import { Link } from '@tanstack/react-router'
import { ArrowLeft, SearchX } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { buildOrgPath } from '#/modules/org-path'

export function WorkflowNotFound({
	organizationSlug,
	workflowId,
}: {
	organizationSlug: string
	workflowId: string
}) {
	return (
		<div className="flex h-full flex-col items-center justify-center gap-1 px-6 text-center">
			<SearchX className="mb-2 size-10 text-muted-foreground" aria-hidden />
			<h1 className="font-semibold text-xl">Workflow introuvable</h1>
			<p className="max-w-md text-muted-foreground text-sm">
				Aucun workflow ne correspond à l’identifiant « {workflowId} ». Il a
				peut-être été archivé ou supprimé.
			</p>
			<Button asChild className="mt-4">
				<Link to={buildOrgPath(organizationSlug, '/automatisation')}>
					<ArrowLeft />
					Retour aux workflows
				</Link>
			</Button>
		</div>
	)
}
