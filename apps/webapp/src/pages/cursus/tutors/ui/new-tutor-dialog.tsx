import { useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import type { CreateParticipantValues } from '#/pages/cursus/participants/lib/participants'
import {
	CreateParticipantForm,
	type HostCompanyOption,
} from '#/pages/cursus/participants/ui/create-participant-dialog'
import type { SchoolTutorValues } from '#/pages/cursus/school-tutors/lib/school-tutors'
import { CreateSchoolTutorForm } from '#/pages/cursus/school-tutors/ui/create-school-tutor-form'
import type { SelectOption } from '#/pages/cursus/shared/lib/options'
import type { TutorKindChoice } from '#/pages/cursus/tutors/lib/tutors'
import { TutorKindChoiceCards } from '#/pages/cursus/tutors/ui/tutor-kind-choice'

export interface MutationState<Values> {
	isPending: boolean
	error: string | null
	onSubmit: (values: Values) => void
}

export interface NewTutorDialogProps {
	open: boolean
	hostCompanies: HostCompanyOption[]
	members: SelectOption[]
	isLoadingMembers: boolean
	company: MutationState<CreateParticipantValues>
	school: MutationState<SchoolTutorValues>
	onOpenChange: (open: boolean) => void
}

type Screen = 'choose' | TutorKindChoice

export function NewTutorDialog({
	open,
	hostCompanies,
	members,
	isLoadingMembers,
	company,
	school,
	onOpenChange,
}: NewTutorDialogProps) {
	const [choice, setChoice] = useState<TutorKindChoice>('company')
	const [screen, setScreen] = useState<Screen>('choose')
	const backToChoice = () => setScreen('choose')

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-lg">
				{screen === 'choose' ? (
					<div className="grid gap-4">
						<DialogHeader>
							<DialogTitle>Nouveau tuteur</DialogTitle>
							<DialogDescription>
								Choisissez le type de tuteur à créer.
							</DialogDescription>
						</DialogHeader>
						<TutorKindChoiceCards value={choice} onChange={setChoice} />
						<DialogFooter>
							<Button
								type="button"
								variant="ghost"
								onClick={() => onOpenChange(false)}
							>
								Annuler
							</Button>
							<Button type="button" onClick={() => setScreen(choice)}>
								Suivant
							</Button>
						</DialogFooter>
					</div>
				) : null}
				{screen === 'company' ? (
					<CreateParticipantForm
						kind="company_tutor"
						hostCompanies={hostCompanies}
						isPending={company.isPending}
						error={company.error}
						onCancel={() => onOpenChange(false)}
						onFirstBack={backToChoice}
						onSubmit={company.onSubmit}
					/>
				) : null}
				{screen === 'school' ? (
					<CreateSchoolTutorForm
						members={members}
						isLoadingMembers={isLoadingMembers}
						isPending={school.isPending}
						error={school.error}
						onBack={backToChoice}
						onSubmit={school.onSubmit}
					/>
				) : null}
			</DialogContent>
		</Dialog>
	)
}
