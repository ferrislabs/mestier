import { type FormEvent, useState } from 'react'
import { FormField } from '#/components/form-field'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogClose,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { hasErrors } from '#/pages/projects/lib/form-model'
import {
	emptyMissionForm,
	type MissionForm,
	missionFormErrors,
	missionFormOf,
	toMissionDraft,
} from '#/pages/projects/lib/mission-form'
import {
	isMissionStatus,
	MISSION_STATUS_LABELS,
	MISSION_STATUSES,
} from '#/pages/projects/lib/missions'
import type { Mission, MissionDraft, Person } from '#/pages/projects/lib/model'
import { useFormBinding } from '#/pages/projects/lib/use-form-binding'
import { PersonSelect } from '#/pages/projects/ui/person-select'
import { ProjectTextarea } from '#/pages/projects/ui/project-textarea'

export function MissionFormDialog({
	open,
	onOpenChange,
	defaultResponsibleId,
	mission,
	people,
	onSubmit,
}: {
	open: boolean
	onOpenChange: (open: boolean) => void
	defaultResponsibleId: string | null
	mission?: Mission
	people: readonly Person[]
	onSubmit: (draft: MissionDraft) => void
}) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="flex max-h-[calc(100dvh-2rem)] flex-col gap-0 p-0 sm:max-w-xl">
				<MissionFormBody
					defaultResponsibleId={defaultResponsibleId}
					mission={mission}
					people={people}
					onSubmit={(draft) => {
						onSubmit(draft)
						onOpenChange(false)
					}}
				/>
			</DialogContent>
		</Dialog>
	)
}

function MissionFormBody({
	defaultResponsibleId,
	mission,
	people,
	onSubmit,
}: {
	defaultResponsibleId: string | null
	mission?: Mission
	people: readonly Person[]
	onSubmit: (draft: MissionDraft) => void
}) {
	const [form, setForm] = useState<MissionForm>(() =>
		mission ? missionFormOf(mission) : emptyMissionForm(defaultResponsibleId),
	)
	const errors = missionFormErrors(form)
	const { bind, baseId, revealAll, formRef } = useFormBinding(errors)
	const title = bind('title')
	const startDate = bind('startDate')
	const endDate = bind('endDate')

	function change(patch: Partial<MissionForm>) {
		setForm((current) => ({ ...current, ...patch }))
	}

	function submit(event: FormEvent) {
		event.preventDefault()
		revealAll()
		if (hasErrors(errors)) return
		onSubmit(toMissionDraft(form))
	}

	return (
		<form
			ref={formRef}
			onSubmit={submit}
			noValidate
			className="flex min-h-0 flex-1 flex-col"
		>
			<DialogHeader className="px-6 pt-6 pb-4">
				<DialogTitle>
					{mission ? 'Modifier la mission' : 'Nouvelle mission'}
				</DialogTitle>
				<DialogDescription>
					Une mission est un lot de travail du projet, avec son avancement et
					son responsable.
				</DialogDescription>
			</DialogHeader>
			<div className="grid min-h-0 flex-1 gap-4 overflow-y-auto border-t px-6 py-5 sm:grid-cols-2">
				<FormField
					id={title.id}
					label="Titre"
					required
					error={title.error}
					className="sm:col-span-2"
				>
					<Input
						{...title.input}
						value={form.title}
						onChange={(event) => change({ title: event.target.value })}
						autoComplete="off"
						placeholder="Fabrication, pose, réception…"
					/>
				</FormField>
				<FormField
					id={`${baseId}-description`}
					label="Description"
					className="sm:col-span-2"
				>
					<ProjectTextarea
						id={`${baseId}-description`}
						value={form.description}
						onChange={(event) => change({ description: event.target.value })}
						placeholder="Ce que couvre la mission."
					/>
				</FormField>
				<FormField id={`${baseId}-status`} label="Statut">
					<Select
						value={form.status}
						onValueChange={(status) => {
							if (isMissionStatus(status)) change({ status })
						}}
					>
						<SelectTrigger id={`${baseId}-status`} className="w-full">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							{MISSION_STATUSES.map((status) => (
								<SelectItem key={status} value={status}>
									{MISSION_STATUS_LABELS[status]}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</FormField>
				<FormField id={`${baseId}-responsible`} label="Responsable">
					<PersonSelect
						id={`${baseId}-responsible`}
						value={form.responsibleId}
						people={people}
						onChange={(responsibleId) => change({ responsibleId })}
					/>
				</FormField>
				<FormField id={startDate.id} label="Début" error={startDate.error}>
					<Input
						{...startDate.input}
						type="date"
						value={form.startDate}
						onChange={(event) => change({ startDate: event.target.value })}
					/>
				</FormField>
				<FormField id={endDate.id} label="Fin" error={endDate.error}>
					<Input
						{...endDate.input}
						type="date"
						value={form.endDate}
						onChange={(event) => change({ endDate: event.target.value })}
					/>
				</FormField>
				{errors.other.length > 0 && (
					<ul className="grid gap-1 text-[13px] text-state-alert sm:col-span-2">
						{errors.other.map((message) => (
							<li key={message}>{message}</li>
						))}
					</ul>
				)}
			</div>
			<div className="flex items-center justify-between gap-3 border-t px-6 py-3">
				<p className="text-[12px] text-muted-foreground">
					Les champs marqués d’une * sont obligatoires.
				</p>
				<div className="flex gap-2">
					<DialogClose asChild>
						<Button type="button" variant="outline">
							Annuler
						</Button>
					</DialogClose>
					<Button type="submit">
						{mission ? 'Enregistrer' : 'Créer la mission'}
					</Button>
				</div>
			</div>
		</form>
	)
}
