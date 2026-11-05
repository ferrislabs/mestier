import { Link } from '@tanstack/react-router'
import { ArrowLeft, FolderX } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { buildOrgPath } from '#/modules/org-path'
import { Page } from '#/pages/projects/ui/page'

export function ProjectNotFound({
	organizationSlug,
}: {
	organizationSlug: string
}) {
	return (
		<Page className="max-w-3xl">
			<div className="rounded-lg border border-dashed bg-card px-6 py-12 text-center">
				<FolderX
					className="mx-auto mb-3 size-8 text-muted-foreground"
					aria-hidden
				/>
				<h1 className="font-medium">Ce projet est introuvable.</h1>
				<p className="mt-1 text-[13px] text-muted-foreground">
					Il n’existe pas ou l’adresse n’est plus valide.
				</p>
				<Button variant="outline" size="sm" className="mt-4" asChild>
					<Link to={buildOrgPath(organizationSlug, '/planification/projects')}>
						<ArrowLeft aria-hidden /> Retour aux projets
					</Link>
				</Button>
			</div>
		</Page>
	)
}
