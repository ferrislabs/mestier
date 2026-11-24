import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useProject } from '#/hooks/use-projects'
import { DafListFeature } from '#/pages/daf/feature/daf-list-feature'
import { DafBackLink } from '#/pages/daf/ui/daf-back-link'
import { DafBreadcrumb } from '#/pages/daf/ui/daf-breadcrumb'
import { Page } from '#/pages/projects/ui/page'

export function DafFeature({ projectId }: { projectId: string }) {
	const { activeOrganization } = useActiveOrganization()
	const project = useProject(activeOrganization.id, projectId)
	const projectName = project.data?.data.name ?? 'Projet'

	return (
		<div className="h-full min-h-0 overflow-y-auto">
			<Page>
				<DafBackLink
					organizationSlug={activeOrganization.slug}
					projectId={projectId}
					to="project"
				/>
				<header className="mb-6">
					<DafBreadcrumb
						organizationSlug={activeOrganization.slug}
						projectId={projectId}
						projectName={projectName}
					/>
					<h1 className="truncate font-normal text-2xl tracking-tight md:text-[28px]">
						DAF – {projectName}
					</h1>
				</header>
				<DafListFeature
					organizationId={activeOrganization.id}
					organizationSlug={activeOrganization.slug}
					projectId={projectId}
				/>
			</Page>
		</div>
	)
}
