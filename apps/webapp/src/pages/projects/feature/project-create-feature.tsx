import { useNavigate } from '@tanstack/react-router'
import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCreateProject } from '#/hooks/use-projects'
import { buildOrgPath } from '#/modules/org-path'
import { saveProjectExtras } from '#/pages/projects/feature/mocks/project-extras'
import { useProjectFormOptions } from '#/pages/projects/feature/use-project-form-options'
import {
	emptyFullForm,
	extrasOfForm,
	type ProjectFullForm,
} from '#/pages/projects/lib/project-form'
import { optionalId } from '#/pages/projects/types'
import { ProjectCreate } from '#/pages/projects/ui/project-create'

export function ProjectCreateFeature({
	customerId,
}: {
	customerId: string | undefined
}) {
	const { activeOrganization } = useActiveOrganization()
	const navigate = useNavigate()
	const createProject = useCreateProject()
	const [form, setForm] = useState<ProjectFullForm>(() =>
		emptyFullForm(null, customerId ?? ''),
	)
	const options = useProjectFormOptions(activeOrganization.id, form.customerId)

	const submit = async () => {
		const created = await createProject.mutateAsync({
			path: { organization_id: activeOrganization.id },
			body: {
				name: form.name.trim(),
				customer_id: optionalId(form.customerId),
				customer_context_id: optionalId(form.customerContextId),
				quote_id: optionalId(form.quoteId),
			},
		})
		await saveProjectExtras(created.data.id, extrasOfForm(form))
		await navigate({
			to: buildOrgPath(
				activeOrganization.slug,
				'/planification/projects/$projectId',
			),
			params: { projectId: created.data.id },
		})
	}

	return (
		<ProjectCreate
			organizationSlug={activeOrganization.slug}
			form={form}
			customers={options.customers}
			quotes={options.quotes}
			customerContexts={options.customerContexts}
			isCustomerContextsLoading={options.isCustomerContextsLoading}
			people={options.people}
			isPending={createProject.isPending}
			error={createProject.error?.message ?? null}
			onChange={(patch) => setForm((current) => ({ ...current, ...patch }))}
			onSubmit={() => {
				void submit().catch(() => {})
			}}
		/>
	)
}
