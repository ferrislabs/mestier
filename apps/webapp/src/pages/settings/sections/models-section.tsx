import { useState } from 'react'
import { RequirePermission } from '#/components/require-permission'
import { DoeModelsFeature } from '#/pages/doe-models/feature/doe-models-feature'
import { ProjectTemplatesFeature } from '#/pages/project-templates/feature/project-templates-feature'
import { DocumentsSection } from '#/pages/settings/sections/documents-section'

/**
 * Every template Mestier produces something from, in one place: project
 * templates, DOE models, and the PDF layout the DOE models can override.
 */
export function ModelsSection() {
	const [templatesArchived, setTemplatesArchived] = useState(false)
	const [doeModelsArchived, setDoeModelsArchived] = useState(false)

	return (
		<div className="flex flex-col gap-10">
			<RequirePermission permission="MANAGE_PLANNING">
				<ProjectTemplatesFeature
					includeArchived={templatesArchived}
					onIncludeArchivedChange={setTemplatesArchived}
				/>
			</RequirePermission>
			<RequirePermission permission="MANAGE_PLANNING">
				<DoeModelsFeature
					includeArchived={doeModelsArchived}
					onIncludeArchivedChange={setDoeModelsArchived}
				/>
			</RequirePermission>
			<DocumentsSection />
		</div>
	)
}
