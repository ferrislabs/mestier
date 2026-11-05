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
	credentialErrorMessage,
	credentialPayload,
	emptyCredentialForm,
} from '#/pages/automation/lib/credential-form'
import {
	kindLabel,
	localizeScheme,
	newestFirst,
} from '#/pages/automation/lib/credential-meta'
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

	const authSchemes = (catalogue.data?.data.auth_schemes ?? []).map(
		localizeScheme,
	)
	const items = credentials.data?.data ?? []

	useEffect(() => {
		rememberKinds(items)
	}, [items, rememberKinds])
	const scheme = authSchemes.find((candidate) => candidate.kind === values.kind)

	const rows: CredentialRow[] = newestFirst(items).map((credential) => ({
		id: credential.id,
		name: credential.name,
		kind: credential.kind,
		kindLabel: kindLabel(
			credential.kind,
			authSchemes.find((candidate) => candidate.kind === credential.kind)
				?.label,
		),
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

	const listError = credentialErrorMessage(credentials.error ?? catalogue.error)
	const saveError = credentialErrorMessage(
		createCredential.error ??
			updateCredential.error ??
			deleteCredential.error ??
			rotateCredential.error,
		scheme?.fields,
	)
	const submittedData = (data: Record<string, string>) =>
		scheme ? credentialPayload(scheme, data) : data

	const resetMutations = () => {
		createCredential.reset()
		updateCredential.reset()
		deleteCredential.reset()
		rotateCredential.reset()
	}

	const handleSubmit = async () => {
		if (formErrors.length > 0) return

		if (mode === 'create') {
			let created: Awaited<ReturnType<typeof createCredential.mutateAsync>>
			try {
				created = await createCredential.mutateAsync({
					path: { organization_id: organizationId },
					body: {
						kind: values.kind,
						name: values.name.trim(),
						origin: values.origin,
						data:
							values.origin === 'supplied'
								? submittedData(values.data)
								: undefined,
					},
				})
			} catch {
				return
			}
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
		try {
			await updateCredential.mutateAsync({
				path: { organization_id: organizationId, credential_id: editingId },
				body: {
					name: values.name.trim(),
					data: hasData ? submittedData(values.data) : undefined,
				},
			})
		} catch {
			return
		}
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
		} catch {
			return
		} finally {
			setRotatingId(null)
		}
	}

	const handleDelete = async (row: CredentialRow) => {
		try {
			await deleteCredential.mutateAsync({
				path: { organization_id: organizationId, credential_id: row.id },
			})
		} catch {
			return
		}
		metaActions.forget(row.id)
		setDialogOpen(false)
	}

	return (
		<AutomationCredentialsUI
			credentials={rows}
			authSchemes={authSchemes}
			services={services.data}
			canManage={canManage}
			isLoading={credentials.isLoading}
			schemesLoading={catalogue.isLoading}
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
			saveError={saveError}
			revealedSecret={revealedSecret}
			onOpenCreate={() => {
				resetMutations()
				setMode('create')
				setEditingId(null)
				setValues(emptyCredentialForm())
				setRevealedSecret(null)
				setDialogOpen(true)
			}}
			onEdit={(row) => {
				resetMutations()
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
			isDeleting={deleteCredential.isPending}
			onDelete={(row) => void handleDelete(row)}
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
