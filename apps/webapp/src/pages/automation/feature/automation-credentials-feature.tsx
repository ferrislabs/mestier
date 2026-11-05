import { useEffect, useState } from 'react'
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
import {
	useCredentialMeta,
	useCredentialMetaActions,
} from '#/pages/automation/feature/use-credential-meta'
import { useExternalServices } from '#/pages/automation/feature/use-external-services'
import {
	buildCredentialFieldErrors,
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

	const meta = useCredentialMeta()
	const metaActions = useCredentialMetaActions()
	const { rememberKinds } = metaActions

	const authSchemes = catalogue.data?.data.auth_schemes ?? []
	const items = credentials.data?.data ?? []

	useEffect(() => {
		rememberKinds(items)
	}, [items, rememberKinds])
	const scheme = authSchemes.find((candidate) => candidate.kind === values.kind)

	const rows: CredentialRow[] = items.map((credential) => ({
		id: credential.id,
		name: credential.name,
		kind: credential.kind,
		kindLabel:
			authSchemes.find((candidate) => candidate.kind === credential.kind)
				?.label ?? credential.kind,
		origin: credential.origin === 'generated' ? 'generated' : 'supplied',
		createdAt: credential.created_at,
		updatedAt: credential.updated_at,
		description: meta.data[credential.id]?.description ?? '',
		usedByWorkflows: meta.data[credential.id]?.usedByWorkflows ?? 0,
		status: meta.data[credential.id]?.status ?? 'valid',
		account: meta.data[credential.id]?.account,
	}))

	const formErrors = buildCredentialFormErrors(values, scheme, mode)
	const fieldErrors = buildCredentialFieldErrors(values, scheme, mode)

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
			metaActions.patch(created.data.id, {
				description: values.description.trim(),
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
		metaActions.patch(editingId, { description: values.description.trim() })
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
				description: row.description,
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
			fieldErrors={fieldErrors}
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
					description: row.description,
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
				metaActions.forget(row.id)
				setDialogOpen(false)
			}}
			onConnect={(row) =>
				metaActions.patch(row.id, {
					status: 'valid',
					account: `autorisé le ${new Date().toLocaleDateString('fr-FR', { day: 'numeric', month: 'short', year: 'numeric' })}`,
				})
			}
			onDisconnect={(row) =>
				metaActions.patch(row.id, {
					status: 'not-connected',
					account: undefined,
				})
			}
		/>
	)
}
