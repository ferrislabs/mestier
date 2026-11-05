import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useAutomationCredentials,
	useConnectorCatalogue,
	useCreateCredential,
	useDeleteCredential,
	useRotateCredential,
	useUpdateCredential,
} from '#/hooks/use-automation'
import { useHasPermission } from '#/hooks/use-permissions'
import { useExternalServices } from '#/pages/automation/feature/use-external-services'
import {
	buildCredentialFormErrors,
	emptyCredentialForm,
} from '#/pages/automation/lib/credential-form'
import type { CredentialFormValues } from '#/pages/automation/types'
import {
	AutomationCredentialsUI,
	type CredentialRow,
} from '#/pages/automation/ui/automation-credentials-ui'

export function AutomationCredentialsFeature() {
	const { activeOrganization } = useActiveOrganization()

	return (
		<AutomationCredentialsWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
		/>
	)
}

function AutomationCredentialsWorkspace({
	organizationId,
}: {
	organizationId: string
}) {
	const services = useExternalServices()
	const canManage = useHasPermission('MANAGE_AUTOMATION')
	const catalogue = useConnectorCatalogue(organizationId)
	const credentials = useAutomationCredentials(organizationId)
	const createCredential = useCreateCredential(organizationId)
	const updateCredential = useUpdateCredential()
	const deleteCredential = useDeleteCredential()
	const rotateCredential = useRotateCredential()

	const [dialogOpen, setDialogOpen] = useState(false)
	const [mode, setMode] = useState<'create' | 'edit'>('create')
	const [editingId, setEditingId] = useState<string | null>(null)
	const [values, setValues] = useState<CredentialFormValues>(
		emptyCredentialForm(),
	)
	const [revealedSecret, setRevealedSecret] = useState<string | null>(null)
	const [rotatingId, setRotatingId] = useState<string | null>(null)

	const authSchemes = catalogue.data?.data.auth_schemes ?? []
	const items = credentials.data?.data ?? []
	const scheme = authSchemes.find((candidate) => candidate.kind === values.kind)

	const rows: CredentialRow[] = items.map((credential) => ({
		id: credential.id,
		name: credential.name,
		kind: credential.kind,
		kindLabel:
			authSchemes.find((candidate) => candidate.kind === credential.kind)
				?.label ?? credential.kind,
		origin: credential.origin === 'generated' ? 'generated' : 'supplied',
		updatedAt: credential.updated_at,
	}))

	const formErrors = buildCredentialFormErrors(values, scheme, mode)

	const listError =
		credentials.error?.message ?? deleteCredential.error?.message ?? null

	const handleSubmit = async () => {
		if (formErrors.length > 0) return

		if (mode === 'create') {
			const created = await createCredential.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					kind: values.kind,
					name: values.name.trim(),
					origin: values.origin,
					data: values.origin === 'supplied' ? values.data : undefined,
				},
			})
			if (values.origin === 'generated') {
				setRevealedSecret(String(created.data.secret))
			} else {
				setDialogOpen(false)
			}
			return
		}

		if (!editingId) return
		const hasData = Object.values(values.data).some(
			(value) => value.trim() !== '',
		)
		await updateCredential.mutateAsync({
			path: { organization_id: organizationId, credential_id: editingId },
			body: {
				name: values.name.trim(),
				data: hasData ? values.data : undefined,
			},
		})
		setDialogOpen(false)
	}

	const handleRotate = async (row: CredentialRow) => {
		setRotatingId(row.id)
		try {
			const rotated = await rotateCredential.mutateAsync({
				path: { organization_id: organizationId, credential_id: row.id },
			})
			setMode('edit')
			setEditingId(row.id)
			setValues({
				kind: row.kind,
				name: row.name,
				origin: 'generated',
				data: {},
			})
			setRevealedSecret(String(rotated.data.secret))
			setDialogOpen(true)
		} finally {
			setRotatingId(null)
		}
	}

	return (
		<AutomationCredentialsUI
			credentials={rows}
			authSchemes={authSchemes}
			services={services.data}
			canManage={canManage}
			isLoading={credentials.isLoading}
			error={listError}
			dialogOpen={dialogOpen}
			dialogMode={mode}
			editing={rows.find((row) => row.id === editingId) ?? null}
			form={{
				values,
				isPending: createCredential.isPending || updateCredential.isPending,
				onChange: (patch) => setValues((current) => ({ ...current, ...patch })),
				onSubmit: () => void handleSubmit(),
			}}
			formErrors={formErrors}
			saveError={
				createCredential.error?.message ??
				updateCredential.error?.message ??
				null
			}
			revealedSecret={revealedSecret}
			onOpenCreate={() => {
				setMode('create')
				setEditingId(null)
				setValues(emptyCredentialForm())
				setRevealedSecret(null)
				setDialogOpen(true)
			}}
			onEdit={(row) => {
				setMode('edit')
				setEditingId(row.id)
				setValues({
					kind: row.kind,
					name: row.name,
					origin: row.origin,
					data: {},
				})
				setRevealedSecret(null)
				setDialogOpen(true)
			}}
			onOpenChange={(open) => {
				setDialogOpen(open)
				if (!open) setRevealedSecret(null)
			}}
			onRotate={(row) => void handleRotate(row)}
			rotatingId={rotatingId}
			onDelete={(row) => {
				deleteCredential.mutate({
					path: { organization_id: organizationId, credential_id: row.id },
				})
				setDialogOpen(false)
			}}
		/>
	)
}
