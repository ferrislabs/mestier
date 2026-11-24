import { useForm, useStore } from '@tanstack/react-form'
import { useMemo } from 'react'
import type { Schemas } from '#/api/api.client'
import { useSaveDafDossier } from '#/hooks/use-daf'
import { useDirtyBaseline } from '#/hooks/use-dirty'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	buildDossierBody,
	type DafDossierValues,
	dossierToForm,
} from '#/pages/daf/lib/dossier-form'
import { DafDossierCard } from '#/pages/daf/ui/daf-dossier-card'

export function DafDossierFeature({
	organizationId,
	projectId,
	dossier,
}: {
	organizationId: string
	projectId: string
	dossier: Schemas.DafDossierResponse
}) {
	const initial = useMemo(() => dossierToForm(dossier), [dossier])
	const save = useSaveDafDossier()
	const form = useForm({
		defaultValues: initial,
		onSubmit: async ({ value }) => {
			const response = await save.mutateAsync({
				path: { organization_id: organizationId, project_id: projectId },
				body: buildDossierBody(value),
			})
			const next = dossierToForm(response.data)
			form.reset(next)
			dirty.commit(next)
		},
	})
	const values = useStore(form.store, (state) => state.values)
	const dirty = useDirtyBaseline(initial, values)

	return (
		<DafDossierCard
			values={values}
			isStored={Boolean(dossier.updated_at)}
			isDirty={dirty.isDirty}
			isSaving={save.isPending}
			error={mutationErrorMessage(save.error)}
			onChange={(patch: Partial<DafDossierValues>) => {
				for (const [key, value] of Object.entries(patch)) {
					form.setFieldValue(key as keyof DafDossierValues, value)
				}
			}}
			onSave={() => {
				void form.handleSubmit().catch(() => {})
			}}
		/>
	)
}
