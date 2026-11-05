import { Link } from '@tanstack/react-router'
import { ArrowLeft, FilePlus, Loader2, Plus } from 'lucide-react'
import { type FormEvent, type ReactNode, useId, useState } from 'react'
import { EmptyState } from '#/components/empty-state'
import { FormField } from '#/components/form-field'
import { RequirePermission } from '#/components/require-permission'
import { Button, buttonVariants } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { Switch } from '#/components/ui/switch'
import { buildOrgPath } from '#/modules/org-path'
import { moveItem } from '#/pages/doe/lib/sections'
import { KIND_LABELS, SECTION_KINDS, type SectionKind } from '#/pages/doe/types'
import {
	canSaveModel,
	modelErrors,
	newModelSectionDraft,
	sameModelDraft,
} from '#/pages/doe-models/lib/model-form'
import type { ModelDraft, ModelSectionDraft } from '#/pages/doe-models/types'
import { SectionRow } from '#/pages/doe-models/ui/section-row'
import { saveStatus } from '#/pages/projects/lib/save-status'
import { StatusBadge } from '#/pages/projects/ui/badges'
import { Page, PageHeader } from '#/pages/projects/ui/page'
import { ProjectTextarea } from '#/pages/projects/ui/project-textarea'
import { SettingsSection } from '#/pages/projects/ui/settings-section'
import { useReportDirty } from '#/pages/projects/ui/unsaved-guard'

export function DoeModelNotFound({
	organizationSlug,
}: {
	organizationSlug: string
}) {
	return (
		<Page>
			<PageHeader title="Modèle de DOE" />
			<div role="alert">
				<EmptyState
					title="Modèle introuvable"
					hint="Ce modèle n’existe plus ou le lien est incorrect."
				>
					<Link
						to={buildOrgPath(organizationSlug, '/settings')}
						className={buttonVariants({ variant: 'outline', size: 'sm' })}
					>
						Voir les modèles de DOE
					</Link>
				</EmptyState>
			</div>
		</Page>
	)
}

export interface DoeModelPageProps {
	organizationSlug: string
	name: string
	themesSummary: string
	archived: boolean
	draft: ModelDraft
	stored: ModelDraft
	notice: string
	isSaving: boolean
	error: string | null
	styleEditor: ReactNode
	onChange: (patch: Partial<ModelDraft>) => void
	onStyleToggle: (enabled: boolean) => void
	onSubmit: () => Promise<void>
	onToggleArchive: () => void
	onCreateDoe: () => void
	onDirtyChange: (dirty: boolean) => void
}

export function DoeModelPage({
	organizationSlug,
	name,
	themesSummary,
	archived,
	draft,
	stored,
	notice,
	isSaving,
	error,
	styleEditor,
	onChange,
	onStyleToggle,
	onSubmit,
	onToggleArchive,
	onCreateDoe,
	onDirtyChange,
}: DoeModelPageProps) {
	const baseId = useId()
	const [saved, setSaved] = useState(false)
	const [newTitle, setNewTitle] = useState('')
	const [newKind, setNewKind] = useState<SectionKind>('supplies')
	const errors = modelErrors(draft)
	const dirty = !sameModelDraft(draft, stored)
	const otherMessages = errors.messages.filter(
		(message) => message !== errors.name,
	)
	useReportDirty(dirty, onDirtyChange)

	function change(patch: Partial<ModelDraft>) {
		onChange(patch)
		setSaved(false)
	}

	function patchSection(key: string, patch: Partial<ModelSectionDraft>) {
		change({
			sections: draft.sections.map((section) =>
				section.key === key ? { ...section, ...patch } : section,
			),
		})
	}

	function appendSection() {
		const title = newTitle.trim()
		if (title === '') return
		change({
			sections: [
				...draft.sections,
				newModelSectionDraft(crypto.randomUUID(), title, newKind),
			],
		})
		setNewTitle('')
	}

	function submit(event: FormEvent) {
		event.preventDefault()
		if (!canSaveModel(draft, stored) || isSaving) return
		void onSubmit().then(() => setSaved(true))
	}

	return (
		<Page>
			<Link
				to={buildOrgPath(organizationSlug, '/settings')}
				className="-ml-1 mb-3 inline-flex items-center gap-1.5 rounded-sm px-1 text-[13px] text-muted-foreground outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
			>
				<ArrowLeft className="size-4" aria-hidden />
				Modèles de DOE
			</Link>
			<PageHeader
				title={name}
				description={themesSummary}
				actions={
					<>
						{archived && <StatusBadge tone="neutral">Archivé</StatusBadge>}
						<RequirePermission permission="MANAGE_REFERENCE">
							{!archived && (
								<Button
									variant="outline"
									disabled={dirty}
									onClick={onCreateDoe}
								>
									<FilePlus aria-hidden />
									Créer un DOE avec ce modèle
								</Button>
							)}
							<Button variant="outline" onClick={onToggleArchive}>
								{archived ? 'Restaurer' : 'Archiver'}
							</Button>
						</RequirePermission>
					</>
				}
			/>
			<output className="mb-3 block text-[13px] text-muted-foreground empty:mb-0">
				{notice ||
					(dirty && !archived
						? 'Enregistre les modifications avant de créer un DOE avec ce modèle.'
						: '')}
			</output>
			{archived && (
				<p className="mb-4 rounded-md border bg-muted/50 px-4 py-3 text-[13px] text-muted-foreground">
					Ce modèle est archivé : il n’est plus proposé pour créer un DOE.
					Restaure-le pour l’utiliser de nouveau.
				</p>
			)}
			<form onSubmit={submit} noValidate className="grid gap-6">
				<SettingsSection
					id={`${baseId}-info`}
					title="Informations"
					description="Le nom et la description servent à reconnaître le modèle dans la liste."
				>
					<div className="grid gap-4 px-5 py-5">
						<FormField
							id={`${baseId}-name`}
							label="Nom"
							required
							error={errors.name}
						>
							<Input
								id={`${baseId}-name`}
								value={draft.name}
								onChange={(event) => change({ name: event.target.value })}
								autoComplete="off"
								placeholder="Paysagiste, Gros œuvre…"
								aria-invalid={errors.name ? true : undefined}
								aria-describedby={
									errors.name ? `${baseId}-name-error` : undefined
								}
							/>
						</FormField>
						<FormField id={`${baseId}-description`} label="Description">
							<ProjectTextarea
								id={`${baseId}-description`}
								rows={3}
								value={draft.description}
								onChange={(event) =>
									change({ description: event.target.value })
								}
							/>
						</FormField>
					</div>
				</SettingsSection>
				<SettingsSection
					id={`${baseId}-themes`}
					title="Thèmes"
					description="Les thèmes sont copiés dans chaque DOE créé avec ce modèle. Un thème non activé est créé désactivé : tu pourras l’activer dans le DOE."
					actions={
						<span className="text-[12px] text-muted-foreground tabular-nums">
							{draft.sections.length}
						</span>
					}
				>
					<div className="grid gap-3 px-5 py-5">
						{draft.sections.length === 0 ? (
							<p className="rounded-md border border-dashed px-4 py-5 text-center text-[13px] text-muted-foreground">
								Aucun thème pour l’instant. Ajoute le premier ci-dessous.
							</p>
						) : (
							<ul className="grid gap-2">
								{draft.sections.map((section, index) => (
									<SectionRow
										key={section.key}
										index={index}
										count={draft.sections.length}
										section={section}
										invalid={errors.sections[index] === true}
										onChange={(patch) => patchSection(section.key, patch)}
										onMove={(delta) =>
											change({
												sections: moveItem(draft.sections, index, delta),
											})
										}
										onRemove={() =>
											change({
												sections: draft.sections.filter(
													(entry) => entry.key !== section.key,
												),
											})
										}
									/>
								))}
							</ul>
						)}
						<div className="grid gap-3 rounded-md border border-dashed p-3 sm:grid-cols-[minmax(0,1fr)_11rem_auto] sm:items-end">
							<FormField
								id={`${baseId}-new-title`}
								label="Titre du nouveau thème"
							>
								<Input
									id={`${baseId}-new-title`}
									value={newTitle}
									onChange={(event) => setNewTitle(event.target.value)}
									onKeyDown={(event) => {
										if (event.key !== 'Enter') return
										event.preventDefault()
										appendSection()
									}}
									autoComplete="off"
								/>
							</FormField>
							<FormField id={`${baseId}-new-kind`} label="Type">
								<Select
									value={newKind}
									onValueChange={(kind) => setNewKind(kind as SectionKind)}
								>
									<SelectTrigger id={`${baseId}-new-kind`} className="w-full">
										<SelectValue />
									</SelectTrigger>
									<SelectContent>
										{SECTION_KINDS.map((kind) => (
											<SelectItem key={kind} value={kind}>
												{KIND_LABELS[kind]}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
							</FormField>
							<Button
								type="button"
								variant="outline"
								disabled={newTitle.trim() === ''}
								onClick={appendSection}
							>
								<Plus aria-hidden />
								Ajouter un thème
							</Button>
						</div>
						{otherMessages.length > 0 && (
							<ul
								role="alert"
								className="grid gap-1 text-[13px] text-state-alert"
							>
								{otherMessages.map((message) => (
									<li key={message}>{message}</li>
								))}
							</ul>
						)}
					</div>
				</SettingsSection>
				<SettingsSection
					id={`${baseId}-style`}
					title="Style propre à ce modèle"
					description="Sans style propre, le style de l’organisation s’applique aux DOE créés depuis ce modèle."
					actions={
						<Switch
							aria-label="Style propre à ce modèle"
							checked={draft.style !== null}
							onCheckedChange={(enabled) => {
								onStyleToggle(enabled)
							}}
						/>
					}
				>
					<div className="px-5 py-5">
						{draft.style === null ? (
							<p className="text-[13px] text-muted-foreground">
								Le style de l’organisation s’applique aux DOE créés depuis ce
								modèle.
							</p>
						) : (
							styleEditor
						)}
					</div>
				</SettingsSection>
				{error ? (
					<p role="alert" className="text-[13px] text-state-alert">
						{error}
					</p>
				) : null}
				<div className="flex flex-wrap items-center justify-between gap-3 rounded-lg border bg-card px-5 py-3">
					<output className="text-[13px] text-muted-foreground">
						{saveStatus({ dirty, saved, savedLabel: 'Modèle enregistré.' })}
					</output>
					<div className="flex items-center gap-2">
						<Button
							type="button"
							variant="outline"
							size="sm"
							disabled={!dirty}
							onClick={() => change({ ...stored })}
						>
							Annuler les modifications
						</Button>
						<RequirePermission permission="MANAGE_REFERENCE">
							<Button
								type="submit"
								size="sm"
								disabled={!canSaveModel(draft, stored) || isSaving}
							>
								{isSaving ? <Loader2 className="animate-spin" /> : null}
								Enregistrer
							</Button>
						</RequirePermission>
					</div>
				</div>
			</form>
		</Page>
	)
}
