import { AlertCircle, FileText, Loader2, Save, Trash2 } from 'lucide-react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { type DafStatus, dafNumberLabel } from '#/pages/daf/lib/daf-status'
import { DafBackLink } from '#/pages/daf/ui/daf-back-link'
import { DafBreadcrumb } from '#/pages/daf/ui/daf-breadcrumb'
import {
	DafEditorForm,
	type DafEditorFormProps,
} from '#/pages/daf/ui/daf-editor-form'
import { DafStatusBadge } from '#/pages/daf/ui/daf-status-badge'
import { Page } from '#/pages/projects/ui/page'

export interface DafEditorUIProps {
	organizationSlug: string
	projectId: string
	projectName: string
	number: number
	designation: string
	status: DafStatus
	isDirty: boolean
	isSaving: boolean
	canSave: boolean
	notice: string | null
	errors: readonly string[]
	error: string | null
	form: DafEditorFormProps
	onSave: () => void
	onPreview: () => void
	onDelete: () => void
}

export function DafEditorUI({
	organizationSlug,
	projectId,
	projectName,
	number,
	designation,
	status,
	isDirty,
	isSaving,
	canSave,
	notice,
	errors,
	error,
	form,
	onSave,
	onPreview,
	onDelete,
}: DafEditorUIProps) {
	return (
		<div className="h-full min-h-0 overflow-y-auto">
			<Page>
				<DafBackLink
					organizationSlug={organizationSlug}
					projectId={projectId}
					to="list"
				/>
				<header className="mb-6 flex flex-wrap items-end justify-between gap-4">
					<div className="min-w-0">
						<DafBreadcrumb
							organizationSlug={organizationSlug}
							projectId={projectId}
							projectName={projectName}
							current={dafNumberLabel(number)}
						/>
						<div className="flex flex-wrap items-center gap-3">
							<h1 className="truncate font-normal text-2xl tracking-tight md:text-[28px]">
								{dafNumberLabel(number)} – {designation}
							</h1>
							<DafStatusBadge status={status} />
						</div>
					</div>
					<div className="flex flex-wrap items-center gap-2">
						<Button
							type="button"
							variant="outline"
							size="sm"
							onClick={onPreview}
						>
							<FileText aria-hidden />
							Aperçu du PDF
						</Button>
						<RequirePermission permission="MANAGE_PLANNING">
							<Button
								type="button"
								variant="outline"
								size="sm"
								onClick={onDelete}
							>
								<Trash2 aria-hidden />
								Supprimer
							</Button>
							<Button
								type="button"
								size="sm"
								disabled={!canSave || !isDirty || isSaving}
								onClick={onSave}
							>
								{isSaving ? <Loader2 className="animate-spin" /> : <Save />}
								Enregistrer
							</Button>
						</RequirePermission>
					</div>
				</header>
				<div className="grid gap-4">
					{notice ? (
						<output className="text-[13px] text-muted-foreground">
							{notice}
						</output>
					) : null}
					{isDirty && errors.length > 0 ? (
						<ul className="list-disc pl-5 text-[13px] text-destructive">
							{errors.map((message) => (
								<li key={message}>{message}</li>
							))}
						</ul>
					) : null}
					{error ? (
						<div
							role="alert"
							className="flex items-start gap-2 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive"
						>
							<AlertCircle className="mt-0.5 size-4 shrink-0" aria-hidden />
							{error}
						</div>
					) : null}
					<DafEditorForm {...form} />
				</div>
			</Page>
		</div>
	)
}
