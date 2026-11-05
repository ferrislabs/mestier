import { Link } from '@tanstack/react-router'
import { AlertCircle, ArrowLeft, Loader2, SearchX } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { SectionCard } from '#/components/ui/surface'
import { buildOrgPath } from '#/modules/org-path'
import { workflowsPath } from '#/pages/automation/lib/automation-paths'

function BackToWorkflows({ organizationSlug }: { organizationSlug: string }) {
	return (
		<Button asChild className="mt-4">
			<Link to={buildOrgPath(organizationSlug, workflowsPath())}>
				<ArrowLeft />
				Retour aux workflows
			</Link>
		</Button>
	)
}

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
			<BackToWorkflows organizationSlug={organizationSlug} />
		</div>
	)
}

export function WorkflowLoadError({
	organizationSlug,
	message,
}: {
	organizationSlug: string
	message: string | null
}) {
	return (
		<SectionCard className="m-6 flex min-h-72 flex-col items-center justify-center gap-1 p-8 text-center">
			<AlertCircle className="mb-2 size-6 text-destructive" aria-hidden />
			<p className="font-semibold">Impossible de charger le workflow</p>
			<p className="text-muted-foreground text-sm">
				{message ?? 'Réessayez dans quelques instants.'}
			</p>
			<BackToWorkflows organizationSlug={organizationSlug} />
		</SectionCard>
	)
}

export function WorkflowLoading() {
	return (
		<div className="flex min-h-0 flex-1 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
			<Loader2 className="size-5 animate-spin" />
			Chargement du workflow…
		</div>
	)
}

export function DraftExecutionsEmpty() {
	return (
		<div className="flex min-h-0 flex-1 items-center justify-center p-8 text-center text-sm text-muted-foreground">
			Aucune exécution : ce workflow n’est pas encore enregistré.
		</div>
	)
}
