import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useAutomationCredentials,
	useAutomationRuns,
	useAutomationSettings,
	useAutomationWorkflows,
	useConnectorCatalogue,
	useCreateCredential,
	useDeleteCredential,
	useReplayRun,
	useRotateCredential,
	useRun,
	useUpdateAutomationSettings,
	useUpdateCredential,
} from '#/hooks/use-automation'
import {
	buildCredentialFormErrors,
	emptyCredentialForm,
	isFieldValueFilled,
	parseSettingsForm,
	settingsToFormValues,
} from '#/pages/settings/lib/automation'
import type { CredentialFormValues } from '#/pages/settings/types'
import {
	AutomationCredentialsUI,
	type CredentialRow,
} from '#/pages/settings/ui/automation-credentials-ui'
import {
	AutomationRunsUI,
	type RunRow,
} from '#/pages/settings/ui/automation-runs-ui'
import { AutomationSettingsUI } from '#/pages/settings/ui/automation-settings-ui'

export function AutomationSection() {
	const { activeOrganization } = useActiveOrganization()

	return (
		<div className="flex flex-col gap-8" key={activeOrganization.id}>
			<CredentialsPanel organizationId={activeOrganization.id} />
			<SettingsPanel organizationId={activeOrganization.id} />
			<RunsPanel organizationId={activeOrganization.id} />
		</div>
	)
}

function CredentialsPanel({ organizationId }: { organizationId: string }) {
	const catalogue = useConnectorCatalogue(organizationId)
	const credentials = useAutomationCredentials(organizationId)
	const createCredential = useCreateCredential(organizationId)
	const updateCredential = useUpdateCredential()
	const deleteCredential = useDeleteCredential()
	const rotateCredential = useRotateCredential()

	const [sheetOpen, setSheetOpen] = useState(false)
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
				setSheetOpen(false)
			}
			return
		}

		if (!editingId) return
		const hasData = Object.values(values.data).some(isFieldValueFilled)
		await updateCredential.mutateAsync({
			path: { organization_id: organizationId, credential_id: editingId },
			body: {
				name: values.name.trim(),
				data: hasData ? values.data : undefined,
			},
		})
		setSheetOpen(false)
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
			setSheetOpen(true)
		} finally {
			setRotatingId(null)
		}
	}

	return (
		<AutomationCredentialsUI
			credentials={rows}
			authSchemes={authSchemes}
			isLoading={credentials.isLoading}
			error={credentials.error?.message ?? null}
			sheetOpen={sheetOpen}
			sheetMode={mode}
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
				setValues(emptyCredentialForm(authSchemes[0]?.kind ?? ''))
				setRevealedSecret(null)
				setSheetOpen(true)
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
				setSheetOpen(true)
			}}
			onOpenChange={(open) => {
				setSheetOpen(open)
				if (!open) setRevealedSecret(null)
			}}
			onRotate={(row) => void handleRotate(row)}
			rotatingId={rotatingId}
			onDelete={(row) =>
				void deleteCredential.mutateAsync({
					path: { organization_id: organizationId, credential_id: row.id },
				})
			}
		/>
	)
}

function SettingsPanel({ organizationId }: { organizationId: string }) {
	const settingsQuery = useAutomationSettings(organizationId)
	const updateSettings = useUpdateAutomationSettings()
	const [draft, setDraft] = useState<ReturnType<
		typeof settingsToFormValues
	> | null>(null)
	const [formError, setFormError] = useState<string | null>(null)

	const settings = settingsQuery.data?.data
	const values = draft ?? (settings ? settingsToFormValues(settings) : null)

	if (!values) {
		return (
			<AutomationSettingsUI
				isLoading={settingsQuery.isLoading}
				values={{
					eventRetentionSeconds: '',
					succeededRunRetentionSeconds: '',
					retryScheduleSeconds: '',
					disableTargetAfter: '',
				}}
				retrySchedulePreview={[]}
				isPending={false}
				formError={null}
				saveError={settingsQuery.error?.message ?? null}
				onChange={() => {}}
				onSubmit={() => {}}
			/>
		)
	}

	const parsed = parseSettingsForm(values)

	return (
		<AutomationSettingsUI
			isLoading={false}
			values={values}
			retrySchedulePreview={parsed.ok ? parsed.body.retry_schedule_seconds : []}
			isPending={updateSettings.isPending}
			formError={formError}
			saveError={updateSettings.error?.message ?? null}
			onChange={(patch) => setDraft({ ...values, ...patch })}
			onSubmit={() => {
				const result = parseSettingsForm(values)
				if (!result.ok) {
					setFormError(result.error)
					return
				}
				setFormError(null)
				void updateSettings.mutateAsync({
					path: { organization_id: organizationId },
					body: result.body,
				})
			}}
		/>
	)
}

function RunsPanel({ organizationId }: { organizationId: string }) {
	const runsQuery = useAutomationRuns(organizationId)
	const workflowsQuery = useAutomationWorkflows(organizationId)
	const replayRun = useReplayRun()

	const [detailRunId, setDetailRunId] = useState<string | null>(null)
	const [replayingConnectorId, setReplayingConnectorId] = useState<
		string | null
	>(null)

	const runDetail = useRun(organizationId, detailRunId)

	const workflowNameById = new Map(
		(workflowsQuery.data?.data ?? []).map((workflow) => [
			workflow.id,
			workflow.name,
		]),
	)

	const runs = runsQuery.data?.data ?? []
	const rows: RunRow[] = runs.map((run) => ({
		id: run.id,
		workflowName: workflowNameById.get(run.workflow_id) ?? run.workflow_id,
		status: run.status,
		startedAt: run.started_at ?? null,
		finishedAt: run.finished_at ?? null,
		nextAttemptAt: run.next_attempt_at ?? null,
		error: run.error ?? null,
	}))

	const detailRun = rows.find((row) => row.id === detailRunId) ?? null
	const detailSteps = (runDetail.data?.data.steps ?? []).map((step) => ({
		id: step.id,
		connectorId: step.connector_id,
		status: step.status,
		attempts: step.attempts,
		error: step.error ?? null,
	}))

	return (
		<AutomationRunsUI
			runs={rows}
			isLoading={runsQuery.isLoading}
			error={runsQuery.error?.message ?? null}
			detailOpen={detailRunId !== null}
			detailRun={detailRun}
			detailSteps={detailSteps}
			detailLoading={runDetail.isLoading}
			onOpenDetail={(run) => setDetailRunId(run.id)}
			onCloseDetail={() => setDetailRunId(null)}
			onReplay={(run, connectorId) => {
				setReplayingConnectorId(connectorId)
				void replayRun
					.mutateAsync({
						path: { organization_id: organizationId, run_id: run.id },
						body: { connector_id: connectorId },
					})
					.finally(() => setReplayingConnectorId(null))
			}}
			replayingConnectorId={replayingConnectorId}
		/>
	)
}
