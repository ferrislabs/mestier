import { Link } from '@tanstack/react-router'
import { ArrowLeft } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { buildOrgPath } from '#/modules/org-path'

/** The visible way out of a DAF page: to the project, or to its DAF list. */
export function DafBackLink({
	organizationSlug,
	projectId,
	to,
}: {
	organizationSlug: string
	projectId: string
	to: 'project' | 'list'
}) {
	return (
		<Button variant="ghost" size="sm" className="-ml-2 mb-2 w-fit" asChild>
			<Link
				to={buildOrgPath(
					organizationSlug,
					to === 'project' ? '/projet/$projectId' : '/projet/$projectId/daf',
				)}
				params={{ projectId }}
			>
				<ArrowLeft aria-hidden />
				{to === 'project' ? 'Retour au projet' : 'Retour aux DAF'}
			</Link>
		</Button>
	)
}
